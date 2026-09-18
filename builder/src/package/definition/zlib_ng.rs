use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe, SYSROOT_ENV};
use crate::package::{Dependency, Package, Role, Source};
use crate::target::Target;

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

    fn role(&self) -> Role {
        Role::Catalog
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/zlib-ng/zlib-ng/archive/refs/tags/2.3.3.tar.gz",
            sha3_512: "b045a4d3358f73db5566951944186293b9a99c5bc9d395866bc0a539617b9584b1499d3d0e9a8943284a7b10a829902dd37e1d6ec55823e6203b000644ba0170",
            blake3: "103cd4e5aa69bb2d0383a242f231bccf64c8a79c21156e7c6187db3cebc1367dc975216cfa092b7d37cdfb303e1fc9cdb2b63d6ef59ce6dc662fbe89286ada42",
            subdir: "zlib-ng-2.3.3",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        let mut dependencies = crate::package::headers_dep();
        dependencies.push(Dependency {
            name: "cmake",
            version: "4.4.3",
        });
        dependencies
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        let mut script = String::from("set -euxo pipefail\n");
        script.push_str(SYSROOT_ENV);
        script.push_str(
            r#"
CMAKE_CROSS=()
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  CMAKE_CROSS=(
    -DCMAKE_SYSTEM_NAME=Linux
    -DCMAKE_SYSTEM_PROCESSOR="${TRIPLE%%-*}"
    -DCMAKE_C_COMPILER_TARGET="$TRIPLE"
  )
fi
configure_cmake() {
  cmake -S "$SOURCE" -B "$1" \
    "${CMAKE_CROSS[@]}" \
    -DCMAKE_BUILD_TYPE=Release \
    -DCMAKE_INSTALL_PREFIX=/usr \
    -DCMAKE_INSTALL_LIBDIR=lib64 \
    -DZLIB_COMPAT=OFF \
    -DWITH_GTEST=OFF \
    -DBUILD_TESTING="$2" \
    -DBUILD_SHARED_LIBS=ON \
    -DCMAKE_C_COMPILER="$CC" \
    -DCMAKE_AR="$AR" \
    -DCMAKE_NM="$NM" \
    -DCMAKE_RANLIB="$RANLIB" \
    -DCMAKE_C_FLAGS="$3" \
    -DCMAKE_EXE_LINKER_FLAGS="$4" \
    -DCMAKE_SHARED_LINKER_FLAGS="$4"
}
if [ "$TRIPLE" = "$HOST_TRIPLE" ]; then
  # ネイティブのみ PGO。instrumented ツリーで minideflate を動かし、
  # 別ツリーで -fprofile-use の成果物を作る。
  export PGODIR=/tmp/pgo
  rm -rf "$PGODIR"
  mkdir -p "$PGODIR"
  configure_cmake "$BUILD/gen" ON \
    "$CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic" \
    "$LDFLAGS -fprofile-generate"
  cmake --build "$BUILD/gen" -j"$(nproc)"
  export LD_LIBRARY_PATH="$BUILD/gen${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  (
    for _i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do cat "$SOURCE"/*.c "$SOURCE"/*.h; done > /tmp/pgo-input
    for _level in 1 6 9; do
      "$BUILD/gen/minideflate" -c "-$_level" < /tmp/pgo-input | "$BUILD/gen/minideflate" -d -c > /dev/null
    done
  ) || true
  unset LD_LIBRARY_PATH
  configure_cmake "$BUILD" OFF \
    "$CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" \
    "$LDFLAGS -fprofile-use"
else
  configure_cmake "$BUILD" OFF "$CFLAGS" "$LDFLAGS"
fi
cmake --build "$BUILD" -j"$(nproc)"
DESTDIR="$OUT" cmake --install "$BUILD"
"#,
        );
        Recipe::Shell(script.into())
    }
}
