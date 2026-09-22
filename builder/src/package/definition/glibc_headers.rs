//! glibc headers の triple 別 build deps とスクリプト。
use super::binutils::VERSION as BINUTILS_VERSION;
use super::stages::dep;
use crate::package::LINUX_HEADERS;
use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Step, Steps, Workdir};
use crate::package::{Dependency, GCC_VERSION};

const X86_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("x86_64-linux-gnu-gcc-stage1", GCC_VERSION),
];

const AARCH64_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("aarch64-linux-gnu-gcc-stage1", GCC_VERSION),
];

const RISCV64_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("riscv64-linux-gnu-gcc-stage1", GCC_VERSION),
];

const PPC64LE_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
    dep("powerpc64le-linux-gnu-gcc-stage1", GCC_VERSION),
];

const S390X_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
    dep("s390x-linux-gnu-gcc-stage1", GCC_VERSION),
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

pub fn recipe(ctx: &BuildContext) -> Steps {
    let stage1 = || {
        Arg::Concat(vec![
            Arg::lit("/dep/"),
            Arg::Var("TRIPLE"),
            Arg::lit("-gcc-stage1/usr/lib/cross/"),
            Arg::Var("TRIPLE"),
            Arg::lit("/stage1"),
        ])
    };
    let sysroot = || Arg::Concat(vec![Arg::Var("OUT"), Arg::lit("/usr/"), Arg::Var("TRIPLE")]);
    let sysroot_dir = |sub: &'static str| Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit(sub)]);
    let stage1_gcc = || {
        Arg::Concat(vec![
            Arg::Var("STAGE1"),
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
        .step(Step::Set {
            name: "STAGE1",
            value: stage1(),
        })
        .step(Step::Set {
            name: "SYSROOT",
            value: sysroot(),
        })
        .step(cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            sysroot_dir("/include"),
            sysroot_dir("/lib"),
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

    // riscv64 のみアーキ別フラグ（case "$TRIPLE" の静的解決）。
    let steps = if ctx.triple == "riscv64-linux-gnu" {
        steps.step(Step::Export {
            name: "CFLAGS",
            value: Arg::quoted(Arg::lit("-O2 -pipe -march=rv64gc -mabi=lp64d")),
        })
    } else {
        steps
    };

    steps
        .step(Step::Export {
            name: "CPP",
            value: Arg::quoted(Arg::Concat(vec![stage1_gcc(), Arg::lit(" -E")])),
        })
        .step(Step::ConfigureScript {
            env: vec![
                env_pair(
                    "BUILD_CC",
                    Arg::Concat(vec![Arg::Var("HOST_TRIPLE"), Arg::lit("-gcc")]),
                ),
                env_pair("CC", stage1_gcc()),
                env_pair("AR", tool("ar")),
                env_pair("RANLIB", tool("ranlib")),
                env_pair("NM", tool("nm")),
                env_pair("OBJCOPY", tool("objcopy")),
                env_pair("OBJDUMP", tool("objdump")),
                env_pair("READELF", tool("readelf")),
            ],
            program: vec![Arg::Concat(vec![
                Arg::Var("SOURCE"),
                Arg::lit("/configure"),
            ])],
            args: vec![
                Arg::lit("--prefix=/"),
                Arg::Opt("host", Box::new(Arg::Var("TRIPLE"))),
                Arg::Opt("build", Box::new(Arg::Var("HOST_TRIPLE"))),
                Arg::Opt("with-headers", Box::new(sysroot_dir("/include"))),
                Arg::lit("--disable-werror"),
                Arg::lit("libc_cv_forced_unwind=yes"),
                Arg::lit("libc_cv_c_cleanup=yes"),
            ],
        })
        .step(Step::Make {
            jobs: false,
            args: vec!["csu/subdir_lib".to_string()],
        })
        .step(Step::Make {
            jobs: false,
            args: vec![
                "install-bootstrap-headers=yes".to_string(),
                "install-headers".to_string(),
                "DESTDIR=\"$SYSROOT\"".to_string(),
            ],
        })
        .step(cmd(vec![
            Arg::lit("install"),
            Arg::lit("-m644"),
            Arg::lit("csu/crt1.o"),
            Arg::lit("csu/crti.o"),
            Arg::lit("csu/crtn.o"),
            sysroot_dir("/lib/"),
        ]))
        .step(cmd(vec![
            stage1_gcc(),
            Arg::lit("-nostdlib"),
            Arg::lit("-nostartfiles"),
            Arg::lit("-shared"),
            Arg::lit("-x"),
            Arg::lit("c"),
            Arg::lit("/dev/null"),
            Arg::lit("-o"),
            sysroot_dir("/lib/libc.so"),
        ]))
        .step(cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            sysroot_dir("/include/gnu"),
        ]))
        .step(cmd(vec![
            Arg::lit("touch"),
            sysroot_dir("/include/gnu/stubs.h"),
        ]))
        .step(Step::Symlink {
            target: "lib".into(),
            link: "$SYSROOT/lib64".into(),
            force: true,
        })
}
