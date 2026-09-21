use crate::package::recipe::{Arg, BuildContext, Env, Recipe, Step, Steps};
use crate::package::{Bootstrap, Package, Source};

pub struct Bash;

impl Package for Bash {
    fn name(&self) -> &'static str {
        "bash"
    }
    fn version(&self) -> &'static str {
        "5.3"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/bash/bash-5.3.tar.gz",
            sha3_512: "8c253baa0accc25a7e83c9d916f0af272b53a368b463afa5035675be7d86e8f63f2e727a1f711615ebed085fe1492fab2ef81490504a7fe9ed09428ffc6fd4c0",
            blake3: "5bf3ac6c18f3dc3fe3e049a1e10ea5ae5c5b39dac2356cd796730f49b273fd0f90dedfa16e5db8348625dbcb4225cb157d6c3bfbc8948e1ae025d3d61b2fed3f",
            subdir: "bash-5.3",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[crate::package::Dependency {
                name: "ncurses",
                version: "6.6",
            }],
        )
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }
    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Profiled
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::Lto)
                .pgo(crate::package::recipe::PgoTemplate::Autotools(
                    pgo_workload(),
                ))
                // クロスでは builtins/mkbuiltins が CC_FOR_BUILD で
                // `CFLAGS_FOR_BUILD`（既定 `-g` のみ）でコンパイルされ、
                // GCC16 の既定 std=gnu23 で `typedef unsigned char bool`
                // が壊れる。GNU make のコマンドライン変数ではなく env に
                // するのは Makefile が @CROSS_COMPILE@ を後置するため。
                .steps(if ctx.cross() {
                    vec![Step::Export {
                        name: "CFLAGS_FOR_BUILD",
                        value: Arg::lit("\"-O2 -std=gnu17\""),
                    }]
                } else {
                    vec![]
                })
                .step(Step::configure(&["--prefix=/usr", "--without-bash-malloc"]))
                .step(Step::make(&[]))
                .step(Step::make_install(&[]))
                .step(Step::Symlink {
                    target: "bash".into(),
                    link: "\"$OUT\"/usr/bin/sh".into(),
                    force: true,
                }),
        )
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
}

impl Bootstrap for Bash {}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::Pipe;

    vec![Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(vec![
            Arg::lit("./bash"),
            Arg::lit("-c"),
            Arg::lit(BASH_C_PROGRAM),
        ]),
    }]
}

/// ワークロードプログラム（シングルクォート込みの字面）。
const BASH_C_PROGRAM: &str = r#"'
x=0; for i in $(seq 1 10000); do x=$((x+i)); done; echo "$x" > /dev/null
s=""; for f in "$SOURCE"/*; do s="$s${f##*/}"; done; echo "${#s}" > /dev/null
for i in $(seq 1 2000); do v="abc$i"; printf "%s" "${v%[0-9]*}${v##*[a-z]}" > /dev/null; done
shopt -s nullglob; for f in "$SOURCE"/*.{c,h} "$SOURCE"/*/*.c; do echo "${#f}" > /dev/null; done
'"#;
