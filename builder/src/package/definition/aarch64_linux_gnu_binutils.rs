use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::binutils;

/// `aarch64-linux-gnu` 向け binutils。
pub struct Aarch64LinuxGnuBinutils;

const TRIPLE: &str = "aarch64-linux-gnu";
const NAME: &str = "aarch64-linux-gnu-binutils";

impl Package for Aarch64LinuxGnuBinutils {
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
}

impl Bootstrap for Aarch64LinuxGnuBinutils {
    fn pacman(&self) -> &'static [&'static str] {
        if self.seed() {
            binutils::PACMAN_SEED
        } else {
            binutils::PACMAN
        }
    }
    fn seed_dependencies(&self, target: Target) -> Option<(Vec<Dependency>, Vec<Dependency>)> {
        Some((vec![], self.build_dependencies(target)))
    }
}
