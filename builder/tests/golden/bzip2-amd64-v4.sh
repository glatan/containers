set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
sanitize_flags() {
  printf '%s' "$1" | sed -E 's/-flto(=full|=auto|=thin)?//g; s/-Werror=(odr|lto-type-mismatch|strict-aliasing)//g; s/-f(no-)?hardened//g'
}
autotools_configure() {
  if ! CFLAGS="$(sanitize_flags "${CFLAGS:-}")" \
       CXXFLAGS="$(sanitize_flags "${CXXFLAGS:-}")" \
       LDFLAGS="$(sanitize_flags "${LDFLAGS:-}")" \
       ./configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"; then
    if [ -f config.log ]; then
      echo "===== config.log (last 80 lines) ====="
      tail -n 80 config.log
    fi
    return 1
  fi
}
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
sed -i 's/$(CC) -shared/$(CC) $(LDFLAGS) -shared/' Makefile-libbz2_so
make -f Makefile-libbz2_so CC="$CC" CFLAGS="$CFLAGS -fpic -fPIC -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" CXXFLAGS="${CXXFLAGS:-}"
make CC="$CC" AR="${AR:-${TRIPLE}-ar}" RANLIB="${RANLIB:-${TRIPLE}-ranlib}" CFLAGS="$CFLAGS -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" libbz2.a bzip2 bzip2recover CXXFLAGS="${CXXFLAGS:-}"
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
(
  for _i in 1 2 3 4 5 6 7 8; do
    cat ./*.c ./*.h 2> /dev/null
  done > /tmp/pgo-input
  for _opt in -1 -6 -9; do
    ./bzip2 -c "$_opt" /tmp/pgo-input | ./bzip2 -d -c > /dev/null
  done
  if ./bzip2 -c -9 /tmp/pgo-input > /tmp/pgo.bz2; then
    ./bzip2recover /tmp/pgo.bz2 > /dev/null 2> /dev/null
  fi
) || true
make clean || make distclean
export CFLAGS="$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
sed -i 's/$(CC) -shared/$(CC) $(LDFLAGS) -shared/' Makefile-libbz2_so
make -f Makefile-libbz2_so CC="$CC" CFLAGS="$CFLAGS -fpic -fPIC -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" CXXFLAGS="${CXXFLAGS:-}"
make CC="$CC" AR="${AR:-${TRIPLE}-ar}" RANLIB="${RANLIB:-${TRIPLE}-ranlib}" CFLAGS="$CFLAGS -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" libbz2.a bzip2 bzip2recover CXXFLAGS="${CXXFLAGS:-}"
make install PREFIX="$OUT/usr" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
mkdir -p "${OUT}/usr/lib"
cp -a libbz2.so* "${OUT}/usr/lib/"
