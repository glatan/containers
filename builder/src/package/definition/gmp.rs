use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Package, Source};

pub struct Gmp;

impl Package for Gmp {
    fn name(&self) -> &'static str {
        "gmp"
    }
    fn version(&self) -> &'static str {
        "6.3.0"
    }
    fn license(&self) -> &'static str {
        "LGPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz",
            sha3_512: "b459a8c258273ff76a2627a64627739d233064e5a9a48de68a544c921a1fe4d617af48528de7b67778f30754d793d645a9ecf9c38a71578bd4f0ab6febf591de",
            blake3: "fffe4996713928ae19331c8ef39129e46d3bf5b7182820656fd4639435cd83a40d7c77aa7c571938ae3f25a58bfdfa44b6a710f2a7e002ab11edf8389cd6928b",
            subdir: "gmp-6.3.0",
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
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Profiled
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let abi = match ctx.target {
            crate::target::Target::Riscv64(_) => "standard",
            crate::target::Target::Ppc64le(_) => "mode64",
            _ => "64",
        };
        // configure が CFLAGS を Makefile へ焼くため Env::Lto（PGO フラグは
        // configure env で注入される）
        Recipe::autotools_pgo_args(
            crate::package::recipe::Env::Lto,
            vec!["--prefix=/usr".into(), format!("ABI={abi}")],
            vec![Step::Make {
                jobs: false,
                args: vec!["check".into()],
            }],
        )
    }
}
