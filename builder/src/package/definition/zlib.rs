use crate::flags::Flag;
use crate::package::recipe::{
    Arg, BuildContext, Cond, Env, Pipe, Recipe, Redirect, Step, Steps, Workdir, argv,
};
use crate::package::{Dependency, Package, Role, Source};
use crate::target::Target;

pub struct Zlib;

impl Package for Zlib {
    fn name(&self) -> &'static str {
        "zlib"
    }

    fn version(&self) -> &'static str {
        "1.3.2"
    }

    fn license(&self) -> &'static str {
        "Zlib"
    }

    fn role(&self) -> Role {
        Role::Catalog
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.gz",
            sha3_512: "860936cb0ddea209f9c14219dbf05efbef7ea83abe93a25bf39dd591a8f075ab5ad0c8225b1455def844d831f2d7b8f645704ebe387e630ad3075f765925dcc7",
            blake3: "ea79f8ae7c98d38ea360b70449fbb0052da4ff4a7201f03387113e5dab8543793587a6ee51d8307b5203aa03bf0a0053fca792f4c99d95c48aecc4b49359dc1b",
            subdir: "zlib-1.3.2",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        const CONFIGURE: &[&str] = &[
            "./configure",
            "--prefix=/usr",
            "--libdir=/usr/lib64",
            "--shared",
        ];
        let mut steps = Steps::new(Env::None)
            .workdir(Workdir::InSource)
            // sysroot:true — 検出・CFLAGS 付与は wrap_script が注入。
            // zlib の configure は $CC を直接使うため CC 側にも反映する。
            .step(Step::If {
                cond: Cond::EnvSet("SYSROOT"),
                then: vec![Step::Export {
                    name: "CC",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("CC"),
                        Arg::lit(" --sysroot="),
                        Arg::Var("SYSROOT"),
                    ])),
                }],
                else_: vec![],
            })
            .step(Step::Set {
                name: "PGO",
                value: Arg::lit("/tmp/zlib-pgo"),
            })
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["rm", "-rf", "\"$PGO\""])),
            })
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["mkdir", "-p", "\"$PGO\""])),
            });
        if ctx.cross() {
            // クロスではプロファイルを実行できない
            steps = steps
                .step(Step::Exec {
                    env: vec![("CHOST", Arg::quoted(Arg::Var("TRIPLE")))],
                    pipe: Pipe::cmd(argv(CONFIGURE)),
                })
                .step(Step::make(&[]));
        } else {
            steps = steps
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(CONFIGURE)),
                })
                .step(Step::Make {
                    jobs: true,
                    args: vec![
                        "CFLAGS=\"$CFLAGS -fprofile-generate=$PGO -fprofile-update=atomic\"".into(),
                        "LDFLAGS=\"$LDFLAGS -fprofile-generate\"".into(),
                    ],
                })
                .step(Step::Export {
                    name: "LD_LIBRARY_PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("PWD"),
                        Arg::lit("${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"),
                    ])),
                })
                .step(Step::Or {
                    lhs: vec![
                        Step::ForEach {
                            var: "_i",
                            items: argv(&[
                                "1", "2", "3", "4", "5", "6", "7", "8", "9", "10", "11", "12",
                                "13", "14", "15", "16",
                            ]),
                            body: vec![Step::Exec {
                                env: vec![],
                                pipe: Pipe::cmd(argv(&["cat", "./*.c", "./*.h"])),
                            }],
                            stdout: Some(Redirect {
                                target: Arg::lit("/tmp/zlib-pgo-input"),
                                append: false,
                            }),
                        },
                        Step::ForEach {
                            var: "_level",
                            items: argv(&["1", "6", "9"]),
                            body: vec![Step::Exec {
                                env: vec![],
                                pipe: Pipe::new(vec![
                                    argv(&["./minigzip", "-c", "\"-$_level\""]),
                                    argv(&["./minigzip", "-d", "-c"]),
                                ])
                                .stdin(Arg::lit("/tmp/zlib-pgo-input"))
                                .stdout(Arg::lit("/dev/null")),
                            }],
                            stdout: None,
                        },
                        Step::Exec {
                            env: vec![],
                            pipe: Pipe::cmd(argv(&["./example"])).stdout(Arg::lit("/dev/null")),
                        },
                    ],
                    rhs: vec![Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(argv(&["true"])),
                    }],
                })
                .step(Step::Make {
                    jobs: false,
                    args: vec!["distclean".to_owned()],
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(CONFIGURE)),
                })
                .step(Step::Make {
                    jobs: true,
                    args: vec![
                        "CFLAGS=\"$CFLAGS -fprofile-use=$PGO -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch\"".into(),
                        "LDFLAGS=\"$LDFLAGS -fprofile-use\"".into(),
                    ],
                });
        }
        Recipe::Steps(steps.step(Step::make_install(&[])))
    }
}
