use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps, Workdir};
use crate::package::{Package, Source};

pub struct Zstd;

impl Package for Zstd {
    fn name(&self) -> &'static str {
        "zstd"
    }
    fn version(&self) -> &'static str {
        "1.5.7"
    }
    fn license(&self) -> &'static str {
        "BSD-3-Clause OR GPL-2.0-only"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://github.com/facebook/zstd/releases/download/v1.5.7/zstd-1.5.7.tar.gz",
            sha3_512: "9fcbd59a61d336c34cd46ff6f335787848d37e8c6d0a4aef468755ef4d31fcce7368ce35650282b31d28303bc985470a97b06110b8fb050b0e72eba66afabca8",
            blake3: "730dca31244abd219e995f03a55d95b2cfb4b3e16cda055a79fa6f30a4f0e1db8febef4887bffd4b8c19d6926fca83c0ae68e6e5abf535c0c0a85193342d8568",
            subdir: "zstd-1.5.7",
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
        crate::flags::Pgo::Profiled(PGO_WORKLOAD)
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .workdir(Workdir::InSource)
                .pgo(crate::package::recipe::PgoTemplate::Make(PGO_WORKLOAD))
                .step(Step::make(&[
                    "PREFIX=/usr",
                    "LIBDIR=/usr/lib",
                    "HAVE_ZLIB=0",
                    "HAVE_LZMA=0",
                    "HAVE_LZ4=0",
                ]))
                .step(Step::make_install(&["PREFIX=/usr", "LIBDIR=/usr/lib"])),
        )
    }
}

const PGO_WORKLOAD: &str = r#"for _i in 1 2 3 4 5 6 7 8; do cat ./lib/*.c ./lib/*/*.c ./lib/*/*.h ./programs/*.c 2>/dev/null; done > /tmp/pgo-input
for _level in 1 3 9 19; do
  ./programs/zstd -b"$_level" -T0 /tmp/pgo-input
done"#;
