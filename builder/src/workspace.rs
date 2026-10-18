use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::target::Target;

pub fn source_dir(name: &str) -> PathBuf {
    PathBuf::from("packages/sources").join(name)
}

#[cfg(test)]
thread_local! {
    static ARTIFACT_ROOT: std::cell::RefCell<Option<PathBuf>> = const { std::cell::RefCell::new(None) };
}

/// テストがワークスペースの `packages/artifacts` を参照しないための差し替え。
/// ガードがドロップされると元の設定に戻る。
#[cfg(test)]
pub struct ArtifactRootGuard;

#[cfg(test)]
impl Drop for ArtifactRootGuard {
    fn drop(&mut self) {
        ARTIFACT_ROOT.with(|slot| *slot.borrow_mut() = None);
    }
}

#[cfg(test)]
pub fn set_artifact_root(root: &Path) -> ArtifactRootGuard {
    ARTIFACT_ROOT.with(|slot| *slot.borrow_mut() = Some(root.to_path_buf()));
    ArtifactRootGuard
}

pub fn artifact_dir(name: &str, version: &str, target: Target) -> PathBuf {
    let rel = format!(
        "{}-{}-{}-{}",
        name,
        version,
        target.architecture(),
        target.variant()
    );
    #[cfg(test)]
    if let Some(root) = ARTIFACT_ROOT.with(|slot| slot.borrow().clone()) {
        return root.join(rel);
    }
    PathBuf::from("packages/artifacts").join(rel)
}

pub fn log_dir() -> PathBuf {
    PathBuf::from("packages/logs")
}

pub fn log_path(name: &str, version: &str, target: Target) -> PathBuf {
    log_dir().join(format!(
        "{}-{}-{}-{}.log",
        name,
        version,
        target.architecture(),
        target.variant()
    ))
}

pub fn bootstrap_log_path() -> PathBuf {
    log_dir().join("bootstrap.log")
}

pub fn artifact_ready(dir: &Path) -> bool {
    let Ok(mut entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.next().is_some()
}

pub fn build_dir(name: &str, version: &str, target: Target) -> PathBuf {
    PathBuf::from("packages/build")
        .join(format!("{}-{}", target.architecture(), target.variant()))
        .join(format!("{name}-{version}"))
}

/// 生成オーバーレイ群のホスト側置き場。`build_dir` 配下には置けない:
/// `/build` はコンテナへ rw マウントされ、openssl の `make clean` が
/// `find . -type l -exec rm` で配下の全シンボリックリンク（`.gen/gcc-b`
/// 等）を掃除してしまうため、マウント外に退避する。
pub fn gen_dir(name: &str, version: &str, target: Target) -> PathBuf {
    PathBuf::from("packages/build")
        .join(format!("{}-{}", target.architecture(), target.variant()))
        .join(".gen")
        .join(format!("{name}-{version}"))
}

pub fn layout_dir(name: &str, version: &str) -> PathBuf {
    PathBuf::from("oci-layouts").join(format!("{name}-{version}"))
}

pub fn compose_layout_dir(name: &str, arch: &str, date: &str, revision: u32) -> PathBuf {
    PathBuf::from("oci-layouts").join(format!("{name}-{arch}-{date}-{revision}"))
}

pub fn compose_current_link(name: &str, arch: &str) -> PathBuf {
    PathBuf::from("oci-layouts").join(format!("{name}-{arch}"))
}

/// index artifact の生成先。`{name}-{version}` 規約とは別の固定パス。
pub fn index_layout_dir() -> PathBuf {
    PathBuf::from("oci-layouts/index")
}

/// skopeo 等の外部ツールが push 中に大きな一時 blob を書く先。
/// `oci-layouts/` と同一 FS（監視対象の VHDX）に固定し、既定の
/// /tmp や別ドライブへの散逸を防ぐ。
pub fn skopeo_tmp_dir() -> Result<PathBuf> {
    let dir = PathBuf::from("oci-layouts/.tmp");
    std::fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
    // TMPDIR は相対だと spawn 先の解釈が不安定になるため絶対化する
    std::fs::canonicalize(&dir).map_err(|source| Error::io(&dir, source))
}

/// build、bootstrap、pack、restore、compose がコマンド全期間共有で持つロック。
/// `clean` の成果物・layout 削除はこれを排他で取れないとき、何も消さず拒否する。
pub fn active_lock_path() -> PathBuf {
    PathBuf::from("packages/artifacts/.locks/active.lock")
}

pub fn active_lock() -> Result<fd_lock::RwLock<std::fs::File>> {
    open_lock(&active_lock_path())
}

/// 成果物 `{name}-{version}-{arch}-{variant}` 単位の flock のパス。
/// build の atomic publish と `restore` の展開、`clean` のセル単位スキップが共有する。
pub fn artifact_lock_path(name: &str, version: &str, target: Target) -> PathBuf {
    PathBuf::from("packages/artifacts/.locks").join(format!(
        "{name}-{version}-{}-{}.lock",
        target.architecture(),
        target.variant()
    ))
}

/// 成果物 `{name}-{version}-{arch}-{variant}` 単位の flock。
/// build の atomic publish と `restore` の展開を直列化するため、
/// `package/build/container.rs` のロック規約と同じ場所・命名を使う。
pub fn artifact_lock(
    name: &str,
    version: &str,
    target: Target,
) -> Result<fd_lock::RwLock<std::fs::File>> {
    open_lock(&artifact_lock_path(name, version, target))
}

/// 排他ロックを、ガードがロック本体より先に解放される形で所有する。
/// 複数セルを削除前にまとめて押さえるときに使う。
pub struct ExclusiveLock {
    // フィールドは宣言順に drop される。ガードを先に解放し、その後で実体を捨てる。
    _guard: fd_lock::RwLockWriteGuard<'static, std::fs::File>,
    _lock: Box<fd_lock::RwLock<std::fs::File>>,
}

impl ExclusiveLock {
    /// 取れれば所有する。他プロセスが共有または排他で持っていれば `Ok(None)`。
    pub fn try_acquire(path: &Path) -> Result<Option<Self>> {
        let mut lock = Box::new(open_lock(path)?);
        let guard = match lock.try_write() {
            Ok(guard) => guard,
            Err(source) if source.kind() == std::io::ErrorKind::WouldBlock => return Ok(None),
            Err(source) => return Err(Error::io(path, source)),
        };
        // ガードはヒープ上の `RwLock` を指す。`Box` の移動は実体の番地を変えない。
        let _guard = unsafe {
            std::mem::transmute::<
                fd_lock::RwLockWriteGuard<'_, std::fs::File>,
                fd_lock::RwLockWriteGuard<'static, std::fs::File>,
            >(guard)
        };
        Ok(Some(Self {
            _guard,
            _lock: lock,
        }))
    }
}

fn open_lock(path: &Path) -> Result<fd_lock::RwLock<std::fs::File>> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let file = std::fs::File::create(path).map_err(|source| Error::io(path, source))?;
    Ok(fd_lock::RwLock::new(file))
}

