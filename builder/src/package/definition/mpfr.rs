use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Mpfr;

impl Package for Mpfr {
    fn name(&self) -> &'static str {
        "mpfr"
    }
    fn version(&self) -> &'static str {
        "4.2.2"
    }
    fn license(&self) -> &'static str {
        "LGPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/mpfr/mpfr-4.2.2.tar.xz",
            sha3_512: "92d8b3f8b16cdb1130e59717e2c8aa8b249070bd34675000af73327b60043358b4238f94cf9c37b97fa8323ec823d518977746afb92573558ce1d40d6121b406",
            blake3: "11d59d061ef8db588650bc7dc5172594a6e5aad013994801c6f63011a62b191dd94685e03abb7e145f97b51c40f75ae11795c1ab6112fc4cf82d638bb68c0588",
            subdir: "mpfr-4.2.2",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(target, &[crate::package::dep("gmp", "6.3.0")])
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
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Profiled
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo_args(
            crate::package::recipe::Env::Autotools,
            vec![
                "--prefix=/usr".into(),
                format!("--with-gmp={}", ctx.overlay("gmp")),
            ],
            Recipe::make_check_workload(),
        )
    }
}
