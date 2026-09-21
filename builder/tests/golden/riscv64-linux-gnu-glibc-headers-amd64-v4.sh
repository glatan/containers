set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
STAGE1=/dep/${TRIPLE}-gcc-stage1/usr/lib/cross/${TRIPLE}/stage1
SYSROOT=${OUT}/usr/${TRIPLE}
mkdir -p ${SYSROOT}/include ${SYSROOT}/lib ${BUILD}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
cp -a ${HDR}/. ${SYSROOT}/include/
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
cd ${BUILD}
unset CFLAGS
unset CXXFLAGS
unset LDFLAGS
export CFLAGS="-O2 -pipe -march=rv64gc -mabi=lp64d"
export CPP="${STAGE1}/bin/${TRIPLE}-gcc -E"
BUILD_CC="${HOST_TRIPLE}-gcc" CC="${STAGE1}/bin/${TRIPLE}-gcc" AR="${TRIPLE}-ar" RANLIB="${TRIPLE}-ranlib" NM="${TRIPLE}-nm" OBJCOPY="${TRIPLE}-objcopy" OBJDUMP="${TRIPLE}-objdump" READELF="${TRIPLE}-readelf" ${SOURCE}/configure --prefix=/ --host=${TRIPLE} --build=${HOST_TRIPLE} --with-headers=${SYSROOT}/include --disable-werror libc_cv_forced_unwind=yes libc_cv_c_cleanup=yes
make csu/subdir_lib
make install-bootstrap-headers=yes install-headers DESTDIR="$SYSROOT"
install -m644 csu/crt1.o csu/crti.o csu/crtn.o ${SYSROOT}/lib/
${STAGE1}/bin/${TRIPLE}-gcc -nostdlib -nostartfiles -shared -x c /dev/null -o ${SYSROOT}/lib/libc.so
mkdir -p ${SYSROOT}/include/gnu
touch ${SYSROOT}/include/gnu/stubs.h
ln -sf lib $SYSROOT/lib64
