use std::path::{Path, PathBuf};

use crate::target::Target;

pub fn source_dir(name: &str) -> PathBuf {
    PathBuf::from("packages/sources").join(name)
}

pub fn artifact_dir(name: &str, version: &str, target: Target) -> PathBuf {
    PathBuf::from("packages/artifacts").join(format!(
        "{}-{}-{}-{}",
        name,
        version,
        target.architecture(),
        target.variant()
    ))
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
