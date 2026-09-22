use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::{gcc, gcc_stage2};

/// `powerpc64le-linux-gnu` 向け GCC stage2（glibc headers 後）。
pub struct Powerpc64leLinuxGnuGccStage2;

const TRIPLE: &str = "powerpc64le-linux-gnu";
const NAME: &str = "powerpc64le-linux-gnu-gcc-stage2";

impl Package for Powerpc64leLinuxGnuGccStage2 {
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
    /// stage コンパイラが動的リンクするランタイムライブラリ。
    /// シード相は distro パッケージが担保し、env では成果物必須。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        gcc::stage_host_deps(self, target)
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
            gcc::PACMAN_STAGE_SEED
        } else {
            gcc::PACMAN_STAGE
        }
    }
}

impl Bootstrap for Powerpc64leLinuxGnuGccStage2 {}
