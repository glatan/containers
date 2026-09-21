use crate::flags::Flag;
use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

pub struct LinuxHeaders;

impl Package for LinuxHeaders {
    fn name(&self) -> &'static str {
        "linux-headers"
    }

    fn version(&self) -> &'static str {
        "6.18.33"
    }

    fn license(&self) -> &'static str {
        "GPL-2.0-only WITH Linux-syscall-note"
    }

    fn build_only(&self) -> bool {
        true
    }

    /// 全アーキテクチャの UAPI ヘッダを内包する単一成果物のため
    /// BUILD_ENV（ホスト）でのみビルドする。
    fn supports(&self, target: Target) -> bool {
        target == Target::BUILD_ENV
    }

    /// 成果物は常に BUILD_ENV スコープ。
    fn runs_on_host(&self) -> bool {
        true
    }

    /// コンパイルを伴わないため cc ラッパを注入しない。
    fn uses_cc_wrapper(&self) -> bool {
        false
    }

    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }

    fn source(&self) -> Source {
        Source {
            url: "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.18.33.tar.xz",
            sha3_512: "ad87ec123afe83db49e304a4f32d1c529d525f2e55d8c78de208cff0de2b1306d1eaa35d0b49a3738ffa170977e252e1e85fa58654710f11de9f4267947258ca",
            blake3: "00d95e1098670a56ee94558692f1e63e0ac3d2113c9e46568e2da9423a37ad8855a530aba3e6f67bc43beacb5edd7a54834cb0f946112630143b7b7bd930f707",
            subdir: "linux-6.18.33",
        }
    }

    fn dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }

    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        // install_uapi karch "$OUT/usr/{triple}/include" の Rust 側展開。
        const UAPI: &[(&str, &str)] = &[
            ("x86_64", "x86_64-linux-gnu"),
            ("arm64", "aarch64-linux-gnu"),
            ("riscv", "riscv64-linux-gnu"),
            ("powerpc", "powerpc64le-linux-gnu"),
            ("s390", "s390x-linux-gnu"),
        ];
        let mut steps = Steps::new(Env::None).sysroot(false).workdir(Workdir::None);
        for (karch, triple) in UAPI {
            let out_dir = format!("$BUILD/{karch}");
            let dest = format!("$OUT/usr/{triple}/include");
            steps = steps
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("mkdir"),
                        Arg::lit("-p"),
                        Arg::quoted(Arg::owned(out_dir.clone())),
                        Arg::quoted(Arg::owned(dest.clone())),
                    ]),
                })
                .step(Step::Make {
                    jobs: false,
                    args: vec![
                        "-C".to_owned(),
                        "\"$SOURCE\"".to_owned(),
                        format!("O=\"{out_dir}\""),
                        format!("ARCH=\"{karch}\""),
                        "headers".to_owned(),
                    ],
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("cp"),
                        Arg::lit("-a"),
                        Arg::quoted(Arg::owned(format!("{out_dir}/usr/include/."))),
                        Arg::quoted(Arg::owned(format!("{dest}/"))),
                    ]),
                });
        }
        Recipe::Steps(
            steps
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("mkdir"),
                        Arg::lit("-p"),
                        Arg::quoted(Arg::lit("$OUT/usr/include")),
                    ]),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("cp"),
                        Arg::lit("-a"),
                        Arg::quoted(Arg::Concat(vec![
                            Arg::lit("$OUT/usr/"),
                            Arg::Var("HOST_TRIPLE"),
                            Arg::lit("/include/."),
                        ])),
                        Arg::quoted(Arg::lit("$OUT/usr/include/")),
                    ]),
                }),
        )
    }
}

impl Bootstrap for LinuxHeaders {
    /// シード（Arch base-devel）相の中間ステージ。toolchain 成果物が
    /// まだ存在しないため、シードの無印コンパイラでビルドする。
    fn seed(&self) -> bool {
        true
    }
}
