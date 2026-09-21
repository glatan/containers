use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::flags::Ldflag;
#[cfg(test)]
use crate::package::recipe::Env;
use crate::package::recipe::{Arg, BuildContext, BuildPlan, Cond, Pipe, Recipe, Step, argv};
use crate::package::{self, Package};
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
    force: bool,
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
    // 同一 (name, target) の同時ビルドを flock で直列化する
    // （artifact_ready チェック〜削除〜作成の TOCTOU を防ぐ。
    // flock はプロセス終了時に自動解放される）。
    let lock_dir = PathBuf::from("packages/artifacts/.locks");
    fs::create_dir_all(&lock_dir).map_err(|source| Error::io(&lock_dir, source))?;
    let lock_path = lock_dir.join(format!(
        "{}-{}-{}-{}.lock",
        package.name(),
        package.version(),
        architecture,
        variant
    ));
    let lock_file = File::create(&lock_path).map_err(|source| Error::io(&lock_path, source))?;
    let mut build_lock = fd_lock::RwLock::new(lock_file);
    let _build_guard = build_lock
        .write()
        .map_err(|source| Error::io(&lock_path, source))?;
    // ロック待ちの間に別プロセスがビルドを終えていれば skip。
    // --force 指定のルートは既存成果物を残したまま再ビルドする
    // （成果物がコンパイラとして自身をマウントする自己ホスト型も
    // 動くよう、ここでは削除しない）。
    if !force && workspace::artifact_ready(&out_dir) {
        return Ok(());
    }
    // 成果物は `<name>.tmp` に構築してから rename で公開する
    // （中断残骸が artifact_ready を騙るのを防ぐ atomic publish。
    // 失敗したリビルドで既存成果物が失われることもない）。
    let tmp_dir = PathBuf::from(format!("{}.tmp", out_dir.display()));
    let gen_dir = workspace::gen_dir(package.name(), package.version(), target);
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    }
    if gen_dir.exists() {
        fs::remove_dir_all(&gen_dir).map_err(|source| Error::io(&gen_dir, source))?;
    }
    if tmp_dir.exists() {
        fs::remove_dir_all(&tmp_dir).map_err(|source| Error::io(&tmp_dir, source))?;
    }
    fs::create_dir_all(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    fs::create_dir_all(&gen_dir).map_err(|source| Error::io(&gen_dir, source))?;
    fs::create_dir_all(&tmp_dir).map_err(|source| Error::io(&tmp_dir, source))?;
    let build_dir = fs::canonicalize(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    // podman の -v は絶対パスを要求する（相対だと named volume と解釈される）。
    let gen_dir = fs::canonicalize(&gen_dir).map_err(|source| Error::io(&gen_dir, source))?;
    let out_dir_mnt = fs::canonicalize(&tmp_dir).map_err(|source| Error::io(&tmp_dir, source))?;
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
        if !workspace::artifact_ready(&dep_dir) {
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
            dep: true,
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
            dep: false,
        });
    }
    for name in compiler_tool_names(package, target, own) {
        if mounted.contains(&name.as_str()) {
            continue;
        }
        let found = package::find(&name)?;
        let mounted_target = if found.target_artifact() {
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
            dep: false,
        });
    }
    for name in extra_host_tool_names(package, target) {
        if mounted.contains(&name) {
            continue;
        }
        let found = package::find(name)?;
        let mounted_target = if found.target_artifact() {
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
            dep: false,
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
                dep: false,
            });
        }
    }
    let ctx = build_context(package, target);
    // .la/.pc オーバーレイと {gmp,mpfr}-prefix をホスト側で生成し、
    // プリアンブルが作っていた /tmp/* パスへそのままマウントする。
    // gen_dir は /build マウントの外に置く（配下だと openssl `make clean` の
    // `find . -type l -exec rm` でシンボリックリンクが消去される）。
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
        .arg(format!("{}:/out", out_dir_mnt.display()))
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
    // 実行経路: コンテナ内 runner（`run-recipe` が plan を直接解釈）。
    // bash 生成は golden・query・デバッグ用途のみ（`wrap_script`）。
    let plan = build_plan(package, !own, target, &mounts, &prep);
    let plan_path = gen_dir.join("plan.json");
    fs::create_dir_all(&gen_dir).map_err(|source| Error::io(&gen_dir, source))?;
    fs::write(&plan_path, serde_json::to_string(&plan)?)
        .map_err(|source| Error::io(&plan_path, source))?;
    command
        .args(["-v"])
        .arg(format!("{}:/tmp/plan.json:ro", plan_path.display()));
    // runner 自身をコンテナへマウント。musl 静的ビルドがあればそれを使い、
    // 無ければホストバイナリを使う（イメージ側 glibc との後方互換が必要）。
    let exe = runner_exe()?;
    command
        .args(["-v"])
        .arg(format!("{}:/tmp/runner:ro", exe.display()));
    let entrypoint: Vec<String> = vec![
        "/tmp/runner".to_string(),
        "run-recipe".to_string(),
        "/tmp/plan.json".to_string(),
        "-t".to_string(),
        format!("{architecture}/{variant}"),
    ];
    let log_path = workspace::log_path(package.name(), package.version(), target);
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let mut log = File::create(&log_path).map_err(|source| Error::io(&log_path, source))?;
    writeln!(
        log,
        "package: {} {}\ntarget: {}/{}\nimage: {image}\nrunner: {}\nstarted: {}\n",
        package.name(),
        package.version(),
        architecture,
        variant,
        exe.display(),
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
                "package: {} {}\ntarget: {}/{}\nimage: {image}\nrunner: {}\nstarted: {}\n",
                package.name(),
                package.version(),
                architecture,
                variant,
                exe.display(),
                util::timestamp()
            );
        }
        process::tee_status(
            command.arg(&image).args(&entrypoint),
            "podman",
            &mut log,
            combined.as_deref_mut(),
            &log_path,
        )
    };
    if result.is_err() {
        let _ = fs::remove_dir_all(&tmp_dir);
        return result;
    }
    // 成功したら tmp を rename して公開する（既存成果物はここで差し替え）
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    }
    fs::rename(&tmp_dir, &out_dir).map_err(|source| Error::io(&out_dir, source))
}

