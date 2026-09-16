use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::flags::Ldflag;
use crate::package::recipe::BuildContext;
use crate::package::{self, Bootstrap, Package, Role, Stage};
use crate::process;
use crate::target::Target;
use crate::util;
use crate::workspace;

use super::{artifact_target, build_env_ready, graph_deps};

pub(super) fn build_target(
    package: &dyn Package,
    target: Target,
    env: &[(&'static str, String)],
    own: bool,
    combined: Option<Arc<Mutex<File>>>,
) -> Result<()> {
    let sources = package.source_dir();
    if !sources.is_dir() {
        return Err(Error::message(format!(
            "ソースが見つかりません: {} (package fetch を実行してください)",
            sources.display()
        )));
    }
    let sources = fs::canonicalize(&sources).map_err(|source| Error::io(&sources, source))?;
    let source = sources.join(package.source().subdir);
    if !source.is_dir() {
        return Err(Error::message(format!(
            "ソースが見つかりません: {} (package fetch を実行してください)",
            source.display()
        )));
    }
    let architecture = target.architecture();
    let variant = target.variant();
    let build_dir = workspace::build_dir(package.name(), package.version(), target);
    let out_dir = package.artifact_dir(target);
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    }
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    }
    fs::create_dir_all(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    fs::create_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    let build_dir = fs::canonicalize(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    let out_dir = fs::canonicalize(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    let (runtime, build) = graph_deps(package, target);
    let mut command = Command::new("podman");
    command.args([
        "run",
        "--rm",
        "--cgroup-manager=cgroupfs",
        "--cgroupns=host",
        "--systemd=false",
    ]);
    if !allow_network(package, target, own) {
        command.args(["--network=none"]);
    }
    command
        .args(["-v"])
        .arg(format!("{}:/src:ro", sources.display()));
    for dependency in runtime.iter().chain(build.iter()) {
        let found = package::find(dependency.name)?;
        let mounted = artifact_target(found, target);
        let dep_dir = found.artifact_dir(mounted);
        if !dep_dir.is_dir() {
            return Err(Error::message(format!(
                "依存パッケージがありません: {} (package build を実行してください)",
                dep_dir.display()
            )));
        }
        let dep_dir = fs::canonicalize(&dep_dir).map_err(|source| Error::io(&dep_dir, source))?;
        command
            .args(["-v"])
            .arg(format!("{}:/dep/{}:ro", dep_dir.display(), dependency.name));
    }
    let mut mounted: Vec<&str> = runtime
        .iter()
        .chain(build.iter())
        .map(|dependency| dependency.name)
        .collect();
    if uses_mold(package, target)
        && let Some(dir) = mold_artifact_dir()
    {
        if !mounted.iter().any(|name| *name == "mold") {
            let dir = fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))?;
            command
                .args(["-v"])
                .arg(format!("{}:/dep/mold:ro", dir.display()));
            mounted.push("mold");
        }
    }
    for name in compiler_tool_names(package, target, own) {
        if mounted.iter().any(|mounted| *mounted == name.as_str()) {
            continue;
        }
        let found = package::find(&name)?;
        let mounted_target = if matches!(
            found.role(),
            Role::Toolchain {
                stage: Stage::GlibcHeaders | Stage::Glibc,
                ..
            }
        ) {
            artifact_target(found, target)
        } else {
            Target::BUILD_ENV
        };
        let dir = found.artifact_dir(mounted_target);
        if !workspace::artifact_ready(&dir) {
            return Err(Error::message(format!(
                "依存パッケージがありません: {} (package build を実行してください)",
                dir.display()
            )));
        }
        let dir = fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))?;
        command
            .args(["-v"])
            .arg(format!("{}:/dep/{name}:ro", dir.display()));
        mounted.push(found.name());
    }
    for name in extra_host_tool_names(package, target) {
        if mounted.iter().any(|mounted| *mounted == name) {
            continue;
        }
        let found = package::find(name)?;
        let mounted_target = if matches!(
            found.role(),
            Role::Toolchain {
                stage: Stage::GlibcHeaders | Stage::Glibc,
                ..
            }
        ) {
            artifact_target(found, target)
        } else {
            Target::BUILD_ENV
        };
        let dir = found.artifact_dir(mounted_target);
        if !workspace::artifact_ready(&dir) {
            continue;
        }
        let dir = fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))?;
        command
            .args(["-v"])
            .arg(format!("{}:/dep/{name}:ro", dir.display()));
        mounted.push(found.name());
    }
    // HOST perl の libperl.so は libcrypt.so.1 を要求する。ターゲット libxcrypt を
    // LD_LIBRARY_PATH に載せると別 ARCH になるので、HOST 専用パスへ載せる。
    if mounted.iter().any(|name| *name == "perl") || package.name() == "perl" {
        let crypt = package::find("libxcrypt")?;
        let dir = crypt.artifact_dir(Target::BUILD_ENV);
        if workspace::artifact_ready(&dir) {
            let dir = fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))?;
            command
                .args(["-v"])
                .arg(format!("{}:/dep/libxcrypt-host:ro", dir.display()));
        }
    }
    let ctx = build_context(package, target);
    command
        .args(["-v"])
        .arg(format!("{}:/build", build_dir.display()))
        .args(["-v"])
        .arg(format!("{}:/out", out_dir.display()))
        .args(["-e"])
        .arg(format!("SOURCE=/src/{}", package.source().subdir))
        .args(["-e", "BUILD=/build"])
        .args(["-e", "OUT=/out"])
        .args(["-e"])
        .arg(format!("ARCH={architecture}"))
        .args(["-e"])
        .arg(format!("VARIANT={variant}"))
        .args(["-e"])
        .arg(format!("TRIPLE={}", ctx.triple))
        .args(["-e"])
        .arg(format!("HOST_TRIPLE={}", ctx.host_triple))
        .args(["-e"])
        .arg(format!("PACKAGE_HOST={}", ctx.package_host))
        .args(["-e"])
        .arg(format!("MARCH={}", target.march()))
        .args(["-e"])
        .arg(format!("CROSS={}", ctx.cross_prefix));
    for (name, value) in compiler_cc(package, target, own) {
        command.args(["-e"]).arg(format!("{name}={value}"));
    }
    for (name, value) in env {
        command.args(["-e"]).arg(format!("{name}={value}"));
        if *name == "CFLAGS" {
            command.args(["-e"]).arg(format!("CXXFLAGS={value}"));
        }
    }
    let image = if own {
        own_image_ref()?
    } else {
        crate::package::env::BUILD_IMAGE.reference()
    };
    let script = wrap_script(package, !own, target);
    let log_path = workspace::log_path(package.name(), package.version(), target);
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let mut log = File::create(&log_path).map_err(|source| Error::io(&log_path, source))?;
    writeln!(
        log,
        "package: {} {}\ntarget: {}/{}\nimage: {image}\nstarted: {}\n",
        package.name(),
        package.version(),
        architecture,
        variant,
        util::timestamp()
    )
    .map_err(|source| Error::io(&log_path, source))?;
    let result = {
        let mut combined = match combined.as_ref() {
            Some(file) => Some(file.lock().unwrap_or_else(|error| error.into_inner())),
            None => None,
        };
        if let Some(combined) = combined.as_mut() {
            let _ = writeln!(
                combined,
                "package: {} {}\ntarget: {}/{}\nimage: {image}\nstarted: {}\n",
                package.name(),
                package.version(),
                architecture,
                variant,
                util::timestamp()
            );
        }
        process::tee_status(
            command
                .arg(&image)
                .args(["/usr/bin/bash", "-c"])
                .arg(script),
            "podman",
            &mut log,
            combined.as_mut().map(|file| &mut **file),
            &log_path,
        )
    };
    if result.is_err() {
        let _ = fs::remove_dir_all(&out_dir);
    }
    result
}

