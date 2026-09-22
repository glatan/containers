use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

use super::{glibc, glibc_headers};

/// `aarch64-linux-gnu` 向け glibc ヘッダ。
pub struct Aarch64LinuxGnuGlibcHeaders;

const TRIPLE: &str = "aarch64-linux-gnu";
const NAME: &str = "aarch64-linux-gnu-glibc-headers";

impl Package for Aarch64LinuxGnuGlibcHeaders {
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

impl Bootstrap for Aarch64LinuxGnuGlibcHeaders {}
