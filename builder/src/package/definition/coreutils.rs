use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Coreutils;

impl Package for Coreutils {
    fn name(&self) -> &'static str {
        "coreutils"
    }
    fn version(&self) -> &'static str {
        "9.12"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/coreutils/coreutils-9.12.tar.xz",
            sha3_512: "d6f1fbd33b86bce8c0eec25875c04192e70a5a946561e2ed5a8fd9a4214e309cb07e0cc4ce1f86b228beeedc4f7fbf1c1218bd5dd3a45a308edd1fefd4e4d82b",
            blake3: "420edafa8b231145442d248c00412727651f20435e85dc80fbf18074fc249624c6ff75aaed836074f668767844667fe0ccc3ffa59975cb5bda8e29348dcb2779",
            subdir: "coreutils-9.12",
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
        Recipe::autotools_pgo(
            &[
                "--prefix=/usr",
                "--disable-acl",
                "--disable-xattr",
                "--without-selinux",
            ],
            pgo_workload(),
        )
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
}

impl Bootstrap for Coreutils {}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        exec(Pipe::cmd(argv(&["./src/ls", "-lR", "\"$SOURCE\""])).stdout(Arg::lit("/dev/null"))),
        Step::Or {
            lhs: vec![exec(
                Pipe::cmd(argv(&["./src/sort", "\"$SOURCE\"/src/*.c"]))
                    .stdout(Arg::lit("/dev/null"))
                    .stderr(Arg::lit("/dev/null")),
            )],
            rhs: vec![exec(Pipe::cmd(argv(&["true"])))],
        },
        exec(Pipe::cmd(argv(&[
            "./src/cp",
            "-a",
            "\"$SOURCE\"",
            "/tmp/coreutils-copy",
        ]))),
        Step::Or {
            lhs: vec![exec(
                Pipe::cmd(argv(&["./src/wc", "-l", "\"$SOURCE\"/src/*.c"]))
                    .stdout(Arg::lit("/dev/null"))
                    .stderr(Arg::lit("/dev/null")),
            )],
            rhs: vec![exec(Pipe::cmd(argv(&["true"])))],
        },
        exec(
            Pipe::new(vec![
                argv(&["./src/cat", "\"$SOURCE\"/configure"]),
                argv(&["./src/md5sum"]),
            ])
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::new(vec![
                argv(&["find", "\"$SOURCE\"", "-name", "'*.h'", "-print0"]),
                argv(&["./src/sort", "-z"]),
            ])
            .stdout(Arg::lit("/dev/null")),
        ),
    ]
}
