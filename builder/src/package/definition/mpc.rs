use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Mpc;

impl Package for Mpc {
    fn name(&self) -> &'static str {
        "mpc"
    }
    fn version(&self) -> &'static str {
        "1.4.1"
    }
    fn license(&self) -> &'static str {
        "LGPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/mpc/mpc-1.4.1.tar.xz",
            sha3_512: "347aa36ed9745dab597e13373ac0c3ff35c8413427a89e3ee1e74a8640a51c893610f3f5d0e30c13a496025142c0539ec936b020138df945f3c8df424dd59ed9",
            blake3: "994e14084833795dbb36ee1abe46059c74cab1845e7ca17585bc7bc8727444f8ac5bec1247722a8a98c0ae0196b0fb813dd39586b7303dc6e612649534c7b8e3",
            subdir: "mpc-1.4.1",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[
                crate::package::Dependency {
                    name: "gmp",
                    version: "6.3.0",
                },
                crate::package::Dependency {
                    name: "mpfr",
                    version: "4.2.2",
                },
            ],
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
            format!("--with-gmp={}", ctx.overlay("gmp")),
            format!("--with-mpfr={}", ctx.overlay("mpfr")),
        ])
    }
}
