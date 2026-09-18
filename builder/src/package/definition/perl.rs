use crate::package::recipe::{AUTOTOOLS_ENV, BuildContext, Recipe, SYSROOT_ENV};
use crate::package::{Package, Source};
use crate::target::Target;

pub struct Perl;

impl Package for Perl {
    fn name(&self) -> &'static str {
        "perl"
    }
    fn version(&self) -> &'static str {
        "5.42.3"
    }
    fn license(&self) -> &'static str {
        "Artistic-1.0-Perl OR GPL-1.0-or-later"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    /// クロス perl ビルドは HOST perl（miniperl）を使うため自分自身をマウントする。
    fn host_tools(&self, target: Target) -> Vec<&'static str> {
        if target == Target::BUILD_ENV {
            vec![]
        } else {
            vec!["perl"]
        }
    }

    fn source(&self) -> Source {
        Source {
            url: "https://www.cpan.org/src/5.0/perl-5.42.3.tar.xz",
            sha3_512: "c827af1973438b9173756f5c0b44e668d0443b2bba01b1edf748947c99a5660ecdf5119e0aeada306e47c848bdc2debe7226f45c38d7158b3cf280b1804021a8",
            blake3: "986547a8f7de7ad55e2d9877ec29461601b12e4642cf5806194f07b805fea547570e9bcbb2f52df883220190c1a891d1eba37a6f2d82f595b833e4223da68b88",
            subdir: "perl-5.42.3",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[crate::package::Dependency {
                name: "libxcrypt",
                version: "4.5.2",
            }],
        )
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }
    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }
    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        let mut script = String::from(AUTOTOOLS_ENV);
        script.push_str(SYSROOT_ENV);
        script.push_str(
            r#"if [ -d /dep/libxcrypt/usr/include ]; then
  export CFLAGS="$CFLAGS -I/dep/libxcrypt/usr/include"
  export CPPFLAGS="${CPPFLAGS:-} -I/dep/libxcrypt/usr/include"
fi
if [ -d /dep/libxcrypt/usr/lib64 ]; then
  export LDFLAGS="$LDFLAGS -L/dep/libxcrypt/usr/lib64"
elif [ -d /dep/libxcrypt/usr/lib ]; then
  export LDFLAGS="$LDFLAGS -L/dep/libxcrypt/usr/lib"
fi
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
_sysroot=/dep/${TRIPLE}-glibc/usr/${TRIPLE}
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  export CFLAGS="$CFLAGS -Wno-incompatible-pointer-types -Wno-int-conversion"
  case "$TRIPLE" in
    s390x-*) _byteorder=87654321 ;;
    *) _byteorder=12345678 ;;
  esac
  LIBRARY_PATH= CFLAGS= LDFLAGS=-fuse-ld=mold "${HOST_TRIPLE}-gcc" -O2 -pipe -o /tmp/generate_uudmap generate_uudmap.c
  _host=/tmp/perl-host
  rm -rf "$_host"
  mkdir -p "$_host"
  (
    cd "$_host"
    export CC="${HOST_TRIPLE}-gcc"
    export CXX="${HOST_TRIPLE}-g++"
    export LDFLAGS="-fuse-ld=mold"
    unset CFLAGS CXXFLAGS CPPFLAGS CROSS LIBRARY_PATH COMPILER_PATH
    export LD_LIBRARY_PATH="/dep/libxcrypt-host/usr/lib:/dep/libxcrypt-host/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    export PATH="/tmp/mold-ld-cc:/dep/mold/usr/bin:/dep/${HOST_TRIPLE}-gcc/usr/bin:/dep/${HOST_TRIPLE}-binutils/usr/bin:/usr/bin:/bin"
    "$SOURCE"/Configure -des -Dusedevel -Dmksymlinks -Dcc="$CC" -Dld="$CC" -Dprefix=/usr -Uusethreads -Dlibs='-lpthread -ldl -lm -lutil -lc'
    make -j"$(nproc)" miniperl
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
  ' Configure > Configure.cross && mv Configure.cross Configure
  sed -i 's/&& \.\/try; then/\&\& true; then/' Configure
  awk '
    /if \$sh -c "\$run \.\/try / {
      print " if true; then"
      getline
      print " xxx=Ok"
      next
    }
    { print }
  ' Configure > Configure.coherency && mv Configure.coherency Configure
  chmod +x Configure
