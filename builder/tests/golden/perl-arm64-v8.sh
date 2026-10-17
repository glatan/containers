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
export CFLAGS="${CFLAGS} -Wno-incompatible-pointer-types -Wno-int-conversion"
LIBRARY_PATH= CFLAGS= LDFLAGS=-fuse-ld=mold "${HOST_TRIPLE}-gcc" -O2 -pipe -o /tmp/generate_uudmap generate_uudmap.c
_host=/tmp/perl-host
rm -rf "${_host}"
mkdir -p "${_host}"
(
  cd "${_host}"
  export CC="${HOST_TRIPLE}-gcc"
  export CXX="${HOST_TRIPLE}-g++"
  export LDFLAGS="-fuse-ld=mold"
  unset CFLAGS
  unset CXXFLAGS
  unset CPPFLAGS
  unset CROSS
  unset LIBRARY_PATH
  unset COMPILER_PATH
  export LD_LIBRARY_PATH="/dep/libxcrypt-host/usr/lib:/dep/libxcrypt-host/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  export PATH="/tmp/mold-ld-cc:/dep/mold/usr/bin:/dep/${HOST_TRIPLE}-gcc/usr/bin:/dep/${HOST_TRIPLE}-binutils/usr/bin:/usr/bin:/bin"
  "${SOURCE}"/Configure -des -Dusedevel -Dmksymlinks -Dcc="${CC}" -Dld="${CC}" -Dprefix=/usr -Uusethreads -Dlibs='-lpthread -ldl -lm -lutil -lc'
  make -j$(nproc) miniperl
)
awk '
    /Skipping the try tests in the rest of Configure as no targethost was defined when cross-compiling/ {
      print
      print "         run=:"
      print "         to=:"
      print "         from=:"
      print "         ;;"
      skip=1
      next
    }
    skip {
      if ($0 ~ /^       ;;/) skip=0
      next
    }
    { print }
  ' Configure > Configure.cross
mv Configure.cross Configure
sed -i 's/&& \.\/try; then/\&\& true; then/' Configure
awk '
    /if \$sh -c "\$run \.\/try / {
      print " if true; then"
      getline
      print " xxx=Ok"
      next
    }
    { print }
  ' Configure > Configure.coherency
mv Configure.coherency Configure
chmod +x Configure
if [ -d "${_sysroot}/lib" ]; then
  export LIBRARY_PATH="${_sysroot}/lib:${_sysroot}/lib64${LIBRARY_PATH:+:$LIBRARY_PATH}"
