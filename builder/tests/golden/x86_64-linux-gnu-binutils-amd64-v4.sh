set -euxo pipefail
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
export PGODIR=/tmp/pgo
rm -rf ${PGODIR}
mkdir -p ${PGODIR} ${BUILD}/gen ${BUILD}/final
cd ${BUILD}/gen
CFLAGS="${CFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" CXXFLAGS="${CXXFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" LDFLAGS="${LDFLAGS:-} -fprofile-generate" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --infodir=/usr/share/info/${TRIPLE} --disable-nls --disable-werror --enable-plugins --with-system-zlib --without-debuginfod --disable-gprofng --enable-new-dtags --enable-relro --enable-default-execstack=no --enable-default-hash-style=gnu --enable-cet
make -j"$(nproc)"
(
  _as=${BUILD}/gen/gas/as-new
  _ld=${BUILD}/gen/ld/ld-new
  _objdump=${BUILD}/gen/binutils/objdump
  mkdir -p /tmp/bu-work
  for _i in $(seq 1 64); do
    printf ".text\n.globl f%s\nf%s:\n  mov $%s, %%eax\n  ret\n" ${_i} ${_i} ${_i} > /tmp/bu-work/t.s
    ${_as} -o /tmp/bu-work/t.o /tmp/bu-work/t.s
    ${_ld} -r -o /tmp/bu-work/t${_i}.o /tmp/bu-work/t.o
  done
  ${_ld} -r -o /tmp/bu-work/all.o /tmp/bu-work/t*.o
  ${_objdump} -d /tmp/bu-work/all.o > /dev/null
  ${_objdump} -h /tmp/bu-work/all.o > /dev/null
) || true
cd ${BUILD}/final
CFLAGS="${CFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" CXXFLAGS="${CXXFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="${LDFLAGS:-} -fprofile-use" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --infodir=/usr/share/info/${TRIPLE} --disable-nls --disable-werror --enable-plugins --with-system-zlib --without-debuginfod --disable-gprofng --enable-new-dtags --enable-relro --enable-default-execstack=no --enable-default-hash-style=gnu --enable-cet
make -j"$(nproc)"
make install DESTDIR="$OUT"
