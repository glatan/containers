use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Sqlite;

impl Package for Sqlite {
    fn name(&self) -> &'static str {
        "sqlite"
    }
    fn version(&self) -> &'static str {
        "3.53.4"
    }
    fn license(&self) -> &'static str {
        "blessing"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://www.sqlite.org/2026/sqlite-autoconf-3530400.tar.gz",
            sha3_512: "2b9595cc6142a529462577b1d120d20a06390402efe97fd88f0a49ec915835853339775a8ff2eaee25c3d46b372724fb77a11cf4490d34439d10b198e59d3380",
            blake3: "e74b1d27f5053bd29ee7e930ad71d99126b59e4f1a45493440c72d2228e32412adad771dccb35bc3ed5990e3412e7bb9e50539bfd7805d978fbdb7e84259b2a6",
            subdir: "sqlite-autoconf-3530400",
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
