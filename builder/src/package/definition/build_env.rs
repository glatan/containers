use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Dependency, Package, Source};
use crate::target::Target;

const fn dep(name: &'static str, version: &'static str) -> Dependency {
    Dependency { name, version }
}

pub struct BuildEnv;

impl Package for BuildEnv {
    fn name(&self) -> &'static str {
        "build-env"
    }
    fn version(&self) -> &'static str {
        "1"
    }
    fn license(&self) -> &'static str {
        "CC0-1.0"
    }
    /// 依存グラフのエントリポイントとしてのみ使う合成ルート。
    fn compose_root(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://example.invalid/build-env",
            sha3_512: "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            blake3: "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
            subdir: "build-env",
        }
    }
    fn dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![
            dep("x86_64-linux-gnu-glibc", "2.44"),
            dep("x86_64-linux-gnu-gcc", "16.2.0"),
            dep("x86_64-linux-gnu-binutils", "2.47"),
            dep("mold", "2.42.1"),
            dep("bash", "5.3"),
            dep("coreutils", "9.12"),
            dep("make", "4.4.1"),
            dep("sed", "4.10"),
            dep("grep", "3.12"),
            dep("gawk", "5.4.1"),
            dep("findutils", "4.11.0"),
            dep("tar", "1.35"),
            dep("gzip", "1.14"),
            dep("xz", "5.8.4"),
            dep("diffutils", "3.12"),
            dep("patch", "2.8"),
            dep("m4", "1.4.21"),
        ]
    }
    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }
    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }
    fn supports(&self, target: Target) -> bool {
        target == Target::BUILD_ENV
    }
    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Noop
    }
}
