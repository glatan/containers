//! gcc ツール固有の定義（`Prefixed{stage: Gcc}` から分離）。
use super::binutils::VERSION as BINUTILS_VERSION;
use super::stages::dep;
use crate::flags::{Cflag, Flag};
use crate::package::GLIBC_VERSION;
use crate::package::LINUX_HEADERS;
use crate::package::recipe::{Arg, BuildContext, Cond, Env, Pipe, Step, Steps, Workdir};
use crate::package::{Dependency, Source};
use crate::target::{Amd64Variant, Target};

pub const LICENSE: &str = "GPL-3.0-or-later WITH GCC-exception-3.1";

/// stage1/stage2 共通の `case "$TRIPLE"` → arch flags（静的解決）。
/// 最終 gcc は `$_baseline` 補間・cet/decimal-float 付きで別処理。
pub fn stage_arch_flags(triple: &str) -> &'static [&'static str] {
    match triple {
        "x86_64-linux-gnu" => &["--with-arch=x86-64"],
        "aarch64-linux-gnu" => &["--with-arch=armv8-a"],
        "riscv64-linux-gnu" => &["--with-arch=rv64gc", "--with-abi=lp64d"],
        "powerpc64le-linux-gnu" => &["--with-cpu=power8"],
        "s390x-linux-gnu" => &["--with-arch=z900"],
        triple => panic!("未対応の triple: {triple}"),
    }
}

/// stage1/stage2 の pacman 一覧（両ステージで共通）。
pub const PACMAN_STAGE_SEED: &[&str] = &[
    "gcc", "make", "bison", "flex", "perl", "texinfo", "m4", "gmp", "mpfr", "libmpc", "libisl",
    "zstd", "zlib", "mold",
];
pub const PACMAN_STAGE: &[&str] = &[
    "make", "bison", "flex", "perl", "texinfo", "m4", "gmp", "mpfr", "libmpc", "libisl", "zstd",
    "zlib",
];
pub const PACMAN: &[&str] = &["make", "bison", "flex", "perl", "texinfo", "m4"];

pub const SOURCE: Source = Source {
    url: "https://ftp.gnu.org/gnu/gcc/gcc-16.2.0/gcc-16.2.0.tar.xz",
    sha3_512: "1995b2fd6c2b19fe8ca47c836fdee8c019909ab8857f81a8045a7726ce724146ab63cd8f836fb460e9b009434ea97eb51b34397760b9ec4bbcac490ca32fa0f6",
    blake3: "c0f53af0bbfd2bf28896ab9bdfd1a534351b851c5a2f0226feb6a4323884c43531cbb207c733e8fd8ac16e2e1b03e6d8f17d043a17cf026857eedf26c31e4e0e",
    subdir: "gcc-16.2.0",
};

pub const RUNTIME_LIBS: [Dependency; 6] = [
    Dependency {
        name: "gmp",
        version: "6.3.0",
    },
    Dependency {
        name: "mpfr",
        version: "4.2.2",
    },
    Dependency {
        name: "mpc",
        version: "1.4.1",
    },
    Dependency {
        name: "isl",
        version: "0.26",
    },
    Dependency {
        name: "zlib",
        version: "1.3.2",
    },
    Dependency {
        name: "zstd",
        version: "1.5.7",
    },
];

const HOST_TOOLS: [Dependency; 5] = [
    dep("bison", "3.8.2"),
    dep("flex", "2.6.4"),
    dep("perl", "5.42.3"),
    dep("texinfo", "7.3"),
    dep("m4", "1.4.21"),
];

const X86_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("x86_64-linux-gnu-glibc", GLIBC_VERSION),
];
const AARCH64_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("aarch64-linux-gnu-glibc", GLIBC_VERSION),
];
const RISCV64_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("riscv64-linux-gnu-glibc", GLIBC_VERSION),
];
const PPC64LE_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
    dep("powerpc64le-linux-gnu-glibc", GLIBC_VERSION),
];
const S390X_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
    dep("s390x-linux-gnu-glibc", GLIBC_VERSION),
];

fn gcc_build_deps(base: &[Dependency]) -> Vec<Dependency> {
    let mut dependencies = base.to_vec();
    dependencies.extend_from_slice(&HOST_TOOLS);
    dependencies
}

