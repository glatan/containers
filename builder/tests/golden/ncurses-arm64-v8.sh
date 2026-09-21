set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export BUILD_CC="$CC_FOR_BUILD"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
autotools_configure --prefix=/usr --with-shared --without-cxx --enable-widec --enable-pc-files --with-pkg-config-libdir=/usr/lib/pkgconfig --without-ada --without-debug --with-versioned-syms --with-build-cc=x86_64-linux-gnu-gcc --with-build-cpp=x86_64-linux-gnu-gcc --with-build-cflags='-O2 -std=gnu17' --with-build-cppflags= --with-build-ldflags=
make -j"$(nproc)"
make install DESTDIR="$OUT"
