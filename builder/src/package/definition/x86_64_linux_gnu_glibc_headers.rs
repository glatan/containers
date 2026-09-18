use crate::flags::{Flag, Pgo};
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Role, Source, Stage};
use crate::target::Target;

use super::toolchain::Prefixed;

pub struct X8664LinuxGnuGlibcHeaders;

const DEF: Prefixed = Prefixed {
    name: "x86_64-linux-gnu-glibc-headers",
    triple: "x86_64-linux-gnu",
    stage: Stage::GlibcHeaders,
};

impl Package for X8664LinuxGnuGlibcHeaders {
    fn name(&self) -> &'static str {
        DEF.name()
    }
    fn version(&self) -> &'static str {
        DEF.version()
    }
    fn license(&self) -> &'static str {
        DEF.license()
    }
    fn build_only(&self) -> bool {
        DEF.build_only()
    }
    fn role(&self) -> Role {
        DEF.role()
    }
    fn source(&self) -> Source {
        DEF.source()
    }
    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        DEF.dependencies(target)
    }
    fn build_dependencies(&self, target: Target) -> Vec<Dependency> {
        DEF.build_dependencies(target)
    }
    fn flags(&self, target: Target) -> Flag {
        DEF.flags(target)
    }
    fn pgo(&self) -> Pgo {
        DEF.pgo()
    }
    fn supports(&self, target: Target) -> bool {
        DEF.supports(target)
    }
    fn host_tools(&self, target: Target) -> Vec<&'static str> {
        DEF.host_tools(target)
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        DEF.recipe(ctx)
    }
}

impl Bootstrap for X8664LinuxGnuGlibcHeaders {
    fn pacman(&self) -> &'static [&'static str] {
        DEF.pacman()
    }
    fn seed(&self) -> bool {
        DEF.seed()
    }
    fn seed_dependencies(&self, target: Target) -> Option<(Vec<Dependency>, Vec<Dependency>)> {
        DEF.seed_dependencies(target)
    }
}
