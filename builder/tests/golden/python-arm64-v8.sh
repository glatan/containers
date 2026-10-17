set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export BUILD_CC="$CC_FOR_BUILD"
export gl_cv_func_strcasecmp_works=yes
export gl_cv_func_strncasecmp_works=yes
export gl_cv_func_memchr_works=yes
export gl_cv_func_working_mktime=yes
export gl_cv_func_working_utimes=yes
export gl_cv_func_getcwd_null=yes
export gl_cv_func_getcwd_path_max=yes
export gl_cv_func_getcwd_abort_bug=no
export gl_cv_func_chown_follows_symlink=yes
export gl_cv_func_fchownat_nofollow_works=yes
export gl_cv_func_lstat_dereferences_slashed_symlink=yes
export gl_cv_func_stat_file_slash=yes
export gl_cv_func_strerror_r_works=yes
export gl_cv_func_strerror_0_works=yes
export gl_cv_func_strtod_works=yes
export gl_cv_func_nanosleep=yes
export gl_cv_func_printf_positions=yes
export gl_cv_func_printf_infinite=yes
export gl_cv_func_printf_enomem=yes
export gl_cv_func_malloc_0_nonnull=yes
export gl_cv_func_malloc_posix=yes
export ac_cv_func_malloc_0_nonnull=yes
export ac_cv_func_realloc_0_nonnull=yes
export ac_cv_func_memcmp_working=yes
sanitize_flags() {
  printf '%s' "$1" | sed -E 's/-flto(=[^[:space:]]*)?//g; s/-Werror=(odr|lto-type-mismatch|strict-aliasing)//g; s/-f(no-)?hardened//g'
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
export CFLAGS="${CFLAGS} -DOPENSSL_NO_SSL3 -DOPENSSL_NO_SSL3_METHOD -DOPENSSL_NO_TLS1_METHOD -DOPENSSL_NO_TLS1_1_METHOD -DOPENSSL_NO_TLS1_2_METHOD"
export ac_cv_buggy_getaddrinfo=no
export ac_cv_file__dev_ptmx=yes
export ac_cv_file__dev_ptc=no
autotools_configure --prefix=/usr --enable-shared --without-ensurepip --with-build-python=/dep/python/usr/bin/python3
sed -i "s|check_extension_modules.py|check_extension_modules.py --cross-compiling|g" Makefile
make -j"$(nproc)" RUNSHARED= CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
make install RUNSHARED= CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
ln -sf python3 "$OUT"/usr/bin/python
