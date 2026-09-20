set -euxo pipefail
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
mkdir -p ${BUILD}
cd ${BUILD}
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --infodir=/usr/share/info/${TRIPLE} --disable-nls --disable-werror --enable-plugins --with-system-zlib --without-debuginfod --disable-gprofng --enable-new-dtags --enable-relro --enable-default-execstack=no --enable-default-hash-style=gnu
make -j"$(nproc)"
make install DESTDIR="$OUT"
