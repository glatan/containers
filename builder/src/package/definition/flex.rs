use crate::package::recipe::{
    Arg, BuildContext, Cond, Env, Pipe, Recipe, Std, Step, Steps, Workdir,
};
use crate::package::{Package, Source};

pub struct Flex;

impl Package for Flex {
    fn name(&self) -> &'static str {
        "flex"
    }
    fn version(&self) -> &'static str {
        "2.6.4"
    }
    fn license(&self) -> &'static str {
        "BSD-2-Clause"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/westes/flex/files/981163/flex-2.6.4.tar.gz",
            sha3_512: "eac223beda4c1629dccfd29666eab9f510e05f5b606587496364ae59622c3bab8104a9ac52f4bb06098a628a4d4fba0c59f5a9d851d65cbc94ed0a660475e162",
            blake3: "d78b714ac38bd9de7f9b44a078efed82e96ed43e7cf9cd33944a7f379a2d09a4d024e6d3a3accb71ae72f36b4f028328f551c6271a231cdd3c307b542cdca143",
            subdir: "flex-2.6.4",
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

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let mut steps = Steps::new(Env::Autotools)
                // gnulib の lib/malloc.c は K&R 宣言のため gnu17 で壊れる
                .std(Std::Gnu11)
                .workdir(Workdir::InSource)
                // config.h fixup（configure〜make 間ステップ）は両フェーズで replay される
                .pgo(crate::package::recipe::PgoTemplate::Autotools(pgo_workload()))
                .step(Step::If {
                    cond: Cond::FileExists(Arg::lit("lib/malloc.c")),
                    then: vec![Step::WriteFile {
                        path: Arg::lit("lib/malloc.c"),
                        lines: vec![
                            Arg::lit("#include <stdlib.h>"),
                            Arg::lit("void *rpl_malloc(size_t n) { if (n == 0) n = 1; return malloc(n); }"),
                        ],
                        append: false,
                        expand: false,
                    }],
                    else_: vec![],
                })
                .step(Step::If {
                    cond: Cond::FileExists(Arg::lit("lib/realloc.c")),
                    then: vec![Step::WriteFile {
                        path: Arg::lit("lib/realloc.c"),
                        lines: vec![
                            Arg::lit("#include <stdlib.h>"),
                            Arg::lit("void *rpl_realloc(void *p, size_t n) { if (n == 0) n = 1; return realloc(p, n); }"),
                        ],
                        append: false,
                        expand: false,
                    }],
                    else_: vec![],
                })
                .step(Step::configure(&["--prefix=/usr"]))
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("find"),
                        Arg::lit("."),
                        Arg::lit("-name"),
                        Arg::lit("config.h"),
                        Arg::lit("-exec"),
                        Arg::lit("sed"),
                        Arg::lit("-i"),
                        Arg::lit("'/rpl_malloc/d;/rpl_realloc/d'"),
                        Arg::lit("{}"),
                        Arg::lit("+"),
                    ]),
                });
        if ctx.cross() {
            // make コマンドラインの `CFLAGS=…` は Makefile の
            // target-specific `CFLAGS=$(CFLAGS_FOR_BUILD)` も上書きするため
            // stage1flex（BUILD 側ツール）がターゲットフラグで壊れる。
            // configure が焼いた Makefile 側の値を使うプレーンな make にする。
            steps = steps
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![Arg::lit("make"), Arg::nproc_jobs("-j")]),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("make"),
                        Arg::lit("install"),
                        Arg::Concat(vec![Arg::lit("DESTDIR="), Arg::quoted(Arg::Var("OUT"))]),
                    ]),
                });
        } else {
            steps = steps.step(Step::make(&[])).step(Step::make_install(&[]));
        }
        Recipe::Steps(steps)
    }
}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![Step::ForEach {
        var: "_l",
        items: vec![Arg::Subst(Box::new(Pipe::cmd(argv(&[
            "find",
            "\"$SOURCE\"",
            "-name",
            "'*.l'",
        ]))))],
        body: vec![exec(
            Pipe::cmd(argv(&["./src/flex", "-o", "/tmp/flex-out.c", "\"$_l\""]))
                .stderr(Arg::lit("/dev/null")),
        )],
        stdout: None,
    }]
}
