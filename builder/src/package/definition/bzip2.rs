use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir};
use crate::package::{Package, Source};

pub struct Bzip2;

impl Package for Bzip2 {
    fn name(&self) -> &'static str {
        "bzip2"
    }
    fn version(&self) -> &'static str {
        "1.0.8"
    }
    fn license(&self) -> &'static str {
        "bzip2-1.0.6"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz",
            sha3_512: "8192e0e7a92c083354d91e114f4e1a0571d56cc106f58a11c92b4028a1bb1ecad2e00892bb81cc1d2d2dd7371e35beebab2e92e2c83ef9ccb8a2490a97a94f52",
            blake3: "97af3f520629c65fe41292f77e6ca798fe594d7987bfb2aebe7c6fcdc7ab5ed25e9035d582b0a344afcd3ae378c3f3f999d66594bd1dd88cfcabf143cd70b8ef",
            subdir: "bzip2-1.0.8",
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
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .workdir(Workdir::InSource)
                .pgo(crate::package::recipe::PgoTemplate::Make(pgo_workload()))
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("sed"),
                        Arg::lit("-i"),
                        Arg::lit("'s/$(CC) -shared/$(CC) $(LDFLAGS) -shared/'"),
                        Arg::lit("Makefile-libbz2_so"),
                    ]),
                })
                .step(Step::Make {
                    jobs: false,
                    args: crate::package::recipe::args(&[
                        "-f",
                        "Makefile-libbz2_so",
                        "CC=\"$CC\"",
                        "CFLAGS=\"$CFLAGS -fpic -fPIC -Wall -Winline -D_FILE_OFFSET_BITS=64\"",
                        "LDFLAGS=\"$LDFLAGS\"",
                    ]),
                })
                .step(Step::Make {
                    jobs: false,
                    args: crate::package::recipe::args(&[
                        "CC=\"$CC\"",
                        "AR=\"${AR:-${TRIPLE}-ar}\"",
                        "RANLIB=\"${RANLIB:-${TRIPLE}-ranlib}\"",
                        "CFLAGS=\"$CFLAGS -Wall -Winline -D_FILE_OFFSET_BITS=64\"",
                        "LDFLAGS=\"$LDFLAGS\"",
                        "libbz2.a",
                        "bzip2",
                        "bzip2recover",
                    ]),
                })
                .step(Step::MakeInstall {
                    vars: crate::package::recipe::args(&["PREFIX=\"$OUT/usr\""]),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("mkdir"),
                        Arg::lit("-p"),
                        Arg::quoted(Arg::Concat(vec![Arg::Var("OUT"), Arg::lit("/usr/lib")])),
                    ]),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("cp"),
                        Arg::lit("-a"),
                        Arg::lit("libbz2.so*"),
                        Arg::quoted(Arg::Concat(vec![Arg::Var("OUT"), Arg::lit("/usr/lib/")])),
                    ]),
                }),
        )
    }
}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Cond, Pipe, Redirect, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        Step::ForEach {
            var: "_i",
            items: argv(&["1", "2", "3", "4", "5", "6", "7", "8"]),
            body: vec![exec(
                Pipe::cmd(argv(&["cat", "./*.c", "./*.h"])).stderr(Arg::lit("/dev/null")),
            )],
            stdout: Some(Redirect {
                target: Arg::lit("/tmp/pgo-input"),
                append: false,
            }),
        },
        Step::ForEach {
            var: "_opt",
            items: argv(&["-1", "-6", "-9"]),
            body: vec![exec(
                Pipe::new(vec![
                    argv(&["./bzip2", "-c", "\"$_opt\"", "/tmp/pgo-input"]),
                    argv(&["./bzip2", "-d", "-c"]),
                ])
                .stdout(Arg::lit("/dev/null")),
            )],
            stdout: None,
        },
        Step::If {
            cond: Cond::Cmd(
                Pipe::cmd(argv(&["./bzip2", "-c", "-9", "/tmp/pgo-input"]))
                    .stdout(Arg::lit("/tmp/pgo.bz2")),
            ),
            then: vec![exec(
                Pipe::cmd(argv(&["./bzip2recover", "/tmp/pgo.bz2"]))
                    .stdout(Arg::lit("/dev/null"))
                    .stderr(Arg::lit("/dev/null")),
            )],
            else_: vec![],
        },
    ]
}
