use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Package, Source};

pub struct Expat;

impl Package for Expat {
    fn name(&self) -> &'static str {
        "expat"
    }
    fn version(&self) -> &'static str {
        "2.9.0"
    }
    fn license(&self) -> &'static str {
        "MIT"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/libexpat/libexpat/releases/download/R_2_9_0/expat-2.9.0.tar.xz",
            sha3_512: "93d3c389be1941a7a0fc6395c2e88d3d4a41a304291e2899c07826f02f28a2ace121c9ac8ea067c5d8d7eaa51b2bc03feec887a5ab504d8db5f782844d26845d",
            blake3: "2b7fee29ba5d7168dbe56fb0a12359cac3729aa2ad0b56ea97f93a45894e868bb05cafc01ec8ec4c269b1aad538ebc8b9d50ec06e9ee9187993888bc5fa2825a",
            subdir: "expat-2.9.0",
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
