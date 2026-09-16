use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

pub struct Cmake;

impl Package for Cmake {
    fn name(&self) -> &'static str {
        "cmake"
    }

    fn version(&self) -> &'static str {
        "4.4.3"
    }

    fn license(&self) -> &'static str {
        "BSD-3-Clause"
    }

    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3.tar.gz",
            sha3_512: "170e8771bffc5d56d42c1702320e5c4dafbe90982cc9bcb8d3e21a16be2bf86b76ca35cef468a0863cb65790b5f6bc4a0e5b30ea8b34a39e56f31fab78ac8162",
            blake3: "347efe575e9f1c7daf9d5888de1c1f1070570d1c1e6beb62522c389b0a408a95be2fca9aaae2d2261a78e68b6777b5fbf08e6a3cce47d64bf12129a13d238f51",
            subdir: "cmake-4.4.3",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[crate::package::gcc_dep(target)])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .step(Step::Shell(
                    r#""$SOURCE"/bootstrap --prefix=/usr --parallel="$(nproc)" --   -DCMAKE_USE_OPENSSL=OFF   -DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON"#,
                ))
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }
}

impl Bootstrap for Cmake {}