pub fn build_deps(triple: &str) -> Vec<Dependency> {
    let base = match triple {
        "x86_64-linux-gnu" => &X86_GCC_BUILD,
        "aarch64-linux-gnu" => &AARCH64_GCC_BUILD,
        "riscv64-linux-gnu" => &RISCV64_GCC_BUILD,
        "powerpc64le-linux-gnu" => &PPC64LE_GCC_BUILD,
        _ => &S390X_GCC_BUILD,
    };
    gcc_build_deps(base)
}

pub fn deps(target: Target) -> Vec<Dependency> {
    crate::package::with_libc(target, &RUNTIME_LIBS)
}

/// コンパイラ自身のバイナリへ hardening。StrictFlexArrays3 は除外
/// （GCC が [1] 末尾配列を使い FORTIFY=3 と相性が悪い）。
pub fn flags(target: Target) -> Flag {
    Flag::new(
        vec![
            Cflag::O2,
            Cflag::Pipe,
            Cflag::March(target),
            Cflag::Hardened,
            Cflag::NoWhardened,
        ],
        vec![],
    )
}

/// 最終 gcc。`case "$TRIPLE"`/`case "$ARCH"`・Canadian wrap・PGO を
/// 静的解決・型付けしたビルド。
pub fn recipe(ctx: &BuildContext) -> Steps {
    let cmd = |argv: Vec<Arg>| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(argv),
    };
    let concat = |parts: Vec<Arg>| Arg::Concat(parts);

    let cet = ctx.triple == "x86_64-linux-gnu";
    let sysroot = || concat(vec![Arg::lit("/usr/"), Arg::Var("TRIPLE")]);
    let dep_binutils = |sub: &'static str| {
        concat(vec![
            Arg::lit("/dep/"),
            Arg::Var("TRIPLE"),
            Arg::lit("-binutils/usr/bin/"),
            Arg::Var("TRIPLE"),
            Arg::lit(sub),
        ])
    };
    let usr_bin = |sub: &'static str| {
        concat(vec![
            Arg::lit("/usr/bin/"),
            Arg::Var("TRIPLE"),
            Arg::lit(sub),
        ])
    };

    // -- Canadian wrap ----------------------------------------------------
    // build-libcpp が HOST の *-g++ を CFLAGS ごと使うため、ネイティブ以外は
    // PACKAGE_HOST 向けフラグを CC/CXX ラッパへ焼き込み環境からは外す。
    // #!/bin/sh は ncurses 警告が stdout に混ざり LD キャッシュを壊すので
    // ラッパは perl。生成器を WriteFile で吐き `perl gen.pl dest real args…`
    // で駆動する（シェル関数 write_canadian_wrap の型付き版）。
    let canadian_steps = || -> Vec<Step> {
        let real = |tool: &'static str| {
            Arg::Subst(Box::new(Pipe::cmd(vec![
                Arg::lit("command"),
                Arg::lit("-v"),
                concat(vec![Arg::Var("PACKAGE_HOST"), Arg::lit(tool)]),
            ])))
        };
        let rewrite = |var: &'static str| -> Step {
            Step::Set {
                name: var,
                value: Arg::Subst(Box::new(Pipe::new(vec![
                    vec![
                        Arg::lit("printf"),
                        Arg::lit("%s"),
                        Arg::quoted(Arg::Var(var)),
                    ],
                    vec![
                        Arg::lit("sed"),
                        Arg::quoted(concat(vec![
                            Arg::lit("s|/dep/"),
                            Arg::Var("PACKAGE_HOST"),
                            Arg::lit("-glibc/usr/"),
                            Arg::Var("PACKAGE_HOST"),
                            Arg::lit("|"),
                            Arg::Var("SYSROOT"),
                            Arg::lit("|g"),
                        ])),
                    ],
                ]))),
            }
        };
        let wrap = |name: &'static str, real_path: Arg, flags: &[&'static str]| {
            let mut argv = vec![
                Arg::lit("perl"),
                Arg::lit("/tmp/gen-wrap.pl"),
                concat(vec![
                    Arg::Var("_wrap"),
                    Arg::lit("/"),
                    Arg::Var("PACKAGE_HOST"),
                    Arg::lit(name),
                ]),
                real_path,
            ];
            argv.extend(flags.iter().map(|f| Arg::lit(*f)));
            cmd(argv)
        };
        vec![
            Step::WriteFile {
                path: Arg::lit("/tmp/gen-wrap.pl"),
                lines: vec![
                    Arg::lit("my ($dest, @cmd) = @ARGV;"),
                    Arg::lit("open my $f, '>', $dest or die $!;"),
                    Arg::lit("print $f \"#!$^X\\n\";"),
                    Arg::lit("print $f \"use strict;\\n\";"),
                    Arg::lit("print $f \"my \\@cmd = (\\n\";"),
                    Arg::lit(
                        "print $f \"  '$_',\\n\" for map { my $q = $_; $q =~ s/'/\\\\'/g; $q } @cmd;",
                    ),
                    Arg::lit("print $f \");\\n\";"),
                    Arg::lit("print $f \"exec \\@cmd, \\@ARGV;\\n\";"),
                    Arg::lit("close $f;"),
                    Arg::lit("chmod 0755, $dest;"),
                ],
                append: false,
                expand: false,
            },
            Step::Set {
                name: "_wrap",
                value: Arg::lit("/tmp/canadian-host-cc"),
            },
            cmd(vec![Arg::lit("mkdir"), Arg::lit("-p"), Arg::Var("_wrap")]),
            Step::Set {
                name: "_real_cc",
                value: real("-gcc"),
            },
            Step::Set {
                name: "_real_cxx",
                value: real("-g++"),
            },
            Step::Set {
                name: "_cflags",
                value: Arg::lit("${CFLAGS:-}"),
            },
            Step::Set {
                name: "_cxxflags",
                value: Arg::lit("${CXXFLAGS:-}"),
            },
            Step::Set {
                name: "_ldflags",
                value: Arg::lit("${LDFLAGS:-}"),
            },
            // host==target の libgcc_s は HOST gcc でリンクするので、
            // wrap の -B/dep/...-glibc（未修正の //lib を持つ）を SYSROOT へ向ける。
            Step::If {
                cond: Cond::Cmd(Pipe::cmd(vec![
                    Arg::lit("["),
                    Arg::quoted(Arg::Var("PACKAGE_HOST")),
                    Arg::lit("="),
                    Arg::quoted(Arg::Var("TRIPLE")),
                    Arg::lit("]"),
                ])),
                then: vec![
                    rewrite("_cflags"),
                    rewrite("_cxxflags"),
                    rewrite("_ldflags"),
                ],
                else_: vec![],
            },
            wrap("-gcc", Arg::Var("_real_cc"), &["${_cflags}", "${_ldflags}"]),
            wrap(
                "-g++",
                Arg::Var("_real_cxx"),
                &["${_cxxflags}", "${_ldflags}"],
            ),
            Step::Export {
                name: "PATH",
                value: Arg::quoted(concat(vec![
                    Arg::Var("_wrap"),
                    Arg::lit(":"),
                    Arg::Var("PATH"),
                ])),
            },
            Step::Export {
                name: "CC",
                value: Arg::quoted(concat(vec![Arg::Var("PACKAGE_HOST"), Arg::lit("-gcc")])),
            },
            Step::Export {
                name: "CXX",
                value: Arg::quoted(concat(vec![Arg::Var("PACKAGE_HOST"), Arg::lit("-g++")])),
            },
            Step::Unset("CFLAGS"),
            Step::Unset("CXXFLAGS"),
            Step::Unset("LDFLAGS"),
            Step::Unset("CPPFLAGS"),
            // TARGET_EXPORTS が CPP を上書きしないため、プリリュードの
            // CPP=gcc -E が target configure へ漏れないよう落とす。
            Step::Unset("CPP"),
            // wrap 動作検査（- は stdin のソース）。
            Step::WriteFile {
                path: Arg::lit("/tmp/canadian-test.c"),
                lines: vec![Arg::lit("int main(void) { return 0; }")],
                append: false,
                expand: false,
            },
            Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(vec![
                    Arg::Var("CC"),
                    Arg::lit("-x"),
                    Arg::lit("c"),
                    Arg::lit("-o"),
                    Arg::lit("/tmp/canadian-cc-ok"),
                    Arg::lit("-"),
                ])
                .stdin(Arg::lit("/tmp/canadian-test.c")),
            },
        ]
    };

    // -- arch flags --------------------------------------------------------
    // $MARCH は PACKAGE_HOST（成果物が動く Platform）の ISA。host arch と
    // codegen arch が一致する場合のみ使い、異種クロスでは triple
    // ベースラインへフォールバックする（_host_arch==_target_arch の静的解決）。
    let same_arch = ctx.package_host.split('-').next() == ctx.triple.split('-').next();
    let baseline_lit = match ctx.triple.as_str() {
        "x86_64-linux-gnu" => "x86-64",
        "aarch64-linux-gnu" => "armv8-a",
        "riscv64-linux-gnu" => "rv64gc",
        "powerpc64le-linux-gnu" => "power8",
        "s390x-linux-gnu" => "z900",
        triple => panic!("未対応の triple: {triple}"),
    };
    let baseline = if same_arch {
        Arg::Var("MARCH")
    } else {
        Arg::lit(baseline_lit)
    };
    let arch_args: Vec<Arg> = match ctx.triple.as_str() {
        "x86_64-linux-gnu" => vec![
            Arg::Opt("with-arch", Box::new(baseline)),
            Arg::lit("--enable-cet"),
        ],
        "aarch64-linux-gnu" => vec![Arg::Opt("with-arch", Box::new(baseline))],
        "riscv64-linux-gnu" => vec![
            Arg::Opt("with-arch", Box::new(baseline)),
            Arg::lit("--with-abi=lp64d"),
        ],
        "powerpc64le-linux-gnu" => vec![Arg::Opt("with-cpu", Box::new(baseline))],
        "s390x-linux-gnu" => vec![
            Arg::Opt("with-arch", Box::new(baseline)),
            Arg::lit("--enable-decimal-float"),
        ],
        _ => unreachable!(),
    };
    let sanitizer = if cet { "" } else { "--disable-libsanitizer" };

    // -- configure_final_gcc（クロージャへ展開） --------------------------
    let configure = |env: Vec<(&'static str, Arg)>| -> Step {
        let mut argv = vec![
            Arg::Concat(vec![Arg::Var("SOURCE"), Arg::lit("/configure")]),
            Arg::Opt("build", Box::new(Arg::Var("HOST_TRIPLE"))),
            // ツールはビルド機で動くクロスツール。--host=HOST_TRIPLE が正しい
            //（=PACKAGE_HOST は host==target になる誤り）。prefixed 名は
            // host!=target 時の自動付与に頼らず --program-prefix で明示する
            //（x86_64 は HOST_TRIPLE==TRIPLE で自動付与が効かない）。
            Arg::Opt("host", Box::new(Arg::Var("HOST_TRIPLE"))),
            Arg::Opt("target", Box::new(Arg::Var("TRIPLE"))),
            Arg::Opt(
                "program-prefix",
                Box::new(Arg::Concat(vec![Arg::Var("TRIPLE"), Arg::lit("-")])),
            ),
            Arg::lit("--prefix=/usr"),
            Arg::Opt("with-sysroot", Box::new(Arg::Var("SYSROOT"))),
            Arg::Opt("with-build-sysroot", Box::new(Arg::Var("BUILD_SYSROOT"))),
            Arg::lit("--with-native-system-header-dir=/include"),
            Arg::Opt("with-as", Box::new(usr_bin("-as"))),
            Arg::Opt("with-ld", Box::new(usr_bin("-ld"))),
            Arg::Concat(vec![
                Arg::lit("--infodir=/usr/share/info/"),
                Arg::Var("TRIPLE"),
            ]),
            Arg::lit("--with-system-zlib"),
            Arg::Var("GCC_LIB_FLAGS"),
        ];
        argv.append(&mut arch_args.clone());
        argv.extend(
            [
                "--disable-nls",
                "--disable-werror",
                "--disable-libssp",
                "--disable-libstdcxx-pch",
                "--disable-bootstrap",
                "--disable-multilib",
            ]
            .iter()
            .map(|f| Arg::lit(*f)),
        );
        if !sanitizer.is_empty() {
            argv.push(Arg::lit(sanitizer));
        }
        argv.extend(
            [
                "--enable-plugin",
                "--enable-languages=c,c++",
                "--enable-lto",
                "--enable-checking=release",
                "--enable-__cxa_atexit",
                "--enable-threads=posix",
                "--enable-shared",
                "--enable-default-pie",
                "--enable-default-ssp",
                "--with-linker-hash-style=gnu",
                "--enable-linker-build-id",
                "--enable-gnu-indirect-function",
                "--enable-gnu-unique-object",
                "--enable-initfini-array",
                "--enable-libstdcxx-backtrace",
                "--enable-host-pie",
                "--enable-host-bind-now",
            ]
            .iter()
            .map(|f| Arg::lit(*f)),
        );
        argv.push(Arg::Opt(
            "with-specs",
            Box::new(Arg::lit("'%{!fno-hardened:-fhardened} -Wno-hardened'")),
        ));
        Step::Exec {
            env,
            pipe: Pipe::cmd(argv),
        }
    };

    // -- 前半ステップ ------------------------------------------------------
    let build_root = concat(vec![
        Arg::lit("/dep/"),
        Arg::Var("HOST_TRIPLE"),
        Arg::lit("-glibc/usr/"),
        Arg::Var("HOST_TRIPLE"),
    ]);
    let glibc_dep = concat(vec![
        Arg::lit("/dep/"),
        Arg::Var("TRIPLE"),
        Arg::lit("-glibc/usr/"),
        Arg::Var("TRIPLE"),
    ]);
    let dep_inc = |name: &'static str| {
        concat(vec![
            Arg::lit("/dep/"),
            Arg::lit(name),
            Arg::lit("/usr/include"),
        ])
    };

    let mut steps: Vec<Step> = vec![
        // GCC の TARGET_EXPORTS は CC/CXX/AR 等を target 向けに上書きするが
        // CPP は含まれない。プリリュードの CPP=gcc -E が target libgcc の
        // configure に漏れてホストヘッダを誤検出するため落とす
        //（target 側は CC -E＝xgcc -E が使われる）。
        Step::Unset("CPP"),
        Step::Export {
            name: "PATH",
            value: Arg::quoted(concat(vec![
                Arg::lit("/dep/"),
                Arg::Var("TRIPLE"),
                Arg::lit("-binutils/usr/bin:"),
                Arg::Var("PATH"),
            ])),
        },
        Step::Export {
            name: "CC_FOR_BUILD",
            value: concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit("-gcc")]),
        },
        Step::Export {
            name: "CXX_FOR_BUILD",
            value: concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit("-g++")]),
        },
        // wrap の CFLAGS は PACKAGE_HOST（成果物が動く Platform）向け。
        // Canadian では build-libcpp が HOST の *-g++ を CFLAGS ごと使うので
        // ビルドマシン用を分ける。
        Step::If {
            cond: Cond::DirExists(build_root.clone()),
            then: vec![
                // build 側コンパイラ（distro/host gcc）が読むのは
                // $root/usr/include のみだが dep glibc の layout は
                // usr/{triple}/include。標準 layout を模した shadow を
                // /tmp/build-sysroot に張って差し替える（#include_next
                // も届くよう CFLAGS 側と同じ手法）。
                cmd(vec![
                    Arg::lit("mkdir"),
                    Arg::lit("-p"),
                    Arg::lit("/tmp/build-sysroot/usr"),
                ]),
                cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::quoted(concat(vec![build_root.clone(), Arg::lit("/include")])),
                    Arg::lit("/tmp/build-sysroot/include"),
                ]),
                cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::quoted(concat(vec![build_root.clone(), Arg::lit("/include")])),
                    Arg::lit("/tmp/build-sysroot/usr/include"),
                ]),
                cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::quoted(concat(vec![build_root.clone(), Arg::lit("/lib")])),
                    Arg::lit("/tmp/build-sysroot/lib"),
                ]),
                cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::quoted(concat(vec![build_root.clone(), Arg::lit("/lib")])),
                    Arg::lit("/tmp/build-sysroot/usr/lib"),
                ]),
                cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::quoted(concat(vec![build_root.clone(), Arg::lit("/lib64")])),
                    Arg::lit("/tmp/build-sysroot/lib64"),
                ]),
                cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::quoted(concat(vec![build_root.clone(), Arg::lit("/lib64")])),
                    Arg::lit("/tmp/build-sysroot/usr/lib64"),
                ]),
                Step::Set {
                    name: "_build_root",
                    value: Arg::lit("/tmp/build-sysroot"),
                },
            ],
            else_: vec![],
        },
        Step::Set {
            name: "_bf",
            value: Arg::quoted(Arg::lit("-O2 -pipe")),
        },
        Step::If {
            cond: Cond::EnvSet("_build_root"),
            then: vec![
                Step::Set {
                    name: "_bf",
                    value: Arg::quoted(concat(vec![
                        Arg::Var("_bf"),
                        Arg::lit(" --sysroot="),
                        Arg::Var("_build_root"),
                    ])),
                },
                Step::If {
                    cond: Cond::FileExists(concat(vec![
                        Arg::Var("_build_root"),
                        Arg::lit("/lib/crti.o"),
                    ])),
                    then: vec![Step::Set {
                        name: "_bf",
                        value: Arg::quoted(concat(vec![
                            Arg::Var("_bf"),
                            Arg::lit(" -B"),
                            Arg::Var("_build_root"),
                            Arg::lit("/lib"),
                        ])),
                    }],
                    else_: vec![Step::If {
                        cond: Cond::FileExists(concat(vec![
                            Arg::Var("_build_root"),
                            Arg::lit("/lib64/crti.o"),
                        ])),
                        then: vec![Step::Set {
                            name: "_bf",
                            value: Arg::quoted(concat(vec![
                                Arg::Var("_bf"),
                                Arg::lit(" -B"),
                                Arg::Var("_build_root"),
                                Arg::lit("/lib64"),
                            ])),
                        }],
                        else_: vec![],
                    }],
                },
            ],
            else_: vec![],
        },
        Step::Export {
            name: "CFLAGS_FOR_BUILD",
            value: Arg::quoted(Arg::Var("_bf")),
        },
        Step::Export {
            name: "CXXFLAGS_FOR_BUILD",
            value: Arg::quoted(Arg::Var("_bf")),
        },
        Step::Export {
            name: "LDFLAGS_FOR_BUILD",
            value: Arg::quoted(Arg::lit("${_build_root:+--sysroot=$_build_root}")),
        },
        // final gcc のみ per-variant。libgcc/libstdc++ 等のランタイムを
        // variant ISA でビルドし、フラグを落としたパッケージの既定も底上げする。
        // stage1/stage2 は BUILD_ENV でしかビルドされないため triple
        // ベースラインのまま。
        Step::Set {
            name: "SYSROOT",
            value: sysroot(),
        },
        Step::Set {
            name: "_profiledir",
            value: Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/pgo-profile-data")]),
        },
        // 前回失敗の config.cache / 壊れた Makefile を残さない。
        cmd(vec![
            Arg::lit("rm"),
            Arg::lit("-rf"),
            Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/instrumented")]),
            Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/final")]),
        ]),
        cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            Arg::Var("SYSROOT"),
            Arg::Var("_profiledir"),
            Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/instrumented")]),
            Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/final")]),
        ]),
        Step::If {
            cond: Cond::DirExists(glibc_dep.clone()),
            then: vec![cmd(vec![
                Arg::lit("cp"),
                Arg::lit("-a"),
                Arg::Concat(vec![glibc_dep, Arg::lit("/.")]),
                Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/")]),
            ])],
            else_: vec![],
        },
        // 既存 glibc 成果物も //lib の ld script を持つ。
        // xgcc の --sysroot が効く形へ直す。
        super::stages::ld_script_fixup(Arg::Var("SYSROOT")),
        Step::Set {
            name: "BUILD_SYSROOT",
            value: Arg::Var("SYSROOT"),
        },
        Step::Set {
            name: "HDR",
            value: Arg::Concat(vec![
                Arg::lit("/dep/linux-headers/usr/"),
                Arg::Var("TRIPLE"),
                Arg::lit("/include"),
            ]),
        },
        Step::Or {
            lhs: vec![cmd(vec![
                Arg::lit("["),
                Arg::lit("-d"),
                Arg::Var("HDR"),
                Arg::lit("]"),
            ])],
            rhs: vec![Step::Set {
                name: "HDR",
                value: Arg::lit("/dep/linux-headers/usr/include"),
            }],
        },
        cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            Arg::Concat(vec![Arg::Var("BUILD_SYSROOT"), Arg::lit("/include")]),
        ]),
        Step::Or {
            lhs: vec![cmd(vec![
                Arg::lit("cp"),
                Arg::lit("-a"),
                Arg::Concat(vec![Arg::Var("HDR"), Arg::lit("/.")]),
                Arg::Concat(vec![Arg::Var("BUILD_SYSROOT"), Arg::lit("/include/")]),
            ])],
            rhs: vec![cmd(vec![Arg::lit("true")])],
        },
        // wrap の CFLAGS は HOST 用。CFLAGS_FOR_TARGET がそれを継承すると
        // /dep の未修正 libc.so を -B で拾う。
        Step::Export {
            name: "CFLAGS_FOR_TARGET",
            value: Arg::quoted(concat(vec![
                Arg::lit("--sysroot="),
                Arg::Var("BUILD_SYSROOT"),
                Arg::lit(" -ftrivial-auto-var-init=uninitialized"),
            ])),
        },
        Step::Export {
            name: "CXXFLAGS_FOR_TARGET",
            value: Arg::quoted(concat(vec![
                Arg::lit("--sysroot="),
                Arg::Var("BUILD_SYSROOT"),
                Arg::lit(" -ftrivial-auto-var-init=uninitialized"),
            ])),
        },
        Step::Export {
            name: "LDFLAGS_FOR_TARGET",
            value: Arg::quoted(concat(vec![
                Arg::lit("--sysroot="),
                Arg::Var("BUILD_SYSROOT"),
            ])),
        },
        cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            dep_binutils("-as"),
            usr_bin("-as"),
        ]),
        cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            dep_binutils("-ld"),
            usr_bin("-ld"),
        ]),
        Step::If {
            cond: Cond::DirExists(Arg::lit("/dep/gmp/usr")),
            then: vec![Step::Set {
                name: "GCC_LIB_FLAGS",
                value: Arg::quoted(Arg::lit(
                    "--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr",
                )),
            }],
            else_: vec![Step::Set {
                name: "GCC_LIB_FLAGS",
                value: Arg::lit("--with-isl"),
            }],
        },
    ];
    for dep in ["zlib", "zstd"] {
        steps.push(Step::If {
            cond: Cond::DirExists(dep_inc(dep)),
            then: vec![
                Step::Export {
                    name: "CFLAGS",
                    value: Arg::quoted(concat(vec![
                        Arg::lit("${CFLAGS:-} -I/dep/"),
                        Arg::lit(dep),
                        Arg::lit("/usr/include"),
                    ])),
                },
                Step::Export {
                    name: "CXXFLAGS",
                    value: Arg::quoted(concat(vec![
                        Arg::lit("${CXXFLAGS:-} -I/dep/"),
                        Arg::lit(dep),
                        Arg::lit("/usr/include"),
                    ])),
                },
                Step::Export {
                    name: "LDFLAGS",
                    value: Arg::quoted(concat(vec![
                        Arg::lit("${LDFLAGS:-} -L/dep/"),
                        Arg::lit(dep),
                        Arg::lit("/usr/lib64 -L/dep/"),
                        Arg::lit(dep),
                        Arg::lit("/usr/lib"),
                    ])),
                },
            ],
            else_: vec![],
        });
    }
    if ctx.canadian() {
        steps.extend(canadian_steps());
    }

    let pgo = !ctx.canadian() && ctx.target == Target::Amd64(Amd64Variant::V4);
    if !pgo {
        if ctx.canadian() {
            for (name, tool) in [
                ("CC_FOR_TARGET", "-gcc"),
                ("CXX_FOR_TARGET", "-g++"),
                ("GCC_FOR_TARGET", "-gcc"),
                ("AR_FOR_TARGET", "-ar"),
                ("RANLIB_FOR_TARGET", "-ranlib"),
                ("NM_FOR_TARGET", "-nm"),
                ("AS_FOR_TARGET", "-as"),
                ("LD_FOR_TARGET", "-ld"),
            ] {
                steps.push(Step::Export {
                    name,
                    value: Arg::quoted(concat(vec![Arg::Var("TRIPLE"), Arg::lit(tool)])),
                });
            }
        }
        steps.extend([
            Step::Cd(Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/final")])),
            configure(vec![]),
            Step::Make {
                jobs: true,
                args: vec![],
            },
            Step::Make {
                jobs: false,
                args: vec!["DESTDIR=\"$OUT\"".to_string(), "install".to_string()],
            },
            cmd(vec![Arg::lit("exit"), Arg::lit("0")]),
        ]);
        let mut s = Steps::new(Env::None).sysroot(true).workdir(Workdir::None);
        s.steps = steps;
        return s;
    }

    // -- PGO（ネイティブ amd64/v4 のみ）------------------------------------
    let gen_env = |var: &'static str| -> (&'static str, Arg) {
        (
            var,
            Arg::quoted(Arg::Concat(vec![
                Arg::owned(format!("${{{var}:-}} ")),
                Arg::lit(if var == "LDFLAGS" {
                    "-fprofile-generate=${_profiledir}"
                } else {
                    "-fprofile-generate=${_profiledir} -fprofile-update=atomic"
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
                    "-fprofile-use=${_profiledir} -fprofile-correction \
                     -Wno-missing-profile -Wno-error=coverage-mismatch"
                }),
            ])),
        )
    };
    steps.extend([
        Step::Cd(Arg::Concat(vec![
            Arg::Var("BUILD"),
            Arg::lit("/instrumented"),
        ])),
        configure(vec![
            gen_env("CFLAGS"),
            gen_env("CXXFLAGS"),
            gen_env("LDFLAGS"),
        ]),
        Step::Make {
            jobs: true,
            args: vec![],
        },
        Step::Set {
            name: "_xgcc",
            value: Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/instrumented/gcc/xgcc")]),
        },
        Step::Set {
            name: "_xgpp",
            value: Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/instrumented/gcc/xg++")]),
        },
        Step::Set {
            name: "_B",
            value: Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/instrumented/gcc/")]),
        },
        Step::Set {
            name: "_ts",
            value: Arg::Concat(vec![Arg::Var("SOURCE"), Arg::lit("/gcc/testsuite")]),
        },
    ]);
    // c-torture を並列コンパイルしてプロファイルを集める。
    steps.push(Step::ForEach {
        var: "_opt",
        items: ["-O0", "-O1", "-O2", "-O3", "-Os"]
            .iter()
            .map(|o| Arg::lit(*o))
            .collect(),
        body: vec![Step::Or {
            lhs: vec![Step::Exec {
                env: vec![],
                pipe: Pipe::new(vec![
                    vec![
                        Arg::lit("find"),
                        Arg::Concat(vec![Arg::Var("_ts"), Arg::lit("/gcc.c-torture/compile")]),
                        Arg::lit("-maxdepth"),
                        Arg::lit("1"),
                        Arg::lit("-name"),
                        Arg::quoted(Arg::lit("*.c")),
                        Arg::lit("-print0"),
                    ],
                    vec![
                        Arg::lit("xargs"),
                        Arg::lit("-0"),
                        Arg::nproc_jobs("-P"),
                        Arg::lit("-I{}"),
                        Arg::Var("_xgcc"),
                        Arg::Concat(vec![Arg::lit("-B"), Arg::Var("_B")]),
                        Arg::Var("_opt"),
                        Arg::lit("-S"),
                        Arg::lit("-o"),
                        Arg::lit("/dev/null"),
                        Arg::lit("-w"),
                        Arg::lit("{}"),
                    ],
                ])
                .stderr(Arg::lit("/dev/null")),
            }],
            rhs: vec![cmd(vec![Arg::lit("true")])],
        }],
        stdout: None,
    });
    steps.push(Step::Or {
        lhs: vec![Step::Exec {
            env: vec![],
            pipe: Pipe::new(vec![
                vec![
                    Arg::lit("find"),
                    Arg::Concat(vec![Arg::Var("_ts"), Arg::lit("/g++.dg")]),
                    Arg::lit("-maxdepth"),
                    Arg::lit("1"),
                    Arg::lit("-name"),
                    Arg::quoted(Arg::lit("*.C")),
                    Arg::lit("-print0"),
                ],
                vec![
                    Arg::lit("xargs"),
                    Arg::lit("-0"),
                    Arg::nproc_jobs("-P"),
                    Arg::lit("-I{}"),
                    Arg::Var("_xgpp"),
                    Arg::Concat(vec![Arg::lit("-B"), Arg::Var("_B")]),
                    Arg::lit("-O2"),
                    Arg::lit("-S"),
                    Arg::lit("-o"),
                    Arg::lit("/dev/null"),
                    Arg::lit("-w"),
                    Arg::lit("{}"),
                ],
            ])
            .stderr(Arg::lit("/dev/null")),
        }],
        rhs: vec![cmd(vec![Arg::lit("true")])],
    });
    steps.extend([
        Step::Cd(Arg::Concat(vec![Arg::Var("BUILD"), Arg::lit("/final")])),
        configure(vec![
            use_env("CFLAGS"),
            use_env("CXXFLAGS"),
            use_env("LDFLAGS"),
        ]),
        Step::Make {
            jobs: true,
            args: vec![],
        },
        Step::Make {
            jobs: false,
            args: vec!["DESTDIR=\"$OUT\"".to_string(), "install".to_string()],
        },
    ]);

    let mut s = Steps::new(Env::None).sysroot(true).workdir(Workdir::None);
    s.steps = steps;
    s
}
