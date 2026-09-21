use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::binutils;

/// `powerpc64le-linux-gnu` 向け binutils。
pub struct Powerpc64leLinuxGnuBinutils;

const TRIPLE: &str = "powerpc64le-linux-gnu";
const NAME: &str = "powerpc64le-linux-gnu-binutils";

impl Package for Powerpc64leLinuxGnuBinutils {
    fn name(&self) -> &'static str {
        NAME
    }
    fn version(&self) -> &'static str {
        binutils::VERSION
    }
    fn license(&self) -> &'static str {
        binutils::LICENSE
    }
    fn source(&self) -> Source {
        binutils::SOURCE
    }
    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        binutils::deps(target)
    }
    fn build_dependencies(&self, target: Target) -> Vec<Dependency> {
        // シード相では glibc 依存を切り離す（循環回避）。
        binutils::build_deps(!(self.seed() && target == Target::BUILD_ENV))
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::Steps(binutils::recipe(ctx))
    }
    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }
    fn gnu_triple(&self) -> Option<&'static str> {
        Some(TRIPLE)
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
    fn pacman(&self) -> &'static [&'static str] {
        if self.seed() {
            binutils::PACMAN_SEED
        } else {
            binutils::PACMAN
        }
    }
}

impl Bootstrap for Powerpc64leLinuxGnuBinutils {
    fn seed_dependencies(&self, target: Target) -> Option<(Vec<Dependency>, Vec<Dependency>)> {
        Some((vec![], self.build_dependencies(target)))
    }
}
