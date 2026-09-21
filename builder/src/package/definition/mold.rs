use crate::flags::{Flag, Ldflag};
use crate::package::recipe::{
    Arg, BuildContext, Cond, Env, Pipe, Recipe, Step, Steps, Workdir, argv,
};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

pub struct Mold;

impl Package for Mold {
    fn name(&self) -> &'static str {
        "mold"
    }

    fn version(&self) -> &'static str {
        "2.42.1"
    }

    fn license(&self) -> &'static str {
        "MIT"
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/rui314/mold/archive/refs/tags/v2.42.1.tar.gz",
            sha3_512: "893446caeb9a904d3d796a8fc3f5c6f6361b7f2ea0d164ee56fcd48a8976857984e10faf51bb4610808faf43d87dd73bb4d02bd022dc6b93b42f80727e258ef4",
            blake3: "ab64ef69b9c842942058e678423ce9fb3901f66725295fbd4f426f14af549f62d6f818fa4161ae6a6bdec58d4f27485f65b2ebfb3374e0f3714a81066555ceca",
            subdir: "mold-2.42.1",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(
            target,
            &[
                crate::package::gcc_dep(target),
                Dependency {
                    name: "zlib",
                    version: "1.3.2",
                },
                Dependency {
                    name: "zstd",
                    version: "1.5.7",
                },
            ],
        )
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![
            crate::package::LINUX_HEADERS,
            Dependency {
                name: "cmake",
                version: "4.4.3",
            },
        ]
    }

    fn flags(&self, target: Target) -> Flag {
        // mold 自身は -fuse-ld=mold でリンクできないため除外する。
        // また bundled TBB が enum の値数違い等の実際の ODR 違反を持ち、
        // さらに LTO オブジェクトのリンクではドライバ内部で `-r` が
        // 使われ spec 由来の `-fhardened` が抑止不可の警告を出す
        // （cmake の cm_cxx_features.cmake が "warning" を機能なしと
        // 誤判定する）。mold は cmake ビルドなので LTO を外す。
        use crate::flags::Cflag;
        Flag::new(
            crate::flags::tool_cflags(target)
                .into_iter()
                .filter(|flag| {
                    !matches!(
                        *flag,
                        Cflag::Lto | Cflag::WerrorOdr | Cflag::WerrorLtoTypeMismatch
                    )
                })
                .collect(),
            crate::flags::tool_ldflags(target)
                .into_iter()
                .filter(|flag| {
                    !matches!(
                        *flag,
                        Ldflag::Lto
                            | Ldflag::WerrorOdr
                            | Ldflag::WerrorLtoTypeMismatch
                            | Ldflag::FuseLdMold
                    )
                })
                .collect(),
        )
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        // CMAKE_CROSS の実行時判定は ctx.cross() で静的に解決する。
        let mut cross_args: Vec<Arg> = Vec::new();
        if ctx.cross() {
            let arch = ctx.triple.split('-').next().unwrap_or(&ctx.triple);
            cross_args = vec![
                Arg::lit("-DCMAKE_SYSTEM_NAME=Linux"),
                Arg::owned(format!("-DCMAKE_SYSTEM_PROCESSOR={arch}")),
                Arg::owned(format!("-DCMAKE_C_COMPILER_TARGET={}", ctx.triple)),
                Arg::owned(format!("-DCMAKE_CXX_COMPILER_TARGET={}", ctx.triple)),
            ];
        }
        // configure_mold() シェル関数を Rust 側の呼び出しへ展開。
        let configure_mold =
            |dir: Arg, cflags: Vec<Arg>, cxxflags: Vec<Arg>, ldflags: Vec<Arg>| -> Step {
                let mut cmd = argv(&["cmake", "-S"]);
                cmd.push(Arg::quoted(Arg::Var("SOURCE")));
                cmd.push(Arg::lit("-B"));
                cmd.push(dir);
                cmd.extend(cross_args.iter().cloned());
                cmd.extend(argv(&[
                    "-DCMAKE_BUILD_TYPE=Release",
                    "-DCMAKE_INSTALL_PREFIX=/usr",
                ]));
                for (flag, var) in [
                    ("-DCMAKE_C_COMPILER=", "CC"),
                    ("-DCMAKE_CXX_COMPILER=", "CXX"),
                ] {
                    cmd.push(Arg::Concat(vec![
                        Arg::lit(flag),
                        Arg::quoted(Arg::Var(var)),
                    ]));
                }
                cmd.push(Arg::Concat(vec![
                    Arg::lit("-DCMAKE_C_FLAGS="),
                    Arg::quoted(Arg::Concat(cflags)),
                ]));
                cmd.push(Arg::Concat(vec![
                    Arg::lit("-DCMAKE_CXX_FLAGS="),
                    Arg::quoted(Arg::Concat(cxxflags)),
                ]));
                for flag in ["-DCMAKE_EXE_LINKER_FLAGS=", "-DCMAKE_SHARED_LINKER_FLAGS="] {
                    cmd.push(Arg::Concat(vec![
                        Arg::lit(flag),
                        Arg::quoted(Arg::Concat(ldflags.clone())),
                    ]));
                }
                cmd.extend(argv(&[
                    "-DMOLD_USE_MIMALLOC=ON",
                    "-DMOLD_USE_SYSTEM_MIMALLOC=OFF",
                    "-DMOLD_USE_SYSTEM_TBB=OFF",
                ]));
                Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(cmd),
                }
            };
        let gen_dir = || Arg::quoted(Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/gen")]));
        let use_dir = || Arg::quoted(Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/use")]));
        let build_dir = || Arg::quoted(Arg::Var("BUILD"));
        let cmake_build = |dir: Arg| Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("cmake"),
                Arg::lit("--build"),
                dir,
                Arg::nproc_jobs("-j"),
            ]),
        };
        let cmake_install = |dir: Arg| Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("cmake"),
                Arg::lit("--install"),
                dir,
                Arg::lit("--prefix"),
                Arg::quoted(Arg::Concat(vec![Arg::Var("OUT"), Arg::lit("/usr")])),
            ]),
        };
        // 依存の /dep/{zlib,zstd}/usr を CMAKE_PREFIX_PATH へ連結する。
        let mut steps = Steps::new(Env::None)
            .workdir(Workdir::None)
            .step(Step::Set {
                name: "CMAKE_PREFIX",
                value: Arg::lit(""),
            })
            .step(Step::ForEach {
                var: "_dep",
                items: argv(&["zlib", "zstd"]),
                body: vec![Step::If {
                    cond: Cond::DirExists(Arg::Concat(vec![
                        Arg::lit("/dep/"),
                        Arg::Var("_dep"),
                        Arg::lit("/usr"),
                    ])),
                    then: vec![Step::Set {
                        name: "CMAKE_PREFIX",
                        value: Arg::quoted(Arg::Concat(vec![
                            Arg::lit("${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/"),
                            Arg::Var("_dep"),
                            Arg::lit("/usr"),
                        ])),
                    }],
                    else_: vec![],
                }],
                stdout: None,
            })
            .step(Step::If {
                cond: Cond::EnvSet("CMAKE_PREFIX"),
                then: vec![Step::Export {
                    name: "CMAKE_PREFIX_PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("CMAKE_PREFIX"),
                        Arg::lit("${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"),
                    ])),
                }],
                else_: vec![],
            })
            .step(Step::If {
                cond: Cond::DirExists(Arg::lit("/dep/cmake/usr/bin")),
                then: vec![Step::Export {
                    name: "PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::lit("/dep/cmake/usr/bin:"),
                        Arg::Var("PATH"),
                    ])),
                }],
                else_: vec![],
            });
        if !ctx.cross() {
            // ネイティブのみ PGO。instrumented mold でリンクを実行して
            // プロファイルを集め、別ツリーで -fprofile-use の成果物を作る。
            let cc = || Arg::quoted(Arg::Var("CC"));
            let moldwork = |suffix: &'static str| {
                Arg::quoted(Arg::Concat(vec![
                    Arg::lit("/tmp/moldwork/"),
                    Arg::lit(suffix),
                ]))
            };
            let moldwork_i = |suffix: &'static str| {
                Arg::quoted(Arg::Concat(vec![
                    Arg::lit("/tmp/moldwork/t"),
                    Arg::Var("_i"),
                    Arg::lit(suffix),
                ]))
            };
            let mold_link = |output: Arg, inputs: Vec<Arg>, extra: Vec<Arg>| {
                let mut a = vec![cc(), Arg::lit("-B/tmp/moldbin"), Arg::lit("-fuse-ld=mold")];
                a.extend(extra);
                a.push(Arg::lit("-o"));
                a.push(output);
                a.extend(inputs);
                Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(a),
                }
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
                .step(configure_mold(
                    gen_dir(),
                    vec![
                        Arg::lit("${CFLAGS:-} -fprofile-generate="),
                        Arg::Var("PGODIR"),
                        Arg::lit(" -fprofile-update=atomic"),
                    ],
                    vec![
                        Arg::lit("${CXXFLAGS:-} -fprofile-generate="),
                        Arg::Var("PGODIR"),
                        Arg::lit(" -fprofile-update=atomic"),
                    ],
                    vec![Arg::lit("${LDFLAGS:-} -fprofile-generate")],
                ))
                .step(cmake_build(gen_dir()))
                .step(Step::Or {
                    lhs: vec![
                        Step::Exec {
                            env: vec![],
                            pipe: Pipe::cmd(argv(&[
                                "mkdir",
                                "-p",
                                "/tmp/moldbin",
                                "/tmp/moldwork",
                            ])),
                        },
                        Step::Exec {
                            env: vec![],
                            pipe: Pipe::cmd(vec![
                                Arg::lit("ln"),
                                Arg::lit("-sf"),
                                Arg::quoted(Arg::Concat(vec![
                                    Arg::Var("BUILD"),
                                    Arg::lit("/gen/mold"),
                                ])),
                                Arg::lit("/tmp/moldbin/ld.mold"),
                            ]),
                        },
                        Step::ForEach {
                            var: "_i",
                            items: argv(&["1", "2", "3", "4", "5"]),
                            body: vec![
                                Step::Exec {
                                    env: vec![],
                                    pipe: Pipe::new(vec![
                                        vec![
                                            Arg::lit("printf"),
                                            Arg::lit("'int fn%s(void){return %s;}\\n'"),
                                            Arg::quoted(Arg::Var("_i")),
                                            Arg::quoted(Arg::Var("_i")),
                                        ],
                                        vec![
                                            cc(),
                                            Arg::lit("-x"),
                                            Arg::lit("c"),
                                            Arg::lit("-"),
                                            Arg::lit("-c"),
                                            Arg::lit("-o"),
                                            moldwork_i(".o"),
                                        ],
                                    ]),
                                },
                                mold_link(
                                    moldwork_i(".so"),
                                    vec![moldwork_i(".o")],
                                    vec![Arg::lit("-shared")],
                                ),
                            ],
                            stdout: None,
                        },
                        Step::Exec {
                            env: vec![],
                            pipe: Pipe::new(vec![
                                vec![
                                    Arg::lit("printf"),
                                    Arg::lit(
                                        "'int main(void){return fn1();}\\nextern int fn1(void);\\n'",
                                    ),
                                ],
                                vec![
                                    cc(),
                                    Arg::lit("-x"),
                                    Arg::lit("c"),
                                    Arg::lit("-"),
                                    Arg::lit("-c"),
                                    Arg::lit("-o"),
                                    moldwork("main.o"),
                                ],
                            ]),
                        },
                        mold_link(
                            moldwork("prog"),
                            vec![moldwork("main.o"), moldwork("t1.o")],
                            vec![],
                        ),
                        mold_link(
                            moldwork("prog-pie"),
                            vec![moldwork("main.o"), moldwork("t1.o")],
                            vec![Arg::lit("-pie")],
                        ),
                    ],
                    rhs: vec![Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(argv(&["true"])),
                    }],
                })
                .step(configure_mold(
                    use_dir(),
                    vec![
                        Arg::lit("${CFLAGS:-} -fprofile-use="),
                        Arg::Var("PGODIR"),
                        Arg::lit(
                            " -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch",
                        ),
                    ],
                    vec![
                        Arg::lit("${CXXFLAGS:-} -fprofile-use="),
                        Arg::Var("PGODIR"),
                        Arg::lit(
                            " -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch",
                        ),
                    ],
                    vec![Arg::lit("${LDFLAGS:-} -fprofile-use")],
                ))
                .step(cmake_build(use_dir()))
                .step(cmake_install(use_dir()));
        } else {
            steps = steps
                .step(configure_mold(
                    build_dir(),
                    vec![Arg::lit("${CFLAGS:-}")],
                    vec![Arg::lit("${CXXFLAGS:-}")],
                    vec![Arg::lit("${LDFLAGS:-}")],
                ))
                .step(cmake_build(build_dir()))
                .step(cmake_install(build_dir()));
        }
        Recipe::Steps(steps)
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
}

impl Bootstrap for Mold {}
