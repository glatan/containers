use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Expat;

impl Package for Expat {
    fn name(&self) -> &'static str {
        "expat"
    }
    fn version(&self) -> &'static str {
        "2.8.4"
    }
    fn license(&self) -> &'static str {
        "MIT"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/libexpat/libexpat/releases/download/R_2_8_4/expat-2.8.4.tar.xz",
            sha3_512: "056d6b25263d95d7375e2fc14f22f161722558c7e695ca3fccc73eb2842bf921e2791fcc1104fd52b0e057dd3be00795af698af5c70a9d8a257b5d6123ac75cc",
            blake3: "1311a1adc8da8ffd10ca65db96be3abe9e7e9d83561d899bd0dbdab17ab599dbddde8e8e3a1a6b9a3223c184a3bf1e71d767750418b304be8b22e8e209348f54",
            subdir: "expat-2.8.4",
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
        Recipe::autotools_pgo(&["--prefix=/usr"], PGO_WORKLOAD)
    }
}

const PGO_WORKLOAD: &str = r#"[ -f "$SOURCE/doc/xmlwf.xml" ] && ./xmlwf/xmlwf "$SOURCE/doc/xmlwf.xml" > /dev/null
find "$SOURCE" -name '*.xml' | head -20 | while read -r _x; do
  ./xmlwf/xmlwf "$_x" > /dev/null 2>&1
done"#;
