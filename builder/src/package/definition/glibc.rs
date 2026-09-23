//! glibc ツール固有の定義（`Prefixed{stage: Glibc|GlibcHeaders}` から分離）。
use super::binutils::VERSION as BINUTILS_VERSION;
use super::stages::dep;
use crate::package::LINUX_HEADERS;
use crate::package::recipe::{Arg, BuildContext, Cond, Env, Pipe, Step, Steps, Workdir};
use crate::package::{Dependency, GCC_VERSION, Source};
use crate::target::Target;

pub const LICENSE: &str = "LGPL-2.1-or-later";
pub const PACMAN: &[&str] = &["python", "bison", "gawk", "make", "mold"];

/// glibc / glibc-headers の HOST 側依存。configure が python3/bison/m4 を
/// 要求する。python→openssl→glibc の循環は via(Seed) で断つ
/// （シード相は distro パッケージが担保し、env では成果物必須）。
pub fn host_deps<P: crate::package::Package + ?Sized>(
    package: &P,
    target: Target,
) -> Vec<Dependency> {
    let mut deps = crate::package::cc_deps(package, target);
    deps.extend([
        dep("python", "3.14.7").via(crate::package::Via::Seed("python")),
        dep("bison", "3.8.2").via(crate::package::Via::Seed("bison")),
        dep("m4", "1.4.21").via(crate::package::Via::Seed("m4")),
    ]);
    deps
}

pub const SOURCE: Source = Source {
    url: "https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz",
    sha3_512: "263dfb5e331b2033368c7455abcd9a9b15ab098ac963d5518ecf4bcb9eeb41298eb9976bf78a64af67d9471a50161bced2fa69d429c69759ea57f76b942ac521",
    blake3: "6d07bc16d8d066b1c0ae713d762175f261e27df9cab8015ec11e1c980c05e79b0fd5bab6cde6a90bf07651c432ca9039629376b57a96c9f3dcb3765f3745dbf4",
    subdir: "glibc-2.44",
};

const X86_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("x86_64-linux-gnu-gcc-stage2", GCC_VERSION),
];

const AARCH64_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("aarch64-linux-gnu-gcc-stage2", GCC_VERSION),
];

const RISCV64_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("riscv64-linux-gnu-gcc-stage2", GCC_VERSION),
];

const PPC64LE_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
    dep("powerpc64le-linux-gnu-gcc-stage2", GCC_VERSION),
];

const S390X_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
    dep("s390x-linux-gnu-gcc-stage2", GCC_VERSION),
];

pub fn build_deps(triple: &str) -> Vec<Dependency> {
    match triple {
        "x86_64-linux-gnu" => X86_BUILD.to_vec(),
        "aarch64-linux-gnu" => AARCH64_BUILD.to_vec(),
        "riscv64-linux-gnu" => RISCV64_BUILD.to_vec(),
        "powerpc64le-linux-gnu" => PPC64LE_BUILD.to_vec(),
        _ => S390X_BUILD.to_vec(),
    }
}