pub(super) fn build_context(package: &dyn Package, target: Target) -> BuildContext {
    BuildContext::of(package.gnu_triple(), target)
}

pub(super) fn host_gcc_ready() -> bool {
    package::find(Target::BUILD_ENV.gcc_package())
        .ok()
        .is_some_and(|gcc| workspace::artifact_ready(&gcc.artifact_dir(Target::BUILD_ENV)))
}

pub(super) fn compiler_cc(
    package: &dyn Package,
    target: Target,
    own: bool,
) -> Vec<(&'static str, String)> {
    // seed 相のパッケージ（`Bootstrap::seed`）は toolchain 成果物が
    // まだ存在しないため、シードの無印コンパイラでビルドする。
    let prefix = if is_seed(package, target) {
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
    target == Target::BUILD_ENV && package.as_bootstrap().is_some_and(|b| b.seed())
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
    // その target の libc 提供者（`{triple}-glibc`）は full
    // `{triple}-gcc` ではなく stage2 でビルドする。full gcc は glibc に
    // 依存するため、ここで要求すると循環する
    // （stage2 は build_deps で既にマウント済み）。
    let gcc = if provides_target_libc(package, target) {
        format!("{prefix}gcc-stage2")
    } else {
        format!("{prefix}gcc")
    };
    [gcc, format!("{prefix}binutils")].into_iter().collect()
}

/// `target` 向けの libc を提供するパッケージか（`{triple}-glibc`
/// 最終成果物）。コード生成 triple 一致・ターゲット成果物・
/// ビルド専用でない（glibc-headers を除く）で判定する。
fn provides_target_libc(package: &dyn Package, target: Target) -> bool {
    package.gnu_triple() == Some(target.gnu_triple())
        && package.target_artifact()
        && !package.build_only()
}

pub(super) fn extra_host_tool_names(package: &dyn Package, target: Target) -> Vec<&'static str> {
    let mut names = package.host_tools(target);
    // ホスト triple 以外の toolchain 構成要素や非 amd64 向けビルドには
    // ホスト toolchain を追加マウントする。seed 相のパッケージ
    // （linux-headers 等）はシードのコンパイラを使うため除外。
    if !is_seed(package, target)
        && (package.gnu_triple() != Some(Target::HOST_GNU_TRIPLE)
            || !matches!(target, Target::Amd64(_)))
    {
        names.push(Target::BUILD_ENV.gcc_package());
        names.push(Target::BUILD_ENV.binutils_package());
    }
    if target != Target::BUILD_ENV && !provides_target_libc(package, target) {
        names.push(target.glibc_package());
    }
    names
}

