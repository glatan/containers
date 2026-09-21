set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
if [ -d /dep/bison/usr/share/bison ]; then
  export BISON_PKGDATADIR=/dep/bison/usr/share/bison
fi
STAGE2=/dep/${TRIPLE}-gcc-stage2/usr/lib/cross/${TRIPLE}/stage2
SYSROOT=${OUT}/usr/${TRIPLE}
mkdir -p ${SYSROOT}/include ${BUILD}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
cp -a ${HDR}/. ${SYSROOT}/include/
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
cd ${BUILD}
unset CFLAGS
unset CXXFLAGS
unset LDFLAGS
export CFLAGS="-O2 -pipe -march=${MARCH}"
export CXXFLAGS="${CFLAGS:-}"
unset LD
export CPP="${STAGE2}/bin/${TRIPLE}-gcc -E"
BUILD_CC="${HOST_TRIPLE}-gcc" CC="${STAGE2}/bin/${TRIPLE}-gcc" AR="${TRIPLE}-ar" RANLIB="${TRIPLE}-ranlib" NM="${TRIPLE}-nm" AS="${TRIPLE}-as" LD="${TRIPLE}-ld" OBJCOPY="${TRIPLE}-objcopy" OBJDUMP="${TRIPLE}-objdump" READELF="${TRIPLE}-readelf" ${SOURCE}/configure --prefix=/ --host=${TRIPLE} --build=${HOST_TRIPLE} --libdir=/lib --with-headers=${SYSROOT}/include --disable-werror --disable-nscd --disable-profile --enable-stack-protector=strong --enable-bind-now --enable-fortify-source --enable-cet libc_cv_forced_unwind=yes libc_cv_c_cleanup=yes
echo "build-programs=no" >> configparms
make -j"$(nproc)"
make DESTDIR="$SYSROOT" install
rm -rf ${SYSROOT}/share ${SYSROOT}/etc ${SYSROOT}/var ${SYSROOT}/bin ${SYSROOT}/sbin
for _so in $(find ${SYSROOT} -type f -name "*.so"); do
  if grep -q "GNU ld script" ${_so} >/dev/null; then
    sed -i "s|//lib/|/lib/|g; s|//lib64/|/lib64/|g" ${_so}
  fi
done
ln -sfn lib ${SYSROOT}/lib64
mkdir -p ${OUT}/lib64 ${OUT}/usr/lib64
ln -sfn /usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2 ${OUT}/lib64/ld-linux-x86-64.so.2 || true
if [ -f ${SYSROOT}/lib/ld-linux-x86-64.so.2 ]; then
  ln -sfn ../usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2 ${OUT}/lib64/ld-linux-x86-64.so.2
fi
for so in ${SYSROOT}/lib/libc.so.6 ${SYSROOT}/lib/libm.so.6 ${SYSROOT}/lib/libdl.so.2 ${SYSROOT}/lib/libpthread.so.0 ${SYSROOT}/lib/librt.so.1 ${SYSROOT}/lib/libresolv.so.2; do
  if [ -f ${so} ]; then
    ln -sfn "$(realpath --relative-to=${OUT}/usr/lib64 ${so})" ${OUT}/usr/lib64/$(basename ${so})
  fi
done
