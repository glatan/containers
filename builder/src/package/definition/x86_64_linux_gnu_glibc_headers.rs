use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::{glibc, glibc_headers};

/// `x86_64-linux-gnu` 向け glibc ヘッダ。
pub struct X8664LinuxGnuGlibcHeaders;

const TRIPLE: &str = "x86_64-linux-gnu";
const NAME: &str = "x86_64-linux-gnu-glibc-headers";

impl Package for X8664LinuxGnuGlibcHeaders {
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
    /// glibc の configure は python3/bison/m4 を要求する。
    /// python→openssl→glibc の循環は via(Seed) で断つ。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        glibc::host_deps(self, target)
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

impl Bootstrap for X8664LinuxGnuGlibcHeaders {
    /// シード（Arch base-devel）相の中間ステージ。
    fn seed(&self) -> bool {
        true
    }
}