fi
if [ -d "$_sysroot/lib" ]; then
  export LIBRARY_PATH="$_sysroot/lib:${_sysroot}/lib64${LIBRARY_PATH:+:$LIBRARY_PATH}"
fi
export LD_LIBRARY_PATH="/dep/libxcrypt/usr/lib:/dep/libxcrypt/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
configure=(
  ./Configure -des
  -Dprefix=/usr
  -Dvendorprefix=/usr
  -Duse64bitall
  -Dusethreads
  -Duseshrplib
  -Dcc="$CC"
  -Dld="$CC"
  -Accflags="$CFLAGS"
  -Aldflags="$LDFLAGS"
)
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  configure+=(
    -Dusecrosscompile
    -Dtargetarch="$TRIPLE"
    -Dhostperl=/tmp/perl-host/miniperl
    -Dhostgenerate=/tmp/generate_uudmap
    -Dsysroot="$_sysroot"
    -Dusrinc="$_sysroot/include"
    -Dincpth="$_sysroot/include"
    -Dlibpth="$_sysroot/lib $_sysroot/lib64"
    -Darchname="$TRIPLE"
    -Dcharsize=1
    -Dshortsize=2
    -Dintsize=4
    -Dlongsize=8
    -Dlonglongsize=8
    -Dptrsize=8
    -Ddoublesize=8
    -Dnvsize=8
    -Duvsize=8
    -Divsize=8
    -Dcharbits=8
    -Dalignbytes=8
    -Dbyteorder="$_byteorder"
    -Dssizetype=ssize_t
    -Dsizetype=size_t
    -Dlseektype=off_t
    -Dlseeksize=8
    -Dtimetype=time_t
    -Dfpostype=fpos_t
    -Dfpossize=16
    -Dlongdblsize=16
    -Ddoublekind=3
    -Duseithreads
    -Dusemultiplicity
    -Duseperlio
    -Uebcdic
    -Dd_PRId64=define
    -Dd_PRIi64=define
    -Divdformat='"ld"'
    -Duvuformat='"lu"'
    -Duvxformat='"lx"'
    -DsPRId64='"ld"'
    -DsPRIi64='"li"'
    -DsPRIo64='"lo"'
    -DsPRIu64='"lu"'
    -DsPRIx64='"lx"'
    -DsPRIXU64='"lX"'
    -Dnveformat='"e"'
    -Dnvgformat='"g"'
    -DnvEUformat='"E"'
    -DnvGUformat='"G"'
  )
fi
"${configure[@]}"
if [ "$TRIPLE" != "$HOST_TRIPLE" ] && [ -f config.sh ]; then
  _dk=3
  _ldk=1
  _bo_s=12345678
  case "$TRIPLE" in
    s390x-*) _dk=4; _ldk=2; _bo_s=87654321 ;;
    x86_64-*) _ldk=3 ;;
    powerpc64le-*) _ldk=5 ;;
  esac
  _patch_cfg() {
    if grep -q "^$1=" config.sh; then
      sed -i "s|^$1=.*|$1='$2'|" config.sh
    else
      echo "$1='$2'" >> config.sh
    fi
  }
  _patch_cfg lseeksize 8
  _patch_cfg ssizetype ssize_t
  _patch_cfg sizetype size_t
  _patch_cfg sizesize 8
  _patch_cfg ptrsize 8
  _patch_cfg ivsize 8
  _patch_cfg uvsize 8
  _patch_cfg nvsize 8
  _patch_cfg doublesize 8
  _patch_cfg longdblsize 16
  _patch_cfg doublekind "$_dk"
  _patch_cfg longdblkind "$_ldk"
  _patch_cfg timetype time_t
  _patch_cfg ivtype long
  _patch_cfg uvtype 'unsigned long'
  _patch_cfg nvtype double
  _patch_cfg d_fork define
  _patch_cfg d_pipe define
  _patch_cfg d_waitpid define
  _patch_cfg d_cplusplus undef
  _patch_cfg extern_C extern
  if [ -f "$SOURCE"/Porting/config.sh ]; then
    awk -F= '/^d_[A-Za-z0-9_]+=.define/ && $0 !~ /quadmath/ { print $1 }' "$SOURCE"/Porting/config.sh > /tmp/d_define.list
    while read -r _k; do
      [ -n "$_k" ] || continue
      _patch_cfg "$_k" define
    done < /tmp/d_define.list
  fi
  ./Configure -S
  sed -i 's/extern "C"/extern/g' Makefile config.sh 2>/dev/null || true
