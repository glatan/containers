use crate::flags::Flag;
use crate::package::recipe::{
    Arg, BuildContext, Env, Pipe, Recipe, Redirect, Step, Steps, Workdir, argv,
};
use crate::package::{Dependency, Package, Source, dep};
use crate::target::Target;

pub struct ZlibNg;

impl Package for ZlibNg {
    fn name(&self) -> &'static str {
        "zlib-ng"
    }

    fn version(&self) -> &'static str {
        "2.3.3"
    }

    fn license(&self) -> &'static str {
        "Zlib"
    }

    fn catalog(&self) -> bool {
        true
    }

    /// シード（Arch base-devel）上のビルドには cmake が必要。
    fn pacman(&self) -> &'static [&'static str] {
        &["cmake"]
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/zlib-ng/zlib-ng/archive/refs/tags/2.3.3.tar.gz",
            sha3_512: "b045a4d3358f73db5566951944186293b9a99c5bc9d395866bc0a539617b9584b1499d3d0e9a8943284a7b10a829902dd37e1d6ec55823e6203b000644ba0170",
            blake3: "103cd4e5aa69bb2d0383a242f231bccf64c8a79c21156e7c6187db3cebc1367dc975216cfa092b7d37cdfb303e1fc9cdb2b63d6ef59ce6dc662fbe89286ada42",
            subdir: "zlib-ng-2.3.3",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    /// cmake ビルド系で cmake を実行するため必須の HOST 依存。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        deps.push(dep("cmake", "4.4.3"));
        deps
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        // configure_cmake() シェル関数を Rust 側の呼び出しへ展開し、
        // CMAKE_CROSS の実行時判定は ctx.cross() で静的に解決する。
        let mut cross_args: Vec<Arg> = Vec::new();
        if ctx.cross() {
            let arch = ctx.triple.split('-').next().unwrap_or(&ctx.triple);
            cross_args = vec![
                Arg::lit("-DCMAKE_SYSTEM_NAME=Linux"),
                Arg::owned(format!("-DCMAKE_SYSTEM_PROCESSOR={arch}")),
                Arg::owned(format!("-DCMAKE_C_COMPILER_TARGET={}", ctx.triple)),
            ];
        }
        let configure_cmake =
            |dir: Arg, testing: &'static str, cflags: Vec<Arg>, ldflags: Vec<Arg>| -> Step {
                let mut args = cross_args.clone();
                args.extend(argv(&[
                    "-DCMAKE_BUILD_TYPE=Release",
                    "-DCMAKE_INSTALL_PREFIX=/usr",
                    "-DCMAKE_INSTALL_LIBDIR=lib64",
                    "-DZLIB_COMPAT=OFF",
                    "-DWITH_GTEST=OFF",
                ]));
                args.push(Arg::Concat(vec![
                    Arg::lit("-DBUILD_TESTING="),
                    Arg::lit(testing),
                ]));
                args.push(Arg::lit("-DBUILD_SHARED_LIBS=ON"));
                for (flag, var) in [
                    ("-DCMAKE_C_COMPILER=", "CC"),
                    ("-DCMAKE_AR=", "AR"),
                    ("-DCMAKE_NM=", "NM"),
                    ("-DCMAKE_RANLIB=", "RANLIB"),
                ] {
                    args.push(Arg::Concat(vec![
                        Arg::lit(flag),
                        Arg::quoted(Arg::Var(var)),
                    ]));
                }
                args.push(Arg::Concat(vec![
                    Arg::lit("-DCMAKE_C_FLAGS="),
                    Arg::quoted(Arg::Concat(cflags)),
                ]));
                for flag in ["-DCMAKE_EXE_LINKER_FLAGS=", "-DCMAKE_SHARED_LINKER_FLAGS="] {
                    args.push(Arg::Concat(vec![
                        Arg::lit(flag),
                        Arg::quoted(Arg::Concat(ldflags.clone())),
                    ]));
                }
                Step::Cmake {
                    source_dir: Arg::quoted(Arg::Var("SOURCE")),
                    build_dir: dir,
                    args,
                    env: vec![],
                }
            };
        let build_dir = || Arg::quoted(Arg::Var("BUILD"));
        let gen_dir = || Arg::quoted(Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/gen")]));
        let cmake_build = |dir: Arg| Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("cmake"),
                Arg::lit("--build"),
                dir,
                Arg::nproc_jobs("-j"),
            ]),
        };
        let mut steps = Steps::new(Env::None).workdir(Workdir::None);
        if !ctx.cross() {
            // ネイティブのみ PGO。instrumented ツリーで minideflate を動かし、
            // 別ツリーで -fprofile-use の成果物を作る。
            let minideflate = || {
                Arg::quoted(Arg::Concat(vec![
                    Arg::Var("BUILD"),
                    Arg::lit("/gen/minideflate"),
                ]))
            };
            steps = steps
                .step(Step::Export {
                    name: "PGODIR",
                    value: Arg::lit("/tmp/pgo"),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("rm"),
                        Arg::lit("-rf"),
                        Arg::quoted(Arg::Var("PGODIR")),
                    ]),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("mkdir"),
                        Arg::lit("-p"),
                        Arg::quoted(Arg::Var("PGODIR")),
                    ]),
                })
                .step(configure_cmake(
                    gen_dir(),
                    "ON",
                    vec![
                        Arg::Var("CFLAGS"),
                        Arg::lit(" -fprofile-generate="),
                        Arg::Var("PGODIR"),
                        Arg::lit(" -fprofile-update=atomic"),
                    ],
                    vec![Arg::Var("LDFLAGS"), Arg::lit(" -fprofile-generate")],
                ))
                .step(cmake_build(gen_dir()))
                .step(Step::Set {
                    name: "PGO_LLP_SAVE",
                    value: Arg::Var("LD_LIBRARY_PATH"),
                })
                .step(Step::Export {
                    name: "LD_LIBRARY_PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("BUILD"),
                        Arg::lit("/gen"),
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
                                pipe: Pipe::cmd(vec![
                                    Arg::lit("cat"),
                                    Arg::Concat(vec![
                                        Arg::quoted(Arg::Var("SOURCE")),
                                        Arg::lit("/*.c"),
                                    ]),
                                    Arg::Concat(vec![
                                        Arg::quoted(Arg::Var("SOURCE")),
                                        Arg::lit("/*.h"),
                                    ]),
                                ]),
                            }],
                            stdout: Some(Redirect {
                                target: Arg::lit("/tmp/pgo-input"),
                                append: false,
                            }),
                        },
                        Step::ForEach {
                            var: "_level",
                            items: argv(&["1", "6", "9"]),
                            body: vec![Step::Exec {
                                env: vec![],
                                pipe: Pipe::new(vec![
                                    vec![
                                        minideflate(),
                                        Arg::lit("-c"),
                                        Arg::quoted(Arg::Concat(vec![
                                            Arg::lit("-"),
                                            Arg::Var("_level"),
                                        ])),
                                    ],
                                    vec![minideflate(), Arg::lit("-d"), Arg::lit("-c")],
                                ])
                                .stdin(Arg::lit("/tmp/pgo-input"))
                                .stdout(Arg::lit("/dev/null")),
                            }],
                            stdout: None,
                        },
                    ],
                    rhs: vec![Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(argv(&["true"])),
                    }],
                })
                .step(Step::Export {
                    name: "LD_LIBRARY_PATH",
                    value: Arg::Var("PGO_LLP_SAVE"),
                })
                .step(configure_cmake(
                    build_dir(),
                    "OFF",
                    vec![
                        Arg::Var("CFLAGS"),
                        Arg::lit(" -fprofile-use="),
                        Arg::Var("PGODIR"),
                        Arg::lit(
                            " -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch",
                        ),
                    ],
                    vec![Arg::Var("LDFLAGS"), Arg::lit(" -fprofile-use")],
                ));
        } else {
            steps = steps.step(configure_cmake(
                build_dir(),
                "OFF",
                vec![Arg::Var("CFLAGS")],
                vec![Arg::Var("LDFLAGS")],
            ));
        }
        Recipe::Steps(steps.step(cmake_build(build_dir())).step(Step::Exec {
            env: vec![("DESTDIR", Arg::quoted(Arg::Var("OUT")))],
            pipe: Pipe::cmd(vec![Arg::lit("cmake"), Arg::lit("--install"), build_dir()]),
        }))
    }
}
