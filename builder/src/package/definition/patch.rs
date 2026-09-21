use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Patch;

impl Package for Patch {
    fn name(&self) -> &'static str {
        "patch"
    }
    fn version(&self) -> &'static str {
        "2.8"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/patch/patch-2.8.tar.xz",
            sha3_512: "d18e96a61a152e03a84d36384931fa3ce01f36240414b2af3bbd5c7ba02a665251cb8300ad2e4d9f7387238e0a717294928f8a30343c7a8647a80e66f543f9b5",
            blake3: "d7dadf0ac720de7560f140a9021928bc0f8089182121c0e65c8bec115cf311240545c913ea3be61b067c31dd4787ac6b2fe7da2d18b1f4047b1ff65a9e55fa60",
            subdir: "patch-2.8",
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
        exec(Pipe::cmd(argv(&["mkdir", "-p", "/tmp/patch-work"]))),
        Step::Or {
            lhs: vec![exec(
                Pipe::cmd(argv(&[
                    "cp",
                    "-a",
                    "\"$SOURCE\"/src/*.c",
                    "/tmp/patch-work/",
                ]))
                .stderr(Arg::lit("/dev/null")),
            )],
            rhs: vec![exec(Pipe::cmd(argv(&[
                "cp",
                "-a",
                "\"$SOURCE\"/*.c",
                "/tmp/patch-work/",
            ])))],
        },
        Step::ForEach {
            var: "_f",
            items: argv(&["/tmp/patch-work/*.c"]),
            body: vec![
                exec(
                    Pipe::cmd(argv(&["sed", "'s/^int /INT /'", "\"$_f\""]))
                        .stdout(Arg::Concat(vec![Arg::Var("_f"), Arg::lit(".new")])),
                ),
                exec(
                    Pipe::new(vec![
                        argv(&["diff", "-u", "\"$_f\"", "\"$_f.new\""]),
                        argv(&["./src/patch", "--dry-run", "-s", "\"$_f\""]),
                    ])
                    .stdout(Arg::lit("/dev/null")),
                ),
                // `< <(diff …)` のプロセス置換はパイプラインと同じ意味。
                exec(
                    Pipe::new(vec![
                        argv(&["diff", "-u", "\"$_f\"", "\"$_f.new\""]),
                        argv(&["./src/patch", "-s", "-o", "\"$_f.out\"", "\"$_f\""]),
                    ])
                    .stdout(Arg::lit("/dev/null")),
                ),
            ],
            stdout: None,
        },
    ]
}

impl Bootstrap for Patch {}
