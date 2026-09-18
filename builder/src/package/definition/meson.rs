use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps, Workdir};
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
                .step(Step::Shell(
                    // python 成果物は --without-ensurepip で pip を持たないため、
                    // mesonbuild を site-packages へ直接配置して launcher を作る。
                    r#"_pyver=$(python3 -c 'import sys; print(f"python{sys.version_info.major}.{sys.version_info.minor}")')
_site="$OUT/usr/lib/$_pyver/site-packages"
mkdir -p "$_site" "$OUT/usr/bin"
cp -a "$SOURCE"/mesonbuild "$_site"/
printf '#!/bin/sh\nexec python3 -m mesonbuild.mesonmain "$@"\n' > "$OUT/usr/bin/meson"
chmod +x "$OUT/usr/bin/meson""#,
                )),
        )
    }
}
