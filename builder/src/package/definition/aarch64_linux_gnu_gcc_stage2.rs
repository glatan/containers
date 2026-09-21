use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::{gcc, gcc_stage2};

/// `aarch64-linux-gnu` 向け GCC stage2（glibc headers 後）。
pub struct Aarch64LinuxGnuGccStage2;

const TRIPLE: &str = "aarch64-linux-gnu";
const NAME: &str = "aarch64-linux-gnu-gcc-stage2";

impl Package for Aarch64LinuxGnuGccStage2 {
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
        gcc_stage2::build_deps(TRIPLE)
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::Steps(gcc_stage2::recipe(ctx))
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
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        gcc::RUNTIME_LIBS.iter().map(|dep| dep.name).collect()
    }
    fn gnu_triple(&self) -> Option<&'static str> {
        Some(TRIPLE)
    }
}

impl Bootstrap for Aarch64LinuxGnuGccStage2 {
    fn pacman(&self) -> &'static [&'static str] {
        if self.seed() {
            gcc::PACMAN_STAGE_SEED
        } else {
            gcc::PACMAN_STAGE
        }
    }
}
