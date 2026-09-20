set -euxo pipefail
export PGODIR=/tmp/pgo
rm -rf "${PGODIR}"
mkdir -p "${PGODIR}"
cmake -S "${SOURCE}" -B "${BUILD}/gen" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=ON -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS} -fprofile-generate=${PGODIR} -fprofile-update=atomic" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS} -fprofile-generate" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS} -fprofile-generate"
cmake --build "${BUILD}/gen" -j$(nproc)
export LD_LIBRARY_PATH="${BUILD}/gen${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
(
  for _i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do
    cat "${SOURCE}"/*.c "${SOURCE}"/*.h
  done > /tmp/pgo-input
  for _level in 1 6 9; do
    "${BUILD}/gen/minideflate" -c "-${_level}" < /tmp/pgo-input | "${BUILD}/gen/minideflate" -d -c > /dev/null
  done
) || true
unset LD_LIBRARY_PATH
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=OFF -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS} -fprofile-use" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS} -fprofile-use"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
