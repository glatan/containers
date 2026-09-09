use crate::flags::Var;
use crate::package::{Dependency, Package, Source};

pub struct Zlib;

impl Package for Zlib {
    fn name(&self) -> &'static str {
        "zlib"
    }

    fn version(&self) -> &'static str {
        "1.3.1"
    }

    fn license(&self) -> &'static str {
        "Zlib"
    }

    fn build_only(&self) -> bool {
        false
    }

    fn network(&self) -> bool {
        false
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/madler/zlib/releases/download/v1.3.1/zlib-1.3.1.tar.gz",
            sha3_512: "d4233237ef02d5a6182e4b3704b5a7da40169076edcc0ff0546441f27ace6f8258fc5668a241263e8f8b7726d974c82654ed06ea1680f82784f79874e6350570",
            blake3: "207c3b0862cb4e3686f8405f76a98c38dbad9c94bcf4be4b9efca0716aba51ecbb74a7a73fb847a40884a5a12be05d3a3d1a9b534b11b16efcec551744bd3fb8",
            subdir: "zlib-1.3.1",
        }
    }

    fn dependencies(&self) -> &'static [Dependency] {
        &[Dependency {
            name: "glibc",
            version: "2.44",
        }]
    }

    fn build_dependencies(&self) -> &'static [Dependency] {
        &[Dependency {
            name: "linux-headers",
            version: "6.12.109",
        }]
    }

    fn flags(&self) -> &'static [Var] {
        &[Var::Cflags]
    }

    fn build_script(&self) -> &'static str {
        r#"set -euxo pipefail
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  pacman -Sy --noconfirm "${CROSS}gcc"
  export CC="$CROSS"gcc
  export AR="${CROSS}gcc-ar"
  export RANLIB="${CROSS}gcc-ranlib"
else
  export CC=gcc
  export AR=gcc-ar
  export RANLIB=gcc-ranlib
fi
export CC="$CC --sysroot=/dep/glibc -idirafter /dep/linux-headers/usr/include -B/dep/glibc/usr/lib64 -L/dep/glibc/usr/lib64 -L/dep/glibc/lib64"
export CFLAGS="$CFLAGS -flto"
export LDFLAGS="-flto -B/dep/glibc/usr/lib64 -L/dep/glibc/usr/lib64 -L/dep/glibc/lib64"
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
"#
    }
}
