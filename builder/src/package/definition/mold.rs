use crate::flags::{Cflag, Flag, Ldflag};
use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir, argv};
use crate::package::{AuxSource, Bootstrap, Dependency, Package, Source, dep};
use crate::target::Target;

pub struct Mold;

/// GNU triple → cargo クロスビルド用の環境変数名対応表。
/// linker は cargo の `CARGO_TARGET_{TRIPLE}_LINKER`、cc crate は
/// `CC_{triple}`（アンダースコア区切り）で target 側コンパイラを解決する。
/// x86_64（ネイティブ）は ambient cc でよいため含まない。
struct CrossEnv {
    gnu: &'static str,
    rust: &'static str,
    linker_env: &'static str,
    cc_env: &'static str,
    cxx_env: &'static str,
    ar_env: &'static str,
    cflags_env: &'static str,
}

/// ホスト triple（x86_64）側のツールチェーン参照。cargo の
/// build-dependencies（build.rs / proc-macro 内の cc-rs）がホスト向けに
/// C を組むときに使う。
fn host_tool(suffix: &'static str) -> Arg {
    Arg::quoted(Arg::Concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit(suffix)]))
}

const CROSS_ENVS: &[CrossEnv] = &[
    CrossEnv {
        gnu: "aarch64-linux-gnu",
        rust: "aarch64-unknown-linux-gnu",
        linker_env: "CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER",
        cc_env: "CC_aarch64_unknown_linux_gnu",
        cxx_env: "CXX_aarch64_unknown_linux_gnu",
        ar_env: "AR_aarch64_unknown_linux_gnu",
        cflags_env: "CFLAGS_aarch64_unknown_linux_gnu",
    },
    CrossEnv {
        gnu: "riscv64-linux-gnu",
        rust: "riscv64gc-unknown-linux-gnu",
        linker_env: "CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER",
        cc_env: "CC_riscv64gc_unknown_linux_gnu",
        cxx_env: "CXX_riscv64gc_unknown_linux_gnu",
        ar_env: "AR_riscv64gc_unknown_linux_gnu",
        cflags_env: "CFLAGS_riscv64gc_unknown_linux_gnu",
    },
    CrossEnv {
        gnu: "powerpc64le-linux-gnu",
        rust: "powerpc64le-unknown-linux-gnu",
        linker_env: "CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER",
        cc_env: "CC_powerpc64le_unknown_linux_gnu",
        cxx_env: "CXX_powerpc64le_unknown_linux_gnu",
        ar_env: "AR_powerpc64le_unknown_linux_gnu",
        cflags_env: "CFLAGS_powerpc64le_unknown_linux_gnu",
    },
    CrossEnv {
        gnu: "s390x-linux-gnu",
        rust: "s390x-unknown-linux-gnu",
        linker_env: "CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER",
        cc_env: "CC_s390x_unknown_linux_gnu",
        cxx_env: "CXX_s390x_unknown_linux_gnu",
        ar_env: "AR_s390x_unknown_linux_gnu",
        cflags_env: "CFLAGS_s390x_unknown_linux_gnu",
    },
];

