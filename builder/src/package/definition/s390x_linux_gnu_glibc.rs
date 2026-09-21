use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::glibc;

/// `s390x-linux-gnu` 向け glibc。
pub struct S390xLinuxGnuGlibc;

const TRIPLE: &str = "s390x-linux-gnu";
const NAME: &str = "s390x-linux-gnu-glibc";

impl Package for S390xLinuxGnuGlibc {
    fn name(&self) -> &'static str {
        NAME
    }
    fn version(&self) -> &'static str {
        crate::package::GLIBC_VERSION
    }
    fn license(&self) -> &'static str {
        glibc::LICENSE
    }
    fn source(&self) -> Source {
        glibc::SOURCE
    }
    fn dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }
    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        glibc::build_deps(TRIPLE)
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::Steps(glibc::recipe(ctx))
    }
    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }
    fn supports(&self, target: Target) -> bool {
        target.gnu_triple() == TRIPLE
    }
    fn target_artifact(&self) -> bool {
        true
    }
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        glibc::HOST_TOOLS.to_vec()
    }
    fn uses_cc_wrapper(&self) -> bool {
        false
    }
    fn gnu_triple(&self) -> Option<&'static str> {
        Some(TRIPLE)
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
    fn pacman(&self) -> &'static [&'static str] {
        glibc::PACMAN
    }
}

impl Bootstrap for S390xLinuxGnuGlibc {}