pub(super) fn build_context(package: &dyn Package, target: Target) -> BuildContext {
    BuildContext::of(package.gnu_triple(), target)
}

pub(super) fn host_gcc_ready() -> bool {
    package::find("x86_64-linux-gnu-gcc")
        .ok()
        .is_some_and(|gcc| workspace::artifact_ready(&gcc.artifact_dir(Target::BUILD_ENV)))
}

pub(super) fn compiler_cc(
    package: &dyn Package,
    target: Target,
    own: bool,
) -> Vec<(&'static str, String)> {
    let prefix = if is_seed(package, target) || matches!(package.role(), Role::LinuxHeaders) {
        String::new()
    } else if !matches!(target, Target::Amd64(_)) {
        target.cross_prefix()
    } else if own || host_gcc_ready() {
        target.cross_prefix()
    } else {
        String::new()
    };
    if prefix.is_empty() {
        vec![
            ("CC", "gcc".into()),
            ("CXX", "g++".into()),
            ("CPP", "gcc -E".into()),
            ("AR", "ar".into()),
            ("AS", "as".into()),
            ("RANLIB", "ranlib".into()),
            ("NM", "nm".into()),
            ("STRIP", "strip".into()),
            ("OBJCOPY", "objcopy".into()),
            ("OBJDUMP", "objdump".into()),
        ]
    } else {
        vec![
            ("CC", format!("{prefix}gcc")),
            ("CXX", format!("{prefix}g++")),
            ("CPP", format!("{prefix}gcc -E")),
            ("AR", format!("{prefix}ar")),
            ("AS", format!("{prefix}as")),
            ("RANLIB", format!("{prefix}ranlib")),
            ("NM", format!("{prefix}nm")),
            ("STRIP", format!("{prefix}strip")),
            ("OBJCOPY", format!("{prefix}objcopy")),
            ("OBJDUMP", format!("{prefix}objdump")),
        ]
    }
}

