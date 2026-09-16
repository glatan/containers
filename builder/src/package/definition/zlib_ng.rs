use crate::flags::{Cflag, Flag, Ldflag};
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
CMAKE_CROSS=()
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  CMAKE_CROSS=(
    -DCMAKE_SYSTEM_NAME=Linux
    -DCMAKE_SYSTEM_PROCESSOR="${TRIPLE%%-*}"
    -DCMAKE_C_COMPILER_TARGET="$TRIPLE"
  )
fi
cmake -S "$SOURCE" -B "$BUILD" \
  "${CMAKE_CROSS[@]}" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=/usr \
  -DCMAKE_INSTALL_LIBDIR=lib64 \
  -DZLIB_COMPAT=OFF \
  -DWITH_GTEST=OFF \
  -DBUILD_TESTING=OFF \
  -DBUILD_SHARED_LIBS=ON \
  -DCMAKE_C_COMPILER="$CC" \
  -DCMAKE_AR="$AR" \
  -DCMAKE_NM="$NM" \
  -DCMAKE_RANLIB="$RANLIB" \
  -DCMAKE_C_FLAGS="$CFLAGS" \
  -DCMAKE_EXE_LINKER_FLAGS="$LDFLAGS" \
  -DCMAKE_SHARED_LINKER_FLAGS="$LDFLAGS"
cmake --build "$BUILD" -j"$(nproc)"
DESTDIR="$OUT" cmake --install "$BUILD"
"#,
        )
    }
}