fi
export LD_LIBRARY_PATH="/dep/libxcrypt/usr/lib:/dep/libxcrypt/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
./Configure -des -Dprefix=/usr -Dvendorprefix=/usr -Duse64bitall -Dusethreads -Duseshrplib -Dcc="${CC}" -Dld="${CC}" -Accflags="${CFLAGS}" -Aldflags="${LDFLAGS}" -Dusecrosscompile -Dhostperl=/tmp/perl-host/miniperl -Dhostgenerate=/tmp/generate_uudmap -Dcharsize=1 -Dshortsize=2 -Dintsize=4 -Dlongsize=8 -Dlonglongsize=8 -Dptrsize=8 -Ddoublesize=8 -Dnvsize=8 -Duvsize=8 -Divsize=8 -Dcharbits=8 -Dalignbytes=8 -Dssizetype=ssize_t -Dsizetype=size_t -Dlseektype=off_t -Dlseeksize=8 -Dtimetype=time_t -Dfpostype=fpos_t -Dfpossize=16 -Dlongdblsize=16 -Ddoublekind=3 -Duseithreads -Dusemultiplicity -Duseperlio -Uebcdic -Dd_PRId64=define -Dd_PRIi64=define -Divdformat='"ld"' -Duvuformat='"lu"' -Duvxformat='"lx"' -DsPRId64='"ld"' -DsPRIi64='"li"' -DsPRIo64='"lo"' -DsPRIu64='"lu"' -DsPRIx64='"lx"' -DsPRIXU64='"lX"' -Dnveformat='"e"' -Dnvgformat='"g"' -DnvEUformat='"E"' -DnvGUformat='"G"' -Dd_PRIfldbl=define -Dd_PRIgldbl=define -Dd_PRIeldbl=define -Dd_PRIFUldbl=define -Dd_PRIGUldbl=define -Dd_PRIEUldbl=define -Dd_SCNfldbl=define -Dd_ldbl_dig=define -DsPRIfldbl='"Lf"' -DsPRIgldbl='"Lg"' -DsPRIeldbl='"Le"' -DsPRIFUldbl='"LF"' -DsPRIGUldbl='"LG"' -DsPRIEUldbl='"LE"' -DsSCNfldbl='"Lf"' -Dtargetarch="${TRIPLE}" -Dsysroot="${_sysroot}" -Dusrinc="${_sysroot}/include" -Dincpth="${_sysroot}/include" -Dlibpth="${_sysroot}/lib /dep/libxcrypt/usr/lib /dep/libxcrypt/usr/lib64 ${_sysroot}/lib64" -Darchname="${TRIPLE}" -Dbyteorder="12345678"
if [ -f config.sh ]; then
  if grep -q ^lseeksize= config.sh >/dev/null; then
    sed -i "s|^lseeksize=.*|lseeksize='8'|" config.sh
  else
    echo "lseeksize='8'" >> config.sh
  fi
  if grep -q ^ssizetype= config.sh >/dev/null; then
    sed -i "s|^ssizetype=.*|ssizetype='ssize_t'|" config.sh
  else
    echo "ssizetype='ssize_t'" >> config.sh
  fi
  if grep -q ^sizetype= config.sh >/dev/null; then
    sed -i "s|^sizetype=.*|sizetype='size_t'|" config.sh
  else
    echo "sizetype='size_t'" >> config.sh
  fi
  if grep -q ^sizesize= config.sh >/dev/null; then
    sed -i "s|^sizesize=.*|sizesize='8'|" config.sh
  else
    echo "sizesize='8'" >> config.sh
  fi
  if grep -q ^ptrsize= config.sh >/dev/null; then
    sed -i "s|^ptrsize=.*|ptrsize='8'|" config.sh
  else
    echo "ptrsize='8'" >> config.sh
  fi
  if grep -q ^ivsize= config.sh >/dev/null; then
    sed -i "s|^ivsize=.*|ivsize='8'|" config.sh
  else
    echo "ivsize='8'" >> config.sh
  fi
  if grep -q ^uvsize= config.sh >/dev/null; then
    sed -i "s|^uvsize=.*|uvsize='8'|" config.sh
  else
    echo "uvsize='8'" >> config.sh
  fi
  if grep -q ^nvsize= config.sh >/dev/null; then
    sed -i "s|^nvsize=.*|nvsize='8'|" config.sh
  else
    echo "nvsize='8'" >> config.sh
  fi
  if grep -q ^doublesize= config.sh >/dev/null; then
    sed -i "s|^doublesize=.*|doublesize='8'|" config.sh
  else
    echo "doublesize='8'" >> config.sh
  fi
  if grep -q ^longdblsize= config.sh >/dev/null; then
    sed -i "s|^longdblsize=.*|longdblsize='16'|" config.sh
  else
    echo "longdblsize='16'" >> config.sh
  fi
  if grep -q ^doublekind= config.sh >/dev/null; then
    sed -i "s|^doublekind=.*|doublekind='3'|" config.sh
  else
    echo "doublekind='3'" >> config.sh
  fi
  if grep -q ^longdblkind= config.sh >/dev/null; then
    sed -i "s|^longdblkind=.*|longdblkind='1'|" config.sh
  else
    echo "longdblkind='1'" >> config.sh
  fi
  if grep -q ^timetype= config.sh >/dev/null; then
    sed -i "s|^timetype=.*|timetype='time_t'|" config.sh
  else
    echo "timetype='time_t'" >> config.sh
  fi
  if grep -q ^ivtype= config.sh >/dev/null; then
    sed -i "s|^ivtype=.*|ivtype='long'|" config.sh
  else
    echo "ivtype='long'" >> config.sh
  fi
  if grep -q ^uvtype= config.sh >/dev/null; then
    sed -i "s|^uvtype=.*|uvtype='unsigned long'|" config.sh
  else
    echo "uvtype='unsigned long'" >> config.sh
  fi
  if grep -q ^nvtype= config.sh >/dev/null; then
    sed -i "s|^nvtype=.*|nvtype='double'|" config.sh
  else
    echo "nvtype='double'" >> config.sh
  fi
  if grep -q ^d_fork= config.sh >/dev/null; then
    sed -i "s|^d_fork=.*|d_fork='define'|" config.sh
  else
    echo "d_fork='define'" >> config.sh
  fi
  if grep -q ^d_pipe= config.sh >/dev/null; then
    sed -i "s|^d_pipe=.*|d_pipe='define'|" config.sh
  else
    echo "d_pipe='define'" >> config.sh
  fi
  if grep -q ^d_waitpid= config.sh >/dev/null; then
    sed -i "s|^d_waitpid=.*|d_waitpid='define'|" config.sh
  else
    echo "d_waitpid='define'" >> config.sh
  fi
  if grep -q ^d_cplusplus= config.sh >/dev/null; then
    sed -i "s|^d_cplusplus=.*|d_cplusplus='undef'|" config.sh
  else
    echo "d_cplusplus='undef'" >> config.sh
  fi
  if grep -q ^extern_C= config.sh >/dev/null; then
    sed -i "s|^extern_C=.*|extern_C='extern'|" config.sh
  else
    echo "extern_C='extern'" >> config.sh
  fi
  if [ -f ${SOURCE}/Porting/config.sh ]; then
    awk -F= '/^d_[A-Za-z0-9_]+=.define/ && $0 !~ /quadmath/ { print $1 }' "${SOURCE}"/Porting/config.sh > /tmp/d_define.list
    for _k in $(cat /tmp/d_define.list); do
      if grep -q ^${_k}= config.sh >/dev/null; then
        sed -i "s|^${_k}=.*|${_k}='define'|" config.sh
      else
        echo "${_k}='define'" >> config.sh
      fi
    done
  fi
  ./Configure -S
  sed -i 's/extern "C"/extern/g' Makefile config.sh 2> /dev/null || true
