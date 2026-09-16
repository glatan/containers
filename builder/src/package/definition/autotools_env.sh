set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
sanitize_flags() {
  printf '%s' "$1" | sed -E 's/-flto(=full|=auto|=thin)?//g; s/-Werror=[^ ]+//g'
}
export CFLAGS="$(sanitize_flags "${CFLAGS:-}") -std=gnu17"
export CXXFLAGS="$(sanitize_flags "${CXXFLAGS:-}")"
export LDFLAGS="$(sanitize_flags "${LDFLAGS:-}")"
# Autoconf 2.72 は AC_RUN_IFELSE の cross 分岐が空だとエラーにする。
# gnulib は推測値を先に入れているが、実行検査の前に落ちる。
if [ "${TRIPLE:-}" != "${HOST_TRIPLE:-}" ]; then
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
fi
autotools_configure() {
  if ! "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"; then
    if [ -f config.log ]; then
      echo "===== config.log (last 80 lines) ====="
      tail -n 80 config.log
    fi
    return 1
  fi
}
