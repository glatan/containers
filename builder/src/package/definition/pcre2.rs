use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Package, Source};

pub struct Pcre2;

impl Package for Pcre2 {
    fn name(&self) -> &'static str {
        "pcre2"
    }
    fn version(&self) -> &'static str {
        "10.49"
    }
    fn license(&self) -> &'static str {
        "BSD-3-Clause"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/PCRE2Project/pcre2/releases/download/pcre2-10.49/pcre2-10.49.tar.bz2",
            sha3_512: "2731650b984a34e41cae95ffaaa4c06d817b4d933e74edd43abbc4fb44e524a452727ee0fd2a8f6011042dd0cd533d52f99210b4557d5ae37812e96991aaff43",
            blake3: "a1b29c27e5d4389e62a37f43df8fe8bcdcfb5dcd293f517ee6749f7d07bad8029cd44b0afd43a890fb9b4fd41b37ea3c66dd97fee3646078835c066b72d82912",
            subdir: "pcre2-10.49",
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
        Step::ForEach {
            var: "_input",
            items: argv(&[
                "\"$SOURCE\"/testdata/testinput1",
                "\"$SOURCE\"/testdata/testinput2",
                "\"$SOURCE\"/testdata/testinput4",
                "\"$SOURCE\"/testdata/testinput5",
            ]),
            body: vec![Step::If {
                cond: Cond::FileExists(Arg::lit("\"$_input\"")),
                then: vec![exec(Pipe::cmd(argv(&[
                    "./pcre2test",
                    "-q",
                    "\"$_input\"",
                    "/tmp/pcre2-out",
                ])))],
                else_: vec![],
            }],
            stdout: None,
        },
        exec(Pipe::cmd(argv(&[
            "./pcre2test",
            "-q",
            "\"$SOURCE/testdata/testinput1\"",
            "/dev/null",
        ]))),
    ]
}
