use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Package, Source};

pub struct Gawk;

impl Package for Gawk {
    fn name(&self) -> &'static str {
        "gawk"
    }
    fn version(&self) -> &'static str {
        "5.4.1"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/gawk/gawk-5.4.1.tar.xz",
            sha3_512: "886ec44952ec9f11ff0eeeeb113479d85d8763a91c83f55f5df7c7e96e71f4f9f20c37f478de6914ed456ff9c1cbc9c19b1a381c5959976d0cfa9b085677a60a",
            blake3: "d2f64ff7d253202c500da1d6c8b33b5f61693385d81cb5126ff546940872e3bcca174d16646dccd8a64da59a5c92123c265a4176d8efbafa414f8dc61bfdd81b",
            subdir: "gawk-5.4.1",
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

const PGO_WORKLOAD: &str = r#"find "$SOURCE" -name '*.c' | while read -r _f; do
  ./gawk '{ n += NF; if ($0 ~ /return/) r++ } END { print n, r }' "$_f" > /dev/null
done
./gawk 'BEGIN { for (i = 0; i < 100000; i++) { s = s i; x[i] = i * 2 } print length(s), length(x) }'
seq 1 50000 | ./gawk '{ sum += $1; a[NR] = $1 } END { asort(a); print sum, a[1], a[NR] }' > /dev/null"#;

impl Bootstrap for Gawk {}
