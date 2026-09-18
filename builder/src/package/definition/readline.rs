use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps};
use crate::package::{Package, Source};

pub struct Readline;

impl Package for Readline {
    fn name(&self) -> &'static str {
        "readline"
    }
    fn version(&self) -> &'static str {
        "8.3"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/readline/readline-8.3.tar.gz",
            sha3_512: "58fed6675aabd02dfab76dd82dddcda75361c96f28782a37883487961352885854dc44fd2868871242c7187ec26837d265c73d131948cffc6eff6238f2d46311",
            blake3: "7109f094062bda387a0c16b4875375b96e36437bebbbd8d8f91bb27ba01d687f0e55e7df793b8b1e47dd4d6d4c990d0428a475e748b86defb20881d01f12a6ad",
            subdir: "readline-8.3",
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
        crate::flags::Pgo::Profiled(PGO_WORKLOAD)
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .pgo(crate::package::recipe::PgoTemplate::Autotools(PGO_WORKLOAD))
                .step(Step::configure(&["--prefix=/usr", "--with-curses"]))
                .step(Step::make(&["SHLIB_LIBS=-lncursesw"]))
                .step(Step::make_install(&["SHLIB_LIBS=-lncursesw"])),
        )
    }
}

const PGO_WORKLOAD: &str = r#"make -C examples SHLIB_LIBS=-lncursesw 2>/dev/null
printf 'exit\n' | ./examples/rl 2>/dev/null
printf 'version\n' | ./examples/rlversion 2>/dev/null"#;
