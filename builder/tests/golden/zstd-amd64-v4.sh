set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
export PGODIR=/tmp/pgo
rm -rf "$PGODIR"
mkdir -p "$PGODIR"
export _PGO_CFLAGS="${CFLAGS:-}" _PGO_CXXFLAGS="${CXXFLAGS:-}" _PGO_LDFLAGS="${LDFLAGS:-}"
export CFLAGS="$_PGO_CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-generate"
make -j"$(nproc)" PREFIX=/usr LIBDIR=/usr/lib HAVE_ZLIB=0 HAVE_LZMA=0 HAVE_LZ4=0
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
(
  for _i in 1 2 3 4 5 6 7 8; do
    cat ./lib/*.c ./lib/*/*.c ./lib/*/*.h ./programs/*.c 2> /dev/null
  done > /tmp/pgo-input
  for _level in 1 3 9 19; do
    ./programs/zstd -b"$_level" -T0 /tmp/pgo-input
  done
) || true
make clean || make distclean
export CFLAGS="$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
make -j"$(nproc)" PREFIX=/usr LIBDIR=/usr/lib HAVE_ZLIB=0 HAVE_LZMA=0 HAVE_LZ4=0
make install PREFIX=/usr LIBDIR=/usr/lib DESTDIR="$OUT"
