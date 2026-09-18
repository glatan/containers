use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Package, Source};

pub struct Tar;

impl Package for Tar {
    fn name(&self) -> &'static str {
        "tar"
    }
    fn version(&self) -> &'static str {
        "1.35"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/tar/tar-1.35.tar.xz",
            sha3_512: "d075aad9bfe5bde143c87388e3b4995775f0b40016e851d1243cc3b6a59fd65c5eeb8a23624ee41b18999dbd850213cbff7ebfb256e85c3a8441c3aa2cd7747c",
            blake3: "55e3853752c56a607ae5dbc9e9190fe5d3f7599b76bb80abe0edba36d712c36bb60b803541ce46293cdf80cb80b8c84fd6940ca9c446fd5db3b04c7ee16664bc",
            subdir: "tar-1.35",
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
        crate::flags::Pgo::Profiled(PGO_WORKLOAD)
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo(&["--prefix=/usr", "--disable-acl"], PGO_WORKLOAD)
    }
}

const PGO_WORKLOAD: &str = r#"mkdir -p /tmp/tar-out
./src/tar -cf /tmp/pgo.tar -C "$SOURCE" .
./src/tar -tf /tmp/pgo.tar > /dev/null
./src/tar -xf /tmp/pgo.tar -C /tmp/tar-out
./src/tar -cf - -C "$SOURCE" . | ./src/tar -tf - > /dev/null"#;

impl Bootstrap for Tar {}
