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
export CFLAGS_FOR_BUILD="-O2 -std=gnu17"
autotools_configure --prefix=/usr --without-bash-malloc
make -j"$(nproc)"
make install DESTDIR="$OUT"
ln -sf bash "$OUT"/usr/bin/sh