/// シード（Arch base-devel）イメージ上で pacman -Sy するパッケージ。
/// 各パッケージの `Package::pacman()` 宣言に加えて、非 bootstrap
/// パッケージの非 amd64 ビルドには distro の cross gcc を入れる
/// （Bootstrap 実装パッケージは pacman() で環境を完結させる）。
pub(super) fn seed_pacman(package: &dyn Package, target: Target) -> Vec<String> {
    let mut pkgs: Vec<String> = package
        .pacman()
        .iter()
        .map(|name| (*name).to_string())
        .collect();
    if package.as_bootstrap().is_none() && !matches!(target, Target::Amd64(_)) {
        pkgs.push(format!("{}gcc", target.cross_prefix()));
    }
    // seed 相 toolchain（gnu_triple 持ち）と、mold 未構築で -fuse-ld=mold
    // を使うパッケージには distro の mold を入れる。
    if pkgs.iter().all(|name| name != "mold")
        && ((is_seed(package, target) && package.gnu_triple().is_some())
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
    /// パッケージの依存宣言由来か。`false` のものは HOST 実行ツールの
    /// マウントで、ターゲット向け `-I`/`-L` には載せない。
    pub dep: bool,
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
        // cc1/lto1 が実行時リンクするライブラリの .la は staging しない。
        // installed=no で staged された dep .la が dependency_libs に入ると
        // libtool が install 時 relink し、relink 出力と同名の .so を
        // lto1 が読み込み中に truncate されて衝突する（mpfr/isl 等）。
        // .so シンボリックリンクは残すため -l 解決は維持される。
        if matches!(name.as_str(), "gmp" | "mpfr" | "mpc" | "isl") {
            prep.la = true;
            continue;
        }
        let content = fs::read_to_string(&host).map_err(|source| Error::io(&host, source))?;
        let rewritten = rewrite_la(&content)
            .replace("/dep/gmp/usr/lib/libgmp.la", "/tmp/la-overlay/libgmp.la")
            .replace("/dep/mpfr/usr/lib/libmpfr.la", "/tmp/la-overlay/libmpfr.la");
        fs::write(la_over.join(file_name), rewritten)
            .map_err(|source| Error::io(&la_over, source))?;
        prep.la = true;
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
    let inject = package.uses_cc_wrapper();
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
            // meson パッケージの bin/meson は `exec python3 -m ...` の
            // シェル shim。shebang を python へ差し替えると shell 構文を
            // python が解釈して壊れるため、shim 形式は python3 の
            // 呼び出しパスだけ差し替える。
            let rewritten = if let Some(body) = content.strip_prefix("#!/bin/sh\nexec python3 ") {
                format!("#!/bin/sh\nexec {py} {body}")
            } else {
                match content.strip_prefix("#!").and_then(|_| content.find('\n')) {
                    Some(nl) => format!("#!{py}{}", &content[nl..]),
                    None if content.starts_with("#!") => format!("#!{py}\n"),
                    None => content,
                }
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
            // ld/as はネイティブビルドでのみ unprefixed 提供する。autotools
            // の AC_PATH_PROG 系 ld 検索が裸の  を要求する一方、
            // クロスビルドでは裸の ld/as がターゲット toolchain と誤認される
            // リスクがあるため除外する。
            const BASE: &[&str] = &["ar", "ranlib", "nm", "strip", "objcopy", "objdump"];
            const NATIVE: &[&str] = &[
                "ar", "ranlib", "nm", "strip", "objcopy", "objdump", "ld", "as",
            ];
            let tools = if ctx.triple == *cc { NATIVE } else { BASE };
            for tool in tools {
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

/// `wrap_script` のプリリュードと recipe 本体を束ねた実行計画を
/// 構築する。bash バックエンドでは `wrap_script` がこれを render し、
/// コンテナ内 runner は `run-recipe` がこれを直接解釈する。
pub(crate) fn build_plan(
    package: &dyn Package,
    seed: bool,
    target: Target,
    mounts: &[Mount],
    prep: &Prep,
) -> BuildPlan {
    let ctx = build_context(package, target);
    let recipe = package.recipe(&ctx);
    let errors = recipe.validate();
    assert!(
        errors.is_empty(),
        "{} の recipe が不変条件違反: {errors:?}",
        package.name()
    );
    let cc = &ctx.package_host;
    let inject = package.uses_cc_wrapper();
    let host_gcc = format!("{}-gcc", ctx.host_triple);
    let exec = |args: &[&'static str]| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(argv(args)),
    };
    let export = |name: &'static str, value: String| Step::Export {
        name,
        value: Arg::owned(value),
    };
    let mut steps = vec![
        Step::Set {
            name: "_cc_triple",
            value: Arg::lit("\"${PACKAGE_HOST:-$TRIPLE}\""),
        },
        Step::Comment(
            "Never export LD=*-gcc: libtool then reports \"is GNU ld... no\" and skips\nthe real -shared link (gmp install: dangling libgmp.so.*).".into(),
        ),
    ];

    // PATH: dep の usr/bin をバイト順に prepend = 逆順結合
    let dep_bins = dep_paths(mounts, "usr/bin");
    if !dep_bins.is_empty() {
        let joined = dep_bins.into_iter().rev().collect::<Vec<_>>().join(":");
        steps.push(export("PATH", format!("\"{joined}:$PATH\"")));
    }
    // gcc は --with-as/--with-ld=/usr/bin/{triple}-{as,ld} で構築されるため、
    // その絶対パスを /dep の binutils へ向ける。glibc/glibc-headers の
    // レシピも同じリンクを張るが、クロスターゲットの汎用パッケージでも
    // 必要になるため prelude で冪等に用意する。
    let triple_binutils = format!("{}-binutils", ctx.triple);
    if mounts.iter().any(|m| m.name == triple_binutils) {
        for tool in ["as", "ld"] {
            steps.push(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::owned(format!(
                        "\"/dep/{triple_binutils}/usr/bin/{}-{tool}\"",
                        ctx.triple
                    )),
                    Arg::owned(format!("/usr/bin/{}-{tool}", ctx.triple)),
                ]),
            });
        }
    }
    // ld が -B/dep 経路（-L$SYSROOT より先に探索される）で libc.so
    // スクリプトを読むと、スクリプト内の絶対パス（/lib/libc.so.6 等）に
    // sysroot は適用されず裸パスになる。環境イメージに /lib が無いため、
    // dep glibc の libdir を指す /lib symlink を冪等に張る。
    // ターゲット glibc を優先し、無ければビルド機側 glibc（Canadian
    // toolchain では host 側リンクが host glibc のスクリプトを読む）。
    let glibc_mount = [
        format!("{}-glibc", ctx.triple),
        format!("{}-glibc", ctx.host_triple),
    ]
    .into_iter()
    .find(|name| mounts.iter().any(|m| m.name == *name));
    if let Some(triple_glibc) = glibc_mount {
        let glibc_triple = triple_glibc.trim_end_matches("-glibc");
        steps.push(Step::If {
            cond: Cond::Not(Box::new(Cond::Exists(Arg::lit("/lib")))),
            then: vec![Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::owned(format!("\"/dep/{triple_glibc}/usr/{glibc_triple}/lib\"")),
                    Arg::lit("/lib"),
                ]),
            }],
            else_: vec![],
        });
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
        steps.push(export(
            "LD_LIBRARY_PATH",
            format!("\"{joined}${{LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}}\""),
        ));
    }
    // bison はビルド時に検出した m4 の絶対パスをバイナリへ焼く
    // （seed では /usr/sbin/m4）。dep マウント環境ではそのパスが
    // 存在しないため、env M4 で実行時に差し替える。
    if mounts.iter().any(|m| m.name == "m4") {
        steps.push(export("M4", "\"/dep/m4/usr/bin/m4\"".to_owned()));
    }
    // PKG_CONFIG_PATH: pkgconfig ディレクトリを順に append
    let mut pc_dirs = dep_paths(mounts, "usr/lib/pkgconfig");
    pc_dirs.extend(dep_paths(mounts, "usr/lib64/pkgconfig"));
    pc_dirs.extend(dep_paths(mounts, "usr/share/pkgconfig"));
    if !pc_dirs.is_empty() {
        let joined = pc_dirs.join(":");
        steps.push(export(
            "PKG_CONFIG_PATH",
            format!("\"{joined}${{PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}}\""),
        ));
    }
    // -I/-L は依存宣言由来のマウントのみ載せる。HOST 実行ツールの
    // マウント（dep=false）は別 arch 成果物であり得るため混ぜない。
    // 依存としての pcre2 等はターゲット成果物なので載せる必要がある
    // （glib リンク時の rpath-link 等）。
    let is_dep = |name: &str| mounts.iter().any(|m| m.name == name && m.dep);
    // -I: include ディレクトリ（case 除外をホスト側で再現）
    let inc_skip = |name: &str| {
        !is_dep(name)
            || name == "linux-headers"
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
        steps.push(export("CPPFLAGS", format!("\"${{CPPFLAGS:-}}{flags}\"")));
        steps.push(export("CFLAGS", format!("\"${{CFLAGS:-}}{flags}\"")));
        steps.push(export("CXXFLAGS", format!("\"${{CXXFLAGS:-}}{flags}\"")));
    }
    // -L/-rpath-link: lib ディレクトリ（case 除外をホスト側で再現）
    let lib_skip = |name: &str| {
        !is_dep(name)
            || name == "linux-headers"
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
        steps.push(export("LDFLAGS", format!("\"${{LDFLAGS:-}}{flags}\"")));
        let joined = libs.into_iter().rev().collect::<Vec<_>>().join(":");
        steps.push(export(
            "LIBRARY_PATH",
            format!("\"{joined}${{LIBRARY_PATH:+:$LIBRARY_PATH}}\""),
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
                    steps.push(export(
                        "LDFLAGS",
                        format!(
                            "\"${{LDFLAGS:-}} -L/dep/{glibc}/{sub} -Wl,-rpath-link=/dep/{glibc}/{sub}\""
                        ),
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
                steps.push(export("CFLAGS", format!("\"${{CFLAGS:-}} -B{slib}\"")));
                steps.push(export("CXXFLAGS", format!("\"${{CXXFLAGS:-}} -B{slib}\"")));
                steps.push(export("LDFLAGS", format!("\"${{LDFLAGS:-}} -B{slib}\"")));
                break;
            }
        }
    }

    // オーバーレイ（中身は /tmp にマウント済み）
    steps.push(exec(&["mkdir", "-p", "/tmp/la-overlay", "/tmp/pc-overlay"]));
    if prep.la {
        steps.push(export(
            "LDFLAGS",
            "\"-L/tmp/la-overlay ${LDFLAGS:-}\"".to_owned(),
        ));
        steps.push(export(
            "LIBRARY_PATH",
            "\"/tmp/la-overlay${LIBRARY_PATH:+:$LIBRARY_PATH}\"".to_owned(),
        ));
    }
    if prep.pc {
        steps.push(export(
            "PKG_CONFIG_PATH",
            "\"/tmp/pc-overlay${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}\"".to_owned(),
        ));
    }
    if prep.pkg_config {
        steps.push(export("PATH", "\"/tmp/pkg-config-bin:$PATH\"".to_owned()));
    } else {
        // /dep/pkgconf が無い場合の command -v フォールバックはイメージ依存
        steps.push(Step::Set {
            name: "_pc_wrap",
            value: Arg::lit("/tmp/pkg-config-bin"),
        });
        steps.push(exec(&["mkdir", "-p", "\"$_pc_wrap\""]));
        let wrapper = |body: &'static str| Step::WriteFile {
            path: Arg::lit("\"$_pc_wrap/pkg-config\""),
            lines: argv(&["#!/bin/sh", body]),
            append: false,
            expand: false,
        };
        steps.push(Step::If {
            cond: Cond::Cmd(
                Pipe::cmd(argv(&["command", "-v", "pkgconf"]))
                    .stdout(Arg::lit("/dev/null"))
                    .stderr(Arg::lit("/dev/null")),
            ),
            then: vec![wrapper("exec pkgconf \"$@\"")],
            else_: vec![Step::If {
                cond: Cond::Cmd(
                    Pipe::cmd(argv(&["command", "-v", "pkg-config"]))
                        .stdout(Arg::lit("/dev/null"))
                        .stderr(Arg::lit("/dev/null")),
                ),
                then: vec![wrapper("exec pkg-config \"$@\"")],
                else_: vec![],
            }],
        });
        steps.push(Step::If {
            cond: Cond::FileExists(Arg::lit("\"$_pc_wrap/pkg-config\"")),
            then: vec![
                exec(&["chmod", "+x", "\"$_pc_wrap/pkg-config\""]),
                export("PATH", "\"$_pc_wrap:$PATH\"".to_owned()),
            ],
            else_: vec![],
        });
    }
    if let Some(py) = &prep.meson {
        // meson 本体は /tmp/meson-bin にマウント済み。コンテナ rootfs への
        // python3 リンク作成はイメージ依存なので存在確認つきにする。
        steps.push(Step::Set {
            name: "_py",
            value: Arg::owned(format!("\"{py}\"")),
        });
        steps.push(Step::If {
            cond: Cond::Not(Box::new(Cond::Exists(Arg::lit("/usr/sbin/python3")))),
            then: vec![
                exec(&["mkdir", "-p", "/usr/sbin"]),
                exec(&["ln", "-sfn", "\"$_py\"", "/usr/sbin/python3"]),
            ],
            else_: vec![],
        });
        steps.push(Step::If {
            cond: Cond::Not(Box::new(Cond::Exists(Arg::lit("/usr/bin/python3")))),
            then: vec![exec(&["ln", "-sfn", "\"$_py\"", "/usr/bin/python3"])],
            else_: vec![],
        });
        steps.push(export("PATH", "\"/tmp/meson-bin:$PATH\"".to_owned()));
    }
    // PYTHONPATH: prepend 各 = 逆順結合
    let mut py_dirs = dep_paths(mounts, "usr/lib/python*/site-packages");
    py_dirs.extend(dep_paths(mounts, "usr/lib64/python*/site-packages"));
    if !py_dirs.is_empty() {
        let joined = py_dirs.into_iter().rev().collect::<Vec<_>>().join(":");
        steps.push(export(
            "PYTHONPATH",
            format!("\"{joined}${{PYTHONPATH:+:$PYTHONPATH}}\""),
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
                steps.push(export(
                    "PERL5LIB",
                    format!(
                        "\"/dep/perl/usr/lib/perl5/{ver}/{arch}:/dep/perl/usr/lib/perl5/{ver}${{PERL5LIB:+:$PERL5LIB}}\""
                    ),
                ));
            }
        }
    }
    // 以降の PATH 追加順は元プリアンブルの prepend 順をそのまま再現する。
    if prep.cc_binutils {
        steps.push(export(
            "PATH",
            format!("\"/dep/{cc}-binutils/usr/bin:$PATH\""),
        ));
        if prep.host_strip {
            steps.push(export("PATH", "\"/tmp/host-strip:$PATH\"".to_owned()));
        }
    }
    if prep.triple_binutils {
        steps.push(export(
            "PATH",
            format!("\"/dep/{}-binutils/usr/bin:$PATH\"", ctx.triple),
        ));
    }
    if prep.cc_gcc {
        steps.push(export("PATH", format!("\"/dep/{cc}-gcc/usr/bin:$PATH\"")));
        if inject {
            if let Some(gcc_lib) = &prep.gcc_lib {
                steps.push(export("PATH", "\"/tmp/gcc-b:$PATH\"".to_owned()));
                steps.push(export("CFLAGS", format!("\"${{CFLAGS:-}} -B{gcc_lib}\"")));
                steps.push(export(
                    "CXXFLAGS",
                    format!("\"${{CXXFLAGS:-}} -B{gcc_lib}\""),
                ));
                steps.push(export(
                    "LDFLAGS",
                    format!("\"${{LDFLAGS:-}} -B{gcc_lib} -L{gcc_lib}\""),
                ));
                steps.push(export(
                    "LIBRARY_PATH",
                    format!("\"{gcc_lib}${{LIBRARY_PATH:+:$LIBRARY_PATH}}\""),
                ));
                steps.push(export(
                    "COMPILER_PATH",
                    format!("\"{gcc_lib}${{COMPILER_PATH:+:$COMPILER_PATH}}\""),
                ));
            }
            if prep.lto_binutils {
                steps.push(export("PATH", "\"/tmp/lto-binutils:$PATH\"".to_owned()));
            }
        }
    }
    if prep.triple_gcc {
        steps.push(export(
            "PATH",
            format!("\"/dep/{}-gcc/usr/bin:$PATH\"", ctx.triple),
        ));
    }
    if prep.unprefixed {
        steps.push(export(
            "PATH",
            "\"/tmp/unprefixed-binutils:$PATH\"".to_owned(),
        ));
    }
    if prep.host_cc {
        steps.push(export("PATH", "\"/tmp/host-cc:$PATH\"".to_owned()));
    }
    if prep.mold {
        steps.push(export("PATH", "\"/dep/mold/usr/bin:$PATH\"".to_owned()));
    }
    // sysroot/カーネルヘッダ付与: Steps{sysroot:true} のみ。実パス検出は
    // マウント一覧からホスト側で行う。
    if let Recipe::Steps(steps_) = &recipe
        && steps_.sysroot
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
            if let Some(m) = hit {
                let dep_root = m.dir.join(sub);
                let sysroot = if sub.is_empty() {
                    format!("/dep/{name}")
                } else {
                    format!("/dep/{name}/{sub}")
                };
                steps.push(Step::Set {
                    name: "SYSROOT",
                    value: Arg::owned(sysroot),
                });
                // dep glibc の layout は usr/{triple}/{include,lib} で、
                // distro/host gcc が見る $SYSROOT/usr/include にはヘッダが
                // ない。C++ の #include_next は -isystem/-idirafter には
                // 届かないため、標準 layout を模した shadow sysroot を
                // /tmp/sysroot に張って SYSROOT を差し替える。
                steps.push(exec(&["mkdir", "-p", "/tmp/sysroot/usr"]));
                for (link, target) in [
                    ("/tmp/sysroot/include", "$SYSROOT/include"),
                    ("/tmp/sysroot/usr/include", "$SYSROOT/include"),
                ] {
                    steps.push(Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(vec![
                            Arg::lit("ln"),
                            Arg::lit("-sfn"),
                            Arg::owned(format!("\"{target}\"")),
                            Arg::lit(link),
                        ]),
                    });
                }
                // ld はリンカスクリプトの実パスが sysroot 配下の時だけ
                // スクリプト内の絶対パスに sysroot を適用する。/dep への
                // シンボリックリンクでは realpath が sysroot 外に出て
                // libc.so の GROUP (/lib/libc.so.6 ...) が解決不能になるため、
                // lib は実ディレクトリへ実コピーする。dep 側が相対
                // シンボリックリンクの場合は shadow 内で再現する
                //（sysroot 内に解決が留まり sysrooted 判定が維持される）。
                for d in ["lib", "lib64"] {
                    let dep_dir = dep_root.join(d);
                    let dst = format!("/tmp/sysroot/{d}");
                    match std::fs::read_link(&dep_dir) {
                        Ok(target) if target.is_relative() => steps.push(Step::Exec {
                            env: vec![],
                            pipe: Pipe::cmd(vec![
                                Arg::lit("ln"),
                                Arg::lit("-sfn"),
                                Arg::owned(target.to_string_lossy().into_owned()),
                                Arg::owned(dst),
                            ]),
                        }),
                        _ if dep_dir.is_dir() => {
                            steps.push(Step::Exec {
                                env: vec![],
                                pipe: Pipe::cmd(vec![
                                    Arg::lit("mkdir"),
                                    Arg::lit("-p"),
                                    Arg::owned(dst.clone()),
                                ]),
                            });
                            steps.push(Step::Exec {
                                env: vec![],
                                pipe: Pipe::cmd(vec![
                                    Arg::lit("cp"),
                                    Arg::lit("-a"),
                                    Arg::owned(format!("\"$SYSROOT/{d}/.\"")),
                                    Arg::owned(dst),
                                ]),
                            });
                        }
                        _ => continue,
                    }
                    steps.push(Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(vec![
                            Arg::lit("ln"),
                            Arg::lit("-sfn"),
                            Arg::owned(format!("../{d}")),
                            Arg::owned(format!("/tmp/sysroot/usr/{d}")),
                        ]),
                    });
                }
                steps.push(Step::Set {
                    name: "SYSROOT",
                    value: Arg::lit("/tmp/sysroot"),
                });
                steps.push(export(
                    "CFLAGS",
                    "\"${CFLAGS:-} --sysroot=$SYSROOT\"".to_owned(),
                ));
                steps.push(export(
                    "CXXFLAGS",
                    "\"${CXXFLAGS:-} --sysroot=$SYSROOT\"".to_owned(),
                ));
                steps.push(export(
                    "CPPFLAGS",
                    "\"${CPPFLAGS:-} --sysroot=$SYSROOT\"".to_owned(),
                ));
                // ld が libc.so 等のスクリプトを sysroot 内から解決できるよう
                // -L は /dep 直接パスより先に shadow を指す（先にヒットした方が
                // 採用されるため prepend）。CFLAGS にも入れるのは gmp の
                // ABI テストのように LDFLAGS を使わないリンクチェックがある
                // ため（-c コンパイルでは無害に無視される）。
                steps.push(export(
                    "CFLAGS",
                    "\"-L$SYSROOT/lib -L$SYSROOT/lib64 ${CFLAGS:-}\"".to_owned(),
                ));
                steps.push(export(
                    "CXXFLAGS",
                    "\"-L$SYSROOT/lib -L$SYSROOT/lib64 ${CXXFLAGS:-}\"".to_owned(),
                ));
                steps.push(export(
                    "LDFLAGS",
                    "\"-L$SYSROOT/lib -L$SYSROOT/usr/lib -L$SYSROOT/lib64 -L$SYSROOT/usr/lib64 ${LDFLAGS:-} --sysroot=$SYSROOT\"".to_owned(),
                ));
                // -isystem/-idirafter $SYSROOT/include は付けない:
                // shadow の usr/include と同一 inode に解決されて dedup
                // され、C++ の #include_next が届くべき標準位置
                // （$SYSROOT/usr/include）が消えてしまう（isl の
                // cstdlib→stdlib.h で実害）。shadow sysroot 単体で
                // host/repo 双方の gcc がヘッダを見つける。
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
                    steps.push(export(
                        "CFLAGS",
                        format!("\"${{CFLAGS:-}} -idirafter {hdr}\""),
                    ));
                    steps.push(export(
                        "CXXFLAGS",
                        format!("\"${{CXXFLAGS:-}} -idirafter {hdr}\""),
                    ));
                    steps.push(export(
                        "CPPFLAGS",
                        format!("\"${{CPPFLAGS:-}} -idirafter {hdr}\""),
                    ));
                    break;
                }
            }
        }
    }
    let mold_steps: Vec<Step> = if uses_mold(package, target) {
        vec![
            Step::Set {
                name: "_mold_wrap",
                value: Arg::lit("/tmp/mold-ld-cc"),
            },
            exec(&["mkdir", "-p", "\"$_mold_wrap\""]),
            Step::If {
                cond: Cond::Cmd(
                    Pipe::cmd(argv(&["command", "-v", "mold"]))
                        .stdout(Arg::lit("/dev/null"))
                        .stderr(Arg::lit("/dev/null")),
                ),
                then: vec![
                    Step::ForEach {
                        var: "_name",
                        items: argv(&[
                            "gcc",
                            "cc",
                            "g++",
                            "c++",
                            "\"${_cc_triple}-gcc\"",
                            "\"${_cc_triple}-g++\"",
                            "\"${TRIPLE}-gcc\"",
                            "\"${TRIPLE}-g++\"",
                            "\"${HOST_TRIPLE}-gcc\"",
                            "\"${HOST_TRIPLE}-g++\"",
                        ]),
                        body: vec![
                            Step::Set {
                                name: "_src",
                                value: Arg::SubstOr(Box::new(Pipe::cmd(argv(&[
                                    "command",
                                    "-v",
                                    "\"$_name\"",
                                ])))),
                            },
                            // 元スクリプトの `case "$_src" in "$_mold_wrap"/*)
                            // continue` ガードは _mold_wrap がコンテナ毎に
                            // 新規空ディレクトリのため到達不能。省略。
                            Step::If {
                                cond: Cond::And(vec![
                                    Cond::NonEmpty(Arg::lit("\"$_src\"")),
                                    Cond::Executable(Arg::lit("\"$_src\"")),
                                ]),
                                then: vec![
                                    Step::Exec {
                                        env: vec![],
                                        pipe: Pipe::cmd(argv(&[
                                            "printf",
                                            "'#!/bin/sh\\nexec %s -fuse-ld=mold \"$@\"\\n'",
                                            "\"$_src\"",
                                        ]))
                                        .stdout(Arg::lit("\"$_mold_wrap/$_name\"")),
                                    },
                                    exec(&["chmod", "+x", "\"$_mold_wrap/$_name\""]),
                                ],
                                else_: vec![],
                            },
                        ],
                        stdout: None,
                    },
                    export("PATH", "\"$_mold_wrap:$PATH\"".to_owned()),
                ],
                else_: vec![],
            },
        ]
    } else {
        vec![]
    };
    let pkgs = if seed {
        seed_pacman(package, target)
    } else {
        Vec::new()
    };
    if pkgs.is_empty() {
        steps.extend(mold_steps);
    } else {
        // 旧 `pacman ... &&` は直後の変数代入のみをゲートしていたが、
        // 意図は pacman 成功時のみ mold 設定を行うことなので `if` で明示する
        // （pacman 失敗時もレシピ本体は実行する）。
        let mut pacman_argv = argv(&["pacman", "-Sy", "--noconfirm"]);
        pacman_argv.extend(pkgs.iter().map(|pkg| Arg::owned(pkg.clone())));
        if mold_steps.is_empty() {
            // gate 対象が無い場合は pacman だけ実行する。空 then の
            // `if ...; then fi` は bash の構文エラーになるため If は使わない
            // （Cond::Cmd の `>/dev/null` と同じく stdout は捨てる）。
            steps.push(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(pacman_argv).stdout(Arg::lit("/dev/null")),
            });
        } else {
            steps.push(Step::If {
                cond: Cond::Cmd(Pipe::cmd(pacman_argv)),
                then: mold_steps,
                else_: vec![],
            });
        }
    }
    // prelude も recipe と同じ不変条件で検査する
    let mut errors = Vec::new();
    let mut sysroot_defined = false;
    crate::package::recipe::validate_steps(
        &steps,
        crate::package::recipe::Env::None,
        &mut sysroot_defined,
        &mut errors,
    );
    assert!(
        errors.is_empty(),
        "{} の prelude が不変条件違反: {errors:?}",
        package.name()
    );
    BuildPlan {
        prelude: steps,
        recipe,
    }
}

