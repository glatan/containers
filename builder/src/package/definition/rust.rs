use crate::flags::{Cflag, Flag, Ldflag};
use crate::package::recipe::{
    Arg, BuildContext, Cond, Env, Pipe, Recipe, Step, Steps, Workdir, argv,
};
use crate::package::{Bootstrap, Dependency, Package, Source, Via, dep};
use crate::target::Target;

pub struct Rust;

/// GNU triple と Rust target triple、対応する toolchain パッケージ名、
/// および seed 相で担保する distro パッケージ接頭辞の対応表。
/// x86_64 は常に必須（ホスト triple）のため別扱いにする。
struct Cross {
    rust: &'static str,
    /// `{gnu}-` 付きツール名接頭辞（`{prefix}gcc` 等）。
    prefix: &'static str,
    gcc_pkg: &'static str,
    binutils_pkg: &'static str,
    glibc_pkg: &'static str,
    /// distro の cross toolchain 接頭辞。`""` は distro 非提供
    /// （seed 相では不マウントとなり、レシピが動的にスキップする）。
    distro: &'static str,
}

const CROSSES: &[Cross] = &[
    Cross {
        rust: "aarch64-unknown-linux-gnu",
        prefix: "aarch64-linux-gnu-",
        gcc_pkg: "aarch64-linux-gnu-gcc",
        binutils_pkg: "aarch64-linux-gnu-binutils",
        glibc_pkg: "aarch64-linux-gnu-glibc",
        distro: "aarch64-linux-gnu-",
    },
    Cross {
        rust: "riscv64gc-unknown-linux-gnu",
        prefix: "riscv64-linux-gnu-",
        gcc_pkg: "riscv64-linux-gnu-gcc",
        binutils_pkg: "riscv64-linux-gnu-binutils",
        glibc_pkg: "riscv64-linux-gnu-glibc",
        distro: "riscv64-linux-gnu-",
    },
    Cross {
        rust: "powerpc64le-unknown-linux-gnu",
        prefix: "powerpc64le-linux-gnu-",
        gcc_pkg: "powerpc64le-linux-gnu-gcc",
        binutils_pkg: "powerpc64le-linux-gnu-binutils",
        glibc_pkg: "powerpc64le-linux-gnu-glibc",
        distro: "",
    },
    Cross {
        rust: "s390x-unknown-linux-gnu",
        prefix: "s390x-linux-gnu-",
        gcc_pkg: "s390x-linux-gnu-gcc",
        binutils_pkg: "s390x-linux-gnu-binutils",
        glibc_pkg: "s390x-linux-gnu-glibc",
        distro: "",
    },
];

/// config.toml の `[target.*]` セクション（無印/distro toolchain 指定）。
fn target_section(rust: &'static str, prefix: &'static str) -> Vec<Arg> {
    vec![
        Arg::lit(""),
        Arg::owned(format!("[target.{rust}]")),
        Arg::owned(format!("cc = \"{prefix}gcc\"")),
        Arg::owned(format!("cxx = \"{prefix}g++\"")),
        Arg::owned(format!("ar = \"/usr/bin/{prefix}ar\"")),
        Arg::owned(format!("ranlib = \"/usr/bin/{prefix}ranlib\"")),
        Arg::owned(format!("linker = \"{prefix}gcc\"")),
    ]
}

/// config.toml の `[target.*]` セクション（sysroot ラッパ指定）。
/// 自前 toolchain（`/dep/{gnu}-gcc`）は libc を内蔵しないため、
/// `/dep/{gnu}-glibc` と linux-headers を畳んだ `/tmp/sysroot-{gnu}`
/// shadow sysroot 経由のラッパを cc/cxx/linker に使う。
fn target_section_wrapped(rust: &'static str, gnu: &str) -> Vec<Arg> {
    vec![
        Arg::lit(""),
        Arg::owned(format!("[target.{rust}]")),
        Arg::owned(format!("cc = \"$BUILD/ccw/{gnu}-gcc\"")),
        Arg::owned(format!("cxx = \"$BUILD/ccw/{gnu}-g++\"")),
        Arg::owned(format!("ar = \"/dep/{gnu}-binutils/usr/bin/{gnu}-ar\"")),
        Arg::owned(format!(
            "ranlib = \"/dep/{gnu}-binutils/usr/bin/{gnu}-ranlib\""
        )),
        Arg::owned(format!("linker = \"$BUILD/ccw/{gnu}-gcc\"")),
    ]
}

