use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps, Workdir};
use crate::package::{Package, Source};

pub struct Ninja;

impl Package for Ninja {
    fn name(&self) -> &'static str {
        "ninja"
    }
    fn version(&self) -> &'static str {
        "1.13.2"
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
            url: "https://github.com/ninja-build/ninja/archive/refs/tags/v1.13.2.tar.gz",
            sha3_512: "4d48df9a0ea4a0b74ac6ec6206ef4743dfabe8acabffb69d66aeee15f9608414f7ba6a4f755728c329fe3827abdbac9d58afbffff4b95af4cf82010f30838b6d",
            blake3: "b32100cf57fde2a19d3c242f363c308c3d7a7ed3167774eefcbd107baada26e4fb5240af8ae22fde9d1cec079f5001098a30bc90f4fb3b5303c3bccd636fe161",
            subdir: "ninja-1.13.2",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(target, &[crate::package::gcc_dep(target)])
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }
    /// `configure.py --bootstrap` の実行に必要（成果物にはリンクしない）。
    fn host_tools(&self, _target: crate::target::Target) -> Vec<&'static str> {
        vec!["python"]
    }

    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let bootstrap = if ctx.cross() {
            "python3 configure.py --bootstrap"
        } else {
            PGO_BOOTSTRAP
        };
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .workdir(Workdir::InSource)
                .step(Step::Shell(bootstrap))
                .step(Step::Install {
                    src: "ninja",
                    dst: "\"$OUT\"/usr/bin/ninja",
                    mode: "755",
                }),
        )
    }
}

/// instrumented bootstrap → ninja で自身を rebuild（workload）→
/// clean → -fprofile-use bootstrap。
const PGO_BOOTSTRAP: &str = r#"export PGODIR=/tmp/pgo
rm -rf "$PGODIR"
mkdir -p "$PGODIR"
export _PGO_CXXFLAGS="${CXXFLAGS:-}" _PGO_LDFLAGS="${LDFLAGS:-}"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-generate"
python3 configure.py --bootstrap
(
  ./ninja --version
  # bootstrap 直後は ninja が最新なので、obj を消して再ビルドさせる
  rm -f build/*.o
  ./ninja -j"$(nproc)" ninja
) || true
rm -rf build ninja
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
python3 configure.py --bootstrap"#;
