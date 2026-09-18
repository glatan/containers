use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe, SYSROOT_ENV};
use crate::package::{Dependency, Package, Role, Source};
use crate::target::Target;

pub struct Zlib;

impl Package for Zlib {
    fn name(&self) -> &'static str {
        "zlib"
    }

    fn version(&self) -> &'static str {
        "1.3.2"
    }

    fn license(&self) -> &'static str {
        "Zlib"
    }

    fn role(&self) -> Role {
        Role::Catalog
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.gz",
            sha3_512: "860936cb0ddea209f9c14219dbf05efbef7ea83abe93a25bf39dd591a8f075ab5ad0c8225b1455def844d831f2d7b8f645704ebe387e630ad3075f765925dcc7",
            blake3: "ea79f8ae7c98d38ea360b70449fbb0052da4ff4a7201f03387113e5dab8543793587a6ee51d8307b5203aa03bf0a0053fca792f4c99d95c48aecc4b49359dc1b",
            subdir: "zlib-1.3.2",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let mut script = String::from("set -euxo pipefail\n");
        script.push_str(SYSROOT_ENV);
        script.push_str(HEAD);
        if ctx.cross() {
            // クロスではプロファイルを実行できない
            script.push_str(
                "CHOST=\"$TRIPLE\" ./configure --prefix=/usr --libdir=/usr/lib64 --shared\nmake -j\"$(nproc)\"\n",
            );
        } else {
            script.push_str(PGO);
        }
        script.push_str("make install DESTDIR=\"$OUT\"\n");
        Recipe::Shell(script.into())
    }
}

const HEAD: &str = r#"
if [ -n "${SYSROOT:-}" ]; then
  export CC="$CC --sysroot=$SYSROOT"
fi
PGO=/tmp/zlib-pgo
rm -rf "$PGO"
mkdir -p "$PGO"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
"#;

const PGO: &str = r#"./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-generate=$PGO -fprofile-update=atomic" LDFLAGS="$LDFLAGS -fprofile-generate"
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
(
  for _i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do cat ./*.c ./*.h; done > /tmp/zlib-pgo-input
  for _level in 1 6 9; do
    ./minigzip -c "-$_level" < /tmp/zlib-pgo-input | ./minigzip -d -c > /dev/null
  done
  ./example > /dev/null
) || true
make distclean
./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-use=$PGO -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="$LDFLAGS -fprofile-use"
"#;
