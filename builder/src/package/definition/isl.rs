use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Isl;

impl Package for Isl {
    fn name(&self) -> &'static str {
        "isl"
    }
    fn version(&self) -> &'static str {
        "0.26"
    }
    fn license(&self) -> &'static str {
        "MIT"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://libisl.sourceforge.io/isl-0.26.tar.xz",
            sha3_512: "4b31f4a9a343cc15319359ad41ef8d4176bcf2f713ea1fb3b2e5a275a6bc6d109943521ef181fab5c21f0ba581f08e753b4e7c71f9a296a559dc71f77754f1aa",
            blake3: "a27da5d097f4e105d3a63c5e81d26c2b00cc35a4a3bf62dd2a49335a0f20ce7f57a3e1152dac76aa4b16d8a4205040f108562aa9647e2d6d27717ed533af673f",
            subdir: "isl-0.26",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[crate::package::Dependency {
                name: "gmp",
                version: "6.3.0",
            }],
        )
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
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::autotools_args(vec![
            "--prefix=/usr".into(),
            format!("--with-gmp-prefix={}", ctx.overlay("gmp")),
        ])
    }
}
