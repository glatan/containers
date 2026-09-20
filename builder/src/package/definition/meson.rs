use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir};
use crate::package::{Package, Source};

pub struct Meson;

impl Package for Meson {
    fn name(&self) -> &'static str {
        "meson"
    }
    fn version(&self) -> &'static str {
        "1.12.0"
    }
    fn license(&self) -> &'static str {
        "Apache-2.0"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/mesonbuild/meson/releases/download/1.12.0/meson-1.12.0.tar.gz",
            sha3_512: "e1832949d31d2e7010a5c3883779dbb0811937a22dac233b4d35dfc6e5e2e7fcbeb0148fc803c795c209a84407269e11df6f123e7f45e397676d1a2100e34e50",
            blake3: "6115839425ffe60c0f036f8f9445e9572a48d207b7c1aa758c624e3f1e66d77ab71d0cff644c4e91c1169c62726baae74625a7832d64e5f0101b6742574e1f8a",
            subdir: "meson-1.12.0",
        }
    }
    fn dependencies(&self, _target: crate::target::Target) -> Vec<crate::package::Dependency> {
        vec![crate::package::Dependency {
            name: "python",
            version: "3.14.7",
        }]
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        vec![]
    }
    fn flags(&self, _target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::Flag::new(vec![], vec![])
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::None)
                .sysroot(false)
                .workdir(Workdir::None)
                // python 成果物は --without-ensurepip で pip を持たないため、
                // mesonbuild を site-packages へ直接配置して launcher を作る。
                .step(Step::Set {
                    name: "_pyver",
                    value: Arg::Subst(Box::new(Pipe::cmd(vec![
                        Arg::lit("python3"),
                        Arg::lit("-c"),
                        Arg::lit("'import sys; print(f\"python{sys.version_info.major}.{sys.version_info.minor}\")'"),
                    ]))),
                })
                .step(Step::Set {
                    name: "_site",
                    value: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("OUT"),
                        Arg::lit("/usr/lib/"),
                        Arg::Var("_pyver"),
                        Arg::lit("/site-packages"),
                    ])),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("mkdir"),
                        Arg::lit("-p"),
                        Arg::quoted(Arg::Var("_site")),
                        Arg::quoted(Arg::Concat(vec![
                            Arg::Var("OUT"),
                            Arg::lit("/usr/bin"),
                        ])),
                    ]),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("cp"),
                        Arg::lit("-a"),
                        Arg::Concat(vec![
                            Arg::quoted(Arg::Var("SOURCE")),
                            Arg::lit("/mesonbuild"),
                        ]),
                        Arg::Concat(vec![Arg::quoted(Arg::Var("_site")), Arg::lit("/")]),
                    ]),
                })
                .step(Step::WriteFile {
                    path: Arg::quoted(Arg::Concat(vec![
                        Arg::Var("OUT"),
                        Arg::lit("/usr/bin/meson"),
                    ])),
                    lines: vec![
                        Arg::lit("#!/bin/sh"),
                        Arg::lit("exec python3 -m mesonbuild.mesonmain \"$@\""),
                    ],
                    append: false,
                    expand: false,
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("chmod"),
                        Arg::lit("+x"),
                        Arg::quoted(Arg::Concat(vec![
                            Arg::Var("OUT"),
                            Arg::lit("/usr/bin/meson"),
                        ])),
                    ]),
                }),
        )
    }
}