/// 成果物ディレクトリ単位の flock（dir の basename をロック名に使う）。
/// `packages/artifacts/{name}-{version}-{arch}-{variant}` の basename は
/// `artifact_lock` のロック名と一致するため同じロックを共有する。
/// pack/compose が層の tar 読み取り中に restore --force の rename と
/// 競合しないよう、読み取り側が read で取得する。
pub fn artifact_dir_lock(dir: &Path) -> Result<fd_lock::RwLock<std::fs::File>> {
    let parent = dir.parent().unwrap_or_else(|| Path::new("."));
    let lock_dir = parent.join(".locks");
    std::fs::create_dir_all(&lock_dir).map_err(|source| Error::io(&lock_dir, source))?;
    let name = dir
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("artifact"));
    let path = lock_dir.join(format!("{}.lock", name.to_string_lossy()));
    let file = std::fs::File::create(&path).map_err(|source| Error::io(&path, source))?;
    Ok(fd_lock::RwLock::new(file))
}

/// `output`（レイアウト等の確定先ディレクトリ）単位の flock。
/// ロックファイルは `{output.parent}/.locks/{name}.lock` に置く。
/// 呼び出し側は返した RwLock の `write()`/`read()` で排他する
/// （プロセス終了で自動解放。merge 読み取り〜確定 rename の
/// 競合防止が目的）。
pub fn output_lock_path(output: &Path) -> PathBuf {
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    let name = output
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("output"));
    parent
        .join(".locks")
        .join(format!("{}.lock", name.to_string_lossy()))
}

pub fn output_lock(output: &Path) -> Result<fd_lock::RwLock<std::fs::File>> {
    open_lock(&output_lock_path(output))
}

pub fn next_compose_revision(name: &str, arch: &str, date: &str) -> u32 {
    let mut revision = 0;
    while compose_layout_dir(name, arch, date, revision).exists() {
        revision += 1;
    }
    revision
}

pub const BUILD_ENV_NAME: &str = "build-env";
pub const BUILD_ENV_ARCH: &str = "amd64";

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn empty_artifact_dir_is_not_ready() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(!artifact_ready(tmp.path()));
        assert!(!artifact_ready(&tmp.path().join("missing")));
        fs::write(tmp.path().join("file"), "ok").unwrap();
        assert!(artifact_ready(tmp.path()));
    }

    #[test]
    fn package_paths_stay_inside_packages_tree() {
        for package in crate::package::PACKAGES {
            for target in Target::all() {
                for (path, prefix) in [
                    (
                        artifact_dir(package.name(), package.version(), target),
                        "packages/artifacts",
                    ),
                    (
                        build_dir(package.name(), package.version(), target),
                        "packages/build",
                    ),
                    (
                        log_path(package.name(), package.version(), target),
                        "packages/logs",
                    ),
                ] {
                    assert!(path.starts_with(prefix), "{path:?}");
                    for component in path.components() {
                        assert!(
                            matches!(
                                component,
                                std::path::Component::Normal(_) | std::path::Component::CurDir
                            ),
                            "{path:?} にパス移動要素があります"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn gen_dir_stays_outside_build_dir() {
        // .gen は /build マウント配下に置けない（コンテナ内の
        // `find . -type l -exec rm` 系の clean がシンボリックリンクを
        // 消すため）。ターゲット dir 直下の .gen/ に退避する。
        for package in crate::package::PACKAGES {
            for target in Target::all() {
                let staged = gen_dir(package.name(), package.version(), target);
                let build = build_dir(package.name(), package.version(), target);
                assert!(!staged.starts_with(&build), "{staged:?}");
            }
        }
    }

    #[test]
    fn log_path_matches_artifact_identity() {
        let target = Target::BUILD_ENV;
        assert_eq!(
            log_path("gmp", "6.3.0", target).as_os_str(),
            "packages/logs/gmp-6.3.0-amd64-v4.log"
        );
        assert_eq!(
            bootstrap_log_path().as_os_str(),
            "packages/logs/bootstrap.log"
        );
    }
}
