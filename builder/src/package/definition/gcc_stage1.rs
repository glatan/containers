//! gcc stage1 の triple 別 build deps とスクリプト。
use super::binutils::VERSION as BINUTILS_VERSION;
use super::stages::dep;
use crate::package::Dependency;
use crate::package::LINUX_HEADERS;
use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Step, Steps, Workdir};

const X86_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
];

const AARCH64_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
];

const RISCV64_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
];

const PPC64LE_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
];

const S390X_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
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
    // case "$TRIPLE" の静的解決（stage2 と共有）。
    let arch_flags = super::gcc::stage_arch_flags(&ctx.triple);
    let sysroot = || Arg::Concat(vec![Arg::lit("/usr/"), Arg::Var("TRIPLE")]);
    let dep_binutils = |sub: &'static str| {
        Arg::Concat(vec![
            Arg::lit("/dep/"),
            Arg::Var("TRIPLE"),
            Arg::lit("-binutils/usr/bin/"),
            Arg::Var("TRIPLE"),
            Arg::lit(sub),
        ])
    };
    let usr_bin = |sub: &'static str| {
        Arg::Concat(vec![
            Arg::lit("/usr/bin/"),
            Arg::Var("TRIPLE"),
            Arg::lit(sub),
        ])
    };
    let cmd = |argv: Vec<Arg>| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(argv),
    };

    let mut args = vec![
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
        Arg::Opt(
            "prefix",
            Box::new(Arg::Concat(vec![
                Arg::lit("/usr/lib/cross/"),
                Arg::Var("TRIPLE"),
                Arg::lit("/stage1"),
            ])),
        ),
        Arg::Opt("with-sysroot", Box::new(sysroot())),
        Arg::Opt("with-build-sysroot", Box::new(sysroot())),
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
    args.extend(arch_flags.iter().map(|f| Arg::lit(f)));
    args.extend(
        [
            "--without-headers",
            "--with-newlib",
            "--disable-shared",
            "--disable-threads",
            "--disable-werror",
            "--disable-libatomic",
            "--disable-libsanitizer",
            "--disable-libssp",
            "--disable-libquadmath",
            "--disable-libgomp",
            "--disable-libstdcxx",
            "--disable-libvtv",
            "--disable-bootstrap",
            "--disable-multilib",
            "--enable-languages=c",
            "--enable-lto",
            "--enable-plugin",
            "--enable-default-pie",
            "--enable-default-ssp",
            "--enable-checking=release",
        ]
        .iter()
        .map(|f| Arg::lit(f)),
    );

    let mut steps = Steps::new(Env::None)
        .sysroot(false)
        .workdir(Workdir::None)
        // GCC の TARGET_EXPORTS は CC/CXX/AR 等を target 向けに上書きするが
        // CPP は含まれない。プリリュードの CPP=gcc -E が target libgcc の
        // configure に漏れてホストヘッダを誤検出するため落とす
        //（target 側は CC -E＝xgcc -E が使われる）。
        .step(Step::Unset("CPP"))
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
            name: "PREFIX",
            value: Arg::Concat(vec![
                Arg::lit("/usr/lib/cross/"),
                Arg::Var("TRIPLE"),
                Arg::lit("/stage1"),
            ]),
        })
        .step(Step::Set {
            name: "SYSROOT",
            value: sysroot(),
        })
        .step(cmd(vec![
            Arg::lit("mkdir"),
            Arg::lit("-p"),
            Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/include")]),
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
            Arg::Concat(vec![Arg::Var("SYSROOT"), Arg::lit("/include/")]),
        ]))
        .step(cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            dep_binutils("-as"),
            usr_bin("-as"),
        ]))
        .step(cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            dep_binutils("-ld"),
            usr_bin("-ld"),
        ]));
    // gmp/mpfr/mpc/isl は cc1 がリンクするビルド機側ライブラリ、
    // zlib/zstd は --with-system-zlib/lto が要求する。合成 env の
    // gcc は /usr/include を検索しないため dep マウント経由で供給する。
    steps.steps.extend(super::gcc::lib_flag_steps());
    steps
        .step(Step::Cd(Arg::Var("BUILD")))
        .step(Step::ConfigureScript {
            env: vec![],
            program: vec![Arg::Concat(vec![
                Arg::Var("SOURCE"),
                Arg::lit("/configure"),
            ])],
            args,
        })
        .step(Step::Make {
            jobs: true,
            args: vec!["all-gcc".to_string(), "all-target-libgcc".to_string()],
        })
        .step(Step::Make {
            jobs: false,
            args: vec![
                "DESTDIR=\"$OUT\"".to_string(),
                "install-gcc".to_string(),
                "install-target-libgcc".to_string(),
            ],
        })
}
