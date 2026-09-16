use crate::flags::{Cflag, Flag, Ldflag};
use crate::package::{Bootstrap, Dependency, Package, Source};
use crate::target::Target;

pub struct Mold;

impl Package for Mold {
    fn name(&self) -> &'static str {
        "mold"
    }

    fn version(&self) -> &'static str {
        "2.42.1"
    }

    fn license(&self) -> &'static str {
        "MIT"
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/rui314/mold/archive/refs/tags/v2.42.1.tar.gz",
            sha3_512: "893446caeb9a904d3d796a8fc3f5c6f6361b7f2ea0d164ee56fcd48a8976857984e10faf51bb4610808faf43d87dd73bb4d02bd022dc6b93b42f80727e258ef4",
            blake3: "ab64ef69b9c842942058e678423ce9fb3901f66725295fbd4f426f14af549f62d6f818fa4161ae6a6bdec58d4f27485f65b2ebfb3374e0f3714a81066555ceca",
            subdir: "mold-2.42.1",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(
            target,
            &[
                crate::package::gcc_dep(target),
                Dependency {
                    name: "zlib",
                    version: "1.3.2",
                },
                Dependency {
                    name: "zstd",
                    version: "1.5.7",
                },
            ],
        )
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![
            crate::package::LINUX_HEADERS,
            Dependency {
                name: "cmake",
                version: "4.4.3",
            },
        ]
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
                Cflag::FormatSecurity,
            ],
            vec![Ldflag::O1, Ldflag::Relro, Ldflag::Now],
        )
    }

    fn build_script(&self) -> &'static str {
        concat!(
            "set -euxo pipefail\n",
            include_str!("sysroot_env.sh"),
            r#"
CMAKE_PREFIX=
for _dep in zlib zstd; do
  if [ -d /dep/$_dep/usr ]; then
    CMAKE_PREFIX="${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/$_dep/usr"
  fi
done
if [ -n "$CMAKE_PREFIX" ]; then
  export CMAKE_PREFIX_PATH="$CMAKE_PREFIX${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
fi
if [ -d /dep/cmake/usr/bin ]; then
  export PATH="/dep/cmake/usr/bin:$PATH"
fi
CMAKE_CROSS=()
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  CMAKE_CROSS=(
    -DCMAKE_SYSTEM_NAME=Linux
    -DCMAKE_SYSTEM_PROCESSOR="${TRIPLE%%-*}"
    -DCMAKE_C_COMPILER_TARGET="$TRIPLE"
    -DCMAKE_CXX_COMPILER_TARGET="$TRIPLE"
  )
fi
cmake -S "$SOURCE" -B "$BUILD" \
  "${CMAKE_CROSS[@]}" \
  -DCMAKE_BUILD_TYPE=Release \
  -DCMAKE_INSTALL_PREFIX=/usr \
  -DCMAKE_C_COMPILER="$CC" \
  -DCMAKE_CXX_COMPILER="$CXX" \
  -DCMAKE_C_FLAGS="${CFLAGS:-}" \
  -DCMAKE_CXX_FLAGS="${CXXFLAGS:-}" \
  -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-}" \
  -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-}" \
  -DMOLD_USE_MIMALLOC=ON \
  -DMOLD_USE_SYSTEM_MIMALLOC=OFF \
  -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "$BUILD" -j"$(nproc)"
cmake --install "$BUILD" --prefix "$OUT/usr"
"#,
        )
    }
}

impl Bootstrap for Mold {}
