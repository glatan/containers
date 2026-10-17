use crate::package::recipe::{
    Arg, BuildContext, Env, PgoTemplate, Pipe, Recipe, Step, Steps, Workdir,
};
use crate::package::{Bootstrap, Package, Source};

pub struct Gzip;

impl Package for Gzip {
    fn name(&self) -> &'static str {
        "gzip"
    }
    fn version(&self) -> &'static str {
        "1.15"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/gzip/gzip-1.15.tar.xz",
            sha3_512: "a9b7f473b0baa272f67f18ac4de788f5457231cde528f9b1f287db9c870bdf67857210bc444fe3f8502cd3b76f44e3fec3544da3aec42b0a665dea3e890b219a",
            blake3: "ed2b2743048530c90a8b93b32bac1f9e06b6fc0988c36ca39d94e10ffb5379d11bab254fe1a13f97a6950991b8d817f4186cada139291aa69c5cf560b72d91e6",
            subdir: "gzip-1.15",
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
                // gzip.c で `gzip.h`（`head` をマクロ定義）が `<signal.h>`
                // より先に include され、aarch64 の asm/sigcontext.h が
                // 持つ `head` フィールドと衝突する（upstream bug#81904、
                // 1.15 後に include 順で修正済み）。signal.h を先行
                // インクルードする sed を当てるため InSource コピーで
                // ビルドする。
                .workdir(Workdir::InSource)
                .pgo(PgoTemplate::Autotools(pgo_workload()))
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("sed"),
                        Arg::lit("-i"),
                        Arg::lit("\"/gzip\\.h/i #include <signal.h>\""),
                        Arg::lit("gzip.c"),
                    ]),
                })
                .step(Step::configure(&["--prefix=/usr"]))
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
}

impl Bootstrap for Gzip {}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Pipe, Redirect, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        Step::ForEach {
            var: "_i",
            items: argv(&["1", "2", "3", "4", "5", "6", "7", "8"]),
            body: vec![exec(
                Pipe::cmd(argv(&["cat", "\"$SOURCE\"/*.c", "\"$SOURCE\"/*.h"]))
                    .stderr(Arg::lit("/dev/null")),
            )],
            stdout: Some(Redirect {
                target: Arg::lit("/tmp/pgo-input"),
                append: false,
            }),
        },
        exec(
            Pipe::new(vec![
                argv(&["./gzip", "-9", "-c", "/tmp/pgo-input"]),
                argv(&["./gzip", "-d", "-c"]),
            ])
            .stdout(Arg::lit("/dev/null")),
        ),
        exec(
            Pipe::new(vec![
                argv(&["./gzip", "-1", "-c", "/tmp/pgo-input"]),
                argv(&["./gzip", "-d", "-c"]),
            ])
            .stdout(Arg::lit("/dev/null")),
        ),
    ]
}
