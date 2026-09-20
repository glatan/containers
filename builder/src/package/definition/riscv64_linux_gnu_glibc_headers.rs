use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::{glibc, glibc_headers};

/// `riscv64-linux-gnu` 向け glibc ヘッダ。
pub struct Riscv64LinuxGnuGlibcHeaders;

const TRIPLE: &str = "riscv64-linux-gnu";
const NAME: &str = "riscv64-linux-gnu-glibc-headers";

impl Package for Riscv64LinuxGnuGlibcHeaders {
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
        glibc_headers::build_deps(TRIPLE)
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::Steps(glibc_headers::recipe(ctx))
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
    fn target_artifact(&self) -> bool {
        true
    }
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        glibc::HOST_TOOLS.to_vec()
    }
    fn gnu_triple(&self) -> Option<&'static str> {
        Some(TRIPLE)
    }
}

impl Bootstrap for Riscv64LinuxGnuGlibcHeaders {
    fn pacman(&self) -> &'static [&'static str] {
        glibc::PACMAN
    }
}
