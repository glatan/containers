use crate::flags::Flag;
use crate::package::recipe::{Arg, BuildContext, Cond, Env, Pipe, Recipe, Step, Steps, Workdir};
use crate::package::{Dependency, Package, Source, dep};
use crate::target::Target;

pub struct Python;

impl Package for Python {
    fn name(&self) -> &'static str {
        "python"
    }

    fn version(&self) -> &'static str {
        "3.15.0"
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
            url: "https://www.python.org/ftp/python/3.15.0/Python-3.15.0.tar.xz",
            sha3_512: "b9ec570f6e23d402f5b445a2dbda7c71611aa217c41c086fe2e2a4ace36e972e65c97d54f6387b8dd0857c4e8fdc35bd1fbc57f3de3db54b3dc49698f4641e5d",
            blake3: "1254fe6e327eaba1572549b1f8d1a1a4cf07d87733411542868cf9bebce6199572072a521e6f261f96f6b054ebac934a88237b955efc7548a8343c4e1d68a574",
            subdir: "Python-3.15.0",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(
            target,
            &[
                dep("libffi", "3.8.0"),
                dep("zlib", "1.3.2"),
                dep("openssl", "4.0.3"),
                dep("expat", "2.9.0"),
                dep("sqlite", "3.54.0"),
                dep("ncurses", "6.6"),
                dep("readline", "8.3"),
                dep("xz", "5.8.4"),
                dep("bzip2", "1.0.8"),
                dep("zstd", "1.5.7"),
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
    /// configure の stdlib 拡張モジュール検出（_ssl/_hashlib/
    /// readline 等）は pkg-config を使うため pkgconf が必須。
    /// クロスでは --with-build-python に同バージョンの BUILD_ENV
    /// python が必須（自己参照。バージョン固定のため distro では賄えない）。
    fn host_dependencies(&self, target: Target) -> Vec<crate::package::Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        deps.push(crate::package::dep("pkgconf", "3.0.8"));
        if !matches!(target, crate::target::Target::Amd64(_)) {
            deps.push(crate::package::dep("python", "3.15.0").via(crate::package::Via::Artifact));
        }
        deps
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
        let steps = Steps::new(Env::Autotools)
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
                            let mut flags = vec![
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
                            ];
                            // ネイティブの拡張 .so import テストが dep の .so を
                            // 実行時解決するため。クロスではホストの /bin/sh が
                            // ターゲット lib を読み、s390x（big-endian）で死ぬ。
                            if !ctx.cross() {
                                flags.push(export(
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
                                ));
                            }
                            flags
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
                .step(Step::configure(&configure_args));
        // クロスの RUNSHARED は LD_LIBRARY_PATH をビルドディレクトリにする。
        // PYTHON_FOR_BUILD（ホストの python3）が s390x の libpython を読み、
        // 動的リンカが big-endian で死ぬ。空にしてホストの libpython のまま起動する。
        // checksharedmods は --cross-compiling でファイルの有無だけを見て、
        // インポート失敗による拡張 .so の改名を避ける。
        let steps = if ctx.cross() {
            steps
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("sed"),
                        Arg::lit("-i"),
                        Arg::quoted(Arg::lit(
                            "s|check_extension_modules.py|check_extension_modules.py --cross-compiling|g",
                        )),
                        Arg::lit("Makefile"),
                    ]),
                })
                .step(Step::make(&["RUNSHARED="]))
                .step(Step::make_install(&["RUNSHARED="]))
        } else {
            steps.step(Step::make(&[])).step(Step::make_install(&[]))
        };
        Recipe::Steps(steps.step(Step::Symlink {
            target: "python3".into(),
            link: "\"$OUT\"/usr/bin/python".into(),
            force: true,
        }))
    }
}
