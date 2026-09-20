use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Make;

impl Package for Make {
    fn name(&self) -> &'static str {
        "make"
    }
    fn version(&self) -> &'static str {
        "4.4.1"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/make/make-4.4.1.tar.gz",
            sha3_512: "a5ad233e908b20121b3db5e39dd228e3f62af3b8d0ca5391dff86640ad82a5c08cc58bbc260c786495315ad5299cc160ba6188d5aafc82051453c0779f929ed6",
            blake3: "a7d8aee97b7e9a525ef561afa84eea0d929f246e3aafa420231c0602151cf9eb034f36820f07eb9c0a8afa7ab1f9fd5c41eed9f0db7d0397d7caf503846302ad",
            subdir: "make-4.4.1",
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

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        exec(Pipe::cmd(argv(&["mkdir", "-p", "/tmp/mk-work"]))),
        Step::WriteFile {
            path: Arg::lit("/tmp/mk-work/Makefile"),
            lines: vec![
                Arg::lit("SRCS := $(wildcard *.c)"),
                Arg::lit("OBJS := $(SRCS:.c=.o)"),
                Arg::lit("all: $(OBJS)"),
                Arg::lit("\t@echo done $(words $(OBJS))"),
                Arg::lit("%.o: %.c"),
                Arg::lit("\t@echo cc $<"),
                Arg::lit("clean:"),
                Arg::lit("\t@rm -f *.o"),
            ],
            append: false,
            expand: false,
        },
        Step::ForEach {
            var: "_i",
            items: argv(&["1", "2", "3", "4", "5"]),
            body: vec![exec(
                Pipe::cmd(argv(&["echo", "\"int f$_i(void){return $_i;}\""])).stdout(Arg::Concat(
                    vec![Arg::lit("/tmp/mk-work/f"), Arg::Var("_i"), Arg::lit(".c")],
                )),
            )],
            stdout: None,
        },
        exec(Pipe::cmd(argv(&[
            "./make",
            "-C",
            "/tmp/mk-work",
            "all",
            "clean",
            "all",
        ]))),
        exec(
            Pipe::cmd(argv(&["./make", "-C", "/tmp/mk-work", "-n", "all"]))
                .stdout(Arg::lit("/dev/null")),
        ),
    ]
}

impl Bootstrap for Make {}
