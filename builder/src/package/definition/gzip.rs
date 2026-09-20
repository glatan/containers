use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Bootstrap, Package, Source};

pub struct Gzip;

impl Package for Gzip {
    fn name(&self) -> &'static str {
        "gzip"
    }
    fn version(&self) -> &'static str {
        "1.14"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/gzip/gzip-1.14.tar.xz",
            sha3_512: "6e7b2e6c5d87a64d2adde0dbd81e0262d638c60088dc5a61be5a0e64b8fa1ed9452bc2a7c3c29667ddb37fac8f8dc35ef16e10154f7a6a57ec8de6aa24ee4162",
            blake3: "4d318e6b8ff830c98b98314bcafb54d540ab940553ef1ad5e0743b21a36a473be1c98b3671dae64e436526b39c2359f623d66dae1a003067c3a5ed9e0c0d0617",
            subdir: "gzip-1.14",
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
