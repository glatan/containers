set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
autotools_configure --prefix=/usr --with-shared --without-cxx --enable-widec --enable-pc-files --with-pkg-config-libdir=/usr/lib/pkgconfig --without-ada --without-debug --with-versioned-syms
make -j"$(nproc)"
make install DESTDIR="$OUT"