pub(super) fn is_seed(package: &dyn Package, target: Target) -> bool {
    target == Target::BUILD_ENV
        && package::find_bootstrap(package.name()).is_some_and(Bootstrap::seed)
}

pub(super) fn mold_artifact_dir() -> Option<std::path::PathBuf> {
    let mold = package::find("mold").ok()?;
    let dir = mold.artifact_dir(Target::BUILD_ENV);
    workspace::artifact_ready(&dir).then_some(dir)
}

pub(super) fn uses_mold(package: &dyn Package, target: Target) -> bool {
    package.name() != "mold"
        && package
            .flags(target)
            .ldflags()
            .contains(&Ldflag::FuseLdMold)
}

pub(super) fn linker_env(package: &dyn Package, target: Target) -> Vec<(&'static str, String)> {
    let mut env = package.flags(target).env();
    if uses_mold(package, target) && mold_artifact_dir().is_none() && build_env_ready() {
        strip_fuse_ld_mold(&mut env);
    }
    env
}

pub(super) fn strip_fuse_ld_mold(env: &mut Vec<(&'static str, String)>) {
    for (name, value) in env.iter_mut() {
        if *name != "LDFLAGS" {
            continue;
        }
        *value = value
            .split_whitespace()
            .filter(|token| *token != "-fuse-ld=mold")
            .collect::<Vec<_>>()
            .join(" ");
    }
    env.retain(|(name, value)| !(*name == "LDFLAGS" && value.is_empty()));
}

pub(super) fn compiler_tool_names(package: &dyn Package, target: Target, own: bool) -> Vec<String> {
    let cc = compiler_cc(package, target, own)
        .into_iter()
        .find(|(name, _)| *name == "CC")
        .map(|(_, value)| value)
        .unwrap_or_default();
    if cc == "gcc" {
        return Vec::new();
    }
    let Some(prefix) = cc.strip_suffix("gcc") else {
        return Vec::new();
    };
    [format!("{prefix}gcc"), format!("{prefix}binutils")]
        .into_iter()
        .collect()
}

pub(super) fn extra_host_tool_names(package: &dyn Package, target: Target) -> Vec<&'static str> {
    let mut names = package.host_tools(target);
    let role = package.role();
    if !matches!(role, Role::LinuxHeaders)
        && (!matches!(
            role,
            Role::Toolchain {
                triple: "x86_64-linux-gnu",
                ..
            }
        ) || !matches!(target, Target::Amd64(_)))
    {
        names.push("x86_64-linux-gnu-gcc");
        names.push("x86_64-linux-gnu-binutils");
    }
    let is_target_glibc = matches!(
        role,
        Role::Toolchain {
            triple,
            stage: Stage::Glibc
        } if triple == target.gnu_triple()
    );
    if target != Target::BUILD_ENV && !is_target_glibc {
        names.push(target.glibc_package());
    }
    if matches!(
        role,
        Role::Toolchain {
            stage: Stage::Gcc,
            ..
        }
    ) && target != Target::BUILD_ENV
        && !names.contains(&package.name())
    {
        names.push(package.name());
    }
    names
}

