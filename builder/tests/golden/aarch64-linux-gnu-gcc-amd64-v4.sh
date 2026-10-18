set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
if [ -d /dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE} ]; then
  mkdir -p /tmp/build-sysroot/usr
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/include
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/usr/include
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib" /tmp/build-sysroot/lib
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib64" /tmp/build-sysroot/lib64
  ln -sfn ../lib /tmp/build-sysroot/usr/lib
  ln -sfn ../lib64 /tmp/build-sysroot/usr/lib64
  _build_root=/tmp/build-sysroot
fi
_bf="-O2 -pipe"
if [ -n "${_build_root:-}" ]; then
  _bf="${_bf} --sysroot=${_build_root}"
  if [ -f ${_build_root}/lib/crti.o ]; then
    _bf="${_bf} -B${_build_root}/lib"
  else
    if [ -f ${_build_root}/lib64/crti.o ]; then
      _bf="${_bf} -B${_build_root}/lib64"
    fi
  fi
fi
export CFLAGS_FOR_BUILD="${_bf}"
export CXXFLAGS_FOR_BUILD="${_bf}"
export LDFLAGS_FOR_BUILD="${_build_root:+--sysroot=$_build_root}"
SYSROOT=/usr/${TRIPLE}
_profiledir=${BUILD}/pgo-profile-data
rm -rf ${BUILD}/instrumented ${BUILD}/final
mkdir -p ${SYSROOT} ${_profiledir} ${BUILD}/instrumented ${BUILD}/final
if [ -d /dep/${TRIPLE}-glibc/usr/${TRIPLE} ]; then
  cp -a /dep/${TRIPLE}-glibc/usr/${TRIPLE}/. ${SYSROOT}/
fi
for _so in $(find ${SYSROOT} -type f -name "*.so"); do
  if grep -q "GNU ld script" ${_so}; then
    sed -i "s|//lib/|/lib/|g; s|//lib64/|/lib64/|g" ${_so}
  fi
done
BUILD_SYSROOT=${SYSROOT}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
mkdir -p ${BUILD_SYSROOT}/include
cp -a ${HDR}/. ${BUILD_SYSROOT}/include/ || true
export CFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export CXXFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export LDFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT}"
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS=--with-isl
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zlib/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zlib/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zstd/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zstd/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
cd ${BUILD}/instrumented
CFLAGS="${CFLAGS:-} -fprofile-generate=${_profiledir} -fprofile-update=atomic" CXXFLAGS="${CXXFLAGS:-} -fprofile-generate=${_profiledir} -fprofile-update=atomic" LDFLAGS="${LDFLAGS:-} -fprofile-generate=${_profiledir}" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --with-sysroot=${SYSROOT} --with-build-sysroot=${BUILD_SYSROOT} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=armv8-a --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib --disable-libsanitizer --enable-plugin --enable-languages=c,c++ --enable-lto --enable-checking=release --enable-__cxa_atexit --enable-threads=posix --enable-shared --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array --enable-libstdcxx-backtrace --enable-host-pie --enable-host-bind-now --with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'
make -j"$(nproc)"
_xgcc=${BUILD}/instrumented/gcc/xgcc
_xgpp=${BUILD}/instrumented/gcc/xg++
_B=${BUILD}/instrumented/gcc/
_ts=${SOURCE}/gcc/testsuite
for _opt in -O0 -O1 -O2 -O3 -Os; do
  find ${_ts}/gcc.c-torture/compile -maxdepth 1 -name "*.c" -print0 | xargs -0 -P$(nproc) -I{} ${_xgcc} -B${_B} ${_opt} -S -o /dev/null -w {} 2> /dev/null || true
done
find ${_ts}/g++.dg -maxdepth 1 -name "*.C" -print0 | xargs -0 -P$(nproc) -I{} ${_xgpp} -B${_B} -O2 -S -o /dev/null -w {} 2> /dev/null || true
cd ${BUILD}/final
CFLAGS="${CFLAGS:-} -fprofile-use=${_profiledir} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" CXXFLAGS="${CXXFLAGS:-} -fprofile-use=${_profiledir} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="${LDFLAGS:-} -fprofile-use" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --with-sysroot=${SYSROOT} --with-build-sysroot=${BUILD_SYSROOT} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=armv8-a --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib --disable-libsanitizer --enable-plugin --enable-languages=c,c++ --enable-lto --enable-checking=release --enable-__cxa_atexit --enable-threads=posix --enable-shared --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array --enable-libstdcxx-backtrace --enable-host-pie --enable-host-bind-now --with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'
make -j"$(nproc)"
make DESTDIR="$OUT" install
