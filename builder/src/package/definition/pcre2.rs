use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Pcre2;

impl Package for Pcre2 {
    fn name(&self) -> &'static str {
        "pcre2"
    }
    fn version(&self) -> &'static str {
        "10.48"
    }
    fn license(&self) -> &'static str {
        "BSD-3-Clause"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/PCRE2Project/pcre2/releases/download/pcre2-10.48/pcre2-10.48.tar.bz2",
            sha3_512: "445ca5ab94350b9c4307405350758e0c148c92160329b0443159274ce12bae0f437a1d788fb566d7731081553a2a31cc9915055e38e4f97ae17b834d7a1c17e6",
            blake3: "5e84ce242f146b8a97ce10e3cf31ac4c2c2b23d67814fa1cf266636d0545f3a50f08634ee1a724793b20cdae76647113f56f13a814328e8c3de48a837168ca45",
            subdir: "pcre2-10.48",
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
