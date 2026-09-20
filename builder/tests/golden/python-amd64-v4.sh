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
for _dep in libffi zlib openssl expat sqlite ncurses readline xz bzip2 zstd; do
  if [ -d /dep/${_dep}/usr/include ]; then
    export CPPFLAGS="${CPPFLAGS:-} -I/dep/${_dep}/usr/include"
    export CFLAGS="${CFLAGS} -I/dep/${_dep}/usr/include"
  fi
  if [ -d /dep/${_dep}/usr/lib64 ]; then
    export LDFLAGS="${LDFLAGS} -L/dep/${_dep}/usr/lib64"
    export PKG_CONFIG_PATH="/dep/${_dep}/usr/lib64/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  else
    if [ -d /dep/${_dep}/usr/lib ]; then
      export LDFLAGS="${LDFLAGS} -L/dep/${_dep}/usr/lib"
      export PKG_CONFIG_PATH="/dep/${_dep}/usr/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
    fi
  fi
done
autotools_configure --prefix=/usr --enable-shared --enable-optimizations --without-ensurepip
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
make install CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
ln -sf python3 "$OUT"/usr/bin/python
