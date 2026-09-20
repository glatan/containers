use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::glibc;

/// `x86_64-linux-gnu` 向け glibc。
pub struct X8664LinuxGnuGlibc;

const TRIPLE: &str = "x86_64-linux-gnu";
const NAME: &str = "x86_64-linux-gnu-glibc";

impl Package for X8664LinuxGnuGlibc {
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
}

impl Bootstrap for X8664LinuxGnuGlibc {
    fn pacman(&self) -> &'static [&'static str] {
        glibc::PACMAN
    }
    /// シード（Arch base-devel）相の中間ステージ。
    fn seed(&self) -> bool {
        true
    }
}