/// `{triple}-gcc` コマンドの存在検査。
fn gcc_probe(bin: &str) -> Cond {
    Cond::Cmd(Pipe::cmd(vec![
        Arg::lit("command"),
        Arg::lit("-v"),
        Arg::owned(bin.to_owned()),
    ]))
}

/// `/dep/{gnu}-glibc` 成果物から shadow sysroot と cc ラッパを生成する。
/// build/prelude の /tmp/sysroot と同じ構造を triple ごとに作る
/// （include は symlink、lib は ld スクリプト解決のため実コピー）。
fn sysroot_wrapper_steps(gnu: &str) -> Vec<Step> {
    let exec = |args: Vec<Arg>| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(args),
    };
    let sysroot = format!("/tmp/sysroot-{gnu}");
    let dep = format!("/dep/{gnu}-glibc/usr/{gnu}");
    vec![
        exec(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            Arg::owned(format!("\"{sysroot}/usr\"")),
            Arg::lit("\"$BUILD/ccw\""),
        ]),
        exec(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            Arg::owned(format!("\"{dep}/include\"")),
            Arg::owned(format!("{sysroot}/include")),
        ]),
        exec(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            Arg::owned(format!("\"{dep}/include\"")),
            Arg::owned(format!("{sysroot}/usr/include")),
        ]),
        exec(vec![
            Arg::lit("cp"),
            Arg::lit("-a"),
            Arg::owned(format!("\"{dep}/lib\"")),
            Arg::owned(format!("{sysroot}/lib")),
        ]),
        Step::If {
            cond: Cond::Exists(Arg::owned(format!("\"{dep}/lib64\""))),
            then: vec![exec(vec![
                Arg::lit("cp"),
                Arg::lit("-a"),
                Arg::owned(format!("\"{dep}/lib64\"")),
                Arg::owned(format!("{sysroot}/lib64")),
            ])],
            else_: vec![],
        },
        exec(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            Arg::lit("../lib"),
            Arg::owned(format!("{sysroot}/usr/lib")),
        ]),
        exec(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            Arg::lit("../lib"),
            Arg::owned(format!("{sysroot}/usr/lib64")),
        ]),
        Step::WriteFile {
            path: Arg::owned(format!("$BUILD/ccw/{gnu}-gcc")),
            lines: vec![
                Arg::lit("#!/bin/sh"),
                Arg::owned(format!(
                    "exec {gnu}-gcc -B/dep/{gnu}-binutils/usr/{gnu}/bin --sysroot={sysroot} -idirafter /dep/linux-headers/usr/{gnu}/include \"$@\""
                )),
            ],
            append: false,
            expand: false,
        },
        Step::WriteFile {
            path: Arg::owned(format!("$BUILD/ccw/{gnu}-g++")),
            lines: vec![
                Arg::lit("#!/bin/sh"),
                Arg::owned(format!(
                    "exec {gnu}-g++ -B/dep/{gnu}-binutils/usr/{gnu}/bin --sysroot={sysroot} -idirafter /dep/linux-headers/usr/{gnu}/include \"$@\""
                )),
            ],
            append: false,
            expand: false,
        },
        exec(vec![
            Arg::lit("chmod"),
            Arg::lit("+x"),
            Arg::owned(format!("\"$BUILD/ccw/{gnu}-gcc\"")),
            Arg::owned(format!("\"$BUILD/ccw/{gnu}-g++\"")),
        ]),
    ]
}

