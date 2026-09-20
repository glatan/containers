use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Findutils;

impl Package for Findutils {
    fn name(&self) -> &'static str {
        "findutils"
    }
    fn version(&self) -> &'static str {
        "4.11.0"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/findutils/findutils-4.11.0.tar.xz",
            sha3_512: "0ffbc8f9fd502abc230fe2df5ebc68aa46297317a9c32eb024cdb044bf018c90bb6dbfeee030dfb5052cea60f8ecb062011b4554d63ea81ca76944406834d13c",
            blake3: "a043f0e36d43c1acbb773fb71bb05342c11f777a34cec2a4ed699c5e578281cd4672a4d409352085f08c1f5c587ddc995dd8322243d4c3a77bac293bad47782f",
            subdir: "findutils-4.11.0",
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
                "./find/find",
                "\"$SOURCE\"",
                "-name",
                "'*.c'",
                "-type",
                "f",
            ]))
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::cmd(argv(&[
                "./find/find",
                "\"$SOURCE\"",
                "-type",
                "f",
                "-size",
                "+10k",
                "-readable",
            ]))
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::cmd(argv(&[
                "./find/find",
                "\"$SOURCE\"",
                "-name",
                "'Make*'",
                "-o",
                "-name",
                "'*.am'",
            ]))
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::new(vec![
                argv(&[
                    "./find/find",
                    "\"$SOURCE\"",
                    "-type",
                    "f",
                    "-newer",
                    "\"$SOURCE\"/configure",
                    "-print0",
                ]),
                argv(&["./xargs/xargs", "-0", "echo"]),
            ])
            .stdout(Arg::lit("/dev/null")),
        ),
    ]
}

impl Bootstrap for Findutils {}
