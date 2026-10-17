use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Isl;

impl Package for Isl {
    fn name(&self) -> &'static str {
        "isl"
    }
    fn version(&self) -> &'static str {
        "0.28"
    }
    fn license(&self) -> &'static str {
        "MIT"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://libisl.sourceforge.io/isl-0.28.tar.xz",
            sha3_512: "c99c4c44b8d026d42d781f950560754b4d9288cfed4a0e35ddf097f5c948a6445f751aec1b38166fe48684deec2a080c3e4f1d30b1b28a2c1e92ef96bd02fd30",
            blake3: "8a4c2423d60a79fd6ac3ff6942181d74d4c47a157a3113e880de4707d7ac75678673ce356a77d1834e1a8390e297cb75579002c6bf4c25624f3c44a6452b10d0",
            subdir: "isl-0.28",
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
                format!("--with-gmp-prefix={}", ctx.overlay("gmp")),
            ],
            Recipe::make_check_workload(),
        )
    }
}