fi
if [ "$TRIPLE" != "$HOST_TRIPLE" ] && [ -f config.h ]; then
  _dk=3
  _ldk=1
  _bo_h=0x12345678
  case "$TRIPLE" in
    s390x-*) _dk=4; _ldk=2; _bo_h=0x87654321 ;;
    x86_64-*) _ldk=3 ;;
    powerpc64le-*) _ldk=5 ;;
  esac
  _inject_h() {
    if grep -qE "^#define $1([[:space:]]|$)" config.h; then
      sed -i "s|^#define $1[[:space:]].*|#define $1 $2\t\t/**/|" config.h
    else
      printf '#define %s %s\t\t/**/\n' "$1" "$2" >> config.h
    fi
  }
  _inject_h LSEEKSIZE 8
  _inject_h Size_t size_t
  _inject_h SSize_t ssize_t
  _inject_h Size_t_size 8
  _inject_h PTRSIZE 8
  _inject_h LONGSIZE 8
  _inject_h IVSIZE 8
  _inject_h UVSIZE 8
  _inject_h NVSIZE 8
  _inject_h I32SIZE 4
  _inject_h U32SIZE 4
  _inject_h INTSIZE 4
  _inject_h SHORTSIZE 2
  _inject_h CHARBITS 8
  _inject_h DOUBLESIZE 8
  _inject_h LONG_DOUBLESIZE 16
  _inject_h DOUBLEKIND "$_dk"
  _inject_h LONG_DOUBLEKIND "$_ldk"
  _inject_h BYTEORDER "$_bo_h"
  _inject_h LONGLONGSIZE 8
  _inject_h FPOSIZE 16
  _inject_h Time_t time_t
  _inject_h Off_t off_t
  _inject_h Off_t_size 8
  _inject_h Mode_t mode_t
  _inject_h Pid_t pid_t
  _inject_h Gid_t gid_t
  _inject_h Uid_t uid_t
  _inject_h IVTYPE long
  _inject_h UVTYPE 'unsigned long'
  _inject_h NVTYPE double
  _inject_h I32TYPE int
  _inject_h U32TYPE 'unsigned int'
  _inject_h PROCSELFEXE_PATH '"/proc/self/exe"'
  sed -i 's/^#define \([A-Za-z0-9_]*\)[[:space:]][[:space:]]*undef.*$/\/\*#define \1 undef \/* *\//' config.h
  _inject_h DOUBLEINFBYTES '0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf0, 0x7f'
  _inject_h DOUBLENANBYTES '0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf8, 0xff'
  _inject_h DOUBLEMANTBITS 52
  _inject_h NVMANTBITS 52
  _host_cfg=/dep/perl/usr/lib/perl5/5.42.3/x86_64-linux-thread-multi/CORE/config.h
  if [ -f "$_host_cfg" ]; then
    awk '/^#define HAS_/ { print $2 }' "$_host_cfg" | while read -r _has; do
      [ -n "$_has" ] || continue
      sed -i "/^#undef ${_has}/d" config.h
      sed -i "s|^/\\*#define ${_has}|#define ${_has}|" config.h
      grep -q "^#define ${_has}" config.h || printf '#define %s\t\t/**/\n' "$_has" >> config.h
    done
  fi
fi
export PERL_USE_UNSAFE_INC=1
export PERL5LIB="$PWD/lib${PERL5LIB:+:$PERL5LIB}"
if [ -x /tmp/perl-host/miniperl ]; then
  ln -sfn /tmp/perl-host/miniperl /tmp/perl-host/perl
  export PATH="/tmp/perl-host:$PATH"
fi
make -j"$(nproc)"
make install DESTDIR="$OUT"
_perl_so=$(find "$OUT/usr/lib" -name libperl.so | head -n1)
if [ -n "$_perl_so" ]; then
  ln -sfn "$(realpath --relative-to="$OUT/usr/lib" "$_perl_so")" "$OUT/usr/lib/libperl.so"
fi
"#,
        );
        Recipe::Shell(script.into())
    }
}