fi
if [ -f config.h ]; then
  if grep -qE "^#define LSEEKSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define LSEEKSIZE[[:space:]].*|#define LSEEKSIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "LSEEKSIZE" "8" >> config.h
  fi
  if grep -qE "^#define Size_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Size_t[[:space:]].*|#define Size_t size_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Size_t" "size_t" >> config.h
  fi
  if grep -qE "^#define SSize_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define SSize_t[[:space:]].*|#define SSize_t ssize_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "SSize_t" "ssize_t" >> config.h
  fi
  if grep -qE "^#define Size_t_size([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Size_t_size[[:space:]].*|#define Size_t_size 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Size_t_size" "8" >> config.h
  fi
  if grep -qE "^#define PTRSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define PTRSIZE[[:space:]].*|#define PTRSIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "PTRSIZE" "8" >> config.h
  fi
  if grep -qE "^#define LONGSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define LONGSIZE[[:space:]].*|#define LONGSIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "LONGSIZE" "8" >> config.h
  fi
  if grep -qE "^#define IVSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define IVSIZE[[:space:]].*|#define IVSIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "IVSIZE" "8" >> config.h
  fi
  if grep -qE "^#define UVSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define UVSIZE[[:space:]].*|#define UVSIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "UVSIZE" "8" >> config.h
  fi
  if grep -qE "^#define NVSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define NVSIZE[[:space:]].*|#define NVSIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "NVSIZE" "8" >> config.h
  fi
  if grep -qE "^#define I32SIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define I32SIZE[[:space:]].*|#define I32SIZE 4\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "I32SIZE" "4" >> config.h
  fi
  if grep -qE "^#define U32SIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define U32SIZE[[:space:]].*|#define U32SIZE 4\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "U32SIZE" "4" >> config.h
  fi
  if grep -qE "^#define INTSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define INTSIZE[[:space:]].*|#define INTSIZE 4\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "INTSIZE" "4" >> config.h
  fi
  if grep -qE "^#define SHORTSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define SHORTSIZE[[:space:]].*|#define SHORTSIZE 2\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "SHORTSIZE" "2" >> config.h
  fi
  if grep -qE "^#define CHARBITS([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define CHARBITS[[:space:]].*|#define CHARBITS 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "CHARBITS" "8" >> config.h
  fi
  if grep -qE "^#define DOUBLESIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define DOUBLESIZE[[:space:]].*|#define DOUBLESIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "DOUBLESIZE" "8" >> config.h
  fi
  if grep -qE "^#define LONG_DOUBLESIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define LONG_DOUBLESIZE[[:space:]].*|#define LONG_DOUBLESIZE 16\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "LONG_DOUBLESIZE" "16" >> config.h
  fi
  if grep -qE "^#define DOUBLEKIND([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define DOUBLEKIND[[:space:]].*|#define DOUBLEKIND 3\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "DOUBLEKIND" "3" >> config.h
  fi
  if grep -qE "^#define LONG_DOUBLEKIND([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define LONG_DOUBLEKIND[[:space:]].*|#define LONG_DOUBLEKIND 1\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "LONG_DOUBLEKIND" "1" >> config.h
  fi
  if grep -qE "^#define BYTEORDER([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define BYTEORDER[[:space:]].*|#define BYTEORDER 0x12345678\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "BYTEORDER" "0x12345678" >> config.h
  fi
  if grep -qE "^#define LONGLONGSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define LONGLONGSIZE[[:space:]].*|#define LONGLONGSIZE 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "LONGLONGSIZE" "8" >> config.h
  fi
  if grep -qE "^#define FPOSIZE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define FPOSIZE[[:space:]].*|#define FPOSIZE 16\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "FPOSIZE" "16" >> config.h
  fi
  if grep -qE "^#define Time_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Time_t[[:space:]].*|#define Time_t time_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Time_t" "time_t" >> config.h
  fi
  if grep -qE "^#define Off_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Off_t[[:space:]].*|#define Off_t off_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Off_t" "off_t" >> config.h
  fi
  if grep -qE "^#define Off_t_size([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Off_t_size[[:space:]].*|#define Off_t_size 8\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Off_t_size" "8" >> config.h
  fi
  if grep -qE "^#define Mode_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Mode_t[[:space:]].*|#define Mode_t mode_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Mode_t" "mode_t" >> config.h
  fi
  if grep -qE "^#define Pid_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Pid_t[[:space:]].*|#define Pid_t pid_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Pid_t" "pid_t" >> config.h
  fi
  if grep -qE "^#define Gid_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Gid_t[[:space:]].*|#define Gid_t gid_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Gid_t" "gid_t" >> config.h
  fi
  if grep -qE "^#define Uid_t([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define Uid_t[[:space:]].*|#define Uid_t uid_t\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "Uid_t" "uid_t" >> config.h
  fi
  if grep -qE "^#define IVTYPE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define IVTYPE[[:space:]].*|#define IVTYPE long\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "IVTYPE" "long" >> config.h
  fi
  if grep -qE "^#define UVTYPE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define UVTYPE[[:space:]].*|#define UVTYPE unsigned long\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "UVTYPE" "unsigned long" >> config.h
  fi
  if grep -qE "^#define NVTYPE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define NVTYPE[[:space:]].*|#define NVTYPE double\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "NVTYPE" "double" >> config.h
  fi
  if grep -qE "^#define I32TYPE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define I32TYPE[[:space:]].*|#define I32TYPE int\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "I32TYPE" "int" >> config.h
  fi
  if grep -qE "^#define U32TYPE([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define U32TYPE[[:space:]].*|#define U32TYPE unsigned int\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "U32TYPE" "unsigned int" >> config.h
  fi
  if grep -qE "^#define PROCSELFEXE_PATH([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define PROCSELFEXE_PATH[[:space:]].*|#define PROCSELFEXE_PATH \"/proc/self/exe\"\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "PROCSELFEXE_PATH" '"/proc/self/exe"' >> config.h
  fi
  sed -i 's/^#define \([A-Za-z0-9_]*\)[[:space:]][[:space:]]*undef.*$/\/\*#define \1 undef \/* *\//' config.h
  if grep -qE "^#define DOUBLEINFBYTES([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define DOUBLEINFBYTES[[:space:]].*|#define DOUBLEINFBYTES 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x7f\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "DOUBLEINFBYTES" "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x7f" >> config.h
  fi
  if grep -qE "^#define DOUBLENANBYTES([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define DOUBLENANBYTES[[:space:]].*|#define DOUBLENANBYTES 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0xff\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "DOUBLENANBYTES" "0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0xff" >> config.h
  fi
  if grep -qE "^#define DOUBLEMANTBITS([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define DOUBLEMANTBITS[[:space:]].*|#define DOUBLEMANTBITS 52\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "DOUBLEMANTBITS" "52" >> config.h
  fi
  if grep -qE "^#define NVMANTBITS([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define NVMANTBITS[[:space:]].*|#define NVMANTBITS 52\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "NVMANTBITS" "52" >> config.h
  fi
  _host_cfg=/dep/perl/usr/lib/perl5/5.44.0/x86_64-linux-thread-multi/CORE/config.h
  if [ -f ${_host_cfg} ]; then
    for _has in $(awk '/^#define HAS_/ { print $2 }' ${_host_cfg}); do
      sed -i "/^#undef ${_has}/d" config.h
      sed -i "s|^/\*#define ${_has}|#define ${_has}|" config.h
      grep -q "^#define ${_has}" config.h || printf '#define %s\t\t/**/\n' "${_has}" >> config.h
    done
  fi
fi
export PERL_USE_UNSAFE_INC=1
export PERL5LIB="${PWD}/lib${PERL5LIB:+:$PERL5LIB}"
if [ -x /tmp/perl-host/miniperl ] >/dev/null; then
  ln -sfn /tmp/perl-host/miniperl /tmp/perl-host/perl
  export PATH="/tmp/perl-host:${PATH}"
fi
make -j$(nproc) PERL=/tmp/perl-host/miniperl
make install DESTDIR="${OUT}"
_perl_so=$(find "${OUT}/usr/lib" -name libperl.so | head -n1)
if [ -n "${_perl_so:-}" ]; then
  ln -sfn $(realpath --relative-to="${OUT}/usr/lib" "${_perl_so}") "${OUT}/usr/lib/libperl.so"
fi
