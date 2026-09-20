use crate::package::recipe::{Arg, BuildContext, Cond, Env, Pipe, Recipe, Step, Steps, Workdir};
use crate::package::{Package, Source};
use crate::target::Target;

pub struct Perl;

/// `grep -q "^K=" config.sh` → `sed -i` 上書き / `echo >>` 追記
/// （旧 `_patch_cfg` シェル関数の展開形）。
fn patch_cfg(key: &str, value: &str) -> Step {
    Step::If {
        cond: Cond::Cmd(Pipe::cmd(vec![
            Arg::lit("grep"),
            Arg::lit("-q"),
            Arg::owned(format!("^{key}=")),
            Arg::lit("config.sh"),
        ])),
        then: vec![Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("sed"),
                Arg::lit("-i"),
                Arg::owned(format!("\"s|^{key}=.*|{key}='{value}'|\"")),
                Arg::lit("config.sh"),
            ]),
        }],
        else_: vec![Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("echo"),
                Arg::owned(format!("\"{key}='{value}'\"")),
            ])
            .append_stdout(Arg::lit("config.sh")),
        }],
    }
}

/// `grep -qE "^#define K( |$)"` → `sed -i` 上書き / `printf >>` 追記
/// （旧 `_inject_h` シェル関数の展開形）。
fn inject_h(key: &str, value: &str) -> Step {
    let arg = if value.contains('"') {
        format!("'{value}'")
    } else {
        format!("\"{value}\"")
    };
    Step::If {
        cond: Cond::Cmd(Pipe::cmd(vec![
            Arg::lit("grep"),
            Arg::lit("-qE"),
            Arg::owned(format!("\"^#define {key}([[:space:]]|$)\"")),
            Arg::lit("config.h"),
        ])),
        then: vec![Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("sed"),
                Arg::lit("-i"),
                Arg::owned(format!(
                    "\"s|^#define {key}[[:space:]].*|#define {key} {}\\t\\t/**/|\"",
                    value.replace('"', "\\\"")
                )),
                Arg::lit("config.h"),
            ]),
        }],
        else_: vec![Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("printf"),
                Arg::lit("'#define %s %s\\t\\t/**/\\n'"),
                Arg::owned(format!("\"{key}\"")),
                Arg::owned(arg),
            ])
            .append_stdout(Arg::lit("config.h")),
        }],
    }
}

fn export_concat(name: &'static str, head: Arg, tail: &'static str) -> Step {
    Step::Export {
        name,
        value: Arg::quoted(Arg::Concat(vec![head, Arg::lit(tail)])),
    }
}

/// Configure の cross 用 try-run 分岐を `run=:` で置き換える awk。
const AWK_SKIP_TRY: &str = r#"'
    /Skipping the try tests in the rest of Configure as no targethost was defined when cross-compiling/ {
      print
      print "         run=:"
      print "         to=:"
      print "         from=:"
      print "         ;;"
      skip=1
      next
    }
    skip {
      if ($0 ~ /^       ;;/) skip=0
      next
    }
    { print }
  '"#;

/// `./try` 実行を `if true` に置き換える awk。
const AWK_TRY_TRUE: &str = r#"'
    /if \$sh -c "\$run \.\/try / {
      print " if true; then"
      getline
      print " xxx=Ok"
      next
    }
    { print }
  '"#;

fn exec(pipe: Pipe) -> Step {
    Step::Exec { env: vec![], pipe }
}

fn cmd(args: Vec<Arg>) -> Step {
    exec(Pipe::cmd(args))
}

