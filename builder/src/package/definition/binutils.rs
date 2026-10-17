//! binutils ツール固有の定義（`Prefixed{stage: Binutils}` から分離）。
use super::stages::dep;
use crate::package::LINUX_HEADERS;
use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Step, Steps, Workdir};
use crate::package::{Dependency, Source};
use crate::target::Target;

pub const VERSION: &str = "2.47";
pub const LICENSE: &str = "GPL-3.0-or-later";
pub const PACMAN_SEED: &[&str] = &[
    "gcc", "make", "bison", "flex", "texinfo", "zlib", "zstd", "mold",
];
pub const PACMAN: &[&str] = &["make", "bison", "flex", "texinfo"];

pub const SOURCE: Source = Source {
    url: "https://ftp.gnu.org/gnu/binutils/binutils-2.47.tar.xz",
    sha3_512: "b2a39d2a1bfff7f27c2619e15c5d9dbf0c7918a85120975f4146c84b4a7d660bd5dc3793ea6d009df89ed865aaef4362805e949438db0f8acf97fd113ca1a1f3",
    blake3: "05a142fb2225e617a3fa30224d5d275682e5cf03a62a15af804c03214cc3a5cfadec1215345da15c94ba7a2e13b9667c094e59a26c39412a47265724cbb62a8e",
    subdir: "binutils-2.47",
};

pub fn deps(target: Target) -> Vec<Dependency> {
    // glibc/zlib/zstd との循環は via(Seed) で断つ。シード相は distro
    // パッケージが担保し、env では成果物必須。
    let distro = |name: &'static str| -> &'static str {
        if matches!(target, Target::Amd64(_)) {
            name
        } else {
            ""
        }
    };
    vec![
        crate::package::glibc_dep(target).via(crate::package::Via::Seed(
            if matches!(target, Target::Amd64(_)) {
                "glibc"
            } else {
                target.glibc_package()
            },
        )),
        dep("zlib", "1.3.2").via(crate::package::Via::Seed(distro("zlib"))),
        dep("zstd", "1.5.7").via(crate::package::Via::Seed(distro("zstd"))),
    ]
}

/// linux-headers のみのビルド時依存（循環は `deps()` 側の via(Seed) で断つ）。
pub fn build_deps() -> Vec<Dependency> {
    vec![LINUX_HEADERS]
}

