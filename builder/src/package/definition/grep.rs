use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Grep;

impl Package for Grep {
    fn name(&self) -> &'static str {
        "grep"
    }
    fn version(&self) -> &'static str {
        "3.12"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/grep/grep-3.12.tar.xz",
            sha3_512: "226b3cd6aeddf50e0f01581c0829033d6a75e4d9ab101252dabbd3fbfc17f4f6c0ce9ec9381bbd00ef6c902f0c7773a24358bf63e37f3112221652d060ef6e1e",
            blake3: "e15c2e03ff6e9e7e8fb7c726539e10b6ff17ecfc3d6961f7419e948d31043f4f953d1acd1988861931ee051d8764723d82bb4c01cd5ee2890d2e9ff7d0e88c16",
            subdir: "grep-3.12",
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
    use crate::package::recipe::{Arg, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        exec(
            Pipe::cmd(argv(&[
                "./src/grep",
                "-r",
                "-E",
                "'int|char'",
                "\"$SOURCE\"",
            ]))
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::cmd(argv(&[
                "./src/grep",
                "-r",
                "-F",
                "'include'",
                "\"$SOURCE\"",
            ]))
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::cmd(argv(&[
                "./src/grep",
                "-r",
                "-i",
                "-w",
                "'main'",
                "\"$SOURCE\"",
            ]))
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::new(vec![
                argv(&["printf", "'a\\0b\\0c'"]),
                argv(&["./src/grep", "-a", "'b'"]),
            ])
            .stdout(Arg::lit("/dev/null")),
        ),
    ]
}

impl Bootstrap for Grep {}
