use crate::package::recipe::{BuildContext, Recipe, Step};
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
        // MPFR 必須: MPFR なしでビルドした gawk 5.4.1 は `a > b` 比較で
        // 文字列値を数値化して破壊するバグがあり、gcc の opt-gather.awk が
        // "###" レコードを "0" に化けさせて optionlist/options.cc を破損する。
        crate::package::with_libc(
            target,
            &[
                crate::package::dep("gmp", "6.3.0"),
                crate::package::dep("mpfr", "4.2.2"),
            ],
        )
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
        Recipe::autotools_pgo(&["--prefix=/usr"], pgo_workload())
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        Step::ForEach {
            var: "_f",
            items: vec![Arg::Subst(Box::new(Pipe::cmd(argv(&[
                "find",
                "\"$SOURCE\"",
                "-name",
                "'*.c'",
            ]))))],
            body: vec![exec(
                Pipe::cmd(argv(&[
                    "./gawk",
                    "'{ n += NF; if ($0 ~ /return/) r++ } END { print n, r }'",
                    "\"$_f\"",
                ]))
                .stdout(Arg::lit("/dev/null")),
            )],
            stdout: None,
        },
        exec(Pipe::cmd(argv(&[
            "./gawk",
            "'BEGIN { for (i = 0; i < 100000; i++) { s = s i; x[i] = i * 2 } print length(s), length(x) }'",
        ]))),
        exec(
            Pipe::new(vec![
                argv(&["seq", "1", "50000"]),
                argv(&[
                    "./gawk",
                    "'{ sum += $1; a[NR] = $1 } END { asort(a); print sum, a[1], a[NR] }'",
                ]),
            ])
            .stdout(Arg::lit("/dev/null")),
        ),
    ]
}

impl Bootstrap for Gawk {}
