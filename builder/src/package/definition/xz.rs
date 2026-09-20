use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Xz;

impl Package for Xz {
    fn name(&self) -> &'static str {
        "xz"
    }
    fn version(&self) -> &'static str {
        "5.8.4"
    }
    fn license(&self) -> &'static str {
        "0BSD"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/tukaani-project/xz/releases/download/v5.8.4/xz-5.8.4.tar.xz",
            sha3_512: "56aff332bbc60da0cb0e075b20cf947f3a6ea2be500a4d05bd27b435c0a40005f90c9a4bed4124c625a374ba0b81473bdf0d7d89cbe381f5f5f36a7687f1e7c4",
            blake3: "4776832814549b37914485ee14979b68282d4ddf1c0b679c6f4e055a57666b96fcbdc36febaf229f670d823e253d1abcbbee9dd4163eec27106770a030609c05",
            subdir: "xz-5.8.4",
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

impl Bootstrap for Xz {}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Pipe, Redirect, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        Step::ForEach {
            var: "_i",
            items: argv(&["1", "2", "3", "4", "5", "6", "7", "8"]),
            body: vec![exec(Pipe::new(vec![
                argv(&[
                    "find",
                    "\"$SOURCE\"",
                    "-name",
                    "'*.c'",
                    "-o",
                    "-name",
                    "'*.h'",
                ]),
                argv(&["xargs", "cat"]),
            ]))],
            stdout: Some(Redirect {
                target: Arg::lit("/tmp/pgo-input"),
                append: false,
            }),
        },
        Step::ForEach {
            var: "_opt",
            items: argv(&["-0", "-6", "-9e"]),
            body: vec![exec(
                Pipe::new(vec![
                    argv(&["./src/xz/xz", "-T4", "\"$_opt\"", "-c", "/tmp/pgo-input"]),
                    argv(&["./src/xz/xz", "-d", "-c"]),
                ])
                .stdout(Arg::lit("/dev/null")),
            )],
            stdout: None,
        },
    ]
}