impl Package for Mold {
    fn name(&self) -> &'static str {
        "mold"
    }

    fn version(&self) -> &'static str {
        "3.0.0"
    }

    fn license(&self) -> &'static str {
        "MIT"
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/rui314/mold/archive/refs/tags/v3.0.0.tar.gz",
            sha3_512: "cd0ccfd5957fbc1329e1a8d0b829326a8a13d5b005e07a4d70a506e1bd775c0eb4b3cd46b3ecfbd78b7418c2e26981c7839fbd96991806d641ff9802229afd41",
            blake3: "6a14f3ebf8e69e73d001b33add174f6b838af6e5557779c952ee74ebeb173a5026a8df100b7663614ffd3651807902e64b33075d052951adcdcbd92eea7f95a3",
            subdir: "mold-3.0.0",
        }
    }

    /// `cargo vendor` 成果物。crates.io 依存に加え git 固定の
    /// `rui314/mimalloc_rust`（submodule の mimalloc C ソース込み）を
    /// 内包し、ネットワーク隔離されたビルドコンテナへ /src/vendor
    /// として供給する。`cargo vendor` 生成物には .cargo-checksum.json が
    /// 付くためファイル単位の完全性も cargo が検証する。
    fn aux_source(&self) -> Option<AuxSource> {
        Some(AuxSource {
            file: "mold-3.0.0-vendor.tar.xz",
            sha3_512: "8604f5847bee09e928d2f7ccdf565d6d02dab2bc3709eeb0efa695dca066e053afe02d3d728e139e437c5e7470faaccce96b5a07eeb3567d785f591cbbf59e2e",
            blake3: "d1d6814932014737cc992315a6c57554411793bd6ea7d98ddf89f011fd4897d85acd07201ce931bc31dcf1754496952f6eb8f8146bfa0faa9418fc9afe44dda9",
            subdir: "vendor",
        })
    }

    /// 3.x は Rust 製。zlib / zstd は libz-sys / zstd-sys が vendored の
    /// C ソースを静的に組むため実行時依存は不要。
    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[crate::package::gcc_dep(target)])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![crate::package::LINUX_HEADERS]
    }

    /// cargo を実行するため rust が必須の HOST 依存（Ordered: mold より先に
    /// ビルドする必要がある。seed 相では distro rust ではなく rust 成果物を
    /// 使う）。cc_deps は cc crate / mimalloc の C コンパイル用。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        deps.push(dep("rust", "1.99.0"));
        deps
    }

    fn flags(&self, target: Target) -> Flag {
        // mold 自身は -fuse-ld=mold でリンクできないため除外する。
        // cargo のリリースプロファイルには LTO 設定がなく、LLVM 規模の
        // ビルドでもないため LTO 系フラグも外す。
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

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        // vendor 設定: crates.io と git 固定の mimalloc_rust を
        // /src/vendor へ向ける。$SOURCE は読み取り専用のため
        // Workdir::InSource で $BUILD へコピーした上で .cargo/config.toml
        // を書く（cargo は cwd から上方へ config を探索する）。
        let mut steps = Steps::new(Env::None)
            .workdir(Workdir::InSource)
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["mkdir", "-p", "\"$BUILD/.cargo\""])),
            })
            // cargo の -C linker は単一プログラムしか取れず
            // CFLAGS/LDFLAGS 経由の --sysroot が載らないため、
            // prelude の /tmp/sysroot を焼いたリンカーラッパーを
            // 生成し CARGO_TARGET_*_LINKER に指定する。$CC は
            // コンテナごとのターゲット cc に解決される。
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["mkdir", "-p", "/build/ccw"])),
            })
            .step(Step::WriteFile {
                path: Arg::lit("/build/ccw/mold-link"),
                lines: argv(&[
                    "#!/bin/sh",
                    "exec \"$CC\" --sysroot=/tmp/sysroot -L/tmp/sysroot/lib -L/tmp/sysroot/lib64 \"$@\"",
                ]),
                append: false,
                expand: false,
            })
            .step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["chmod", "+x", "/build/ccw/mold-link"])),
            })
            .step(Step::WriteFile {
                path: Arg::lit("$BUILD/.cargo/config.toml"),
                lines: argv(&[
                    "[source.crates-io]",
                    "replace-with = \"vendored-sources\"",
                    "",
                    "[source.\"git+https://github.com/rui314/mimalloc_rust?rev=3979460494f1cd1e7f936cb8e10f41e927c9f698\"]",
                    "git = \"https://github.com/rui314/mimalloc_rust\"",
                    "rev = \"3979460494f1cd1e7f936cb8e10f41e927c9f698\"",
                    "replace-with = \"vendored-sources\"",
                    "",
                    "[source.vendored-sources]",
                    "directory = \"/src/vendor\"",
                ]),
                append: false,
                expand: false,
            });

        if !ctx.cross() {
            // ネイティブ（x86_64-unknown-linux-gnu）。cargo は build
            // script のリンクに PATH 上の `cc` を使い、/dep の生 gcc が
            // sysroot 無しで選ばれて crt ファイルを見失うため、host
            // triple の linker / cc を prelude のラップ済み $CC に
            // 固定する。
            steps = steps
                .step(Step::Exec {
                    env: vec![
                        (
                            "CARGO_HOME",
                            Arg::quoted(Arg::Concat(vec![
                                Arg::Var("BUILD"),
                                Arg::lit("/cargo-home"),
                            ])),
                        ),
                        (
                            "CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER",
                            Arg::lit("/build/ccw/mold-link"),
                        ),
                        ("CC_x86_64_unknown_linux_gnu", Arg::Var("CC")),
                        ("CXX_x86_64_unknown_linux_gnu", Arg::Var("CXX")),
                        ("AR_x86_64_unknown_linux_gnu", Arg::Var("AR")),
                        ("CFLAGS_x86_64_unknown_linux_gnu", Arg::Var("CFLAGS")),
                    ],
                    pipe: Pipe::cmd(argv(&[
                        "cargo",
                        "build",
                        "--release",
                        "--locked",
                        "--offline",
                    ])),
                })
                .step(Step::Exec {
                    env: vec![
                        ("DESTDIR", Arg::quoted(Arg::Var("OUT"))),
                        ("PREFIX", Arg::lit("/usr")),
                        ("MOLD_LIBDIR", Arg::lit("/usr/lib")),
                    ],
                    pipe: Pipe::cmd(argv(&["sh", "\"$BUILD/install-mold.sh\""])),
                });
        } else {
            let cross = CROSS_ENVS
                .iter()
                .find(|cross| cross.gnu == ctx.triple)
                .unwrap_or_else(|| panic!("mold: 未対応のクロス triple: {}", ctx.triple));
            steps = steps
                .step(Step::Exec {
                    env: vec![
                        (
                            "CARGO_HOME",
                            Arg::quoted(Arg::Concat(vec![
                                Arg::Var("BUILD"),
                                Arg::lit("/cargo-home"),
                            ])),
                        ),
                        (cross.linker_env, Arg::lit("/build/ccw/mold-link")),
                        (cross.cc_env, Arg::Var("CC")),
                        (cross.cxx_env, Arg::Var("CXX")),
                        (cross.ar_env, Arg::Var("AR")),
                        (cross.cflags_env, Arg::Var("CFLAGS")),
                        // cargo build-dependencies は HOST triple 向けに
                        // コンパイルされる。ambient の CC/CFLAGS はターゲット
                        // 向け（クロス gcc + sysroot フラグ）なので、cc-rs が
                        // 参照するホスト側変数を固定して分離する。
                        ("CC_x86_64_unknown_linux_gnu", host_tool("-gcc")),
                        ("CXX_x86_64_unknown_linux_gnu", host_tool("-g++")),
                        ("AR_x86_64_unknown_linux_gnu", host_tool("-ar")),
                        ("HOST_CC", host_tool("-gcc")),
                        ("HOST_CXX", host_tool("-g++")),
                        ("CFLAGS_x86_64_unknown_linux_gnu", Arg::lit("")),
                        ("CXXFLAGS_x86_64_unknown_linux_gnu", Arg::lit("")),
                        ("LDFLAGS_x86_64_unknown_linux_gnu", Arg::lit("")),
                    ],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("cargo"),
                        Arg::lit("build"),
                        Arg::lit("--release"),
                        Arg::lit("--locked"),
                        Arg::lit("--offline"),
                        Arg::lit("--target"),
                        Arg::lit(cross.rust),
                    ]),
                })
                .step(Step::Exec {
                    env: vec![
                        ("DESTDIR", Arg::quoted(Arg::Var("OUT"))),
                        ("PREFIX", Arg::lit("/usr")),
                        ("MOLD_LIBDIR", Arg::lit("/usr/lib")),
                        (
                            "CARGO_TARGET_DIR",
                            Arg::owned(format!("\"$BUILD/target/{}\"", cross.rust)),
                        ),
                    ],
                    pipe: Pipe::cmd(argv(&["sh", "\"$BUILD/install-mold.sh\""])),
                });
        }
        Recipe::Steps(steps)
    }

    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
}

impl Bootstrap for Mold {}
