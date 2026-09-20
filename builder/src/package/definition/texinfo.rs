use crate::package::recipe::{Arg, BuildContext, Cond, Env, Pipe, Recipe, Step, Steps};
use crate::package::{Package, Source};
use crate::target::Target;

pub struct Texinfo;

impl Package for Texinfo {
    fn name(&self) -> &'static str {
        "texinfo"
    }
    fn version(&self) -> &'static str {
        "7.3"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        vec!["perl"]
    }

    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/texinfo/texinfo-7.3.tar.xz",
            sha3_512: "04243a6f04eec6f208fecea57c16fc4a9f5fd0487c670370295d18edc6d266bbff3fa60d8a547687d8c17de19218c28434ac5359a3d29dde9720f53d86f33938",
            blake3: "01eae8c848df2fbd6903aeba43fef7ac45a019b4c72d4cd853f936940408102561499b58ecfc36753ea84e31dbbc79fc8c43d950380dc33f3bf8402c7d56aa49",
            subdir: "texinfo-7.3",
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
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let mut args = vec!["--prefix=/usr".to_owned()];
        if ctx.cross() {
            args.push("--disable-perl-xs".into());
        }
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .pgo(crate::package::recipe::PgoTemplate::Autotools(
                    pgo_workload(),
                ))
                .step(Step::Configure(args))
                .step(Step::If {
                    cond: Cond::Not(Box::new(Cond::FileExists(Arg::lit("tools/Makefile")))),
                    then: vec![
                        Step::Exec {
                            env: vec![],
                            pipe: Pipe::cmd(vec![
                                Arg::lit("mkdir"),
                                Arg::lit("-p"),
                                Arg::lit("tools"),
                            ]),
                        },
                        Step::WriteFile {
                            path: Arg::lit("tools/Makefile"),
                            lines: vec![
                                Arg::lit("all:"),
                                Arg::lit("install:"),
                                Arg::lit("check:"),
                                Arg::lit(".PHONY: all install check"),
                            ],
                            append: false,
                            expand: false,
                        },
                    ],
                    else_: vec![],
                })
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Profiled
    }
}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Cond, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        Step::Set {
            name: "_texi2any",
            value: Arg::lit("\"$BUILD/tp/texi2any\""),
        },
        // texi2any は configure が生成するラッパー。out-of-tree で
        // 見つからない場合に備えてビルドツリーから解決する。
        Step::If {
            cond: Cond::Not(Box::new(Cond::Cmd(Pipe::cmd(argv(&[
                "test",
                "-x",
                "\"$_texi2any\"",
            ]))))),
            then: vec![Step::Set {
                name: "_texi2any",
                value: Arg::Subst(Box::new(Pipe::cmd(argv(&[
                    "find",
                    "\"$BUILD\"",
                    "-name",
                    "texi2any",
                    "-type",
                    "f",
                    "-print",
                    "-quit",
                ])))),
            }],
            else_: vec![],
        },
        Step::ForEach {
            var: "_texi",
            items: argv(&["\"$SOURCE\"/doc/*.texi"]),
            body: vec![Step::If {
                cond: Cond::FileExists(Arg::lit("\"$_texi\"")),
                then: vec![exec(
                    Pipe::cmd(argv(&["\"$_texi2any\"", "--plaintext", "\"$_texi\""]))
                        .stdout(Arg::lit("/dev/null"))
                        .stderr(Arg::lit("/dev/null")),
                )],
                else_: vec![],
            }],
            stdout: None,
        },
        Step::ForEach {
            var: "_t",
            items: vec![Arg::Subst(Box::new(Pipe::new(vec![
                argv(&["find", "\"$SOURCE\"", "-name", "'*.texi'"]),
                argv(&["head", "-5"]),
            ])))],
            body: vec![exec(
                Pipe::cmd(argv(&[
                    "\"$_texi2any\"",
                    "--html",
                    "\"$_t\"",
                    "-o",
                    "/tmp/texi-out",
                ]))
                .stderr(Arg::lit("/dev/null")),
            )],
            stdout: None,
        },
    ]
}