impl Package for Rust {
    fn name(&self) -> &'static str {
        "rust"
    }

    fn version(&self) -> &'static str {
        "1.99.0"
    }

    fn license(&self) -> &'static str {
        "MIT OR Apache-2.0"
    }

    /// 5 つの Rust target の std を内包する単一成果物のため
    /// BUILD_ENV（ホスト）でのみビルドする。
    fn supports(&self, target: Target) -> bool {
        target == Target::BUILD_ENV
    }

    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    /// rustbuild は LLVM / compiler-builtins 向けに独自に cc を解決する
    /// （config.toml の [target.*].cc）。cc ラッパの --sysroot/-idirafter
    /// 注入は rustbuild の検出と干渉するため無効化する。
    fn uses_cc_wrapper(&self) -> bool {
        false
    }

    /// `Bootstrap` を実装（bootstrap パッケージ）。seed 相は distro rust
    /// を stage0 として使い、env 相は自前の rust 成果物で自己ホストする。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }

    fn source(&self) -> Source {
        Source {
            url: "https://static.rust-lang.org/dist/rustc-1.99.0-src.tar.xz",
            sha3_512: "9e135098ddcae1040361c6f87dd95aed9d83f7e51d37a7671e67b30bdc4deed7b9c3bd79fc7b646ab5d8edbed6e5abe2a1cf8df499eb1796944abeede0e4dbfd",
            blake3: "3ca49872e960518052d1e709cc6860224356ce79f184d8f2b1ef900591ec2695b7d227f169e38204c6e31e42c93ded1f57a283537cf3c25e4b6540bfbbcd6d19",
            subdir: "rustc-1.99.0-src",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[crate::package::gcc_dep(target)])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    /// rustbuild（x.py + LLVM + cargo bootstrap）の HOST 依存。
    /// 全て `Via::Seed`: Ordered にすると Phase-1 閉包が非 HOST
    /// toolchain（cross gcc/glibc）と python 系を巻き込み bootstrap
    /// 不変条件に違反するため。seed 相では distro 名の pacman パッケージが
    /// 担保し、env 相では成果物が必須になる。
    ///
    /// - rust（自己参照）: stage0。seed は distro rust、env は自前成果物。
    /// - python/cmake/ninja: x.py / LLVM ビルド系。
    /// - perl: cargo が静的リンクする vendored OpenSSL（openssl-src）の
    ///   Configure で必要。
    /// - {triple}-{gcc,binutils,glibc}: 各 Rust target の std を組む
    ///   cross toolchain。aarch64/riscv64 は distro 提供あり、
    ///   powerpc64le/s390x/x86_64 は `Seed("")`（seed 相では不マウント
    ///   = x86_64 はベースの無印 cc が担保、cross はスキップ）。
    fn host_dependencies(&self, _target: Target) -> Vec<Dependency> {
        let mut deps = vec![
            dep("rust", "1.99.0").via(Via::Seed("rust")),
            dep("python", "3.15.0").via(Via::Seed("python")),
            dep("cmake", "4.4.4").via(Via::Seed("cmake")),
            dep("ninja", "1.13.2").via(Via::Seed("ninja")),
            dep("perl", "5.44.0").via(Via::Seed("perl")),
            // rustc 実行時の libz 解決用。成果物 usr/lib に同梱する。
            dep("zlib", "1.3.2").via(Via::Seed("zlib")),
        ];
        // x86_64 は distro 側に -linux-gnu- 命名のパッケージがなく、
        // ベースイメージ / build-env の無印 gcc が担保するため Seed("")。
        deps.push(dep("x86_64-linux-gnu-gcc", crate::package::GCC_VERSION).via(Via::Seed("")));
        deps.push(
            dep(
                "x86_64-linux-gnu-binutils",
                crate::package::BINUTILS_VERSION,
            )
            .via(Via::Seed("")),
        );
        deps.push(dep("x86_64-linux-gnu-glibc", crate::package::GLIBC_VERSION).via(Via::Seed("")));
        for cross in CROSSES {
            deps.push(
                dep(cross.gcc_pkg, crate::package::GCC_VERSION).via(Via::Seed(
                    if cross.distro.is_empty() {
                        ""
                    } else {
                        cross.gcc_pkg
                    },
                )),
            );
            deps.push(
                dep(cross.binutils_pkg, crate::package::BINUTILS_VERSION).via(Via::Seed(
                    if cross.distro.is_empty() {
                        ""
                    } else {
                        cross.binutils_pkg
                    },
                )),
            );
            deps.push(
                dep(cross.glibc_pkg, crate::package::GLIBC_VERSION).via(Via::Seed(
                    if cross.distro.is_empty() {
                        ""
                    } else {
                        cross.glibc_pkg
                    },
                )),
            );
        }
        deps
    }

    fn flags(&self, target: Target) -> Flag {
        // mold への依存を持たず（rust は自身の std/LLVM を組む最下層の
        // toolchain）、`-fuse-ld=mold` は使わない。LLVM 規模のビルドで
        // LTO はコストに見合わないため外す。
        use crate::flags::tool_flags;
        Flag::new(
            tool_flags(target)
                .cflags()
                .iter()
                .filter(|flag| {
                    !matches!(
                        **flag,
                        Cflag::Lto | Cflag::WerrorOdr | Cflag::WerrorLtoTypeMismatch
                    )
                })
                .copied()
                .collect(),
            tool_flags(target)
                .ldflags()
                .iter()
                .filter(|flag| {
                    !matches!(
                        **flag,
                        Ldflag::Lto
                            | Ldflag::WerrorOdr
                            | Ldflag::WerrorLtoTypeMismatch
                            | Ldflag::FuseLdMold
                    )
                })
                .copied()
                .collect(),
        )
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        // config.toml を動的に生成する。stage0 rustc/cargo は
        // /dep/*/usr/bin の PATH 先頭挿入により `command -v` で
        // 統一的に解決する（env 相: 自己成果物、seed 相: distro rust）。
        // std 対象は {gnu}-gcc の存在で選択し、seed 相では distro 提供の
        // aarch64/riscv64 + x86_64、env 相では全 5 triple が揃う。
        let mut steps = Steps::new(Env::None)
            .sysroot(false)
            .workdir(Workdir::InSource)
            .step(Step::Comment(
                "stage0 と cross toolchain を検出し config.toml を生成".into(),
            ))
            // bundled LLVM の bind_back は static_assert(ConstFn != nullptr)
            // を if constexpr 内で評価するが、関数ポインタの nullptr 比較は
            // GCC 16 では constexpr として受理されず LLVM ビルドが失敗する。
            // 防御的 assert のみなので削除する（upstream 側で実質的な
            // 制約を持たない）。
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(vec![
                    Arg::lit("sed"),
                    Arg::lit("-i"),
                    Arg::lit(
                        "\"/if constexpr (std::is_pointer_v/,/static_assert(ConstFn != nullptr)/d\"",
                    ),
                    Arg::lit(
                        "src/llvm-project/llvm/include/llvm/ADT/STLForwardCompat.h",
                    ),
                ]),
            })
            .step(Step::Set {
                name: "RUSTC",
                value: Arg::Subst(Box::new(Pipe::cmd(argv(&["command", "-v", "rustc"])))),
            })
            .step(Step::Set {
                name: "STAGE0_CARGO",
                value: Arg::Subst(Box::new(Pipe::cmd(argv(&["command", "-v", "cargo"])))),
            })
            // 変数値に TOML の引用符を残すため \\" エスケープで代入する
            // （素の `"…"` はシェル構文として剥がれてしまう）。
            .step(Step::Set {
                name: "RUST_TARGETS",
                value: Arg::lit("\\\"x86_64-unknown-linux-gnu\\\""),
            });

        // 各 triple は「自前 toolchain（dep gcc+glibc が揃う）なら
        // sysroot ラッパ、それ以外で distro の {prefix}gcc があれば
        // distro そのまま、無ければ std 対象から外す」という選択。
        // x86_64 はホスト triple で常に std 対象。dep glibc が無い
        // seed 初回は distro 無印 gcc にフォールバックする。
        let x86_gnu = "x86_64-linux-gnu";
        let x86_dep = format!("/dep/{x86_gnu}-glibc/usr/{x86_gnu}/lib");
        steps = steps.step(Step::If {
            cond: Cond::And(vec![
                Cond::DirExists(Arg::owned(x86_dep)),
                Cond::DirExists(Arg::owned(
                    "/dep/x86_64-linux-gnu-binutils/usr/bin".to_owned()
                )),
                gcc_probe("x86_64-linux-gnu-gcc"),
            ]),
            then: {
                let mut v = sysroot_wrapper_steps(x86_gnu);
                // LLVM（ホスト）ビルドの cc 解決にも sysroot を届ける。
                // prelude の sysroot 注入と同じ flag 構成。
                let sysroot = format!("/tmp/sysroot-{x86_gnu}");
                v.extend([
                    Step::Export {
                        name: "CFLAGS",
                        value: Arg::owned(format!(
                            "\"${{CFLAGS:-}} --sysroot={sysroot} -idirafter /dep/linux-headers/usr/{x86_gnu}/include\""
                        )),
                    },
                    Step::Export {
                        name: "CXXFLAGS",
                        value: Arg::owned(format!(
                            "\"${{CXXFLAGS:-}} --sysroot={sysroot} -idirafter /dep/linux-headers/usr/{x86_gnu}/include\""
                        )),
                    },
                    Step::Export {
                        name: "LDFLAGS",
                        value: Arg::owned(format!(
                            "\"-L{sysroot}/lib -L{sysroot}/usr/lib -L{sysroot}/lib64 -L{sysroot}/usr/lib64 ${{LDFLAGS:-}} --sysroot={sysroot}\""
                        )),
                    },
                    Step::WriteFile {
                        path: Arg::lit("$BUILD/targets.conf"),
                        lines: target_section_wrapped("x86_64-unknown-linux-gnu", x86_gnu),
                        append: true,
                        expand: true,
                    },
                ]);
                v
            },
            else_: vec![Step::WriteFile {
                path: Arg::lit("$BUILD/targets.conf"),
                lines: target_section("x86_64-unknown-linux-gnu", ""),
                append: true,
                expand: false,
            }],
        });

        // cross triple: toolchain が使える場合のみ std 対象に追加。
        for cross in CROSSES {
            let gnu = cross.prefix.trim_end_matches('-');
            steps = steps.step(Step::If {
                cond: Cond::And(vec![
                    Cond::DirExists(Arg::owned(format!("/dep/{gnu}-glibc/usr/{gnu}/lib"))),
                    Cond::DirExists(Arg::owned(format!("/dep/{gnu}-binutils/usr/bin"))),
                    gcc_probe(&format!("{gnu}-gcc")),
                ]),
                then: {
                    let mut v = sysroot_wrapper_steps(gnu);
                    v.extend([
                        Step::Set {
                            name: "RUST_TARGETS",
                            value: Arg::owned(format!("${{RUST_TARGETS}},\\\"{}\\\"", cross.rust)),
                        },
                        Step::WriteFile {
                            path: Arg::lit("$BUILD/targets.conf"),
                            lines: target_section_wrapped(cross.rust, gnu),
                            append: true,
                            expand: true,
                        },
                    ]);
                    v
                },
                // dep toolchain が揃わない seed 相では distro の cross gcc
                // （自己完結型）にフォールバックする。
                else_: vec![Step::If {
                    cond: gcc_probe(&format!("{}gcc", cross.prefix)),
                    then: vec![
                        Step::Set {
                            name: "RUST_TARGETS",
                            value: Arg::owned(format!("${{RUST_TARGETS}},\\\"{}\\\"", cross.rust)),
                        },
                        Step::WriteFile {
                            path: Arg::lit("$BUILD/targets.conf"),
                            lines: target_section(cross.rust, cross.prefix),
                            append: true,
                            expand: false,
                        },
                    ],
                    else_: vec![],
                }],
            });
        }

        steps = steps
            .step(Step::WriteFile {
                path: Arg::lit("$BUILD/config.toml"),
                lines: argv(&[
                    "",
                    "[build]",
                    "build = \"x86_64-unknown-linux-gnu\"",
                    "host = [\"x86_64-unknown-linux-gnu\"]",
                    "target = [${RUST_TARGETS}]",
                    "rustc = \"${RUSTC}\"",
                    "cargo = \"${STAGE0_CARGO}\"",
                    "vendor = true",
                    // cargo が distro の libgit2/libcurl/libssl に動的
                    // リンクされるのを防ぎ成果物を自己完結させる。
                    "cargo-native-static = true",
                    "docs = false",
                    "submodules = false",
                    "extended = true",
                    "tools = [\"cargo\"]",
                    "",
                    "[install]",
                    "prefix = \"/usr\"",
                    "sysconfdir = \"etc\"",
                    "",
                    "[llvm]",
                    "ninja = true",
                    "targets = \"X86;AArch64;RISCV;PowerPC;SystemZ\"",
                    "download-ci-llvm = false",
                ]),
                append: false,
                expand: true,
            })
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["cat", "\"$BUILD/targets.conf\""]))
                    .append_stdout(Arg::lit("\"$BUILD/config.toml\"")),
            })
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(vec![
                    Arg::lit("python3"),
                    Arg::lit("x.py"),
                    Arg::lit("build"),
                    Arg::lit("--stage"),
                    Arg::lit("2"),
                    Arg::nproc_jobs("-j"),
                ]),
            })
            .step(Step::Exec {
                env: vec![("DESTDIR", Arg::quoted(Arg::Var("OUT")))],
                pipe: Pipe::cmd(vec![
                    Arg::lit("python3"),
                    Arg::lit("x.py"),
                    Arg::lit("install"),
                    Arg::nproc_jobs("-j"),
                ]),
            })
            // rust 成果物の RUNPATH は $ORIGIN/../lib のみ。依存側
            // コンテナで rustc/cargo が自己完結するよう、実行時依存
            // .so を usr/lib に同梱する（env 相は dep 成果物、
            // seed 相は distro /usr/lib から取る）。
            // dep 側の片方の glob だけ失敗すると混在コピーになるため、
            // || のフォールバックではなくディレクトリ有無で分岐する。
            .step(Step::If {
                cond: Cond::DirExists(Arg::lit("/dep/x86_64-linux-gnu-gcc/usr/lib64")),
                then: vec![Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(&[
                        "cp",
                        "-a",
                        "/dep/x86_64-linux-gnu-gcc/usr/lib64/libstdc++.so.6*",
                        "/dep/x86_64-linux-gnu-gcc/usr/lib64/libgcc_s.so.1*",
                        "\"$OUT/usr/lib/\"",
                    ])),
                }],
                else_: vec![Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(&[
                        "cp",
                        "-a",
                        "/usr/lib/libstdc++.so.6*",
                        "/usr/lib/libgcc_s.so.1*",
                        "\"$OUT/usr/lib/\"",
                    ])),
                }],
            })
            .step(Step::If {
                cond: Cond::DirExists(Arg::lit("/dep/zlib/usr/lib64")),
                then: vec![Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(&[
                        "cp",
                        "-a",
                        "/dep/zlib/usr/lib64/libz.so.1*",
                        "\"$OUT/usr/lib/\"",
                    ])),
                }],
                else_: vec![Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(&[
                        "cp",
                        "-a",
                        "/usr/lib/libz.so.1*",
                        "\"$OUT/usr/lib/\"",
                    ])),
                }],
            });

        Recipe::Steps(steps)
    }
}

impl Bootstrap for Rust {
    /// シード相は distro rust を stage0 に ambient cc でビルドする。
    fn seed(&self) -> bool {
        true
    }
}
