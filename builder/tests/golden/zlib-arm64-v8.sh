set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
PGO=/tmp/zlib-pgo
rm -rf "$PGO"
mkdir -p "$PGO"
CHOST="${TRIPLE}" ./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)"
make install DESTDIR="$OUT"