/// 計画を bash スクリプトへ render する互換バックエンド。
/// golden テスト・デバッグ表示用。実実行は runner のみ。
#[cfg(test)]
pub(super) fn wrap_script(
    package: &dyn Package,
    seed: bool,
    target: Target,
    mounts: &[Mount],
    prep: &Prep,
) -> String {
    let ctx = build_context(package, target);
    let plan = build_plan(package, seed, target, mounts, prep);
    let mut out = String::new();
    for step in &plan.prelude {
        step.render(&mut out, Env::None);
    }
    out.push_str(&plan.recipe.render(&ctx));
    out
}

pub(super) fn own_image_ref() -> Result<String> {
    let current =
        workspace::compose_current_link(workspace::BUILD_ENV_NAME, workspace::BUILD_ENV_ARCH);
    let path = fs::canonicalize(&current).map_err(|source| Error::io(&current, source))?;
    Ok(format!("oci:{}", path.display()))
}

/// コンテナ内で `run-recipe` を実行する runner バイナリを解決する。
///
/// 優先順:
/// 1. `CONTAINERS_BUILDER_RUNNER`（明示指定。存在しないパスはエラー）
/// 2. cargo 規約の musl 静的ビルド
///    `target/x86_64-unknown-linux-musl/<profile>/<name>`
///    （完全静的でコンテナ側 libc に依存しない）
/// 3. ホストバイナリ自身（フォールバック。ホスト glibc がコンテナ側より
///    新しいとコンテナ内で起動しない）
fn runner_exe() -> Result<PathBuf> {
    let exe = std::env::current_exe()
        .and_then(fs::canonicalize)
        .map_err(|source| Error::message(format!("current_exe: {source}")))?;
    let override_path = std::env::var_os("CONTAINERS_BUILDER_RUNNER");
    let runner = resolve_runner(override_path.as_deref(), &exe)?;
    if override_path.is_none() && runner == exe {
        static WARNED: std::sync::Once = std::sync::Once::new();
        WARNED.call_once(|| {
            eprintln!(
                "警告: musl 静的 runner が見つかりません。ホストバイナリをマウントします\
                 （コンテナ側 glibc がホストより古いと起動しません）。\
                 `cargo build --target x86_64-unknown-linux-musl` で生成できます"
            );
        });
    }
    Ok(runner)
}

pub(super) fn resolve_runner(override_path: Option<&OsStr>, exe: &Path) -> Result<PathBuf> {
    if let Some(path) = override_path {
        let path = PathBuf::from(path);
        if !path.is_file() {
            return Err(Error::message(format!(
                "CONTAINERS_BUILDER_RUNNER が存在しません: {}",
                path.display()
            )));
        }
        return fs::canonicalize(&path).map_err(|source| Error::io(&path, source));
    }
    Ok(musl_runner_candidate(exe).unwrap_or_else(|| exe.to_path_buf()))
}

/// `exe` が `target/[<triple>/]<profile>/<name>` にあるとき、同じ target dir の
/// `x86_64-unknown-linux-musl/<profile>/<name>` を返す。
fn musl_runner_candidate(exe: &Path) -> Option<PathBuf> {
    let profile = exe.parent()?.file_name()?;
    let name = exe.file_name()?;
    exe.ancestors()
        .skip(2)
        .take(2)
        .map(|dir| {
            dir.join("x86_64-unknown-linux-musl")
                .join(profile)
                .join(name)
        })
        .find(|candidate| candidate.is_file())
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
