use crate::flags::{Cflag, Flag, Ldflag};
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
        Flag::new(
            vec![
                Cflag::O2,
                Cflag::Pipe,
                Cflag::March(target),
                Cflag::TreeVectorize,
                Cflag::NoSemanticInterposition,
                Cflag::FortifySource,
                Cflag::StackProtectorStrong,
                Cflag::Lto,
                Cflag::WerrorOdr,
                Cflag::WerrorLtoTypeMismatch,
                Cflag::WerrorStrictAliasing,
            ],
            vec![
                Ldflag::FuseLdMold,
                Ldflag::O1,
                Ldflag::AsNeeded,
                Ldflag::Relro,
                Ldflag::Now,
                Ldflag::Lto,
                Ldflag::WerrorOdr,
                Ldflag::WerrorLtoTypeMismatch,
                Ldflag::WerrorStrictAliasing,
            ],
        )
    }

    fn build_script(&self) -> &'static str {
        concat!(
            "set -euxo pipefail\n",
            include_str!("sysroot_env.sh"),
            r#"
if [ -n "${SYSROOT:-}" ]; then
  export CC="$CC --sysroot=$SYSROOT"
fi
PGO=/tmp/zlib-pgo
rm -rf "$PGO"
mkdir -p "$PGO"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  CHOST="$TRIPLE" ./configure --prefix=/usr --libdir=/usr/lib64 --shared
else
  ./configure --prefix=/usr --libdir=/usr/lib64 --shared
fi
if [ "$TRIPLE" = "$HOST_TRIPLE" ]; then
  make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-generate -fprofile-dir=$PGO" LDFLAGS="$LDFLAGS -fprofile-generate"
  echo 'pgo' | ./minigzip | ./minigzip -d >/dev/null
  ./example >/dev/null
  make distclean
  ./configure --prefix=/usr --libdir=/usr/lib64 --shared
  make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-use -fprofile-dir=$PGO -fprofile-correction -Wno-missing-profile" LDFLAGS="$LDFLAGS -fprofile-use"
else
  make -j"$(nproc)"
fi
make install DESTDIR="$OUT"
"#,
        )
    }
}
