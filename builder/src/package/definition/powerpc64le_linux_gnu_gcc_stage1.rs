use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::{gcc, gcc_stage1};

/// `powerpc64le-linux-gnu` 向け GCC stage1（最小構成 C コンパイラ）。
pub struct Powerpc64leLinuxGnuGccStage1;

const TRIPLE: &str = "powerpc64le-linux-gnu";
const NAME: &str = "powerpc64le-linux-gnu-gcc-stage1";

impl Package for Powerpc64leLinuxGnuGccStage1 {
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
    fn dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }
    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        gcc_stage1::build_deps(TRIPLE)
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::Steps(gcc_stage1::recipe(ctx))
    }
    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }
    fn build_only(&self) -> bool {
        true
    }
    fn supports(&self, target: Target) -> bool {
        target == Target::BUILD_ENV
    }
    fn gnu_triple(&self) -> Option<&'static str> {
        Some(TRIPLE)
    }
}

impl Bootstrap for Powerpc64leLinuxGnuGccStage1 {
    fn pacman(&self) -> &'static [&'static str] {
        if self.seed() {
            gcc::PACMAN_STAGE_SEED
        } else {
            gcc::PACMAN_STAGE
        }
    }
}
