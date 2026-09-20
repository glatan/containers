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
./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-generate=$PGO -fprofile-update=atomic" LDFLAGS="$LDFLAGS -fprofile-generate"
export LD_LIBRARY_PATH="${PWD}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
(
  for _i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do
    cat ./*.c ./*.h
  done > /tmp/zlib-pgo-input
  for _level in 1 6 9; do
    ./minigzip -c "-$_level" < /tmp/zlib-pgo-input | ./minigzip -d -c > /dev/null
  done
  ./example > /dev/null
) || true
make distclean
./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-use=$PGO -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="$LDFLAGS -fprofile-use"
make install DESTDIR="$OUT"
