set -euxo pipefail
CMAKE_PREFIX=
for _dep in zlib zstd; do
  if [ -d /dep/${_dep}/usr ]; then
    CMAKE_PREFIX="${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/${_dep}/usr"
  fi
done
if [ -n "${CMAKE_PREFIX:-}" ]; then
  export CMAKE_PREFIX_PATH="${CMAKE_PREFIX}${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
fi
if [ -d /dep/cmake/usr/bin ]; then
  export PATH="/dep/cmake/usr/bin:${PATH}"
fi
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=aarch64 -DCMAKE_C_COMPILER_TARGET=aarch64-linux-gnu -DCMAKE_CXX_COMPILER_TARGET=aarch64-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-}" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-}" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}" -j$(nproc)
cmake --install "${BUILD}" --prefix "${OUT}/usr"
