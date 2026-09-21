use crate::flags::Flag;
use crate::package::recipe::{Arg, BuildContext, Cond, Env, Recipe, Step, Steps, Workdir};
use crate::package::{Dependency, Package, Source};
use crate::target::Target;

pub struct Python;

impl Package for Python {
    fn name(&self) -> &'static str {
        "python"
    }

    fn version(&self) -> &'static str {
        "3.14.7"
    }

    fn license(&self) -> &'static str {
        "Python-2.0"
    }

    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://www.python.org/ftp/python/3.14.7/Python-3.14.7.tar.xz",
            sha3_512: "2da7cc048103dfd9f3681848e13fd3dfa06f25cc9afd76f89f772c1e3a8f14410ba44546754ee177672d0f86fac551bbd26685335830e316ca0e917629934420",
            blake3: "4406a4956bb3265c370b011383004cc2dcc94ed2c99b5f3ed67338eeab9e5eaceedabc538098da6772b361383e84ccf0274451296970e8d7c9b4261e9a30de59",
            subdir: "Python-3.14.7",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(
            target,
            &[
                Dependency {
                    name: "libffi",
                    version: "3.8.0",
                },
                Dependency {
                    name: "zlib",
                    version: "1.3.2",
                },
                Dependency {
                    name: "openssl",
                    version: "4.0.2",
                },
                Dependency {
                    name: "expat",
                    version: "2.8.4",
                },
                Dependency {
                    name: "sqlite",
                    version: "3.53.4",
                },
                Dependency {
                    name: "ncurses",
                    version: "6.6",
                },
                Dependency {
                    name: "readline",
                    version: "8.3",
                },
                Dependency {
                    name: "xz",
                    version: "5.8.4",
                },
                Dependency {
                    name: "bzip2",
                    version: "1.0.8",
                },
                Dependency {
                    name: "zstd",
                    version: "1.5.7",
                },
                crate::package::gcc_dep(target),
            ],
        )
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    /// configure の stdlib 拡張モジュール検出（_ssl/_hashlib/readline 等）は
    /// pkg-config を使うため、pkgconf を HOST ツールとしてマウントする。
    /// （dep グラフ外のため artifact 未作成なら mount されない点に注意）
    fn host_tools(&self, target: Target) -> Vec<&'static str> {
        let mut tools = vec!["pkgconf"];
        if target != Target::BUILD_ENV {
            // クロスでは --with-build-python に BUILD_ENV の python が必要
            tools.push("python");
        }
        tools
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    /// `--enable-optimizations` の自己完結 PGO（gen→PROFILE_TASK→use を内部で回す）。
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        // --enable-optimizations は PROFILE_TASK でビルドした python を
        // 実行するため、生成バイナリを動かせないクロスビルドでは付けない。
        let mut configure_args = vec!["--prefix=/usr", "--enable-shared"];
        if !ctx.cross() {
            configure_args.push("--enable-optimizations");
        }
        configure_args.push("--without-ensurepip");
        if ctx.cross() {
            configure_args.push("--with-build-python=/dep/python/usr/bin/python3");
        }
        Recipe::Steps(
            Steps::new(Env::Autotools)
                // --enable-optimizations の PROFILE_TASK（test.regrtest）は
                // Lib 配下に .pyc を書き込み forget() で消すため、ro マウントの
                // /src を直接使うと EROFS で失敗する。InSource で書き込み可能な
                // /build へソースをコピーしてからビルドする。
                .workdir(Workdir::InSource)
                .step(Step::ForEach {
                    var: "_dep",
                    items: [
                        "libffi", "zlib", "openssl", "expat", "sqlite", "ncurses", "readline",
                        "xz", "bzip2", "zstd",
                    ]
                    .iter()
                    .map(|name| Arg::lit(name))
                    .collect(),
                    body: {
                        let dep_dir = |sub: &'static str| {
                            Arg::Concat(vec![Arg::lit("/dep/"), Arg::Var("_dep"), Arg::lit(sub)])
                        };
                        let export = |name: &'static str, parts: Vec<Arg>| Step::Export {
                            name,
                            value: Arg::quoted(Arg::Concat(parts)),
                        };
                        let lib_flags = |libdir: &'static str| {
                            vec![
                                export(
                                    "LDFLAGS",
                                    vec![
                                        Arg::Var("LDFLAGS"),
                                        Arg::lit(" -L/dep/"),
                                        Arg::Var("_dep"),
                                        Arg::lit(match libdir {
                                            "lib64" => "/usr/lib64",
                                            _ => "/usr/lib",
                                        }),
                                    ],
                                ),
                                export(
                                    "PKG_CONFIG_PATH",
                                    vec![
                                        Arg::lit("/dep/"),
                                        Arg::Var("_dep"),
                                        Arg::lit(match libdir {
                                            "lib64" => "/usr/lib64/pkgconfig",
                                            _ => "/usr/lib/pkgconfig",
                                        }),
                                        Arg::lit("${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"),
                                    ],
                                ),
                                // ビルド済み拡張 .so の import テストが dep の
                                // .so（libreadline.so.8 等）を実行時解決するため、
                                // dep lib dir を LD_LIBRARY_PATH にも載せる。
                                // （runs_on_host なので dep は常にホスト arch）
                                export(
                                    "LD_LIBRARY_PATH",
                                    vec![
                                        Arg::lit("/dep/"),
                                        Arg::Var("_dep"),
                                        Arg::lit(match libdir {
                                            "lib64" => "/usr/lib64",
                                            _ => "/usr/lib",
                                        }),
                                        Arg::lit("${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"),
                                    ],
                                ),
                            ]
                        };
                        vec![
                            Step::If {
                                cond: Cond::DirExists(dep_dir("/usr/include")),
                                then: vec![
                                    export(
                                        "CPPFLAGS",
                                        vec![
                                            Arg::lit("${CPPFLAGS:-}"),
                                            Arg::lit(" -I/dep/"),
                                            Arg::Var("_dep"),
                                            Arg::lit("/usr/include"),
                                        ],
                                    ),
                                    export(
                                        "CFLAGS",
                                        vec![
                                            Arg::Var("CFLAGS"),
                                            Arg::lit(" -I/dep/"),
                                            Arg::Var("_dep"),
                                            Arg::lit("/usr/include"),
                                        ],
                                    ),
                                ],
                                else_: vec![],
                            },
                            Step::If {
                                cond: Cond::DirExists(dep_dir("/usr/lib64")),
                                then: lib_flags("lib64"),
                                else_: vec![Step::If {
                                    cond: Cond::DirExists(dep_dir("/usr/lib")),
                                    then: lib_flags("lib"),
                                    else_: vec![],
                                }],
                            },
                        ]
                    },
                    stdout: None,
                })
                // openssl 4.x では SSLv3/TLSv1/1.1/1.2 の *_method() が完全に
                // 削除されており OPENSSL_NO_*_METHOD も定義されない。_ssl.c が
                // マクロ未定義だと extern 宣言して未定義シンボル参照になるため、
                // 明示的に無効化する（3.x では deprecated 機能を外すだけで安全）。
                .step(Step::Export {
                    name: "CFLAGS",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("CFLAGS"),
                        Arg::lit(
                            " -DOPENSSL_NO_SSL3 -DOPENSSL_NO_SSL3_METHOD -DOPENSSL_NO_TLS1_METHOD -DOPENSSL_NO_TLS1_1_METHOD -DOPENSSL_NO_TLS1_2_METHOD",
                        ),
                    ])),
                })
                // getaddrinfo bug は実行テストのためクロスでは常に
                // 「bug あり」と判定され configure が死ぬ。
                // glibc の getaddrinfo は健全なので cache 値を与える。
                .steps(if ctx.cross() {
                    vec![
                        Step::Export {
                            name: "ac_cv_buggy_getaddrinfo",
                            value: Arg::lit("no"),
                        },
                        // /dev/ptmx はファイル存在確認のためクロスでは
                        // CONFIG_SITE 相当の値が必要（glibc/Linux では有）。
                        Step::Export {
                            name: "ac_cv_file__dev_ptmx",
                            value: Arg::lit("yes"),
                        },
                        Step::Export {
                            name: "ac_cv_file__dev_ptc",
                            value: Arg::lit("no"),
                        },
                    ]
                } else {
                    vec![]
                })
                .step(Step::configure(&configure_args))
                .step(Step::make(&[]))
                .step(Step::make_install(&[]))
                .step(Step::Symlink {
                    target: "python3".into(),
                    link: "\"$OUT\"/usr/bin/python".into(),
                    force: true,
                }),
        )
    }
}
