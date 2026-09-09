use crate::flags::Var;
use crate::package::{Dependency, Package, Source};

pub struct ZlibNg;

impl Package for ZlibNg {
    fn name(&self) -> &'static str {
        "zlib-ng"
    }

    fn version(&self) -> &'static str {
        "2.3.3"
    }

    fn license(&self) -> &'static str {
        "Zlib"
    }

    fn build_only(&self) -> bool {
        false
    }

    fn network(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/zlib-ng/zlib-ng/archive/refs/tags/2.3.3.tar.gz",
            sha3_512: "b045a4d3358f73db5566951944186293b9a99c5bc9d395866bc0a539617b9584b1499d3d0e9a8943284a7b10a829902dd37e1d6ec55823e6203b000644ba0170",
            blake3: "103cd4e5aa69bb2d0383a242f231bccf64c8a79c21156e7c6187db3cebc1367dc975216cfa092b7d37cdfb303e1fc9cdb2b63d6ef59ce6dc662fbe89286ada42",
            subdir: "zlib-ng-2.3.3",
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
  pacman -Sy --noconfirm cmake "${CROSS}gcc"
  COMPILER="$CROSS"gcc
  AR="${CROSS}gcc-ar"
  RANLIB="${CROSS}gcc-ranlib"
  NM="${CROSS}gcc-nm"
else
  pacman -Sy --noconfirm cmake
  COMPILER=gcc
  AR=gcc-ar
  RANLIB=gcc-ranlib
  NM=gcc-nm
fi
SYSROOT_FLAGS="--sysroot=/dep/glibc -idirafter /dep/linux-headers/usr/include -B/dep/glibc/usr/lib64 -L/dep/glibc/usr/lib64 -L/dep/glibc/lib64"
cmake -S "$SOURCE" -B "$BUILD" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=/usr \
  -DCMAKE_INSTALL_LIBDIR=lib64 \
  -DZLIB_COMPAT=OFF \
  -DWITH_GTEST=OFF \
  -DBUILD_TESTING=OFF \
  -DBUILD_SHARED_LIBS=ON \
  -DCMAKE_INTERPROCEDURAL_OPTIMIZATION=ON \
  -DCMAKE_C_COMPILER="$COMPILER" \
  -DCMAKE_AR="$AR" \
  -DCMAKE_NM="$NM" \
  -DCMAKE_RANLIB="$RANLIB" \
  -DCMAKE_C_FLAGS="$CFLAGS $SYSROOT_FLAGS" \
  -DCMAKE_EXE_LINKER_FLAGS="$SYSROOT_FLAGS" \
  -DCMAKE_SHARED_LINKER_FLAGS="$SYSROOT_FLAGS"
cmake --build "$BUILD" -j"$(nproc)"
DESTDIR="$OUT" cmake --install "$BUILD"
"#
    }
}
