set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
PREFIX=/usr/lib/cross/${TRIPLE}/stage2
SYSROOT=/usr/${TRIPLE}
mkdir -p ${SYSROOT} ${BUILD}
if [ -d /dep/${TRIPLE}-glibc-headers/usr/${TRIPLE} ]; then
  cp -a /dep/${TRIPLE}-glibc-headers/usr/${TRIPLE}/. ${SYSROOT}/
fi
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
mkdir -p ${SYSROOT}/include
cp -a ${HDR}/. ${SYSROOT}/include/
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
cd ${BUILD}
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr/lib/cross/${TRIPLE}/stage2 --with-sysroot=/usr/${TRIPLE} --with-build-sysroot=/usr/${TRIPLE} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-cpu=power8 --disable-nls --disable-multilib --disable-werror --disable-libatomic --disable-libsanitizer --disable-libssp --disable-libquadmath --disable-libgomp --disable-libstdcxx --disable-libvtv --disable-bootstrap --enable-shared --enable-threads=posix --enable-languages=c --enable-lto --enable-plugin --enable-checking=release --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array
make -j"$(nproc)" all-gcc all-target-libgcc
make DESTDIR="$OUT" install-gcc install-target-libgcc
