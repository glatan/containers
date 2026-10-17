use crate::flags::Flag;
use crate::package::recipe::{
    Arg, BuildContext, Cond, Env, Pipe, Recipe, Step, Steps, Workdir, args, argv,
};
use crate::package::{Dependency, Package, Source, dep};
use crate::target::Target;

pub struct Openssl;

impl Package for Openssl {
    fn name(&self) -> &'static str {
        "openssl"
    }

    fn version(&self) -> &'static str {
        "4.0.3"
    }

    fn license(&self) -> &'static str {
        "Apache-2.0"
    }

    fn catalog(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/openssl/openssl/releases/download/openssl-4.0.3/openssl-4.0.3.tar.gz",
            sha3_512: "11583faf5718638acdca493ca593d382302fab032db7ec1c6adedb07d073394e891ddf236591070f3fb5d48fe7fad149ee990d87397c4c3a6f95748db80d20cc",
            blake3: "92a6d5e2e74a989c1405de8a7770c7fb469ddcf9c9ed74b768d51c274af6487b5816f7dd05f15714baf55b17bb948175891ecfc24da8117fb5527462795abe2b",
            subdir: "openssl-4.0.3",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    /// Configure スクリプトが perl を実行するため必須の HOST 依存。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        deps.push(dep("perl", "5.44.0"));
        deps
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        // case "$ARCH" の TGT 解決を Rust 側へ移す。
        let tgt = match ctx.target.architecture() {
            "amd64" => "linux-x86_64",
            "arm64" => "linux-aarch64",
            "riscv64" => "linux64-riscv64",
            "ppc64le" => "linux-ppc64le",
            "s390x" => "linux64-s390x",
            arch => panic!("未対応の architecture です: {arch}"),
        };
        // sysroot:true デフォルト（検出・CFLAGS 付与は wrap_script が注入）。
        // openssl の Configure は $CC を直接使うため CC 側にも反映する。
        let mut steps = Steps::new(Env::None)
            .workdir(Workdir::Build)
            .step(Step::If {
                cond: Cond::EnvSet("SYSROOT"),
                then: vec![Step::Export {
                    name: "CC",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("CC"),
                        Arg::lit(" --sysroot="),
                        Arg::Var("SYSROOT"),
                    ])),
                }],
                else_: vec![],
            })
            .step(Step::ConfigureScript {
                env: vec![],
                program: vec![Arg::quoted(Arg::Concat(vec![
                    Arg::Var("SOURCE"),
                    Arg::lit("/Configure"),
                ]))],
                args: vec![
                    Arg::lit(tgt),
                    Arg::lit("--prefix=/usr"),
                    Arg::lit("--openssldir=/etc/ssl"),
                    Arg::lit("--libdir=lib64"),
                ],
            });
        if ctx.cross() {
            steps = steps.step(Step::make(&[]));
        } else {
            // ネイティブのみ PGO。OPENSSL_MODULES が無いと EVP 系
            // アルゴリズムが実行できずプロファイルが空になる。
            steps = steps
                .step(Step::Export {
                    name: "PGODIR",
                    value: Arg::lit("/tmp/pgo"),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("rm"),
                        Arg::lit("-rf"),
                        Arg::quoted(Arg::Var("PGODIR")),
                    ]),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("mkdir"),
                        Arg::lit("-p"),
                        Arg::quoted(Arg::Var("PGODIR")),
                    ]),
                })
                .step(Step::Make {
                    jobs: true,
                    args: args(&[
                        "CFLAGS=\"$CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic\"",
                        "LDFLAGS=\"$LDFLAGS -fprofile-generate\"",
                    ]),
                })
                .step(Step::Export {
                    name: "LD_LIBRARY_PATH",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("PWD"),
                        Arg::lit("${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"),
                    ])),
                })
                .step(Step::Export {
                    name: "OPENSSL_MODULES",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("PWD"),
                        Arg::lit("/providers"),
                    ])),
                })
                .step(Step::Export {
                    name: "OPENSSL_CONF",
                    value: Arg::lit("/dev/null"),
                })
                .step(Step::Or {
                    lhs: [
                        "./apps/openssl speed -seconds 1 rsa2048 ecdsap256 ed25519",
                        "./apps/openssl speed -seconds 1 -evp aes-128-gcm -evp aes-256-gcm -evp chacha20-poly1305 -evp sha256 -evp sha512",
                        "./apps/openssl speed -seconds 1 aes-128-cbc aes-256-cbc sha256 sha512",
                    ]
                    .iter()
                    .map(|line| Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(argv(
                            &line.split(' ').collect::<Vec<_>>(),
                        )),
                    })
                    .collect(),
                    rhs: vec![Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(argv(&["true"])),
                    }],
                })
                .step(Step::Make {
                    jobs: false,
                    args: args(&["clean"]),
                })
                .step(Step::Make {
                    jobs: true,
                    args: args(&[
                        "CFLAGS=\"$CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch\"",
                        "LDFLAGS=\"$LDFLAGS -fprofile-use\"",
                    ]),
                });
        }
        steps = steps.step(Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![
                Arg::lit("make"),
                Arg::lit("install_sw"),
                Arg::Concat(vec![Arg::lit("DESTDIR="), Arg::quoted(Arg::Var("OUT"))]),
            ]),
        });
        Recipe::Steps(steps)
    }
}