pub(super) fn seed_pacman(package: &dyn Package, target: Target) -> Vec<String> {
    let mut pkgs: Vec<String> = Vec::new();
    if let Some(bootstrap) = package::find_bootstrap(package.name()) {
        pkgs.extend(bootstrap.pacman().iter().map(|name| (*name).to_string()));
    } else {
        if package.name() == "zlib-ng" {
            pkgs.push("cmake".into());
        }
        if !matches!(target, Target::Amd64(_)) {
            pkgs.push(format!("{}gcc", target.cross_prefix()));
        }
    }
    if pkgs.iter().all(|name| name != "mold")
        && (is_seed(package, target)
            || (uses_mold(package, target) && mold_artifact_dir().is_none()))
    {
        pkgs.push("mold".into());
    }
    pkgs
}

pub(super) fn wrap_script(package: &dyn Package, seed: bool, target: Target) -> String {
    let script = package.rendered_script(target);
    let inject_triple_gcc_specs = match package.role() {
        Role::LinuxHeaders
        | Role::Toolchain {
            stage: Stage::Gcc | Stage::Glibc,
            ..
        } => "no",
        _ => "yes",
    };
    let mut prefix = format!("INJECT_TRIPLE_GCC_SPECS={inject_triple_gcc_specs}\n");
    prefix.push_str(
        r#"_cc_triple="${PACKAGE_HOST:-$TRIPLE}"
# Never export LD=*-gcc: libtool then reports "is GNU ld... no" and skips
# the real -shared link (gmp install: dangling libgmp.so.*).
for _dep_bin in /dep/*/usr/bin; do
  [ -d "$_dep_bin" ] || continue
  export PATH="$_dep_bin:$PATH"
done
for _dep_lib in /dep/*/usr/lib /dep/*/usr/lib64 /dep/*/usr/lib/perl5/*/*/CORE; do
  [ -d "$_dep_lib" ] || continue
  # Only HOST-run libraries. Target gmp on LD_LIBRARY_PATH makes cc1 load
  # big-endian libgmp.so ("ELF file data encoding not little-endian").
  case "$_dep_lib" in
    */python/*|*/perl/*|*/libxcrypt-host/*|*/cmake/*|*/meson/*|*/ninja/*|*/qemu/*|*/pkgconf/*|*/"${HOST_TRIPLE}-gcc"/*) ;;
    *) continue ;;
  esac
  export LD_LIBRARY_PATH="$_dep_lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
done
_pcs=
for _pc in /dep/*/usr/lib/pkgconfig /dep/*/usr/lib64/pkgconfig /dep/*/usr/share/pkgconfig; do
  [ -d "$_pc" ] || continue
  _pcs="${_pcs:+$_pcs:}$_pc"
done
if [ -n "$_pcs" ]; then
  export PKG_CONFIG_PATH="$_pcs${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
fi
for _inc in /dep/*/usr/include; do
  [ -d "$_inc" ] || continue
  case "$_inc" in
    */linux-headers/*|*/*-gcc/*|*/*-glibc/*|*/*-binutils/*|*/python/*|*/perl/*|*/meson/*|*/cmake/*|*/qemu/*|*/glib/*|*/pcre2/*|*/libxcrypt-host/*) continue ;;
  esac
  export CPPFLAGS="${CPPFLAGS:-} -I$_inc"
  export CFLAGS="${CFLAGS:-} -I$_inc"
  export CXXFLAGS="${CXXFLAGS:-} -I$_inc"
done
for _lib in /dep/*/usr/lib /dep/*/usr/lib64; do
  [ -d "$_lib" ] || continue
  case "$_lib" in
    */linux-headers/*|*/*-gcc/*|*/*-binutils/*|*/qemu/*|*/glib/*|*/pcre2/*|*/perl/*|*/libxcrypt-host/*) continue ;;
  esac
  export LDFLAGS="${LDFLAGS:-} -L$_lib -Wl,-rpath-link=$_lib"
  export LIBRARY_PATH="$_lib${LIBRARY_PATH:+:$LIBRARY_PATH}"
done
for _t in "$_cc_triple" "$TRIPLE"; do
  for _slib in /dep/${_t}-glibc/usr/${_t}/lib /dep/${_t}-glibc/usr/${_t}/lib64 /dep/${_t}-glibc/usr/lib /dep/${_t}-glibc/usr/lib64; do
    [ -d "$_slib" ] || continue
    export LDFLAGS="${LDFLAGS:-} -L$_slib -Wl,-rpath-link=$_slib"
  done
done
for _slib in /dep/${_cc_triple}-glibc/usr/${_cc_triple}/lib /dep/${_cc_triple}-glibc/usr/${_cc_triple}/lib64; do
  if [ -e "$_slib/crti.o" ]; then
    export CFLAGS="${CFLAGS:-} -B$_slib"
    export CXXFLAGS="${CXXFLAGS:-} -B$_slib"
    export LDFLAGS="${LDFLAGS:-} -B$_slib"
    break
  fi
done
_la_over=/tmp/la-overlay
mkdir -p "$_la_over"
for _la in /dep/*/usr/lib/*.la /dep/*/usr/lib64/*.la; do
  [ -f "$_la" ] || continue
  case "$_la" in
    */linux-headers/*|*/*-gcc/*|*/*-glibc/*|*/*-binutils/*|*/python/*|*/perl/*|*/meson/*|*/cmake/*) continue ;;
  esac
  _dir=$(dirname "$_la")
  _base=$(basename "$_la" .la)
  sed -e "s|^libdir='[^']*'|libdir='$_la_over'|" \
      -e "s|^installed=yes|installed=no|" \
      -e "s|-L/usr/lib|-L$_la_over|g" \
    "$_la" > "$_la_over/$(basename "$_la")"
  for _so in "$_dir"/$_base.so "$_dir"/$_base.so.* "$_dir"/$_base.a; do
    [ -e "$_so" ] || continue
    ln -sfn "$_so" "$_la_over/$(basename "$_so")"
  done
done
for _la in "$_la_over"/*.la; do
  [ -f "$_la" ] || continue
  sed -i -e 's|/dep/gmp/usr/lib/libgmp.la|/tmp/la-overlay/libgmp.la|g' \
         -e 's|/dep/mpfr/usr/lib/libmpfr.la|/tmp/la-overlay/libmpfr.la|g' \
    "$_la"
done
if [ -d /dep/gmp/usr ]; then
  mkdir -p /tmp/gmp-prefix/include /tmp/gmp-prefix/lib
  for _h in /dep/gmp/usr/include/*; do
    [ -e "$_h" ] || continue
    ln -sfn "$_h" /tmp/gmp-prefix/include/
  done
  for _g in /tmp/la-overlay/libgmp.* /tmp/la-overlay/libgmp.la; do
    [ -e "$_g" ] || continue
    ln -sfn "$_g" /tmp/gmp-prefix/lib/
  done
fi
if [ -d /dep/mpfr/usr ]; then
  mkdir -p /tmp/mpfr-prefix/include /tmp/mpfr-prefix/lib
  for _h in /dep/mpfr/usr/include/*; do
    [ -e "$_h" ] || continue
    ln -sfn "$_h" /tmp/mpfr-prefix/include/
  done
  for _g in /tmp/la-overlay/libmpfr.* /tmp/la-overlay/libmpfr.la; do
    [ -e "$_g" ] || continue
    ln -sfn "$_g" /tmp/mpfr-prefix/lib/
  done
fi
if [ -n "$(ls -A "$_la_over" 2>/dev/null)" ]; then
  export LDFLAGS="-L$_la_over ${LDFLAGS:-}"
  export LIBRARY_PATH="$_la_over${LIBRARY_PATH:+:$LIBRARY_PATH}"
fi
_pc_over=/tmp/pc-overlay
mkdir -p "$_pc_over"
for _pc in /dep/*/usr/lib/pkgconfig/*.pc /dep/*/usr/lib64/pkgconfig/*.pc /dep/*/usr/share/pkgconfig/*.pc; do
  [ -f "$_pc" ] || continue
  _pref=$(cd "$(dirname "$_pc")/../.." && pwd)
  sed -e "s|^prefix=/usr|prefix=$_pref|" \
      -e "s|^exec_prefix=/usr|exec_prefix=$_pref|" \
      -e "s|^libdir=/usr/lib64|libdir=$_pref/lib64|" \
      -e "s|^sharedlibdir=/usr/lib64|sharedlibdir=$_pref/lib64|" \
      -e "s|^libdir=/usr/lib|libdir=$_pref/lib|" \
      -e "s|^sharedlibdir=/usr/lib|sharedlibdir=$_pref/lib|" \
      -e "s|^includedir=/usr/include|includedir=$_pref/include|" \
    "$_pc" > "$_pc_over/$(basename "$_pc")"
done
if [ -n "$(ls -A "$_pc_over" 2>/dev/null)" ]; then
  export PKG_CONFIG_PATH="$_pc_over${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
fi
_pc_wrap=/tmp/pkg-config-bin
mkdir -p "$_pc_wrap"
if [ -x /dep/pkgconf/usr/bin/pkgconf ]; then
  printf '#!/bin/sh\nexec /dep/pkgconf/usr/bin/pkgconf "$@"\n' > "$_pc_wrap/pkg-config"
elif command -v pkgconf >/dev/null 2>&1; then
  printf '#!/bin/sh\nexec pkgconf "$@"\n' > "$_pc_wrap/pkg-config"
elif command -v pkg-config >/dev/null 2>&1; then
  printf '#!/bin/sh\nexec pkg-config "$@"\n' > "$_pc_wrap/pkg-config"
fi
if [ -f "$_pc_wrap/pkg-config" ]; then
  chmod +x "$_pc_wrap/pkg-config"
  export PATH="$_pc_wrap:$PATH"
fi
if [ -f /dep/meson/usr/bin/meson ]; then
  _meson_wrap=/tmp/meson-bin
  mkdir -p "$_meson_wrap"
  _py=/usr/bin/python3
  if [ -x /dep/python/usr/bin/python3 ]; then
    _py=/dep/python/usr/bin/python3
  fi
  if [ ! -e /usr/sbin/python3 ]; then
    mkdir -p /usr/sbin
    ln -sfn "$_py" /usr/sbin/python3
  fi
  if [ ! -e /usr/bin/python3 ]; then
    ln -sfn "$_py" /usr/bin/python3
  fi
  sed "1s|^#!.*|#!$_py|" /dep/meson/usr/bin/meson > "$_meson_wrap/meson"
  chmod +x "$_meson_wrap/meson"
  export PATH="$_meson_wrap:$PATH"
fi
for _py in /dep/*/usr/lib/python*/site-packages /dep/*/usr/lib64/python*/site-packages; do
  [ -d "$_py" ] || continue
  export PYTHONPATH="$_py${PYTHONPATH:+:$PYTHONPATH}"
done
if [ -d /dep/perl/usr/lib/perl5 ]; then
  _perl_arch=$(ls -d /dep/perl/usr/lib/perl5/[0-9]*/[a-z0-9_]*-linux* 2>/dev/null | head -n1 || true)
  _perl_ver=$(ls -d /dep/perl/usr/lib/perl5/[0-9]* 2>/dev/null | head -n1 || true)
  if [ -n "$_perl_arch" ] && [ -n "$_perl_ver" ]; then
    export PERL5LIB="$_perl_arch:$_perl_ver${PERL5LIB:+:$PERL5LIB}"
  fi
fi
if [ -d /dep/${_cc_triple}-binutils/usr/bin ]; then
  export PATH="/dep/${_cc_triple}-binutils/usr/bin:$PATH"
  if [ -x "/dep/${_cc_triple}-binutils/usr/bin/${_cc_triple}-strip" ]; then
    _strip_wrap=/tmp/host-strip
    mkdir -p "$_strip_wrap"
    ln -sfn "/dep/${_cc_triple}-binutils/usr/bin/${_cc_triple}-strip" "$_strip_wrap/strip"
    export PATH="$_strip_wrap:$PATH"
  fi
fi
if [ -d /dep/${TRIPLE}-binutils/usr/bin ]; then
  export PATH="/dep/${TRIPLE}-binutils/usr/bin:$PATH"
fi
if [ -d /dep/${_cc_triple}-gcc/usr/bin ]; then
  export PATH="/dep/${_cc_triple}-gcc/usr/bin:$PATH"
  if [ "${INJECT_TRIPLE_GCC_SPECS:-yes}" = yes ]; then
  for _gcc_lib in /dep/${_cc_triple}-gcc/usr/lib/gcc/${_cc_triple}/*; do
    [ -d "$_gcc_lib" ] || continue
    _gcc_bin="/dep/${_cc_triple}-gcc/usr/bin/${_cc_triple}-gcc"
    _gxx_bin="/dep/${_cc_triple}-gcc/usr/bin/${_cc_triple}-g++"
    _gcc_wrap=/tmp/gcc-b
    mkdir -p "$_gcc_wrap"
    if [ -x "$_gcc_bin" ]; then
      ln -sfn "$_gcc_bin" "$_gcc_wrap/${_cc_triple}-gcc"
    fi
    if [ -x "$_gxx_bin" ]; then
      ln -sfn "$_gxx_bin" "$_gcc_wrap/${_cc_triple}-g++"
    fi
    export PATH="$_gcc_wrap:$PATH"
    export CFLAGS="${CFLAGS:-} -B$_gcc_lib"
    export CXXFLAGS="${CXXFLAGS:-} -B$_gcc_lib"
    export LDFLAGS="${LDFLAGS:-} -B$_gcc_lib -L$_gcc_lib"
    export LIBRARY_PATH="$_gcc_lib${LIBRARY_PATH:+:$LIBRARY_PATH}"
    export COMPILER_PATH="$_gcc_lib${COMPILER_PATH:+:$COMPILER_PATH}"
    break
  done
  for _lto_plugin in /dep/${_cc_triple}-gcc/usr/libexec/gcc/${_cc_triple}/*/liblto_plugin.so; do
    [ -f "$_lto_plugin" ] || continue
    _lto_wrap=/tmp/lto-binutils
    mkdir -p "$_lto_wrap"
    for _tool in ar ranlib nm; do
      _real="/dep/${_cc_triple}-binutils/usr/bin/${_cc_triple}-$_tool"
      [ -x "$_real" ] || continue
      printf '#!/bin/sh\nexec %s --plugin %s "$@"\n' "$_real" "$_lto_plugin" > "$_lto_wrap/${_cc_triple}-$_tool"
      chmod +x "$_lto_wrap/${_cc_triple}-$_tool"
    done
    export PATH="$_lto_wrap:$PATH"
    break
  done
  fi
fi
if [ "$TRIPLE" != "$_cc_triple" ] && [ -d /dep/${TRIPLE}-gcc/usr/bin ]; then
  export PATH="/dep/${TRIPLE}-gcc/usr/bin:$PATH"
fi
if [ -d /dep/${_cc_triple}-binutils/usr/bin ]; then
  _plain=/tmp/unprefixed-binutils
  mkdir -p "$_plain"
  for _tool in ar ranlib nm strip objcopy objdump; do
    _wrap="/tmp/lto-binutils/${_cc_triple}-$_tool"
    _real="/dep/${_cc_triple}-binutils/usr/bin/${_cc_triple}-$_tool"
    if [ -x "$_wrap" ]; then
      ln -sfn "$_wrap" "$_plain/$_tool"
      ln -sfn "$_wrap" "$_plain/gcc-$_tool"
    elif [ -x "$_real" ]; then
      ln -sfn "$_real" "$_plain/$_tool"
      ln -sfn "$_real" "$_plain/gcc-$_tool"
    fi
  done
  export PATH="$_plain:$PATH"
fi
if [ -x "/dep/${HOST_TRIPLE}-gcc/usr/bin/${HOST_TRIPLE}-gcc" ]; then
  _hcc=/tmp/host-cc
  mkdir -p "$_hcc"
  ln -sfn "/dep/${HOST_TRIPLE}-gcc/usr/bin/${HOST_TRIPLE}-gcc" "$_hcc/gcc"
  ln -sfn "/dep/${HOST_TRIPLE}-gcc/usr/bin/${HOST_TRIPLE}-gcc" "$_hcc/cc"
  if [ -x "/dep/${HOST_TRIPLE}-gcc/usr/bin/${HOST_TRIPLE}-g++" ]; then
    ln -sfn "/dep/${HOST_TRIPLE}-gcc/usr/bin/${HOST_TRIPLE}-g++" "$_hcc/g++"
    ln -sfn "/dep/${HOST_TRIPLE}-gcc/usr/bin/${HOST_TRIPLE}-g++" "$_hcc/c++"
  fi
  export PATH="$_hcc:$PATH"
fi
if [ -x /dep/mold/usr/bin/mold ]; then
  export PATH="/dep/mold/usr/bin:$PATH"
fi
"#,
    );
    let mold_wrap = if uses_mold(package, target) {
        r#"
_mold_wrap=/tmp/mold-ld-cc
mkdir -p "$_mold_wrap"
if command -v mold >/dev/null 2>&1; then
  for _name in gcc cc g++ c++ "${_cc_triple}-gcc" "${_cc_triple}-g++" "${TRIPLE}-gcc" "${TRIPLE}-g++" "${HOST_TRIPLE}-gcc" "${HOST_TRIPLE}-g++"; do
    _src=$(command -v "$_name" || true)
    [ -n "$_src" ] && [ -x "$_src" ] || continue
    case "$_src" in
      "$_mold_wrap"/*) continue ;;
    esac
    printf '#!/bin/sh\nexec %s -fuse-ld=mold "$@"\n' "$_src" > "$_mold_wrap/$_name"
    chmod +x "$_mold_wrap/$_name"
  done
  export PATH="$_mold_wrap:$PATH"
fi
"#
    } else {
        ""
    };
    if !seed {
        prefix.push_str(mold_wrap);
        prefix.push_str(&script);
        return prefix;
    }
    let pkgs = seed_pacman(package, target);
    if pkgs.is_empty() {
        prefix.push_str(mold_wrap);
        prefix.push_str(&script);
        return prefix;
    }
    prefix.push_str(&format!("pacman -Sy --noconfirm {} && ", pkgs.join(" ")));
    prefix.push_str(mold_wrap);
    prefix.push_str(&script);
    prefix
}

pub(super) fn own_image_ref() -> Result<String> {
    let current =
        workspace::compose_current_link(workspace::BUILD_ENV_NAME, workspace::BUILD_ENV_ARCH);
    let path = fs::canonicalize(&current).map_err(|source| Error::io(&current, source))?;
    Ok(format!("oci:{}", path.display()))
}

pub(super) fn allow_network(package: &dyn Package, target: Target, own: bool) -> bool {
    if own {
        return false;
    }
    if !matches!(target, Target::Amd64(_)) {
        return true;
    }
    !seed_pacman(package, target).is_empty()
}
