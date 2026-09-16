use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Package, Source};

pub struct Diffutils;

impl Package for Diffutils {
    fn name(&self) -> &'static str {
        "diffutils"
    }
    fn version(&self) -> &'static str {
        "3.12"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/diffutils/diffutils-3.12.tar.xz",
            sha3_512: "24fde52f69eebea2c5807bf3ab532dd5334ec6714203e9dbeca6cd71f7b31abc0fc1ebe8fa20a17aca8bac41b7b66344c092310bfe5af240fbf45f2e4e47e6fb",
            blake3: "363fb378c85505b43f91d6e7c7382d571f2f7b47e4d334c18d522baf55599d18a186e566dcd9cdcf7f3580edf8381ddfc7fd7c0b1100e20ee6ba6dd7f3e6e80c",
            subdir: "diffutils-3.12",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(target, &[])
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }
    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }
    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools(&["--prefix=/usr"])
    }
}

impl Bootstrap for Diffutils {}
