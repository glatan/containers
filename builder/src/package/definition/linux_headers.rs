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
        "6.18.56"
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
            url: "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.18.56.tar.xz",
            sha3_512: "1cd94e02690a2e548c49d1f9f10d874b174f8e9f6ad69dab98dea41f58133b2867775dbf1dea44761abb3a673065b27ff8a8fdd5aaf0d695fc30ded5b3cc8a30",
            blake3: "e7d2895ae0c1a3151f5a8795643113c736f94996fe86956e5174246b9f018a8f2e82256f215a5c851bb364720235b32fe85e7c1bccd3837d1f346825477841cd",
            subdir: "linux-6.18.56",
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
                        // kbuild は HOSTCC を `=` で代入するため env でなく
                        // make 引数として渡す。seed 相は無印 gcc、env 相は
                        // prefix 付き自前 gcc を指す `$CC` をそのまま使う。
                        "HOSTCC=\"$CC\"".to_owned(),
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
