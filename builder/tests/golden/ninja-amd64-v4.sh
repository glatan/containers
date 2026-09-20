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
rm -rf "${PGODIR}"
mkdir -p "${PGODIR}"
export _PGO_CXXFLAGS="${CXXFLAGS:-}"
export _PGO_LDFLAGS="${LDFLAGS:-}"
export CXXFLAGS="${_PGO_CXXFLAGS} -fprofile-generate=${PGODIR} -fprofile-update=atomic"
export LDFLAGS="${_PGO_LDFLAGS} -fprofile-generate"
python3 configure.py --bootstrap
(
  ./ninja --version
  rm -f build/*.o
  ./ninja -j$(nproc) ninja
) || true
rm -rf build ninja
export CXXFLAGS="${_PGO_CXXFLAGS} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="${_PGO_LDFLAGS} -fprofile-use"
python3 configure.py --bootstrap
install -D -m 755 ninja "$OUT"/usr/bin/ninja