/// `--enable-cet` は x86_64 のみ。PGO も x86_64 triple のみ:
/// instrumented な as/ld/objdump をビルドホスト上で実行して
/// プロファイルを採取するため、生成バイナリがビルドホストで動かせる
/// ことが前提。Canadian（`$PACKAGE_HOST` != `$HOST_TRIPLE`）では
/// 生成バイナリが別 arch のため実行できず適用しない。
/// x86_64 triple の binutils は amd64 向けでのみ発火する。
pub fn recipe(ctx: &BuildContext) -> Steps {
    let cet = ctx.triple == "x86_64-linux-gnu";
    let pgo = cet && !ctx.cross() && !ctx.canadian();

    let configure = |env: Vec<(&'static str, Arg)>| -> Step {
        let mut args = vec![
            Arg::Opt("build", Box::new(Arg::Var("HOST_TRIPLE"))),
            // amd64 向けはビルド機で動くクロスツール（--host=$HOST_TRIPLE）。
            // 非 amd64 向けは成果物ターゲットの arch で動くバイナリを
            // canadian cross で組むため --host=$PACKAGE_HOST が正しい
            //（dep:true 依存が成果物ターゲットの glibc/zlib/zstd を供給
            // しており、プリリュードの CC も PACKAGE_HOST 向けクロス
            // コンパイラを指す。HOST_TRIPLE のままでは生成バイナリと
            // --host が不整合になる）。prefixed 名は --program-prefix
            // で明示する（x86_64 は HOST_TRIPLE==TRIPLE で自動付与が
            // 効かない）。
            Arg::Opt(
                "host",
                Box::new(if ctx.canadian() {
                    Arg::Var("PACKAGE_HOST")
                } else {
                    Arg::Var("HOST_TRIPLE")
                }),
            ),
            Arg::Opt("target", Box::new(Arg::Var("TRIPLE"))),
            Arg::Opt(
                "program-prefix",
                Box::new(Arg::Concat(vec![Arg::Var("TRIPLE"), Arg::lit("-")])),
            ),
            Arg::lit("--prefix=/usr"),
            Arg::Concat(vec![
                Arg::lit("--infodir=/usr/share/info/"),
                Arg::Var("TRIPLE"),
            ]),
            Arg::lit("--disable-nls"),
            Arg::lit("--disable-werror"),
            Arg::lit("--enable-plugins"),
            Arg::lit("--with-system-zlib"),
            Arg::lit("--without-debuginfod"),
            Arg::lit("--disable-gprofng"),
            Arg::lit("--enable-new-dtags"),
            Arg::lit("--enable-relro"),
            Arg::lit("--enable-default-execstack=no"),
            Arg::lit("--enable-default-hash-style=gnu"),
        ];
        if cet {
            args.push(Arg::lit("--enable-cet"));
        }
        Step::ConfigureScript {
            program: vec![Arg::Concat(vec![
                Arg::Var("SOURCE"),
                Arg::lit("/configure"),
            ])],
            args,
            env,
        }
    };
    let cmd = |argv: Vec<Arg>| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(argv),
    };

    let steps = Steps::new(Env::None)
        .sysroot(true)
        .workdir(Workdir::None)
        .step(Step::Export {
            name: "CC_FOR_BUILD",
            value: Arg::Concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit("-gcc")]),
        })
        .step(Step::Export {
            name: "CXX_FOR_BUILD",
            value: Arg::Concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit("-g++")]),
        });

    if !pgo {
        return steps
            .step(cmd(vec![
                Arg::lit("mkdir"),
                Arg::lit("-p"),
                Arg::Var("BUILD"),
            ]))
            .step(Step::Cd(Arg::Var("BUILD")))
            .step(configure(vec![]))
            .step(Step::Make {
                jobs: true,
                args: vec![],
            })
            .step(Step::MakeInstall { vars: vec![] });
    }

    let gen_env = |var: &'static str| -> (&'static str, Arg) {
        (
            var,
            Arg::quoted(Arg::Concat(vec![
                Arg::owned(format!("${{{var}:-}} ")),
                Arg::lit(if var == "LDFLAGS" {
                    "-fprofile-generate"
                } else {
                    "-fprofile-generate=${PGODIR} -fprofile-update=atomic"
                }),
            ])),
        )
    };
    let use_env = |var: &'static str| -> (&'static str, Arg) {
        (
            var,
            Arg::quoted(Arg::Concat(vec![
                Arg::owned(format!("${{{var}:-}} ")),
                Arg::lit(if var == "LDFLAGS" {
                    "-fprofile-use"
                } else {
                    "-fprofile-use=${PGODIR} -fprofile-correction \
                     -Wno-missing-profile -Wno-error=coverage-mismatch"
                }),
            ])),
        )
    };

    let var_i = Arg::Var("_i");
    let workload = vec![
        Step::Set {
            name: "_as",
            value: Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/gen/gas/as-new")]),
        },
        Step::Set {
            name: "_ld",
            value: Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/gen/ld/ld-new")]),
        },
        Step::Set {
            name: "_objdump",
            value: Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/gen/binutils/objdump")]),
        },
        cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            Arg::lit("/tmp/bu-work"),
        ]),
        Step::ForEach {
            var: "_i",
            items: vec![Arg::Subst(Box::new(Pipe::cmd(vec![
                Arg::lit("seq"),
                Arg::lit("1"),
                Arg::lit("64"),
            ])))],
            body: vec![
                Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("printf"),
                        Arg::quoted(Arg::lit(
                            ".text\\n.globl f%s\\nf%s:\\n  mov $%s, %%eax\\n  ret\\n",
                        )),
                        var_i.clone(),
                        var_i.clone(),
                        var_i.clone(),
                    ])
                    .stdout(Arg::lit("/tmp/bu-work/t.s")),
                },
                cmd(vec![
                    Arg::Var("_as"),
                    Arg::lit("-o"),
                    Arg::lit("/tmp/bu-work/t.o"),
                    Arg::lit("/tmp/bu-work/t.s"),
                ]),
                cmd(vec![
                    Arg::Var("_ld"),
                    Arg::lit("-r"),
                    Arg::lit("-o"),
                    Arg::Concat(vec![Arg::lit("/tmp/bu-work/t"), var_i, Arg::lit(".o")]),
                    Arg::lit("/tmp/bu-work/t.o"),
                ]),
            ],
            stdout: None,
        },
        cmd(vec![
            Arg::Var("_ld"),
            Arg::lit("-r"),
            Arg::lit("-o"),
            Arg::lit("/tmp/bu-work/all.o"),
            Arg::lit("/tmp/bu-work/t*.o"),
        ]),
        Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::Var("_objdump"),
                Arg::lit("-d"),
                Arg::lit("/tmp/bu-work/all.o"),
            ])
            .stdout(Arg::lit("/dev/null")),
        },
        Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::Var("_objdump"),
                Arg::lit("-h"),
                Arg::lit("/tmp/bu-work/all.o"),
            ])
            .stdout(Arg::lit("/dev/null")),
        },
    ];

    steps
        .step(Step::Export {
            name: "PGODIR",
            value: Arg::lit("/tmp/pgo"),
        })
        .step(cmd(vec![
            Arg::lit("rm"),
            Arg::lit("-rf"),
            Arg::Var("PGODIR"),
        ]))
        .step(cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            Arg::Var("PGODIR"),
            Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/gen")]),
            Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/final")]),
        ]))
        .step(Step::Cd(Arg::Concat(vec![
            Arg::Var("BUILD"),
            Arg::lit("/gen"),
        ])))
        .step(configure(vec![
            gen_env("CFLAGS"),
            gen_env("CXXFLAGS"),
            gen_env("LDFLAGS"),
        ]))
        .step(Step::Make {
            jobs: true,
            args: vec![],
        })
        .step(Step::Or {
            lhs: workload,
            rhs: vec![cmd(vec![Arg::lit("true")])],
        })
        .step(Step::Cd(Arg::Concat(vec![
            Arg::Var("BUILD"),
            Arg::lit("/final"),
        ])))
        .step(configure(vec![
            use_env("CFLAGS"),
            use_env("CXXFLAGS"),
            use_env("LDFLAGS"),
        ]))
        .step(Step::Make {
            jobs: true,
            args: vec![],
        })
        .step(Step::MakeInstall { vars: vec![] })
}
