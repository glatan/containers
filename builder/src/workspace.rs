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

pub fn layout_dir(name: &str, version: &str) -> PathBuf {
    PathBuf::from("oci-layouts").join(format!("{name}-{version}"))
}

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
}
