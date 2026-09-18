use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};

pub struct Pkgconf;

impl Package for Pkgconf {
    fn name(&self) -> &'static str {
        "pkgconf"
    }
    fn version(&self) -> &'static str {
        "2.5.1"
    }
    fn license(&self) -> &'static str {
        "ISC"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://distfiles.ariadne.space/pkgconf/pkgconf-2.5.1.tar.xz",
            sha3_512: "9c27796bf156adec40eaf2a10be4f30c53ef8f99f810984116dfc77a2f8f0868a9c203337bbd6661ad08da58b6f192ce33a360476c181479062d804e25490974",
            blake3: "c524d73c938e7382d3c12c9cfba16ed32ee092ab21b3e9659a095b5628ea2e8854d574e1591345eb70d3eeb5e08cd99af007751a488adc9b16ad9d6dd24c7081",
            subdir: "pkgconf-2.5.1",
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

const PGO_WORKLOAD: &str = r#"find /dep /usr -name '*.pc' 2>/dev/null | while read -r _pc; do
  _dir=$(dirname "$_pc")
  _name=$(basename "$_pc" .pc)
  PKG_CONFIG_PATH="$_dir" ./pkgconf --cflags --libs "$_name" > /dev/null 2>&1
  PKG_CONFIG_PATH="$_dir" ./pkgconf --modversion "$_name" > /dev/null 2>&1
done"#;
