use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir};
use crate::package::{Dependency, Package, Source};
use crate::target::Target;

pub struct Meson;

impl Package for Meson {
    fn name(&self) -> &'static str {
        "meson"
    }
    fn version(&self) -> &'static str {
        "1.12.1"
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
            url: "https://github.com/mesonbuild/meson/releases/download/1.12.1/meson-1.12.1.tar.gz",
            sha3_512: "82f6c97fdbf6877c326cb1c82cc9b3f1c3e9d0f2de16a11e9cc4e4c72d47bce89ca53c4d13802f579f47b50359a2b3171762e8c2d7561ce11789086baf2d9221",
            blake3: "f6d4541d0a5129992e93ce9e55a5ec7608781381451e84e9a4c2afd6400f13e077412def1775d18c6ba3b84a9fbe28edaef8d3b6621df794c7a3edbd339564d9",
            subdir: "meson-1.12.1",
        }
    }
    /// 成果物ランチャー（`python3 -m mesonbuild.mesonmain`）が実行時に
    /// python を要求するため、消費側イメージ向けの実行時依存として宣言する。
    fn dependencies(&self, _target: crate::target::Target) -> Vec<crate::package::Dependency> {
        vec![crate::package::dep("python", "3.15.0")]
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        vec![]
    }
    /// meson 本体は python スクリプトであり、レシピ内の `python3 -c`
    /// （バージョン検出）と成果物の `python3` ランチャー双方が HOST 側で
    /// python を実行する。dependencies()（dep:true マウント）では
    /// LD_LIBRARY_PATH に /dep/python/usr/lib が載らず libpython を
    /// 解決できないため、ビルド時は HOST 依存として宣言する
    /// （実行時依存は dependencies() に残す）。
    fn host_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![crate::package::dep("python", "3.15.0")]
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
