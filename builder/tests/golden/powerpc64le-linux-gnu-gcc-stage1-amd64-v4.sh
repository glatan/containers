set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
PREFIX=/usr/lib/cross/${TRIPLE}/stage1
SYSROOT=/usr/${TRIPLE}
mkdir -p ${SYSROOT}/include ${BUILD}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
cp -a ${HDR}/. ${SYSROOT}/include/
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS=--with-isl
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS=" -I/dep/zlib/usr/include"
  export CXXFLAGS=" -I/dep/zlib/usr/include"
  export LDFLAGS=" -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS=" -I/dep/zstd/usr/include"
  export CXXFLAGS=" -I/dep/zstd/usr/include"
  export LDFLAGS=" -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
cd ${BUILD}
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr/lib/cross/${TRIPLE}/stage1 --with-sysroot=/usr/${TRIPLE} --with-build-sysroot=/usr/${TRIPLE} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-cpu=power8 --without-headers --with-newlib --disable-shared --disable-threads --disable-werror --disable-libatomic --disable-libsanitizer --disable-libssp --disable-libquadmath --disable-libgomp --disable-libstdcxx --disable-libvtv --disable-bootstrap --disable-multilib --enable-languages=c --enable-lto --enable-plugin --enable-default-pie --enable-default-ssp --enable-checking=release
make -j"$(nproc)" all-gcc all-target-libgcc
make DESTDIR="$OUT" install-gcc install-target-libgcc
