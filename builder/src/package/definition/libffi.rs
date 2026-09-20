use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Libffi;

impl Package for Libffi {
    fn name(&self) -> &'static str {
        "libffi"
    }
    fn version(&self) -> &'static str {
        "3.8.0"
    }
    fn license(&self) -> &'static str {
        "MIT"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/libffi/libffi/releases/download/v3.8.0/libffi-3.8.0.tar.gz",
            sha3_512: "adad40a2b8be7e69762ba21e0d6d1a51da4ab55340ee2924cb9e74d292ee0fbcf9b9f52bc096c6d6ae285c39ca02fb12508add63e92c505efb0f7490483022ad",
            blake3: "df92f5720298710e22ffbfbb7d9a9969e90b1f95c01e02e6356a56de162b57ea6585c6fedb65ca37cf816e9d8f12dbadf706f75e266b04cdaab06ee4c270bb87",
            subdir: "libffi-3.8.0",
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

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo(&["--prefix=/usr"], Recipe::make_check_workload())
    }
}
