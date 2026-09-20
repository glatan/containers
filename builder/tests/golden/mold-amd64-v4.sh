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
export PGODIR=/tmp/pgo
rm -rf "${PGODIR}"
mkdir -p "${PGODIR}"
cmake -S "${SOURCE}" -B "${BUILD}/gen" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-} -fprofile-generate" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-} -fprofile-generate" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}/gen" -j$(nproc)
(
  mkdir -p /tmp/moldbin /tmp/moldwork
  ln -sf "${BUILD}/gen/mold" /tmp/moldbin/ld.mold
  for _i in 1 2 3 4 5; do
    printf 'int fn%s(void){return %s;}\n' "${_i}" "${_i}" | "${CC}" -x c - -c -o "/tmp/moldwork/t${_i}.o"
    "${CC}" -B/tmp/moldbin -fuse-ld=mold -shared -o "/tmp/moldwork/t${_i}.so" "/tmp/moldwork/t${_i}.o"
  done
  printf 'int main(void){return fn1();}\nextern int fn1(void);\n' | "${CC}" -x c - -c -o "/tmp/moldwork/main.o"
  "${CC}" -B/tmp/moldbin -fuse-ld=mold -o "/tmp/moldwork/prog" "/tmp/moldwork/main.o" "/tmp/moldwork/t1.o"
  "${CC}" -B/tmp/moldbin -fuse-ld=mold -pie -o "/tmp/moldwork/prog-pie" "/tmp/moldwork/main.o" "/tmp/moldwork/t1.o"
) || true
cmake -S "${SOURCE}" -B "${BUILD}/use" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-} -fprofile-use" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-} -fprofile-use" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}/use" -j$(nproc)
cmake --install "${BUILD}/use" --prefix "${OUT}/usr"
