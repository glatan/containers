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
       "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"; then
    if [ -f config.log ]; then
      echo "===== config.log (last 80 lines) ====="
      tail -n 80 config.log
    fi
    return 1
  fi
}
mkdir -p "$BUILD"
cd "$BUILD"
export PGODIR=/tmp/pgo
rm -rf "$PGODIR"
mkdir -p "$PGODIR"
autotools_configure --prefix=/usr
export _PGO_CFLAGS="${CFLAGS:-}" _PGO_CXXFLAGS="${CXXFLAGS:-}" _PGO_LDFLAGS="${LDFLAGS:-}"
export CFLAGS="$_PGO_CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-generate"
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
(
  for _i in 1 2 3 4 5 6 7 8; do
    find "$SOURCE" -name '*.c' -o -name '*.h' | xargs cat
  done > /tmp/pgo-input
  for _opt in -0 -6 -9e; do
    ./src/xz/xz -T4 "$_opt" -c /tmp/pgo-input | ./src/xz/xz -d -c > /dev/null
  done
) || true
make distclean || make clean
export CFLAGS="$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
autotools_configure --prefix=/usr
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
make install CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
