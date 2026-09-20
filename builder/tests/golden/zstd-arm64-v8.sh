set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
make -j"$(nproc)" PREFIX=/usr LIBDIR=/usr/lib HAVE_ZLIB=0 HAVE_LZMA=0 HAVE_LZ4=0
make install PREFIX=/usr LIBDIR=/usr/lib DESTDIR="$OUT"
