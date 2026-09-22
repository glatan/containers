use crate::flags::Flag;
use crate::package::recipe::{
    Arg, BuildContext, Cond, Env, Pipe, Recipe, Step, Steps, Workdir, argv,
};
use crate::package::{Dependency, Package, Source, dep};
use crate::target::Target;

pub struct Qemu;

impl Package for Qemu {
    fn name(&self) -> &'static str {
        "qemu"
    }
    fn version(&self) -> &'static str {
        "11.1.1"
    }
    fn license(&self) -> &'static str {
        "GPL-2.0-only"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://download.qemu.org/qemu-11.1.1.tar.xz",
            sha3_512: "8b53993040fe8ce2e6c69e2d616a952fcd3da00ddcd114b64dea2d5b1cc0a4f6df03bc899ba12437b6166559be8f2379b97a737608659f6b2c6ab56605bd20a2",
            blake3: "9cda9512c997d0ec176add30ff4f4531d4882681eb12e98ba5c7d11cb4cf610ca079689e5def5ae36c1c1a173dee7eb001ce8fffeba77f418048e4b4773dff67",
            subdir: "qemu-11.1.1",
        }
    }
    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(
            target,
            &[
                dep("glib", "2.90.0"),
                dep("zlib", "1.3.2"),
                dep("zstd", "1.5.7"),
            ],
        )
    }
    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![
            crate::package::LINUX_HEADERS,
            dep("pcre2", "10.48"),
            dep("libffi", "3.8.0"),
        ]
    }
    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }
    fn supports(&self, target: Target) -> bool {
        target == Target::BUILD_ENV
    }
    /// configure は meson を内部起動し、python3 / pkg-config /
    /// ninja を要求する。イメージには無いため HOST 成果物を使う。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        deps.extend([
            dep("python", "3.14.7"),
            dep("meson", "1.12.0"),
            dep("ninja", "1.13.2"),
            dep("pkgconf", "2.5.1"),
        ]);
        deps
    }
    /// meson の sizeof 検査はリンクもする。--sysroot は HOST glibc に
    /// /usr/include が無く sizeof(void*)=-1 になり、64-bit linux-user の
    /// TCG が全部落ちるため CFLAGS 系から strip する。
    /// （sysroot(false) で wrap_script 側の --sysroot 注入も抑制）
    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        // printf '%s' "${VAR:-}" | sed -E 's/--sysroot=[^ ]+//g' を $( ) に展開し
        // " -flto=auto" を連結してダブルクォートで囲んだ export。
        let strip_export = |var: &'static str| Step::Export {
            name: var,
            value: Arg::quoted(Arg::Concat(vec![
                Arg::Subst(Box::new(Pipe::new(vec![
                    vec![
                        Arg::lit("printf"),
                        Arg::lit("'%s'"),
                        Arg::quoted(Arg::owned(format!("${{{var}:-}}"))),
                    ],
                    argv(&["sed", "-E", "'s/--sysroot=[^ ]+//g'"]),
                ]))),
                Arg::lit(" -flto=auto"),
            ])),
        };
        Recipe::Steps(
            Steps::new(Env::None)
                .sysroot(false)
                // pythondeps.toml を sed するためソースを /build へコピーする
                //（/src は read-only マウント）。
                .workdir(Workdir::InSource)
                .step(strip_export("CFLAGS"))
                .step(strip_export("CXXFLAGS"))
                .step(strip_export("LDFLAGS"))
                .step(Step::If {
                    cond: Cond::DirExists(Arg::lit("/dep/${TRIPLE}-glibc")),
                    then: vec![
                        Step::Set {
                            name: "_glibc",
                            value: Arg::lit("/dep/${TRIPLE}-glibc"),
                        },
                        Step::Export {
                            name: "LDFLAGS",
                            value: Arg::quoted(Arg::Concat(vec![
                                Arg::Var("LDFLAGS"),
                                Arg::lit(" -B"),
                                Arg::Var("_glibc"),
                                Arg::lit("/usr/lib64 -L"),
                                Arg::Var("_glibc"),
                                Arg::lit("/usr/lib64 -L"),
                                Arg::Var("_glibc"),
                                Arg::lit("/lib64"),
                            ])),
                        },
                    ],
                    else_: vec![],
                })
                .step(Step::If {
                    cond: Cond::DirExists(Arg::lit("/dep/linux-headers/usr/include")),
                    then: ["CFLAGS", "CXXFLAGS"]
                        .iter()
                        .map(|name| Step::Export {
                            name,
                            value: Arg::quoted(Arg::Concat(vec![
                                Arg::Var(name),
                                Arg::lit(" -idirafter /dep/linux-headers/usr/include"),
                            ])),
                        })
                        .collect(),
                    else_: vec![],
                })
                .step(Step::Set {
                    name: "PKG_CONFIG_LIBDIR",
                    value: Arg::lit(""),
                })
                .step(Step::ForEach {
                    var: "_dep",
                    items: argv(&["glib", "pcre2", "zlib", "libffi"]),
                    body: vec![Step::ForEach {
                        var: "_pc",
                        items: vec![
                            Arg::Concat(vec![
                                Arg::lit("/dep/\""),
                                Arg::Var("_dep"),
                                Arg::lit("\"/usr/lib/pkgconfig"),
                            ]),
                            Arg::Concat(vec![
                                Arg::lit("/dep/\""),
                                Arg::Var("_dep"),
                                Arg::lit("\"/usr/lib64/pkgconfig"),
                            ]),
                        ],
                        body: vec![Step::If {
                            cond: Cond::DirExists(Arg::quoted(Arg::Var("_pc"))),
                            then: vec![Step::Set {
                                name: "PKG_CONFIG_LIBDIR",
                                value: Arg::quoted(Arg::Concat(vec![
                                    Arg::lit("${PKG_CONFIG_LIBDIR:+$PKG_CONFIG_LIBDIR:}"),
                                    Arg::Var("_pc"),
                                ])),
                            }],
                            else_: vec![],
                        }],
                        stdout: None,
                    }],
                    stdout: None,
                })
                .step(Step::If {
                    cond: Cond::EnvSet("PKG_CONFIG_LIBDIR"),
                    then: vec![
                        Step::ExportBare("PKG_CONFIG_LIBDIR"),
                        Step::Export {
                            name: "PKG_CONFIG_PATH",
                            value: Arg::lit(""),
                        },
                    ],
                    else_: vec![],
                })
                .step(Step::If {
                    cond: Cond::DirExists(Arg::lit("/dep/glib")),
                    then: vec![Step::Export {
                        name: "PKG_CONFIG_SYSROOT_DIR",
                        value: Arg::lit("/dep/glib"),
                    }],
                    else_: vec![],
                })
                // tooling グループの path install（qemu）と setuptools /
                // wheel / pip の解決は PyPI 到達が必要で network=none では
                // 失敗する。qemu.qmp の bundled wheel のみ残して configure
                // 時の mkvenv を完走させる。
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(&[
                        "sed",
                        "-i",
                        "'/^\\[tooling\\]/,/^\\[/ { /^\"qemu\" =/d; /^\"setuptools\"/d; /^\"wheel\"/d; /^\"pip\"/d }'",
                        "pythondeps.toml",
                    ])),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(&[
                        "./configure",
                        "--prefix=/usr",
                        "--target-list=x86_64-linux-user,aarch64-linux-user,riscv64-linux-user,ppc64le-linux-user,s390x-linux-user",
                        "--enable-tcg",
                        "--disable-system",
                        "--disable-tools",
                        "--disable-docs",
                        "--disable-guest-agent",
                        "--disable-slirp",
                        "--disable-vnc",
                        "--disable-sdl",
                        "--disable-gtk",
                        "--disable-curses",
                        "--disable-brlapi",
                        "--disable-spice",
                        "--disable-libnfs",
                        "--disable-libusb",
                        "--disable-vde",
                        "--disable-nettle",
                        "--disable-gnutls",
                        "--disable-gcrypt",
                        "--disable-auth-pam",
                    ])),
                })
                .step(Step::make(&[]))
                .step(Step::make_install(&[]))
                .step(Step::If {
                    cond: Cond::Cmd(Pipe::cmd(argv(&["command", "-v", "qemu-s390x"]))),
                    then: vec![Step::Or {
                        lhs: vec![Step::Exec {
                            env: vec![],
                            pipe: Pipe::new(vec![
                                argv(&["qemu-s390x", "-cpu", "help"]),
                                argv(&["grep", "-E", "'z900|z990|z9EC'"]),
                            ]),
                        }],
                        rhs: vec![Step::Exec {
                            env: vec![],
                            pipe: Pipe::cmd(argv(&["true"])),
                        }],
                    }],
                    else_: vec![],
                }),
        )
    }
}
