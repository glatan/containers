use std::path::PathBuf;

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

pub fn build_dir(name: &str, version: &str, target: Target) -> PathBuf {
    PathBuf::from("packages/build")
        .join(format!("{}-{}", target.architecture(), target.variant()))
        .join(format!("{name}-{version}"))
}

pub fn layout_dir(name: &str, version: &str) -> PathBuf {
    PathBuf::from("oci-layouts").join(format!("{name}-{version}"))
}
