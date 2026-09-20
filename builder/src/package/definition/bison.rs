use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Package, Source};

pub struct Bison;

impl Package for Bison {
    fn name(&self) -> &'static str {
        "bison"
    }
    fn version(&self) -> &'static str {
        "3.8.2"
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
            url: "https://ftp.gnu.org/gnu/bison/bison-3.8.2.tar.xz",
            sha3_512: "8025a9224d91172b8262318bb8df7c6e1dd154407e0743855da383f22c5a0f76c3145687fa93fb0acbd71c26026d34876a05bd16234071638efffa08272df593",
            blake3: "9dd90be8df4d0474b941e2ca14ac76d11b7ccb46edb26344b60d866178bbcc9872561d223817a88247e5cc30f573f08f0243eb2f43c8a8928d0c6fc25f532d67",
            subdir: "bison-3.8.2",
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
        Recipe::autotools_pgo(&["--prefix=/usr"], pgo_workload())
    }
}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Cond, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        Step::Export {
            name: "BISON_PKGDATADIR",
            value: Arg::lit("\"$SOURCE/data\""),
        },
        Step::ForEach {
            var: "_y",
            items: argv(&[
                "\"$SOURCE\"/src/parse-gram.y",
                "\"$SOURCE\"/examples/*/calc.y",
                "\"$SOURCE\"/tests/*.y",
            ]),
            body: vec![Step::If {
                cond: Cond::FileExists(Arg::lit("\"$_y\"")),
                then: vec![exec(
                    Pipe::cmd(argv(&["./src/bison", "-o", "/tmp/bison-out.c", "\"$_y\""]))
                        .stderr(Arg::lit("/dev/null")),
                )],
                else_: vec![],
            }],
            stdout: None,
        },
    ]
}
