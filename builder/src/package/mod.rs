use std::path::PathBuf;

use crate::target::Target;
use crate::workspace;

pub(crate) mod build;
pub mod definition;
pub(crate) mod env;
pub(crate) mod fetch;
pub(crate) mod validate;

pub use build::build_package;
pub use definition::{PACKAGES, all, find, find_image, images};
pub use fetch::fetch_package;

pub trait Package: Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn license(&self) -> &'static str;
    fn build_only(&self) -> bool;
    fn network(&self) -> bool;
    fn source(&self) -> Source;
    fn dependencies(&self) -> &'static [Dependency];
    fn build_dependencies(&self) -> &'static [Dependency];
    fn build_script(&self) -> &'static str;

    fn flags(&self) -> &'static [crate::flags::Var];
    fn source_dir(&self) -> PathBuf {
        workspace::source_dir(self.name())
    }

    fn artifact_dir(&self, target: Target) -> PathBuf {
        workspace::artifact_dir(self.name(), self.version(), target)
    }

    fn depends_on(&self, name: &str) -> bool {
        lists_name(self.dependencies(), name)
    }

    fn build_depends_on(&self, name: &str) -> bool {
        lists_name(self.build_dependencies(), name)
    }
}

fn lists_name(dependencies: &[Dependency], name: &str) -> bool {
    dependencies
        .iter()
        .any(|dependency| dependency.name == name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub url: &'static str,
    pub sha3_512: &'static str,
    pub blake3: &'static str,
    pub subdir: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: &'static str,
    pub version: &'static str,
}

impl Dependency {
    pub fn artifact_dir(self, target: Target) -> PathBuf {
        workspace::artifact_dir(self.name, self.version, target)
    }
}
