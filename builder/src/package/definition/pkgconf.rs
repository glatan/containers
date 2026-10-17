use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Package, Source};

pub struct Pkgconf;

impl Package for Pkgconf {
    fn name(&self) -> &'static str {
        "pkgconf"
    }
    fn version(&self) -> &'static str {
        "3.0.8"
    }
    fn license(&self) -> &'static str {
        "ISC"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://distfiles.ariadne.space/pkgconf/pkgconf-3.0.8.tar.xz",
            sha3_512: "c4fb2615c0508dfe6b5bc5d49cdb904c25dcc8eb57e874d4829a67b843f3fb5d650b7f2c89dba4c2713dc69bb7414d257137c9e9fb2a292addd321e8a897c2c8",
            blake3: "778da292b878edf09b9d8c30d9491c51b70ca4f1b881a15fdb29b23f482e936b446d9ea6ca2825768b3e7c8f81e5cbe800212d8055d07e9aef7f949ddcad6aab",
            subdir: "pkgconf-3.0.8",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(target, &[])
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

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo(&["--prefix=/usr"], pgo_workload())
    }
}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Pipe, argv};

    vec![Step::ForEach {
        var: "_pc",
        items: vec![Arg::Subst(Box::new(
            Pipe::cmd(argv(&["find", "/dep", "/usr", "-name", "'*.pc'"]))
                .stderr(Arg::lit("/dev/null")),
        ))],
        body: vec![
            Step::Set {
                name: "_dir",
                value: Arg::Subst(Box::new(Pipe::cmd(argv(&["dirname", "\"$_pc\""])))),
            },
            Step::Set {
                name: "_name",
                value: Arg::Subst(Box::new(Pipe::cmd(argv(&["basename", "\"$_pc\"", ".pc"])))),
            },
            Step::Exec {
                env: vec![("PKG_CONFIG_PATH", Arg::Var("_dir"))],
                pipe: Pipe::cmd(argv(&["./pkgconf", "--cflags", "--libs", "\"$_name\""]))
                    .stdout(Arg::lit("/dev/null"))
                    .stderr(Arg::lit("/dev/null")),
            },
            Step::Exec {
                env: vec![("PKG_CONFIG_PATH", Arg::Var("_dir"))],
                pipe: Pipe::cmd(argv(&["./pkgconf", "--modversion", "\"$_name\""]))
                    .stdout(Arg::lit("/dev/null"))
                    .stderr(Arg::lit("/dev/null")),
            },
        ],
        stdout: None,
    }]
}