/// `case "$ARCH"`/`case "$TRIPLE"`・`find|while` 修復ループを
/// 静的解決・型付けした glibc ビルド。
pub fn recipe(ctx: &BuildContext) -> Steps {
    let stage2 = || {
        Arg::Concat(vec![
            Arg::lit("/dep/"),
            Arg::Var("TRIPLE"),
            Arg::lit("-gcc-stage2/usr/lib/cross/"),
            Arg::Var("TRIPLE"),
            Arg::lit("/stage2"),
        ])
    };
    let sysroot = || Arg::Concat(vec![Arg::Var("OUT"), Arg::lit("/usr/"), Arg::Var("TRIPLE")]);
    let sysroot_dir = |sub: &'static str| Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit(sub)]);
    let stage2_gcc = || {
        Arg::Concat(vec![
            Arg::Var("STAGE2"),
            Arg::lit("/bin/"),
            Arg::Var("TRIPLE"),
            Arg::lit("-gcc"),
        ])
    };
    let tool =
        |name: &'static str| Arg::Concat(vec![Arg::Var("TRIPLE"), Arg::lit("-"), Arg::lit(name)]);
    let cmd = |argv: Vec<Arg>| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(argv),
    };
    let env_pair = |name: &'static str, value: Arg| (name, Arg::quoted(value));

    let cet = ctx.triple == "x86_64-linux-gnu";

    let mut configure_args = vec![
        Arg::lit("--prefix=/"),
        Arg::Opt("host", Box::new(Arg::Var("TRIPLE"))),
        Arg::Opt("build", Box::new(Arg::Var("HOST_TRIPLE"))),
        Arg::lit("--libdir=/lib"),
        Arg::Opt("with-headers", Box::new(sysroot_dir("/include"))),
        Arg::lit("--disable-werror"),
        Arg::lit("--disable-nscd"),
        Arg::lit("--disable-profile"),
        Arg::lit("--enable-stack-protector=strong"),
        Arg::lit("--enable-bind-now"),
        Arg::lit("--enable-fortify-source"),
    ];
    if cet {
        configure_args.push(Arg::lit("--enable-cet"));
    }
    configure_args.push(Arg::lit("libc_cv_forced_unwind=yes"));
    configure_args.push(Arg::lit("libc_cv_c_cleanup=yes"));

    let steps = Steps::new(Env::None)
        .sysroot(false)
        .workdir(Workdir::None)
        .step(Step::Export {
            name: "PATH",
            value: Arg::quoted(Arg::Concat(vec![
                Arg::lit("/dep/"),
                Arg::Var("TRIPLE"),
                Arg::lit("-binutils/usr/bin:"),
                Arg::Var("PATH"),
            ])),
        })
        .step(Step::If {
            cond: Cond::DirExists(Arg::lit("/dep/bison/usr/share/bison")),
            then: vec![Step::Export {
                name: "BISON_PKGDATADIR",
                value: Arg::lit("/dep/bison/usr/share/bison"),
            }],
            else_: vec![],
        })
        .step(Step::Set {
            name: "STAGE2",
            value: stage2(),
        })
        .step(Step::Set {
            name: "SYSROOT",
            value: sysroot(),
        })
        .step(cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            sysroot_dir("/include"),
            Arg::Var("BUILD"),
        ]))
        .step(Step::Set {
            name: "HDR",
            value: Arg::Concat(vec![
                Arg::lit("/dep/linux-headers/usr/"),
                Arg::Var("TRIPLE"),
                Arg::lit("/include"),
            ]),
        })
        .step(Step::Or {
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
        })
        .step(cmd(vec![
            Arg::lit("cp"),
            Arg::lit("-a"),
            Arg::Concat(vec![Arg::Var("HDR"), Arg::lit("/.")]),
            sysroot_dir("/include/"),
        ]))
        // stage1/stage2 gcc は /usr/bin/{triple}-{as,ld} を内蔵パスで呼ぶ
        .steps(super::stages::toolchain_links())
        .step(Step::Cd(Arg::Var("BUILD")))
        .step(Step::Unset("CFLAGS"))
        .step(Step::Unset("CXXFLAGS"))
        .step(Step::Unset("LDFLAGS"));

    // ビルド環境向け CFLAGS は HOST 用なので、ターゲット libc には
    // パッケージ Platform の ISA を付ける（case "$ARCH" の静的解決）。
    let cflags = if ctx.target.architecture() == "ppc64le" {
        Arg::quoted(Arg::Concat(vec![
            Arg::lit("-O2 -pipe -mcpu="),
            Arg::Var("VARIANT"),
        ]))
    } else {
        Arg::quoted(Arg::Concat(vec![
            Arg::lit("-O2 -pipe -march="),
            Arg::Var("MARCH"),
        ]))
    };
    let steps = steps
        .step(Step::Export {
            name: "CFLAGS",
            value: cflags,
        })
        .step(Step::Export {
            name: "CXXFLAGS",
            value: Arg::quoted(Arg::lit("${CFLAGS:-}")),
        })
        // ld.so のリンクに mold は使えない（_PROCEDURE_LINKAGE_TABLE_ が
        // 未定義のまま symtab に残り rtld の UND チェックに落ちる）。
        // LDFLAGS は先に Unset 済みなので dep binutils の bfd が使われる。
        .step(Step::Unset("LD"))
        .step(Step::Export {
            name: "CPP",
            value: Arg::quoted(Arg::Concat(vec![stage2_gcc(), Arg::lit(" -E")])),
        })
        .step(Step::ConfigureScript {
            env: vec![
                env_pair(
                    "BUILD_CC",
                    Arg::Concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit("-gcc")]),
                ),
                env_pair("CC", stage2_gcc()),
                env_pair("AR", tool("ar")),
                env_pair("RANLIB", tool("ranlib")),
                env_pair("NM", tool("nm")),
                env_pair("AS", tool("as")),
                env_pair("LD", tool("ld")),
                env_pair("OBJCOPY", tool("objcopy")),
                env_pair("OBJDUMP", tool("objdump")),
                env_pair("READELF", tool("readelf")),
            ],
            program: vec![Arg::Concat(vec![
                Arg::Var("SOURCE"),
                Arg::lit("/configure"),
            ])],
            args: configure_args,
        })
        .step(Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("echo"),
                Arg::quoted(Arg::lit("build-programs=no")),
            ])
            .append_stdout(Arg::lit("configparms")),
        })
        .step(Step::Make {
            jobs: true,
            args: vec![],
        })
        .step(Step::Make {
            jobs: false,
            args: vec!["DESTDIR=\"$SYSROOT\"".to_string(), "install".to_string()],
        })
        .step(cmd(vec![
            Arg::lit("rm"),
            Arg::lit("-rf"),
            Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/share")]),
            Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/etc")]),
            Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/var")]),
            Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/bin")]),
            Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/sbin")]),
        ]))
        // --prefix=/ --libdir=/lib は GROUP ( //lib/libc.so.6 ) を書く。
        // ld は // を sysroot しないので ld script の // を / へ修正する。
        .step(super::stages::ld_script_fixup(Arg::Var("SYSROOT")))
        .step(cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            Arg::lit("lib"),
            sysroot_dir("/lib64"),
        ]))
        // usrmerge: 合成 rootfs では /bin・/sbin は usr への symlink。
        // libc は全実行時閉包に入るため、ここで持たせれば Dockerfile 側の
        // `FROM scratch` + COPY 合成（RUN 不可）でもリンクが揃う。
        .step(cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-s"),
            Arg::lit("usr/bin"),
            Arg::Concat(vec![Arg::Var("OUT"), Arg::lit("/bin")]),
        ]))
        .step(cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-s"),
            Arg::lit("usr/sbin"),
            Arg::Concat(vec![Arg::Var("OUT"), Arg::lit("/sbin")]),
        ]));

    if !cet {
        return steps;
    }

    // x86_64 のみ: 動的リンカと主要 .so を $OUT の lib64 へも揃える。
    let out_lib64 = |sub: &'static str| Arg::Concat(vec![Arg::Var("OUT"), Arg::lit(sub)]);
    steps
        .step(cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            out_lib64("/lib64"),
            out_lib64("/usr/lib64"),
        ]))
        .step(Step::Or {
            lhs: vec![cmd(vec![
                Arg::lit("ln"),
                Arg::lit("-sfn"),
                Arg::lit("/usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2"),
                out_lib64("/lib64/ld-linux-x86-64.so.2"),
            ])],
            rhs: vec![cmd(vec![Arg::lit("true")])],
        })
        .step(Step::If {
            cond: Cond::FileExists(sysroot_dir("/lib/ld-linux-x86-64.so.2")),
            then: vec![cmd(vec![
                Arg::lit("ln"),
                Arg::lit("-sfn"),
                Arg::lit("../usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2"),
                out_lib64("/lib64/ld-linux-x86-64.so.2"),
            ])],
            else_: vec![],
        })
        .step(Step::ForEach {
            var: "so",
            items: [
                "libc.so.6",
                "libm.so.6",
                "libdl.so.2",
                "libpthread.so.0",
                "librt.so.1",
                "libresolv.so.2",
            ]
            .iter()
            .map(|name| Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/lib/"), Arg::lit(name)]))
            .collect(),
            body: vec![Step::If {
                cond: Cond::FileExists(Arg::Var("so")),
                then: vec![cmd(vec![
                    Arg::lit("ln"),
                    Arg::lit("-sfn"),
                    Arg::quoted(Arg::Subst(Box::new(Pipe::cmd(vec![
                        Arg::lit("realpath"),
                        Arg::Concat(vec![Arg::lit("--relative-to="), out_lib64("/usr/lib64")]),
                        Arg::Var("so"),
                    ])))),
                    Arg::Concat(vec![
                        out_lib64("/usr/lib64/"),
                        Arg::Subst(Box::new(Pipe::cmd(vec![
                            Arg::lit("basename"),
                            Arg::Var("so"),
                        ]))),
                    ]),
                ])],
                else_: vec![],
            }],
            stdout: None,
        })
}