impl Package for Perl {
    fn name(&self) -> &'static str {
        "perl"
    }
    fn version(&self) -> &'static str {
        "5.42.3"
    }
    fn license(&self) -> &'static str {
        "Artistic-1.0-Perl OR GPL-1.0-or-later"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    /// クロス perl ビルドは HOST perl（miniperl）を使うため自分自身をマウントする。
    fn host_tools(&self, target: Target) -> Vec<&'static str> {
        if target == Target::BUILD_ENV {
            vec![]
        } else {
            vec!["perl"]
        }
    }

    fn source(&self) -> Source {
        Source {
            url: "https://www.cpan.org/src/5.0/perl-5.42.3.tar.xz",
            sha3_512: "c827af1973438b9173756f5c0b44e668d0443b2bba01b1edf748947c99a5660ecdf5119e0aeada306e47c848bdc2debe7226f45c38d7158b3cf280b1804021a8",
            blake3: "986547a8f7de7ad55e2d9877ec29461601b12e4642cf5806194f07b805fea547570e9bcbb2f52df883220190c1a891d1eba37a6f2d82f595b833e4223da68b88",
            subdir: "perl-5.42.3",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[crate::package::Dependency {
                name: "libxcrypt",
                version: "4.5.2",
            }],
        )
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
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let cross = ctx.cross();
        let mut steps = Steps::new(Env::Autotools).workdir(Workdir::None);

        // perl は autotools_configure を経由しないため、旧 AUTOTOOLS_ENV
        // 相当のフラグ sanitize を冒頭で export する。
        let sanitize = |var: &'static str| {
            Arg::Subst(Box::new(Pipe::cmd(vec![
                Arg::lit("sanitize_flags"),
                Arg::quoted(Arg::owned(format!("${{{var}:-}}"))),
            ])))
        };
        steps.steps.push(Step::Export {
            name: "CFLAGS",
            value: Arg::quoted(Arg::Concat(vec![
                sanitize("CFLAGS"),
                Arg::lit(" -std=gnu17"),
            ])),
        });
        steps.steps.push(Step::Export {
            name: "CXXFLAGS",
            value: Arg::quoted(sanitize("CXXFLAGS")),
        });
        steps.steps.push(Step::Export {
            name: "LDFLAGS",
            value: Arg::quoted(sanitize("LDFLAGS")),
        });

        steps.steps.push(Step::If {
            cond: Cond::DirExists(Arg::lit("/dep/libxcrypt/usr/include")),
            then: vec![
                export_concat(
                    "CFLAGS",
                    Arg::Var("CFLAGS"),
                    " -I/dep/libxcrypt/usr/include",
                ),
                export_concat(
                    "CPPFLAGS",
                    Arg::lit("${CPPFLAGS:-}"),
                    " -I/dep/libxcrypt/usr/include",
                ),
            ],
            else_: vec![],
        });
        let ldflags = |dir: &'static str| {
            export_concat(
                "LDFLAGS",
                Arg::Var("LDFLAGS"),
                match dir {
                    "lib64" => " -L/dep/libxcrypt/usr/lib64",
                    _ => " -L/dep/libxcrypt/usr/lib",
                },
            )
        };
        steps.steps.push(Step::If {
            cond: Cond::DirExists(Arg::lit("/dep/libxcrypt/usr/lib64")),
            then: vec![ldflags("lib64")],
            else_: vec![Step::If {
                cond: Cond::DirExists(Arg::lit("/dep/libxcrypt/usr/lib")),
                then: vec![ldflags("lib")],
                else_: vec![],
            }],
        });
        // in-source コピー（libxcrypt export 後に元スクリプトと同順で実行）。
        steps.steps.push(cmd(vec![
            Arg::lit("cp"),
            Arg::lit("-a"),
            Arg::Concat(vec![Arg::quoted(Arg::Var("SOURCE")), Arg::lit("/.")]),
            Arg::Concat(vec![Arg::quoted(Arg::Var("BUILD")), Arg::lit("/")]),
        ]));
        steps.steps.push(Step::Cd(Arg::quoted(Arg::Var("BUILD"))));
        steps.steps.push(Step::Set {
            name: "_sysroot",
            value: Arg::lit("/dep/${TRIPLE}-glibc/usr/${TRIPLE}"),
        });

        if cross {
            steps.steps.push(export_concat(
                "CFLAGS",
                Arg::Var("CFLAGS"),
                " -Wno-incompatible-pointer-types -Wno-int-conversion",
            ));
            // generate_uudmap はホストマシンで実行するためホスト CC。
            steps.steps.push(Step::Exec {
                env: vec![
                    ("LIBRARY_PATH", Arg::lit("")),
                    ("CFLAGS", Arg::lit("")),
                    ("LDFLAGS", Arg::lit("-fuse-ld=mold")),
                ],
                pipe: Pipe::cmd(vec![
                    Arg::quoted(Arg::Concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit("-gcc")])),
                    Arg::lit("-O2"),
                    Arg::lit("-pipe"),
                    Arg::lit("-o"),
                    Arg::lit("/tmp/generate_uudmap"),
                    Arg::lit("generate_uudmap.c"),
                ]),
            });
            // ホスト miniperl を環境分離したサブシェルでビルド。
            steps.steps.push(Step::Set {
                name: "_host",
                value: Arg::lit("/tmp/perl-host"),
            });
            steps.steps.push(cmd(vec![
                Arg::lit("rm"),
                Arg::lit("-rf"),
                Arg::quoted(Arg::Var("_host")),
            ]));
            steps.steps.push(cmd(vec![
                Arg::lit("mkdir"),
                Arg::lit("-p"),
                Arg::quoted(Arg::Var("_host")),
            ]));
            steps.steps.push(Step::Subshell(vec![
                Step::Cd(Arg::quoted(Arg::Var("_host"))),
                Step::Export {
                    name: "CC",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("HOST_TRIPLE"),
                        Arg::lit("-gcc"),
                    ])),
                },
                Step::Export {
                    name: "CXX",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("HOST_TRIPLE"),
                        Arg::lit("-g++"),
                    ])),
                },
                Step::Export {
                    name: "LDFLAGS",
                    value: Arg::quoted(Arg::lit("-fuse-ld=mold")),
                },
                Step::Unset("CFLAGS"),
                Step::Unset("CXXFLAGS"),
                Step::Unset("CPPFLAGS"),
                Step::Unset("CROSS"),
                Step::Unset("LIBRARY_PATH"),
                Step::Unset("COMPILER_PATH"),
                Step::Export {
                    name: "LD_LIBRARY_PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::lit("/dep/libxcrypt-host/usr/lib:/dep/libxcrypt-host/usr/lib64"),
                        Arg::lit("${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"),
                    ])),
                },
                Step::Export {
                    name: "PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::lit("/tmp/mold-ld-cc:/dep/mold/usr/bin:/dep/"),
                        Arg::Var("HOST_TRIPLE"),
                        Arg::lit("-gcc/usr/bin:/dep/"),
                        Arg::Var("HOST_TRIPLE"),
                        Arg::lit("-binutils/usr/bin:/usr/bin:/bin"),
                    ])),
                },
                cmd(vec![
                    Arg::Concat(vec![
                        Arg::quoted(Arg::Var("SOURCE")),
                        Arg::lit("/Configure"),
                    ]),
                    Arg::lit("-des"),
                    Arg::lit("-Dusedevel"),
                    Arg::lit("-Dmksymlinks"),
                    Arg::Concat(vec![Arg::lit("-Dcc="), Arg::quoted(Arg::Var("CC"))]),
                    Arg::Concat(vec![Arg::lit("-Dld="), Arg::quoted(Arg::Var("CC"))]),
                    Arg::lit("-Dprefix=/usr"),
                    Arg::lit("-Uusethreads"),
                    Arg::lit("-Dlibs='-lpthread -ldl -lm -lutil -lc'"),
                ]),
                cmd(vec![
                    Arg::lit("make"),
                    Arg::nproc_jobs("-j"),
                    Arg::lit("miniperl"),
                ]),
            ]));
            // Configure の try-run を cross 向けに潰す。
            steps.steps.push(exec(
                Pipe::cmd(vec![
                    Arg::lit("awk"),
                    Arg::lit(AWK_SKIP_TRY),
                    Arg::lit("Configure"),
                ])
                .stdout(Arg::lit("Configure.cross")),
            ));
            steps.steps.push(cmd(vec![
                Arg::lit("mv"),
                Arg::lit("Configure.cross"),
                Arg::lit("Configure"),
            ]));
            steps.steps.push(cmd(vec![
                Arg::lit("sed"),
                Arg::lit("-i"),
                Arg::lit("'s/&& \\.\\/try; then/\\&\\& true; then/'"),
                Arg::lit("Configure"),
            ]));
            steps.steps.push(exec(
                Pipe::cmd(vec![
                    Arg::lit("awk"),
                    Arg::lit(AWK_TRY_TRUE),
                    Arg::lit("Configure"),
                ])
                .stdout(Arg::lit("Configure.coherency")),
            ));
            steps.steps.push(cmd(vec![
                Arg::lit("mv"),
                Arg::lit("Configure.coherency"),
                Arg::lit("Configure"),
            ]));
            steps.steps.push(cmd(vec![
                Arg::lit("chmod"),
                Arg::lit("+x"),
                Arg::lit("Configure"),
            ]));
        }

        steps.steps.push(Step::If {
            cond: Cond::DirExists(Arg::quoted(Arg::Concat(vec![
                Arg::Var("_sysroot"),
                Arg::lit("/lib"),
            ]))),
            then: vec![Step::Export {
                name: "LIBRARY_PATH",
                value: Arg::quoted(Arg::Concat(vec![
                    Arg::Var("_sysroot"),
                    Arg::lit("/lib:"),
                    Arg::Var("_sysroot"),
                    Arg::lit("/lib64"),
                    Arg::lit("${LIBRARY_PATH:+:$LIBRARY_PATH}"),
                ])),
            }],
            else_: vec![],
        });
        steps.steps.push(Step::Export {
            name: "LD_LIBRARY_PATH",
            value: Arg::quoted(Arg::Concat(vec![
                Arg::lit("/dep/libxcrypt/usr/lib:/dep/libxcrypt/usr/lib64"),
                Arg::lit("${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"),
            ])),
        });

        let mut configure = vec![
            Arg::lit("./Configure"),
            Arg::lit("-des"),
            Arg::lit("-Dprefix=/usr"),
            Arg::lit("-Dvendorprefix=/usr"),
            Arg::lit("-Duse64bitall"),
            Arg::lit("-Dusethreads"),
            Arg::lit("-Duseshrplib"),
            Arg::Concat(vec![Arg::lit("-Dcc="), Arg::quoted(Arg::Var("CC"))]),
            Arg::Concat(vec![Arg::lit("-Dld="), Arg::quoted(Arg::Var("CC"))]),
            Arg::Concat(vec![
                Arg::lit("-Accflags="),
                Arg::quoted(Arg::Var("CFLAGS")),
            ]),
            Arg::Concat(vec![
                Arg::lit("-Aldflags="),
                Arg::quoted(Arg::Var("LDFLAGS")),
            ]),
        ];
        if cross {
            let byteorder = if ctx.triple.starts_with("s390x-") {
                "87654321"
            } else {
                "12345678"
            };
            for arg in [
                "-Dusecrosscompile",
                "-Dhostperl=/tmp/perl-host/miniperl",
                "-Dhostgenerate=/tmp/generate_uudmap",
                "-Dcharsize=1",
                "-Dshortsize=2",
                "-Dintsize=4",
                "-Dlongsize=8",
                "-Dlonglongsize=8",
                "-Dptrsize=8",
                "-Ddoublesize=8",
                "-Dnvsize=8",
                "-Duvsize=8",
                "-Divsize=8",
                "-Dcharbits=8",
                "-Dalignbytes=8",
                "-Dssizetype=ssize_t",
                "-Dsizetype=size_t",
                "-Dlseektype=off_t",
                "-Dlseeksize=8",
                "-Dtimetype=time_t",
                "-Dfpostype=fpos_t",
                "-Dfpossize=16",
                "-Dlongdblsize=16",
                "-Ddoublekind=3",
                "-Duseithreads",
                "-Dusemultiplicity",
                "-Duseperlio",
                "-Uebcdic",
                "-Dd_PRId64=define",
                "-Dd_PRIi64=define",
                "-Divdformat='\"ld\"'",
                "-Duvuformat='\"lu\"'",
                "-Duvxformat='\"lx\"'",
                "-DsPRId64='\"ld\"'",
                "-DsPRIi64='\"li\"'",
                "-DsPRIo64='\"lo\"'",
                "-DsPRIu64='\"lu\"'",
                "-DsPRIx64='\"lx\"'",
                "-DsPRIXU64='\"lX\"'",
                "-Dnveformat='\"e\"'",
                "-Dnvgformat='\"g\"'",
                "-DnvEUformat='\"E\"'",
                "-DnvGUformat='\"G\"'",
            ] {
                configure.push(Arg::lit(arg));
            }
            configure.extend([
                Arg::Concat(vec![
                    Arg::lit("-Dtargetarch="),
                    Arg::quoted(Arg::Var("TRIPLE")),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dsysroot="),
                    Arg::quoted(Arg::Var("_sysroot")),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dusrinc="),
                    Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_sysroot"),
                        Arg::lit("/include"),
                    ])),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dincpth="),
                    Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_sysroot"),
                        Arg::lit("/include"),
                    ])),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dlibpth="),
                    Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_sysroot"),
                        Arg::lit("/lib "),
                        Arg::Var("_sysroot"),
                        Arg::lit("/lib64"),
                    ])),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Darchname="),
                    Arg::quoted(Arg::Var("TRIPLE")),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dbyteorder="),
                    Arg::quoted(Arg::lit(byteorder)),
                ]),
            ]);
        }
        if cross {
            steps.steps.push(exec(Pipe::cmd(configure)));
        } else {
            // ネイティブでも dep glibc sysroot があるならヘッダ/ライブラリ
            // プローブを sysroot に限定する。incpth 既定値の /usr/include は
            // イメージ側の gdbm ヘッダ（ndbm.h 等）を検出して拡張を有効化
            // するが、実ビルドの --sysroot では見えず NDBM_File 等が失敗する。
            let mut sysroot_configure = configure.clone();
            // Configure の `cppstdin -v` プローブがホスト既定の incpth を
            // 拾うと findhdr がイメージ側 /usr/include の gdbm ヘッダを
            // 誤検出する。`-Dsysroot` を渡せば Configure が cppflags に
            // `--sysroot=` を自前で足し、cppstdin（`$cc $cppflags -E`）の
            // プローブも sysroot 化される（cppstdin は Configure が
            // 再計算するため -D プリセットは効かない）。
            // 値は prelude の shadow sysroot `$SYSROOT`（/tmp/sysroot）を
            // 使う: dep パス直指定だと Makefile が末尾に付ける
            // `--sysroot=<dep>` が prelude 注入を上書きし
            // $SYSROOT/usr/include 不在でヘッダ欠落するため。
            sysroot_configure.extend([
                Arg::Concat(vec![
                    Arg::lit("-Dsysroot="),
                    Arg::quoted(Arg::Var("SYSROOT")),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dusrinc="),
                    Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_sysroot"),
                        Arg::lit("/include"),
                    ])),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dincpth="),
                    Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_sysroot"),
                        Arg::lit("/include"),
                    ])),
                ]),
                Arg::Concat(vec![
                    Arg::lit("-Dlibpth="),
                    Arg::quoted(Arg::Concat(vec![
                        Arg::Var("_sysroot"),
                        Arg::lit("/lib "),
                        Arg::Var("_sysroot"),
                        Arg::lit("/lib64"),
                    ])),
                ]),
            ]);
            steps.steps.push(Step::If {
                cond: Cond::DirExists(Arg::quoted(Arg::Concat(vec![
                    Arg::Var("_sysroot"),
                    Arg::lit("/include"),
                ]))),
                then: vec![exec(Pipe::cmd(sysroot_configure))],
                else_: vec![exec(Pipe::cmd(configure))],
            });
        }

        if cross {
            let (dk, ldk, bo_h) = if ctx.triple.starts_with("s390x-") {
                ("4", "2", "0x87654321")
            } else if ctx.triple.starts_with("x86_64-") {
                ("3", "3", "0x12345678")
            } else if ctx.triple.starts_with("powerpc64le-") {
                ("3", "5", "0x12345678")
            } else {
                ("3", "1", "0x12345678")
            };
            let mut patch = vec![
                patch_cfg("lseeksize", "8"),
                patch_cfg("ssizetype", "ssize_t"),
                patch_cfg("sizetype", "size_t"),
                patch_cfg("sizesize", "8"),
                patch_cfg("ptrsize", "8"),
                patch_cfg("ivsize", "8"),
                patch_cfg("uvsize", "8"),
                patch_cfg("nvsize", "8"),
                patch_cfg("doublesize", "8"),
                patch_cfg("longdblsize", "16"),
                patch_cfg("doublekind", dk),
                patch_cfg("longdblkind", ldk),
                patch_cfg("timetype", "time_t"),
                patch_cfg("ivtype", "long"),
                patch_cfg("uvtype", "unsigned long"),
                patch_cfg("nvtype", "double"),
                patch_cfg("d_fork", "define"),
                patch_cfg("d_pipe", "define"),
                patch_cfg("d_waitpid", "define"),
                patch_cfg("d_cplusplus", "undef"),
                patch_cfg("extern_C", "extern"),
                Step::If {
                    cond: Cond::FileExists(Arg::Concat(vec![
                        Arg::Var("SOURCE"),
                        Arg::lit("/Porting/config.sh"),
                    ])),
                    then: vec![
                        exec(
                            Pipe::cmd(vec![
                                Arg::lit("awk"),
                                Arg::lit("-F="),
                                Arg::lit(
                                    "'/^d_[A-Za-z0-9_]+=.define/ && $0 !~ /quadmath/ { print $1 }'",
                                ),
                                Arg::Concat(vec![
                                    Arg::quoted(Arg::Var("SOURCE")),
                                    Arg::lit("/Porting/config.sh"),
                                ]),
                            ])
                            .stdout(Arg::lit("/tmp/d_define.list")),
                        ),
                        Step::ForEach {
                            var: "_k",
                            items: vec![Arg::Subst(Box::new(Pipe::cmd(vec![
                                Arg::lit("cat"),
                                Arg::lit("/tmp/d_define.list"),
                            ])))],
                            body: vec![patch_cfg("${_k}", "define")],
                            stdout: None,
                        },
                    ],
                    else_: vec![],
                },
                cmd(vec![Arg::lit("./Configure"), Arg::lit("-S")]),
                Step::Or {
                    lhs: vec![exec(
                        Pipe::cmd(vec![
                            Arg::lit("sed"),
                            Arg::lit("-i"),
                            Arg::lit("'s/extern \"C\"/extern/g'"),
                            Arg::lit("Makefile"),
                            Arg::lit("config.sh"),
                        ])
                        .stderr(Arg::lit("/dev/null")),
                    )],
                    rhs: vec![cmd(vec![Arg::lit("true")])],
                },
            ];
            steps.steps.push(Step::If {
                cond: Cond::FileExists(Arg::lit("config.sh")),
                then: patch.drain(..).collect(),
                else_: vec![],
            });

            let mut inject = vec![
                inject_h("LSEEKSIZE", "8"),
                inject_h("Size_t", "size_t"),
                inject_h("SSize_t", "ssize_t"),
                inject_h("Size_t_size", "8"),
                inject_h("PTRSIZE", "8"),
                inject_h("LONGSIZE", "8"),
                inject_h("IVSIZE", "8"),
                inject_h("UVSIZE", "8"),
                inject_h("NVSIZE", "8"),
                inject_h("I32SIZE", "4"),
                inject_h("U32SIZE", "4"),
                inject_h("INTSIZE", "4"),
                inject_h("SHORTSIZE", "2"),
                inject_h("CHARBITS", "8"),
                inject_h("DOUBLESIZE", "8"),
                inject_h("LONG_DOUBLESIZE", "16"),
                inject_h("DOUBLEKIND", dk),
                inject_h("LONG_DOUBLEKIND", ldk),
                inject_h("BYTEORDER", bo_h),
                inject_h("LONGLONGSIZE", "8"),
                inject_h("FPOSIZE", "16"),
                inject_h("Time_t", "time_t"),
                inject_h("Off_t", "off_t"),
                inject_h("Off_t_size", "8"),
                inject_h("Mode_t", "mode_t"),
                inject_h("Pid_t", "pid_t"),
                inject_h("Gid_t", "gid_t"),
                inject_h("Uid_t", "uid_t"),
                inject_h("IVTYPE", "long"),
                inject_h("UVTYPE", "unsigned long"),
                inject_h("NVTYPE", "double"),
                inject_h("I32TYPE", "int"),
                inject_h("U32TYPE", "unsigned int"),
                inject_h("PROCSELFEXE_PATH", "\"/proc/self/exe\""),
                cmd(vec![
                    Arg::lit("sed"),
                    Arg::lit("-i"),
                    Arg::lit(
                        "'s/^#define \\([A-Za-z0-9_]*\\)[[:space:]][[:space:]]*undef.*$/\\/\\*#define \\1 undef \\/* *\\//'",
                    ),
                    Arg::lit("config.h"),
                ]),
                inject_h(
                    "DOUBLEINFBYTES",
                    "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x7f",
                ),
                inject_h(
                    "DOUBLENANBYTES",
                    "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0xff",
                ),
                inject_h("DOUBLEMANTBITS", "52"),
                inject_h("NVMANTBITS", "52"),
                Step::Set {
                    name: "_host_cfg",
                    value: Arg::lit(
                        "/dep/perl/usr/lib/perl5/5.42.3/x86_64-linux-thread-multi/CORE/config.h",
                    ),
                },
                Step::If {
                    cond: Cond::FileExists(Arg::Var("_host_cfg")),
                    then: vec![Step::ForEach {
                        var: "_has",
                        items: vec![Arg::Subst(Box::new(Pipe::cmd(vec![
                            Arg::lit("awk"),
                            Arg::lit("'/^#define HAS_/ { print $2 }'"),
                            Arg::Var("_host_cfg"),
                        ])))],
                        body: vec![
                            cmd(vec![
                                Arg::lit("sed"),
                                Arg::lit("-i"),
                                Arg::lit("\"/^#undef ${_has}/d\""),
                                Arg::lit("config.h"),
                            ]),
                            cmd(vec![
                                Arg::lit("sed"),
                                Arg::lit("-i"),
                                Arg::lit("\"s|^/\\*#define ${_has}|#define ${_has}|\""),
                                Arg::lit("config.h"),
                            ]),
                            Step::Or {
                                lhs: vec![cmd(vec![
                                    Arg::lit("grep"),
                                    Arg::lit("-q"),
                                    Arg::lit("\"^#define ${_has}\""),
                                    Arg::lit("config.h"),
                                ])],
                                rhs: vec![exec(
                                    Pipe::cmd(vec![
                                        Arg::lit("printf"),
                                        Arg::lit("'#define %s\\t\\t/**/\\n'"),
                                        Arg::lit("\"${_has}\""),
                                    ])
                                    .append_stdout(Arg::lit("config.h")),
                                )],
                            },
                        ],
                        stdout: None,
                    }],
                    else_: vec![],
                },
            ];
            steps.steps.push(Step::If {
                cond: Cond::FileExists(Arg::lit("config.h")),
                then: inject.drain(..).collect(),
                else_: vec![],
            });
        }

        steps.steps.push(Step::Export {
            name: "PERL_USE_UNSAFE_INC",
            value: Arg::lit("1"),
        });
        steps.steps.push(Step::Export {
            name: "PERL5LIB",
            value: Arg::quoted(Arg::Concat(vec![
                Arg::Var("PWD"),
                Arg::lit("/lib${PERL5LIB:+:$PERL5LIB}"),
            ])),
        });
        steps.steps.push(Step::If {
            cond: Cond::Cmd(Pipe::cmd(vec![
                Arg::lit("["),
                Arg::lit("-x"),
                Arg::lit("/tmp/perl-host/miniperl"),
                Arg::lit("]"),
            ])),
            then: vec![
                cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::lit("/tmp/perl-host/miniperl"),
                    Arg::lit("/tmp/perl-host/perl"),
                ]),
                Step::Export {
                    name: "PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::lit("/tmp/perl-host:"),
                        Arg::Var("PATH"),
                    ])),
                },
            ],
            else_: vec![],
        });
        // Step::Make の CFLAGS 系変数上書きは perl の Makefile が焼いた
        // フラグを潰すため、素の make を Exec で呼ぶ。
        steps
            .steps
            .push(cmd(vec![Arg::lit("make"), Arg::nproc_jobs("-j")]));
        steps.steps.push(cmd(vec![
            Arg::lit("make"),
            Arg::lit("install"),
            Arg::lit("DESTDIR=\"${OUT}\""),
        ]));
        steps.steps.push(Step::Set {
            name: "_perl_so",
            value: Arg::Subst(Box::new(Pipe::new(vec![
                vec![
                    Arg::lit("find"),
                    Arg::quoted(Arg::lit("${OUT}/usr/lib")),
                    Arg::lit("-name"),
                    Arg::lit("libperl.so"),
                ],
                vec![Arg::lit("head"), Arg::lit("-n1")],
            ]))),
        });
        steps.steps.push(Step::If {
            cond: Cond::EnvSet("_perl_so"),
            then: vec![cmd(vec![
                Arg::lit("ln"),
                Arg::lit("-sfn"),
                Arg::Subst(Box::new(Pipe::cmd(vec![
                    Arg::lit("realpath"),
                    Arg::Concat(vec![
                        Arg::lit("--relative-to="),
                        Arg::quoted(Arg::lit("${OUT}/usr/lib")),
                    ]),
                    Arg::quoted(Arg::Var("_perl_so")),
                ]))),
                Arg::quoted(Arg::lit("${OUT}/usr/lib/libperl.so")),
            ])],
            else_: vec![],
        });

        Recipe::Steps(steps)
    }
}
