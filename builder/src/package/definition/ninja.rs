use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir};
use crate::package::{Package, Source};

pub struct Ninja;

impl Package for Ninja {
    fn name(&self) -> &'static str {
        "ninja"
    }
    fn version(&self) -> &'static str {
        "1.13.2"
    }
    fn license(&self) -> &'static str {
        "Apache-2.0"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/ninja-build/ninja/archive/refs/tags/v1.13.2.tar.gz",
            sha3_512: "4d48df9a0ea4a0b74ac6ec6206ef4743dfabe8acabffb69d66aeee15f9608414f7ba6a4f755728c329fe3827abdbac9d58afbffff4b95af4cf82010f30838b6d",
            blake3: "b32100cf57fde2a19d3c242f363c308c3d7a7ed3167774eefcbd107baada26e4fb5240af8ae22fde9d1cec079f5001098a30bc90f4fb3b5303c3bccd636fe161",
            subdir: "ninja-1.13.2",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(target, &[crate::package::gcc_dep(target)])
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }
    /// `configure.py --bootstrap` の実行に必要（成果物にはリンクしない）。
    fn host_tools(&self, _target: crate::target::Target) -> Vec<&'static str> {
        vec!["python"]
    }

    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let cmd = |args: Vec<Arg>| Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(args),
        };
        let mut steps = Steps::new(Env::Autotools).workdir(Workdir::InSource);
        if ctx.cross() {
            steps.steps.push(cmd(vec![
                Arg::lit("python3"),
                Arg::lit("configure.py"),
                Arg::lit("--bootstrap"),
            ]));
        } else {
            // instrumented bootstrap → ninja で自身を rebuild（workload）→
            // clean → -fprofile-use bootstrap。
            steps.steps.extend([
                Step::Export {
                    name: "PGODIR",
                    value: Arg::lit("/tmp/pgo"),
                },
                cmd(vec![
                    Arg::lit("rm"),
                    Arg::lit("-rf"),
                    Arg::quoted(Arg::Var("PGODIR")),
                ]),
                cmd(vec![
                    Arg::lit("mkdir"),
                    Arg::lit("-p"),
                    Arg::quoted(Arg::Var("PGODIR")),
                ]),
                Step::Export {
                    name: "_PGO_CXXFLAGS",
                    value: Arg::quoted(Arg::lit("${CXXFLAGS:-}")),
                },
                Step::Export {
                    name: "_PGO_LDFLAGS",
                    value: Arg::quoted(Arg::lit("${LDFLAGS:-}")),
                },
                Step::Export {
                    name: "CXXFLAGS",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_PGO_CXXFLAGS"),
                        Arg::lit(" -fprofile-generate=${PGODIR} -fprofile-update=atomic"),
                    ])),
                },
                Step::Export {
                    name: "LDFLAGS",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_PGO_LDFLAGS"),
                        Arg::lit(" -fprofile-generate"),
                    ])),
                },
                cmd(vec![
                    Arg::lit("python3"),
                    Arg::lit("configure.py"),
                    Arg::lit("--bootstrap"),
                ]),
                Step::Or {
                    lhs: vec![
                        cmd(vec![Arg::lit("./ninja"), Arg::lit("--version")]),
                        // bootstrap 直後は ninja が最新なので、obj を消して
                        // 再ビルドさせる。
                        cmd(vec![Arg::lit("rm"), Arg::lit("-f"), Arg::lit("build/*.o")]),
                        cmd(vec![
                            Arg::lit("./ninja"),
                            Arg::nproc_jobs("-j"),
                            Arg::lit("ninja"),
                        ]),
                    ],
                    rhs: vec![cmd(vec![Arg::lit("true")])],
                },
                cmd(vec![
                    Arg::lit("rm"),
                    Arg::lit("-rf"),
                    Arg::lit("build"),
                    Arg::lit("ninja"),
                ]),
                Step::Export {
                    name: "CXXFLAGS",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_PGO_CXXFLAGS"),
                        Arg::lit(" -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"),
                    ])),
                },
                Step::Export {
                    name: "LDFLAGS",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_PGO_LDFLAGS"),
                        Arg::lit(" -fprofile-use"),
                    ])),
                },
                cmd(vec![
                    Arg::lit("python3"),
                    Arg::lit("configure.py"),
                    Arg::lit("--bootstrap"),
                ]),
            ]);
        }
        steps.steps.push(Step::Install {
            src: "ninja".into(),
            dst: "\"$OUT\"/usr/bin/ninja".into(),
            mode: "755".into(),
        });
        Recipe::Steps(steps)
    }
}
