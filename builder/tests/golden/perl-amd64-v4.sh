set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
sanitize_flags() {
  printf '%s' "$1" | sed -E 's/-flto(=[^[:space:]]*)?//g; s/-Werror=(odr|lto-type-mismatch|strict-aliasing)//g; s/-f(no-)?hardened//g'
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
export CFLAGS="$(sanitize_flags "${CFLAGS:-}") -std=gnu17"
export CXXFLAGS="$(sanitize_flags "${CXXFLAGS:-}")"
export LDFLAGS="$(sanitize_flags "${LDFLAGS:-}")"
if [ -d /dep/libxcrypt/usr/include ]; then
  export CFLAGS="${CFLAGS} -I/dep/libxcrypt/usr/include"
  export CPPFLAGS="${CPPFLAGS:-} -I/dep/libxcrypt/usr/include"
fi
if [ -d /dep/libxcrypt/usr/lib64 ]; then
  export LDFLAGS="${LDFLAGS} -L/dep/libxcrypt/usr/lib64"
else
  if [ -d /dep/libxcrypt/usr/lib ]; then
    export LDFLAGS="${LDFLAGS} -L/dep/libxcrypt/usr/lib"
  fi
fi
cp -a "${SOURCE}"/. "${BUILD}"/
cd "${BUILD}"
_sysroot=/dep/${TRIPLE}-glibc/usr/${TRIPLE}
if [ -d "${_sysroot}/lib" ]; then
  export LIBRARY_PATH="${_sysroot}/lib:${_sysroot}/lib64${LIBRARY_PATH:+:$LIBRARY_PATH}"
fi
export LD_LIBRARY_PATH="/dep/libxcrypt/usr/lib:/dep/libxcrypt/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
if [ -d "${_sysroot}/include" ]; then
  ./Configure -des -Dprefix=/usr -Dvendorprefix=/usr -Duse64bitall -Dusethreads -Duseshrplib -Dcc="${CC}" -Dld="${CC}" -Accflags="${CFLAGS}" -Aldflags="${LDFLAGS}" -Dsysroot="${SYSROOT}" -Dusrinc="${_sysroot}/include" -Dincpth="${_sysroot}/include" -Dlibpth="${_sysroot}/lib ${_sysroot}/lib64"
else
  ./Configure -des -Dprefix=/usr -Dvendorprefix=/usr -Duse64bitall -Dusethreads -Duseshrplib -Dcc="${CC}" -Dld="${CC}" -Accflags="${CFLAGS}" -Aldflags="${LDFLAGS}"
fi
export PERL_USE_UNSAFE_INC=1
export PERL5LIB="${PWD}/lib${PERL5LIB:+:$PERL5LIB}"
if [ -x /tmp/perl-host/miniperl ] >/dev/null; then
  ln -sfn /tmp/perl-host/miniperl /tmp/perl-host/perl
  export PATH="/tmp/perl-host:${PATH}"
fi
make -j$(nproc)
make install DESTDIR="${OUT}"
_perl_so=$(find "${OUT}/usr/lib" -name libperl.so | head -n1)
if [ -n "${_perl_so:-}" ]; then
  ln -sfn $(realpath --relative-to="${OUT}/usr/lib" "${_perl_so}") "${OUT}/usr/lib/libperl.so"
fi
