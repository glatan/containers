use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::flags::Ldflag;
use crate::package::recipe::{BuildContext, Recipe};
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
            "ソースが見つかりません: {} (fetch を実行してください)",
            sources.display()
        )));
    }
    let sources = fs::canonicalize(&sources).map_err(|source| Error::io(&sources, source))?;
    let source = sources.join(package.source().subdir);
    if !source.is_dir() {
        return Err(Error::message(format!(
            "ソースが見つかりません: {} (fetch を実行してください)",
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
    let mut mounts: Vec<Mount> = Vec::new();
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
                "依存パッケージがありません: {} (build を実行してください)",
                dep_dir.display()
            )));
        }
        let dep_dir = fs::canonicalize(&dep_dir).map_err(|source| Error::io(&dep_dir, source))?;
        command
            .args(["-v"])
            .arg(format!("{}:/dep/{}:ro", dep_dir.display(), dependency.name));
        mounts.push(Mount {
            name: dependency.name.to_owned(),
            dir: dep_dir,
        });
    }
    let mut mounted: Vec<&str> = runtime
        .iter()
        .chain(build.iter())
        .map(|dependency| dependency.name)
        .collect();
    if uses_mold(package, target)
        && let Some(dir) = mold_artifact_dir()
        && !mounted.contains(&"mold")
    {
        let dir = fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))?;
        command
            .args(["-v"])
            .arg(format!("{}:/dep/mold:ro", dir.display()));
        mounted.push("mold");
        mounts.push(Mount {
            name: "mold".to_owned(),
            dir,
        });
    }
    for name in compiler_tool_names(package, target, own) {
        if mounted.contains(&name.as_str()) {
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
                "依存パッケージがありません: {} (build を実行してください)",
                dir.display()
            )));
        }
        let dir = fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))?;
        command
            .args(["-v"])
            .arg(format!("{}:/dep/{name}:ro", dir.display()));
        mounted.push(found.name());
        mounts.push(Mount {
            name: found.name().to_owned(),
            dir,
        });
    }
    for name in extra_host_tool_names(package, target) {
        if mounted.contains(&name) {
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
        mounts.push(Mount {
            name: found.name().to_owned(),
            dir,
        });
    }
    // HOST perl の libperl.so は libcrypt.so.1 を要求する。ターゲット libxcrypt を
    // LD_LIBRARY_PATH に載せると別 ARCH になるので、HOST 専用パスへ載せる。
    if mounted.contains(&"perl") || package.name() == "perl" {
        let crypt = package::find("libxcrypt")?;
        let dir = crypt.artifact_dir(Target::BUILD_ENV);
        if workspace::artifact_ready(&dir) {
            let dir = fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))?;
            command
                .args(["-v"])
                .arg(format!("{}:/dep/libxcrypt-host:ro", dir.display()));
            mounts.push(Mount {
                name: "libxcrypt-host".to_owned(),
                dir,
            });
        }
    }
    let ctx = build_context(package, target);
    // .la/.pc オーバーレイと {gmp,mpfr}-prefix をホスト側で生成し、
    // プリアンブルが作っていた /tmp/* パスへそのままマウントする。
    let gen_dir = build_dir.join(".gen");
    let prep = stage_generated(package, &mounts, &gen_dir, target)?;
    for (sub, dst) in [
        ("la-overlay", "/tmp/la-overlay"),
        ("pc-overlay", "/tmp/pc-overlay"),
        ("gmp-prefix", "/tmp/gmp-prefix"),
        ("mpfr-prefix", "/tmp/mpfr-prefix"),
        ("pkg-config-bin", "/tmp/pkg-config-bin"),
        ("meson-bin", "/tmp/meson-bin"),
        ("host-strip", "/tmp/host-strip"),
        ("gcc-b", "/tmp/gcc-b"),
        ("lto-binutils", "/tmp/lto-binutils"),
        ("unprefixed-binutils", "/tmp/unprefixed-binutils"),
        ("host-cc", "/tmp/host-cc"),
    ] {
        let dir = gen_dir.join(sub);
        if dir.is_dir() {
            command
                .args(["-v"])
                .arg(format!("{}:{}:ro", dir.display(), dst));
        }
    }
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
    let script = wrap_script(package, !own, target, &mounts, &prep);
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
        let mut combined = combined
            .as_ref()
            .map(|file| file.lock().unwrap_or_else(|error| error.into_inner()));
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
            combined.as_deref_mut(),
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
    } else if !matches!(target, Target::Amd64(_)) || own || host_gcc_ready() {
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

/// パッケージの CFLAGS/LDFLAGS を環境へ出す。`Env::Autotools` の
/// configure probe 向け剥がしは recipe 側の `sanitize_flags` が担うため、
/// ここでは常にフルフラグを渡す（make/install が var override で受ける）。
pub(super) fn linker_env(package: &dyn Package, target: Target) -> Vec<(&'static str, String)> {
    let flags = package.flags(target);
    let mut env = flags.env();
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

/// `/dep` にマウントされる依存成果物。ホスト側が把握しているため、
/// wrap_script は `/dep/*/` glob ではなくこの一覧から環境を生成する。
pub(super) struct Mount {
    /// コンテナ側名（`/dep/{name}`）。
    pub name: String,
    /// ホスト側の成果物ディレクトリ。
    pub dir: std::path::PathBuf,
}

/// `*` のみを解釈する最小限の glob コンポーネントマッチ。
fn glob_match(pattern: &str, name: &str) -> bool {
    if !pattern.contains('*') {
        return pattern == name;
    }
    let parts: Vec<&str> = pattern.split('*').collect();
    let mut rest = name;
    let (mut start, mut end) = (0, parts.len());
    if !pattern.starts_with('*') {
        if !rest.starts_with(parts[0]) {
            return false;
        }
        rest = &rest[parts[0].len()..];
        start = 1;
    }
    if !pattern.ends_with('*') {
        let last = parts[parts.len() - 1];
        if !rest.ends_with(last) {
            return false;
        }
        rest = &rest[..rest.len() - last.len()];
        end -= 1;
    }
    for part in &parts[start..end] {
        if part.is_empty() {
            continue;
        }
        match rest.find(part) {
            Some(pos) => rest = &rest[pos + part.len()..],
            None => return false,
        }
    }
    true
}

/// `dir` 配下を `/` 区切りの glob パターンで列挙する（`*` のみ解釈）。
/// bash の glob と同じくバイト順ソート。
fn glob_in(dir: &std::path::Path, pattern: &str) -> Vec<std::path::PathBuf> {
    let mut current = vec![dir.to_path_buf()];
    for component in pattern.split('/') {
        let mut next = Vec::new();
        for dir in &current {
            if component.contains('*') {
                let mut matched: Vec<std::path::PathBuf> = fs::read_dir(dir)
                    .map(|entries| {
                        entries
                            .flatten()
                            .map(|entry| entry.path())
                            .filter(|path| {
                                path.file_name().is_some_and(|name| {
                                    let name = name.to_string_lossy();
                                    // bash glob は dotfile を '*' にマッチさせない
                                    !name.starts_with('.') && glob_match(component, &name)
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                matched.sort_by(|a, b| {
                    a.file_name()
                        .unwrap_or_default()
                        .as_encoded_bytes()
                        .cmp(b.file_name().unwrap_or_default().as_encoded_bytes())
                });
                next.extend(matched);
            } else {
                let path = dir.join(component);
                if path.symlink_metadata().is_ok() {
                    next.push(path);
                }
            }
        }
        current = next;
    }
    current
}

/// マウント済み dep のパターンを展開し、（マウント名、`/dep` 側パス、
/// ホスト側パス）を bash glob と同じバイト順で返す。
fn dep_glob(mounts: &[Mount], pattern: &str) -> Vec<(String, String, std::path::PathBuf)> {
    let mut out = Vec::new();
    for mount in mounts {
        for path in glob_in(&mount.dir, pattern) {
            let Ok(rel) = path.strip_prefix(&mount.dir) else {
                continue;
            };
            out.push((
                mount.name.clone(),
                format!("/dep/{}/{}", mount.name, rel.display()),
                path,
            ));
        }
    }
    out.sort_by(|a, b| a.1.cmp(&b.1));
    out
}

/// マウント済み dep のパターンを展開し、`/dep` 側パスを bash glob と
/// 同じバイト順で返す。
pub(super) fn dep_paths(mounts: &[Mount], pattern: &str) -> Vec<String> {
    dep_glob(mounts, pattern)
        .into_iter()
        .map(|(_, container, _)| container)
        .collect()
}

/// ホスト側で生成したオーバーレイ・ラッパの内容フラグ。空のディレクトリは
/// マウントしない（元シェルの `[ -d ... ]` / `ls -A` 条件に相当）。
#[derive(Default)]
pub(super) struct Prep {
    /// `/tmp/la-overlay` に .la 複写/シンボリックリンクがある
    pub la: bool,
    /// `/tmp/pc-overlay` に .pc 複写がある
    pub pc: bool,
    /// `/tmp/pkg-config-bin/pkg-config`（/dep/pkgconf 版）を生成済み
    pub pkg_config: bool,
    /// `/tmp/meson-bin/meson` を生成済み（値は shebang の python パス）
    pub meson: Option<String>,
    /// `/dep/{cc}-binutils/usr/bin` が存在する
    pub cc_binutils: bool,
    /// `/tmp/host-strip/strip` を生成済み
    pub host_strip: bool,
    /// `/dep/{triple}-binutils/usr/bin` が存在する
    pub triple_binutils: bool,
    /// `/dep/{cc}-gcc/usr/bin` が存在する
    pub cc_gcc: bool,
    /// -B/-L 対象の gcc libdir（コンテナ側パス）。Some なら `/tmp/gcc-b` 生成済み
    pub gcc_lib: Option<String>,
    /// `/tmp/lto-binutils` を生成済み（liblto_plugin 検出）
    pub lto_binutils: bool,
    /// lto-binutils に生成したツール名（unprefixed の参照先決定用）
    lto_tools: Vec<String>,
    /// `/dep/{triple}-gcc/usr/bin` が存在する（triple != cc のときのみ）
    pub triple_gcc: bool,
    /// `/tmp/unprefixed-binutils` を生成済み
    pub unprefixed: bool,
    /// `/tmp/host-cc` を生成済み
    pub host_cc: bool,
    /// `/dep/mold/usr/bin/mold` が存在する
    pub mold: bool,
}

/// `-x` テスト相当（シンボリックリンクを辿る）。
fn is_exec(path: &std::path::Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// 実行ビット付きでファイルを書き出す（`printf > file && chmod +x` 相当）。
fn write_exec(path: &std::path::Path, content: &str) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, content).map_err(|source| Error::io(path, source))?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
        .map_err(|source| Error::io(path, source))
}

/// `ln -sfn` 相当: 既存リンクを置き換えてシンボリックリンクを作る。
fn force_symlink(target: &str, link: &std::path::Path) -> std::io::Result<()> {
    if link.symlink_metadata().is_ok() {
        fs::remove_file(link)?;
    }
    std::os::unix::fs::symlink(target, link)
}

/// .la の sed 書き換えを Rust で再現する。
/// `s|^libdir='[^']*'|libdir='/tmp/la-overlay'|`、
/// `s|^installed=yes|installed=no|`、`s|-L/usr/lib|-L/tmp/la-overlay|g`、
/// および gmp/mpfr の参照先付け替え（元は第二パスの `sed -i`）。
fn rewrite_la(content: &str) -> String {
    let mut out = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        let (body, eol) = match line.strip_suffix('\n') {
            Some(body) => (body, "\n"),
            None => (line, ""),
        };
        let mut body = body.to_owned();
        if let Some(rest) = body.strip_prefix("libdir='")
            && let Some(pos) = rest.find('\'')
        {
            body = format!("libdir='/tmp/la-overlay'{}", &rest[pos + 1..]);
        }
        if let Some(rest) = body.strip_prefix("installed=yes") {
            body = format!("installed=no{rest}");
        }
        body = body.replace("-L/usr/lib", "-L/tmp/la-overlay");
        out.push_str(&body);
        out.push_str(eol);
    }
    out
}

/// 依存パッケージオーバーレイをホスト側で生成し、gen 配下に配置する。
/// 元のプリアンブルがコンテナ内で行っていた sed 書き換えと ln -sfn を
/// ここで行い、`/tmp/la-overlay` 等へマウントする。
pub(super) fn stage_generated(
    package: &dyn Package,
    mounts: &[Mount],
    gen_dir: &std::path::Path,
    target: Target,
) -> Result<Prep> {
    let mut prep = Prep::default();
    let la_over = gen_dir.join("la-overlay");
    // .la 複写（*/linux-headers/* 等の case 除外を Rust 側で再現）
    let la_skip = |name: &str| {
        name == "linux-headers"
            || name == "python"
            || name == "perl"
            || name == "meson"
            || name == "cmake"
            || name.ends_with("-gcc")
            || name.ends_with("-glibc")
            || name.ends_with("-binutils")
    };
    for (name, container, host) in dep_glob(mounts, "usr/lib/*.la")
        .into_iter()
        .chain(dep_glob(mounts, "usr/lib64/*.la"))
    {
        if la_skip(&name) {
            continue;
        }
        fs::create_dir_all(&la_over).map_err(|source| Error::io(&la_over, source))?;
        let file_name = host.file_name().unwrap_or_default();
        let content = fs::read_to_string(&host).map_err(|source| Error::io(&host, source))?;
        let rewritten = rewrite_la(&content)
            .replace("/dep/gmp/usr/lib/libgmp.la", "/tmp/la-overlay/libgmp.la")
            .replace("/dep/mpfr/usr/lib/libmpfr.la", "/tmp/la-overlay/libmpfr.la");
        fs::write(la_over.join(file_name), rewritten)
            .map_err(|source| Error::io(&la_over, source))?;
        prep.la = true;
        // 隣接する $_base.so / $_base.so.* / $_base.a へのリンク
        let stem = file_name
            .to_string_lossy()
            .strip_suffix(".la")
            .unwrap_or_default()
            .to_owned();
        let dir = host.parent().unwrap_or(&host);
        let container_dir = container.rsplit_once('/').map(|(d, _)| d).unwrap_or("/");
        for pattern in [
            format!("{stem}.so"),
            format!("{stem}.so.*"),
            format!("{stem}.a"),
        ] {
            for so in glob_in(dir, &pattern) {
                let Some(so_name) = so.file_name() else {
                    continue;
                };
                let target = format!("{container_dir}/{}", so_name.to_string_lossy());
                force_symlink(&target, &la_over.join(so_name))
                    .map_err(|source| Error::io(&la_over, source))?;
            }
        }
    }
    // {dep}-prefix: include/* へのリンク + la-overlay 内の lib{dep}.* へのリンク
    for dep in ["gmp", "mpfr"] {
        let Some(mount) = mounts.iter().find(|m| m.name == dep) else {
            continue;
        };
        if !mount.dir.join("usr").is_dir() {
            continue;
        }
        let prefix = gen_dir.join(format!("{dep}-prefix"));
        let include = prefix.join("include");
        let lib = prefix.join("lib");
        fs::create_dir_all(&include).map_err(|source| Error::io(&include, source))?;
        fs::create_dir_all(&lib).map_err(|source| Error::io(&lib, source))?;
        for h in glob_in(&mount.dir, "usr/include/*") {
            let Some(h_name) = h.file_name() else {
                continue;
            };
            let target = format!("/dep/{dep}/usr/include/{}", h_name.to_string_lossy());
            force_symlink(&target, &include.join(h_name))
                .map_err(|source| Error::io(&include, source))?;
        }
        for g in glob_in(&la_over, &format!("lib{dep}.*")) {
            let Some(g_name) = g.file_name() else {
                continue;
            };
            let target = format!("/tmp/la-overlay/{}", g_name.to_string_lossy());
            force_symlink(&target, &lib.join(g_name)).map_err(|source| Error::io(&lib, source))?;
        }
    }
    // .pc オーバーレイ（フィルタなし）
    let pc_over = gen_dir.join("pc-overlay");
    for (name, _, host) in dep_glob(mounts, "usr/lib/pkgconfig/*.pc")
        .into_iter()
        .chain(dep_glob(mounts, "usr/lib64/pkgconfig/*.pc"))
        .chain(dep_glob(mounts, "usr/share/pkgconfig/*.pc"))
    {
        fs::create_dir_all(&pc_over).map_err(|source| Error::io(&pc_over, source))?;
        let content = fs::read_to_string(&host).map_err(|source| Error::io(&host, source))?;
        fs::write(
            pc_over.join(host.file_name().unwrap_or_default()),
            rewrite_pc(&content, &format!("/dep/{name}/usr")),
        )
        .map_err(|source| Error::io(&pc_over, source))?;
        prep.pc = true;
    }

    // --- ラッパ類 ---
    let ctx = build_context(package, target);
    let cc = &ctx.package_host;
    let cc_binutils = format!("{cc}-binutils");
    let cc_gcc = format!("{cc}-gcc");
    let inject = !matches!(
        package.role(),
        Role::LinuxHeaders
            | Role::Toolchain {
                stage: Stage::Gcc | Stage::Glibc,
                ..
            }
    );
    let mount = |name: &str| mounts.iter().find(|m| m.name == name);

    // pkg-config ラッパ。/dep/pkgconf があればホスト側で生成。
    // 無い場合の command -v フォールバックはイメージ依存なのでシェルに残す。
    if let Some(pkgconf) = mount("pkgconf")
        && is_exec(&pkgconf.dir.join("usr/bin/pkgconf"))
    {
        let dir = gen_dir.join("pkg-config-bin");
        fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
        write_exec(
            &dir.join("pkg-config"),
            "#!/bin/sh\nexec /dep/pkgconf/usr/bin/pkgconf \"$@\"\n",
        )?;
        prep.pkg_config = true;
    }

    // meson ラッパ: shebang を利用可能な python3 へ書き換えて複写。
    if let Some(meson) = mount("meson") {
        let meson_bin = meson.dir.join("usr/bin/meson");
        if meson_bin.is_file() {
            let py = match mount("python") {
                Some(p) if is_exec(&p.dir.join("usr/bin/python3")) => "/dep/python/usr/bin/python3",
                _ => "/usr/bin/python3",
            };
            let content =
                fs::read_to_string(&meson_bin).map_err(|source| Error::io(&meson_bin, source))?;
            let rewritten = match content.strip_prefix("#!").and_then(|_| content.find('\n')) {
                Some(nl) => format!("#!{py}{}", &content[nl..]),
                None if content.starts_with("#!") => format!("#!{py}\n"),
                None => content,
            };
            let dir = gen_dir.join("meson-bin");
            fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
            write_exec(&dir.join("meson"), &rewritten)?;
            prep.meson = Some(py.to_owned());
        }
    }

    if let Some(bu) = mount(&cc_binutils)
        && bu.dir.join("usr/bin").is_dir()
    {
        prep.cc_binutils = true;
        let strip = format!("usr/bin/{cc}-strip");
        if is_exec(&bu.dir.join(&strip)) {
            let dir = gen_dir.join("host-strip");
            fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
            force_symlink(&format!("/dep/{cc_binutils}/{strip}"), &dir.join("strip"))
                .map_err(|source| Error::io(&dir, source))?;
            prep.host_strip = true;
        }
    }
    let triple_binutils = format!("{}-binutils", ctx.triple);
    if let Some(bu) = mount(&triple_binutils)
        && bu.dir.join("usr/bin").is_dir()
    {
        prep.triple_binutils = true;
    }

    if let Some(gcc) = mount(&cc_gcc)
        && gcc.dir.join("usr/bin").is_dir()
    {
        prep.cc_gcc = true;
        if inject {
            // -B/-L 対象: usr/lib/gcc/{cc}/* の最初のディレクトリ
            if let Some(lib) = glob_in(&gcc.dir, &format!("usr/lib/gcc/{cc}/*"))
                .into_iter()
                .find(|p| p.is_dir())
            {
                let rel = lib.strip_prefix(&gcc.dir).unwrap_or(&lib);
                prep.gcc_lib = Some(format!("/dep/{cc_gcc}/{}", rel.display()));
                let dir = gen_dir.join("gcc-b");
                fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
                for tool in ["gcc", "g++"] {
                    let bin = format!("usr/bin/{cc}-{tool}");
                    if is_exec(&gcc.dir.join(&bin)) {
                        force_symlink(
                            &format!("/dep/{cc_gcc}/{bin}"),
                            &dir.join(format!("{cc}-{tool}")),
                        )
                        .map_err(|source| Error::io(&dir, source))?;
                    }
                }
            }
            // liblto_plugin: 最初のファイルヒットで ar/ranlib/nm ラッパ生成
            if let Some(plugin) = glob_in(
                &gcc.dir,
                &format!("usr/libexec/gcc/{cc}/*/liblto_plugin.so"),
            )
            .into_iter()
            .find(|p| p.is_file())
            {
                let rel = plugin.strip_prefix(&gcc.dir).unwrap_or(&plugin);
                let plugin = format!("/dep/{cc_gcc}/{}", rel.display());
                let dir = gen_dir.join("lto-binutils");
                fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
                if let Some(bu) = mount(&cc_binutils) {
                    for tool in ["ar", "ranlib", "nm"] {
                        let real = format!("usr/bin/{cc}-{tool}");
                        if is_exec(&bu.dir.join(&real)) {
                            write_exec(
                                &dir.join(format!("{cc}-{tool}")),
                                &format!(
                                    "#!/bin/sh\nexec /dep/{cc_binutils}/{real} --plugin {plugin} \"$@\"\n"
                                ),
                            )?;
                            prep.lto_tools.push(tool.to_owned());
                        }
                    }
                }
                prep.lto_binutils = true;
            }
        }
    }
    if ctx.triple != *cc {
        let triple_gcc = format!("{}-gcc", ctx.triple);
        if mount(&triple_gcc).is_some_and(|g| g.dir.join("usr/bin").is_dir()) {
            prep.triple_gcc = true;
        }
    }

    // プレフィックス無し binutils: lto ラッパ優先、無ければ実体へリンク
    if prep.cc_binutils {
        let dir = gen_dir.join("unprefixed-binutils");
        fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
        if let Some(bu) = mount(&cc_binutils) {
            for tool in ["ar", "ranlib", "nm", "strip", "objcopy", "objdump"] {
                let real = format!("usr/bin/{cc}-{tool}");
                let target_path = if prep.lto_tools.iter().any(|t| t == tool) {
                    Some(format!("/tmp/lto-binutils/{cc}-{tool}"))
                } else if is_exec(&bu.dir.join(&real)) {
                    Some(format!("/dep/{cc_binutils}/{real}"))
                } else {
                    None
                };
                if let Some(target_path) = target_path {
                    force_symlink(&target_path, &dir.join(tool))
                        .map_err(|source| Error::io(&dir, source))?;
                    force_symlink(&target_path, &dir.join(format!("gcc-{tool}")))
                        .map_err(|source| Error::io(&dir, source))?;
                }
            }
        }
        prep.unprefixed = true;
    }

    // ホスト {host_triple}-gcc の unprefixed ラッパ
    let host_gcc = format!("{}-gcc", ctx.host_triple);
    if let Some(gcc) = mount(&host_gcc) {
        let bin = format!("usr/bin/{}-gcc", ctx.host_triple);
        if is_exec(&gcc.dir.join(&bin)) {
            let dir = gen_dir.join("host-cc");
            fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
            for name in ["gcc", "cc"] {
                force_symlink(&format!("/dep/{host_gcc}/{bin}"), &dir.join(name))
                    .map_err(|source| Error::io(&dir, source))?;
            }
            let gxx = format!("usr/bin/{}-g++", ctx.host_triple);
            if is_exec(&gcc.dir.join(&gxx)) {
                for name in ["g++", "c++"] {
                    force_symlink(&format!("/dep/{host_gcc}/{gxx}"), &dir.join(name))
                        .map_err(|source| Error::io(&dir, source))?;
                }
            }
            prep.host_cc = true;
        }
    }

    if mount("mold").is_some_and(|m| is_exec(&m.dir.join("usr/bin/mold"))) {
        prep.mold = true;
    }
    Ok(prep)
}

/// .pc の sed 書き換えを Rust で再現する。各行の先頭リテラル一致を
/// `prefix=/dep/{name}/usr` 系へ置き換える（残りの行内容は保持）。
fn rewrite_pc(content: &str, pref: &str) -> String {
    let rules: [(String, String); 7] = [
        ("prefix=/usr".into(), format!("prefix={pref}")),
        ("exec_prefix=/usr".into(), format!("exec_prefix={pref}")),
        ("libdir=/usr/lib64".into(), format!("libdir={pref}/lib64")),
        (
            "sharedlibdir=/usr/lib64".into(),
            format!("sharedlibdir={pref}/lib64"),
        ),
        ("libdir=/usr/lib".into(), format!("libdir={pref}/lib")),
        (
            "sharedlibdir=/usr/lib".into(),
            format!("sharedlibdir={pref}/lib"),
        ),
        (
            "includedir=/usr/include".into(),
            format!("includedir={pref}/include"),
        ),
    ];
    let mut out = String::with_capacity(content.len());
    for line in content.split_inclusive('\n') {
        let (body, eol) = match line.strip_suffix('\n') {
            Some(body) => (body, "\n"),
            None => (line, ""),
        };
        let mut rewritten = body.to_owned();
        for (from, to) in &rules {
            if let Some(rest) = body.strip_prefix(from.as_str()) {
                rewritten = format!("{to}{rest}");
                break;
            }
        }
        out.push_str(&rewritten);
        out.push_str(eol);
    }
    out
}

pub(super) fn wrap_script(
    package: &dyn Package,
    seed: bool,
    target: Target,
    mounts: &[Mount],
    prep: &Prep,
) -> String {
    let script = package.rendered_script(target);
    let ctx = build_context(package, target);
    let cc = &ctx.package_host;
    let inject = !matches!(
        package.role(),
        Role::LinuxHeaders
            | Role::Toolchain {
                stage: Stage::Gcc | Stage::Glibc,
                ..
            }
    );
    let host_gcc = format!("{}-gcc", ctx.host_triple);
    let mut prefix = String::new();
    prefix.push_str(
        r#"_cc_triple="${PACKAGE_HOST:-$TRIPLE}"
# Never export LD=*-gcc: libtool then reports "is GNU ld... no" and skips
# the real -shared link (gmp install: dangling libgmp.so.*).
"#,
    );

    // PATH: dep の usr/bin をバイト順に prepend = 逆順結合
    let dep_bins = dep_paths(mounts, "usr/bin");
    if !dep_bins.is_empty() {
        let joined = dep_bins.into_iter().rev().collect::<Vec<_>>().join(":");
        prefix.push_str(&format!("export PATH=\"{joined}:$PATH\"\n"));
    }
    // LD_LIBRARY_PATH: HOST 実行ライブラリのみ（case フィルタをホスト側で再現）
    let dep_lib_ok = |name: &str| {
        matches!(
            name,
            "python" | "perl" | "libxcrypt-host" | "cmake" | "meson" | "ninja" | "qemu" | "pkgconf"
        ) || name == host_gcc
    };
    let mut dep_libs = dep_glob(mounts, "usr/lib");
    dep_libs.extend(dep_glob(mounts, "usr/lib64"));
    dep_libs.extend(dep_glob(mounts, "usr/lib/perl5/*/*/CORE"));
    let dep_libs: Vec<String> = dep_libs
        .into_iter()
        .filter(|(name, _, _)| dep_lib_ok(name))
        .map(|(_, c, _)| c)
        .collect();
    if !dep_libs.is_empty() {
        let joined = dep_libs.into_iter().rev().collect::<Vec<_>>().join(":");
        prefix.push_str(&format!(
            "export LD_LIBRARY_PATH=\"{joined}${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}\"\n"
        ));
    }
    // PKG_CONFIG_PATH: pkgconfig ディレクトリを順に append
    let mut pc_dirs = dep_paths(mounts, "usr/lib/pkgconfig");
    pc_dirs.extend(dep_paths(mounts, "usr/lib64/pkgconfig"));
    pc_dirs.extend(dep_paths(mounts, "usr/share/pkgconfig"));
    if !pc_dirs.is_empty() {
        let joined = pc_dirs.join(":");
        prefix.push_str(&format!(
            "export PKG_CONFIG_PATH=\"{joined}${{PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}}\"\n"
        ));
    }
    // -I: include ディレクトリ（case 除外をホスト側で再現）
    let inc_skip = |name: &str| {
        name == "linux-headers"
            || matches!(
                name,
                "python"
                    | "perl"
                    | "meson"
                    | "cmake"
                    | "qemu"
                    | "glib"
                    | "pcre2"
                    | "libxcrypt-host"
            )
            || name.ends_with("-gcc")
            || name.ends_with("-glibc")
            || name.ends_with("-binutils")
    };
    let incs: Vec<String> = dep_glob(mounts, "usr/include")
        .into_iter()
        .filter(|(name, _, _)| !inc_skip(name))
        .map(|(_, c, _)| c)
        .collect();
    if !incs.is_empty() {
        let flags = incs.iter().map(|i| format!(" -I{i}")).collect::<String>();
        prefix.push_str(&format!(
            "export CPPFLAGS=\"${{CPPFLAGS:-}}{flags}\"\n\
             export CFLAGS=\"${{CFLAGS:-}}{flags}\"\n\
             export CXXFLAGS=\"${{CXXFLAGS:-}}{flags}\"\n"
        ));
    }
    // -L/-rpath-link: lib ディレクトリ（case 除外をホスト側で再現）
    let lib_skip = |name: &str| {
        name == "linux-headers"
            || matches!(name, "qemu" | "glib" | "pcre2" | "perl" | "libxcrypt-host")
            || name.ends_with("-gcc")
            || name.ends_with("-binutils")
    };
    let mut lib_globs = dep_glob(mounts, "usr/lib");
    lib_globs.extend(dep_glob(mounts, "usr/lib64"));
    let libs: Vec<String> = lib_globs
        .into_iter()
        .filter(|(name, _, _)| !lib_skip(name))
        .map(|(_, c, _)| c)
        .collect();
    if !libs.is_empty() {
        let flags = libs
            .iter()
            .map(|l| format!(" -L{l} -Wl,-rpath-link={l}"))
            .collect::<String>();
        prefix.push_str(&format!("export LDFLAGS=\"${{LDFLAGS:-}}{flags}\"\n"));
        let joined = libs.into_iter().rev().collect::<Vec<_>>().join(":");
        prefix.push_str(&format!(
            "export LIBRARY_PATH=\"{joined}${{LIBRARY_PATH:+:$LIBRARY_PATH}}\"\n"
        ));
    }
    // glibc sysroot libdir（_t in cc, triple の順、重複も元通り再現）
    for t in [cc.as_str(), ctx.triple.as_str()] {
        let glibc = format!("{t}-glibc");
        if let Some(m) = mounts.iter().find(|m| m.name == glibc) {
            for sub in [
                format!("usr/{t}/lib"),
                format!("usr/{t}/lib64"),
                "usr/lib".to_owned(),
                "usr/lib64".to_owned(),
            ] {
                if m.dir.join(&sub).is_dir() {
                    prefix.push_str(&format!(
                        "export LDFLAGS=\"${{LDFLAGS:-}} -L/dep/{glibc}/{sub} -Wl,-rpath-link=/dep/{glibc}/{sub}\"\n"
                    ));
                }
            }
        }
    }
    // crti.o のある sysroot libdir に -B（最初のヒットで break 相当）
    let cc_glibc = format!("{cc}-glibc");
    if let Some(m) = mounts.iter().find(|m| m.name == cc_glibc) {
        for sub in [format!("usr/{cc}/lib"), format!("usr/{cc}/lib64")] {
            if m.dir.join(&sub).join("crti.o").exists() {
                let slib = format!("/dep/{cc_glibc}/{sub}");
                prefix.push_str(&format!(
                    "export CFLAGS=\"${{CFLAGS:-}} -B{slib}\"\n\
                     export CXXFLAGS=\"${{CXXFLAGS:-}} -B{slib}\"\n\
                     export LDFLAGS=\"${{LDFLAGS:-}} -B{slib}\"\n"
                ));
                break;
            }
        }
    }

    // オーバーレイ（中身は /tmp にマウント済み）
    prefix.push_str("mkdir -p /tmp/la-overlay /tmp/pc-overlay\n");
    if prep.la {
        prefix.push_str(
            r#"export LDFLAGS="-L/tmp/la-overlay ${LDFLAGS:-}"
export LIBRARY_PATH="/tmp/la-overlay${LIBRARY_PATH:+:$LIBRARY_PATH}"
"#,
        );
    }
    if prep.pc {
        prefix.push_str(
            r#"export PKG_CONFIG_PATH="/tmp/pc-overlay${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
"#,
        );
    }
    if prep.pkg_config {
        prefix.push_str("export PATH=\"/tmp/pkg-config-bin:$PATH\"\n");
    } else {
        // /dep/pkgconf が無い場合の command -v フォールバックはイメージ依存
        prefix.push_str(
            r#"_pc_wrap=/tmp/pkg-config-bin
mkdir -p "$_pc_wrap"
if command -v pkgconf >/dev/null 2>&1; then
  printf '#!/bin/sh\nexec pkgconf "$@"\n' > "$_pc_wrap/pkg-config"
elif command -v pkg-config >/dev/null 2>&1; then
  printf '#!/bin/sh\nexec pkg-config "$@"\n' > "$_pc_wrap/pkg-config"
fi
if [ -f "$_pc_wrap/pkg-config" ]; then
  chmod +x "$_pc_wrap/pkg-config"
  export PATH="$_pc_wrap:$PATH"
fi
"#,
        );
    }
    if let Some(py) = &prep.meson {
        // meson 本体は /tmp/meson-bin にマウント済み。コンテナ rootfs への
        // python3 リンク作成だけイメージ依存なのでシェルに残す。
        prefix.push_str(&format!(
            r#"_py={py}
if [ ! -e /usr/sbin/python3 ]; then
  mkdir -p /usr/sbin
  ln -sfn "$_py" /usr/sbin/python3
fi
if [ ! -e /usr/bin/python3 ]; then
  ln -sfn "$_py" /usr/bin/python3
fi
export PATH="/tmp/meson-bin:$PATH"
"#,
        ));
    }
    // PYTHONPATH: prepend 各 = 逆順結合
    let mut py_dirs = dep_paths(mounts, "usr/lib/python*/site-packages");
    py_dirs.extend(dep_paths(mounts, "usr/lib64/python*/site-packages"));
    if !py_dirs.is_empty() {
        let joined = py_dirs.into_iter().rev().collect::<Vec<_>>().join(":");
        prefix.push_str(&format!(
            "export PYTHONPATH=\"{joined}${{PYTHONPATH:+:$PYTHONPATH}}\"\n"
        ));
    }
    // PERL5LIB: perl5 のバージョン/アーキディレクトリ（ls -d | head -n1 相当）
    if let Some(perl) = mounts.iter().find(|m| m.name == "perl") {
        let perl5 = perl.dir.join("usr/lib/perl5");
        if perl5.is_dir() {
            let first = |pattern_match: &dyn Fn(&str) -> bool| -> Option<String> {
                let mut names: Vec<String> = fs::read_dir(&perl5)
                    .map(|entries| {
                        entries
                            .flatten()
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .filter(|n| !n.starts_with('.') && pattern_match(n))
                            .collect()
                    })
                    .unwrap_or_default();
                names.sort();
                names.into_iter().next()
            };
            // [0-9]*
            let ver = first(&|n| n.as_bytes().first().is_some_and(u8::is_ascii_digit));
            // [0-9]*/[a-z0-9_]*-linux*
            let arch = ver.as_ref().and_then(|ver| {
                let dir = perl5.join(ver);
                let mut names: Vec<String> = fs::read_dir(&dir)
                    .map(|entries| {
                        entries
                            .flatten()
                            .map(|e| e.file_name().to_string_lossy().into_owned())
                            .filter(|n| {
                                !n.starts_with('.')
                                    && n.as_bytes().first().is_some_and(|b| {
                                        b.is_ascii_lowercase() || b.is_ascii_digit() || *b == b'_'
                                    })
                                    && n[1..].contains("-linux")
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                names.sort();
                names.into_iter().next()
            });
            if let (Some(arch), Some(ver)) = (arch, ver) {
                prefix.push_str(&format!(
                    "export PERL5LIB=\"/dep/perl/usr/lib/perl5/{ver}/{arch}:/dep/perl/usr/lib/perl5/{ver}${{PERL5LIB:+:$PERL5LIB}}\"\n"
                ));
            }
        }
    }
    // 以降の PATH 追加順は元プリアンブルの prepend 順をそのまま再現する。
    if prep.cc_binutils {
        prefix.push_str(&format!(
            "export PATH=\"/dep/{cc}-binutils/usr/bin:$PATH\"\n"
        ));
        if prep.host_strip {
            prefix.push_str("export PATH=\"/tmp/host-strip:$PATH\"\n");
        }
    }
    if prep.triple_binutils {
        prefix.push_str(&format!(
            "export PATH=\"/dep/{}-binutils/usr/bin:$PATH\"\n",
            ctx.triple
        ));
    }
    if prep.cc_gcc {
        prefix.push_str(&format!("export PATH=\"/dep/{cc}-gcc/usr/bin:$PATH\"\n"));
        if inject {
            if let Some(gcc_lib) = &prep.gcc_lib {
                prefix.push_str(&format!(
                    r#"export PATH="/tmp/gcc-b:$PATH"
export CFLAGS="${{CFLAGS:-}} -B{gcc_lib}"
export CXXFLAGS="${{CXXFLAGS:-}} -B{gcc_lib}"
export LDFLAGS="${{LDFLAGS:-}} -B{gcc_lib} -L{gcc_lib}"
export LIBRARY_PATH="{gcc_lib}${{LIBRARY_PATH:+:$LIBRARY_PATH}}"
export COMPILER_PATH="{gcc_lib}${{COMPILER_PATH:+:$COMPILER_PATH}}"
"#
                ));
            }
            if prep.lto_binutils {
                prefix.push_str("export PATH=\"/tmp/lto-binutils:$PATH\"\n");
            }
        }
    }
    if prep.triple_gcc {
        prefix.push_str(&format!(
            "export PATH=\"/dep/{}-gcc/usr/bin:$PATH\"\n",
            ctx.triple
        ));
    }
    if prep.unprefixed {
        prefix.push_str("export PATH=\"/tmp/unprefixed-binutils:$PATH\"\n");
    }
    if prep.host_cc {
        prefix.push_str("export PATH=\"/tmp/host-cc:$PATH\"\n");
    }
    if prep.mold {
        prefix.push_str("export PATH=\"/dep/mold/usr/bin:$PATH\"\n");
    }
    // sysroot_env.sh 相当: Steps{sysroot:true} のみ。実パス検出は
    // マウント一覧からホスト側で行う。
    if let Recipe::Steps(steps) = package.recipe(&ctx)
        && steps.sysroot
    {
        let candidates = [
            (format!("{cc}-glibc"), format!("usr/{cc}")),
            (
                format!("{}-glibc", ctx.triple),
                format!("usr/{}", ctx.triple),
            ),
            (format!("{cc}-glibc"), String::new()),
            (format!("{}-glibc", ctx.triple), String::new()),
        ];
        for (name, sub) in &candidates {
            let hit = mounts
                .iter()
                .find(|m| m.name == *name)
                .filter(|m| m.dir.join(sub).is_dir());
            if hit.is_some() {
                let sysroot = if sub.is_empty() {
                    format!("/dep/{name}")
                } else {
                    format!("/dep/{name}/{sub}")
                };
                prefix.push_str(&format!(
                    "SYSROOT={sysroot}\n\
                         export CFLAGS=\"${{CFLAGS:-}} --sysroot=$SYSROOT\"\n\
                         export CXXFLAGS=\"${{CXXFLAGS:-}} --sysroot=$SYSROOT\"\n\
                         export CPPFLAGS=\"${{CPPFLAGS:-}} --sysroot=$SYSROOT\"\n\
                         export LDFLAGS=\"${{LDFLAGS:-}} --sysroot=$SYSROOT\"\n"
                ));
                break;
            }
        }
        if let Some(headers) = mounts.iter().find(|m| m.name == "linux-headers") {
            for sub in [
                format!("usr/{cc}/include"),
                format!("usr/{}/include", ctx.triple),
                "usr/include".to_owned(),
            ] {
                if headers.dir.join(&sub).is_dir() {
                    let hdr = format!("/dep/linux-headers/{sub}");
                    prefix.push_str(&format!(
                        "export CFLAGS=\"${{CFLAGS:-}} -idirafter {hdr}\"\n\
                             export CXXFLAGS=\"${{CXXFLAGS:-}} -idirafter {hdr}\"\n\
                             export CPPFLAGS=\"${{CPPFLAGS:-}} -idirafter {hdr}\"\n"
                    ));
                    break;
                }
            }
        }
    }
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
