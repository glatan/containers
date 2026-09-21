use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Sed;

impl Package for Sed {
    fn name(&self) -> &'static str {
        "sed"
    }
    fn version(&self) -> &'static str {
        "4.10"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/sed/sed-4.10.tar.xz",
            sha3_512: "a5ae1b5ce201013c51e4f6b8fd132d86f63087a5d5f0908d3cd8d6c03b9a49f67c24ce24d759599dda81e88b3b6a9d40a70e124e24ad53ce5fdee3864308a579",
            blake3: "b48115aadef05df0018c3081bc5d7d2d55bf455b0679c6bae665e8498df38be2ea4771112f4fb48783952aa8aef8f8bd184129228e26e95755ba247b97c60ea1",
            subdir: "sed-4.10",
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
                "-o",
                "-name",
                "'*.h'",
            ]))))],
            body: vec![exec(
                Pipe::cmd(argv(&[
                    "./sed/sed",
                    "-e",
                    "'s/int/INT/g'",
                    "-e",
                    "'s/return/RET/g'",
                    "\"$_f\"",
                ]))
                .stdout(Arg::lit("/dev/null")),
            )],
            stdout: None,
        },
        exec(
            Pipe::cmd(argv(&[
                "./sed/sed",
                "-n",
                "'1,50p'",
                "\"$SOURCE\"/configure",
            ]))
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::cmd(argv(&[
                "./sed/sed",
                "-e",
                "'y/abc/ABC/'",
                "-e",
                "'/^#/d'",
                "\"$SOURCE\"/config.h.in",
            ]))
            .stdout(Arg::lit("/dev/null"))
            .stderr(Arg::lit("/dev/null")),
        ),
    ]
}

impl Bootstrap for Sed {}
