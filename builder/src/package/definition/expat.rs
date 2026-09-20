use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Package, Source};

pub struct Expat;

impl Package for Expat {
    fn name(&self) -> &'static str {
        "expat"
    }
    fn version(&self) -> &'static str {
        "2.8.4"
    }
    fn license(&self) -> &'static str {
        "MIT"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/libexpat/libexpat/releases/download/R_2_8_4/expat-2.8.4.tar.xz",
            sha3_512: "056d6b25263d95d7375e2fc14f22f161722558c7e695ca3fccc73eb2842bf921e2791fcc1104fd52b0e057dd3be00795af698af5c70a9d8a257b5d6123ac75cc",
            blake3: "1311a1adc8da8ffd10ca65db96be3abe9e7e9d83561d899bd0dbdab17ab599dbddde8e8e3a1a6b9a3223c184a3bf1e71d767750418b304be8b22e8e209348f54",
            subdir: "expat-2.8.4",
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
    use crate::package::recipe::{Arg, Cond, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        Step::If {
            cond: Cond::FileExists(Arg::lit("\"$SOURCE/doc/xmlwf.xml\"")),
            then: vec![exec(
                Pipe::cmd(argv(&["./xmlwf/xmlwf", "\"$SOURCE/doc/xmlwf.xml\""]))
                    .stdout(Arg::lit("/dev/null")),
            )],
            else_: vec![],
        },
        Step::ForEach {
            var: "_x",
            items: vec![Arg::Subst(Box::new(Pipe::new(vec![
                argv(&["find", "\"$SOURCE\"", "-name", "'*.xml'"]),
                argv(&["head", "-20"]),
            ])))],
            body: vec![exec(
                Pipe::cmd(argv(&["./xmlwf/xmlwf", "\"$_x\""]))
                    .stdout(Arg::lit("/dev/null"))
                    .stderr(Arg::lit("/dev/null")),
            )],
            stdout: None,
        },
    ]
}
