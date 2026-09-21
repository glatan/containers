use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::gcc;

/// `s390x-linux-gnu` 向け 最終 GCC。
pub struct S390xLinuxGnuGcc;

const TRIPLE: &str = "s390x-linux-gnu";
const NAME: &str = "s390x-linux-gnu-gcc";

impl Package for S390xLinuxGnuGcc {
    fn name(&self) -> &'static str {
        NAME
    }
    fn version(&self) -> &'static str {
        crate::package::GCC_VERSION
    }
    fn license(&self) -> &'static str {
        gcc::LICENSE
    }
    fn source(&self) -> Source {
        gcc::SOURCE
    }
    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        gcc::deps(target)
    }
    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        gcc::build_deps(TRIPLE)
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::Steps(gcc::recipe(ctx))
    }
    fn flags(&self, target: Target) -> Flag {
        gcc::flags(target)
    }
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }
    fn uses_cc_wrapper(&self) -> bool {
        false
    }
    fn host_tools(&self, target: Target) -> Vec<&'static str> {
        if target == Target::BUILD_ENV {
            vec![]
        } else {
            vec![NAME]
        }
    }
    fn gnu_triple(&self) -> Option<&'static str> {
        Some(TRIPLE)
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
    fn pacman(&self) -> &'static [&'static str] {
        gcc::PACMAN
    }
}

impl Bootstrap for S390xLinuxGnuGcc {}
