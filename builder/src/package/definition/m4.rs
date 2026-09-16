use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Package, Source};

pub struct M4;

impl Package for M4 {
    fn name(&self) -> &'static str {
        "m4"
    }
    fn version(&self) -> &'static str {
        "1.4.21"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/m4/m4-1.4.21.tar.xz",
            sha3_512: "b39e704ae1e37add21b68ff8754897cc1e2684181436f02d90de9bff292d27314b28fad194806ae4ea17a42296a2f024e9381e8aa4cdbbbf2104a63ca2c607fd",
            blake3: "a23e9503084fa4087f45bb7bb9c39f0cf8e0f046e0f94e55c40a8da124c1fd686bfa9a76df6ac536a55028b3fcc156494d0ff41c87c993715fbe3c99eb42986d",
            subdir: "m4-1.4.21",
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

impl Bootstrap for M4 {}
