<!-- GENERATED -->
# perl

| 項目 | 値 |
|-|-|
| パッケージ名 | `perl` |
| バージョン | 5.44.0 |
| ライセンス | Artistic-1.0-Perl OR GPL-1.0-or-later |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| host 実行 | 依存として常に BUILD_ENV 側で実行 |
| 定義 | [`builder/src/package/definition/perl.rs`](../../../../builder/src/package/definition/perl.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://www.cpan.org/src/5.0/perl-5.44.0.tar.xz |
| sha3-512 | `0a5b1ffa2b8ba86065e9206db8a60c42ee8e7e7b5c27cc610e2fe4d8c3675fd7f6f40444bca99a2a5eb9c3c2c61365f1caa1aa2c7702b8c46d9d007a1678e9c9` |
| blake3 | `2b1be108d901a7b5832e17292631b975f98d04d7c615dcde9d7f61c32c79063c3456d1eb921ea62c472be9e6748ed1ec2cc718caeb8a24844acb44d5c6cb4847` |
| 展開ディレクトリ | `perl-5.44.0` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [libxcrypt](./libxcrypt.md) | 4.5.2 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.56 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [mold](./mold.md) | 3.0.0 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::Autotools`（configure のみ `-flto*`・LTO 系 `-Werror=*`・`-f{,no-}hardened` を `sanitize_flags` で除去。`make`/`make install` に `CFLAGS`/`CXXFLAGS`/`LDFLAGS` の変数上書きを自動付与） |
| `-std` 付与 | `-std=gnu17`（CFLAGS 末尾） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | なし |

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

### `"${SOURCE}"/Configure`

- 対象: arm64, ppc64le, riscv64, s390x

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-des` |  | arm64, ppc64le, riscv64, s390x |
| `-Dusedevel` |  | arm64, ppc64le, riscv64, s390x |
| `-Dmksymlinks` |  | arm64, ppc64le, riscv64, s390x |
| `-Dcc` | `"${CC}"` | arm64, ppc64le, riscv64, s390x |
| `-Dld` | `"${CC}"` | arm64, ppc64le, riscv64, s390x |
| `-Dprefix` | `/usr` | arm64, ppc64le, riscv64, s390x |
| `-Uusethreads` |  | arm64, ppc64le, riscv64, s390x |
| `-Dlibs` | `'-lpthread -ldl -lm -lutil -lc'` | arm64, ppc64le, riscv64, s390x |

### `./Configure`（1回目の呼出）

- 対象: arm64, ppc64le, riscv64, s390x

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-des` |  | arm64, ppc64le, riscv64, s390x |
| `-Dprefix` | `/usr` | arm64, ppc64le, riscv64, s390x |
| `-Dvendorprefix` | `/usr` | arm64, ppc64le, riscv64, s390x |
| `-Duse64bitall` |  | arm64, ppc64le, riscv64, s390x |
| `-Dusethreads` |  | arm64, ppc64le, riscv64, s390x |
| `-Duseshrplib` |  | arm64, ppc64le, riscv64, s390x |
| `-Dcc` | `"${CC}"` | arm64, ppc64le, riscv64, s390x |
| `-Dld` | `"${CC}"` | arm64, ppc64le, riscv64, s390x |
| `-Accflags` | `"${CFLAGS}"` | arm64, ppc64le, riscv64, s390x |
| `-Aldflags` | `"${LDFLAGS}"` | arm64, ppc64le, riscv64, s390x |
| `-Dusecrosscompile` |  | arm64, ppc64le, riscv64, s390x |
| `-Dhostperl` | `/tmp/perl-host/miniperl` | arm64, ppc64le, riscv64, s390x |
| `-Dhostgenerate` | `/tmp/generate_uudmap` | arm64, ppc64le, riscv64, s390x |
| `-Dcharsize` | `1` | arm64, ppc64le, riscv64, s390x |
| `-Dshortsize` | `2` | arm64, ppc64le, riscv64, s390x |
| `-Dintsize` | `4` | arm64, ppc64le, riscv64, s390x |
| `-Dlongsize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Dlonglongsize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Dptrsize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Ddoublesize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Dnvsize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Duvsize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Divsize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Dcharbits` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Dalignbytes` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Dssizetype` | `ssize_t` | arm64, ppc64le, riscv64, s390x |
| `-Dsizetype` | `size_t` | arm64, ppc64le, riscv64, s390x |
| `-Dlseektype` | `off_t` | arm64, ppc64le, riscv64, s390x |
| `-Dlseeksize` | `8` | arm64, ppc64le, riscv64, s390x |
| `-Dtimetype` | `time_t` | arm64, ppc64le, riscv64, s390x |
| `-Dfpostype` | `fpos_t` | arm64, ppc64le, riscv64, s390x |
| `-Dfpossize` | `16` | arm64, ppc64le, riscv64, s390x |
| `-Dlongdblsize` | `16` | arm64, ppc64le, riscv64, s390x |
| `-Ddoublekind` | `3` | arm64, ppc64le, riscv64, s390x |
| `-Duseithreads` |  | arm64, ppc64le, riscv64, s390x |
| `-Dusemultiplicity` |  | arm64, ppc64le, riscv64, s390x |
| `-Duseperlio` |  | arm64, ppc64le, riscv64, s390x |
| `-Uebcdic` |  | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRId64` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRIi64` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Divdformat` | `'"ld"'` | arm64, ppc64le, riscv64, s390x |
| `-Duvuformat` | `'"lu"'` | arm64, ppc64le, riscv64, s390x |
| `-Duvxformat` | `'"lx"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRId64` | `'"ld"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIi64` | `'"li"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIo64` | `'"lo"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIu64` | `'"lu"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIx64` | `'"lx"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIXU64` | `'"lX"'` | arm64, ppc64le, riscv64, s390x |
| `-Dnveformat` | `'"e"'` | arm64, ppc64le, riscv64, s390x |
| `-Dnvgformat` | `'"g"'` | arm64, ppc64le, riscv64, s390x |
| `-DnvEUformat` | `'"E"'` | arm64, ppc64le, riscv64, s390x |
| `-DnvGUformat` | `'"G"'` | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRIfldbl` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRIgldbl` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRIeldbl` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRIFUldbl` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRIGUldbl` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_PRIEUldbl` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_SCNfldbl` | `define` | arm64, ppc64le, riscv64, s390x |
| `-Dd_ldbl_dig` | `define` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIfldbl` | `'"Lf"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIgldbl` | `'"Lg"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIeldbl` | `'"Le"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIFUldbl` | `'"LF"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIGUldbl` | `'"LG"'` | arm64, ppc64le, riscv64, s390x |
| `-DsPRIEUldbl` | `'"LE"'` | arm64, ppc64le, riscv64, s390x |
| `-DsSCNfldbl` | `'"Lf"'` | arm64, ppc64le, riscv64, s390x |
| `-Dtargetarch` | `"${TRIPLE}"` | arm64, ppc64le, riscv64, s390x |
| `-Dsysroot` | `"${_sysroot}"` | arm64, ppc64le, riscv64, s390x |
| `-Dusrinc` | `"${_sysroot}/include"` | arm64, ppc64le, riscv64, s390x |
| `-Dincpth` | `"${_sysroot}/include"` | arm64, ppc64le, riscv64, s390x |
| `-Dlibpth` | `"${_sysroot}/lib /dep/libxcrypt/usr/lib /dep/libxcrypt/usr/lib64 ${_sysroot}/lib64"` | arm64, ppc64le, riscv64, s390x |
| `-Darchname` | `"${TRIPLE}"` | arm64, ppc64le, riscv64, s390x |
| `-Dbyteorder` | `"12345678"` | arm64, ppc64le, riscv64 |
| `-Dbyteorder` | `"87654321"` | s390x |

### `./Configure`（2回目の呼出）

- 対象: arm64, ppc64le, riscv64, s390x
- 実行時条件: `[ -f config.sh ]`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-S` |  | arm64, ppc64le, riscv64, s390x |

### `./Configure`（3回目の呼出）

- 対象: amd64
- 実行時条件: `[ -d "${_sysroot}/include" ]`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-des` |  | amd64 |
| `-Dprefix` | `/usr` | amd64 |
| `-Dvendorprefix` | `/usr` | amd64 |
| `-Duse64bitall` |  | amd64 |
| `-Dusethreads` |  | amd64 |
| `-Duseshrplib` |  | amd64 |
| `-Dcc` | `"${CC}"` | amd64 |
| `-Dld` | `"${CC}"` | amd64 |
| `-Accflags` | `"${CFLAGS}"` | amd64 |
| `-Aldflags` | `"${LDFLAGS}"` | amd64 |
| `-Dsysroot` | `"${SYSROOT}"` | amd64 |
| `-Dusrinc` | `"${_sysroot}/include"` | amd64 |
| `-Dincpth` | `"${_sysroot}/include"` | amd64 |
| `-Dlibpth` | `"${_sysroot}/lib /dep/libxcrypt/usr/lib /dep/libxcrypt/usr/lib64 ${_sysroot}/lib64"` | amd64 |

### `./Configure`（4回目の呼出）

- 対象: amd64
- 実行時条件: `! [ -d "${_sysroot}/include" ]`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-des` |  | amd64 |
| `-Dprefix` | `/usr` | amd64 |
| `-Dvendorprefix` | `/usr` | amd64 |
| `-Duse64bitall` |  | amd64 |
| `-Dusethreads` |  | amd64 |
| `-Duseshrplib` |  | amd64 |
| `-Dcc` | `"${CC}"` | amd64 |
| `-Dld` | `"${CC}"` | amd64 |
| `-Accflags` | `"${CFLAGS}"` | amd64 |
| `-Aldflags` | `"${LDFLAGS}"` | amd64 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 4 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
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
  ./Configure -des -Dprefix=/usr -Dvendorprefix=/usr -Duse64bitall -Dusethreads -Duseshrplib -Dcc="${CC}" -Dld="${CC}" -Accflags="${CFLAGS}" -Aldflags="${LDFLAGS}" -Dsysroot="${SYSROOT}" -Dusrinc="${_sysroot}/include" -Dincpth="${_sysroot}/include" -Dlibpth="${_sysroot}/lib /dep/libxcrypt/usr/lib /dep/libxcrypt/usr/lib64 ${_sysroot}/lib64"
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
```

</details>

<details>
<summary>arm64, riscv64（クロス・Canadian）</summary>

```bash
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
```

</details>

<details>
<summary>ppc64le（クロス・Canadian）</summary>

```bash
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
    sed -i "s|^longdblkind=.*|longdblkind='5'|" config.sh
  else
    echo "longdblkind='5'" >> config.sh
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
    sed -i "s|^#define LONG_DOUBLEKIND[[:space:]].*|#define LONG_DOUBLEKIND 5\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "LONG_DOUBLEKIND" "5" >> config.h
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
```

</details>

<details>
<summary>s390x（クロス・Canadian）</summary>

```bash
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
./Configure -des -Dprefix=/usr -Dvendorprefix=/usr -Duse64bitall -Dusethreads -Duseshrplib -Dcc="${CC}" -Dld="${CC}" -Accflags="${CFLAGS}" -Aldflags="${LDFLAGS}" -Dusecrosscompile -Dhostperl=/tmp/perl-host/miniperl -Dhostgenerate=/tmp/generate_uudmap -Dcharsize=1 -Dshortsize=2 -Dintsize=4 -Dlongsize=8 -Dlonglongsize=8 -Dptrsize=8 -Ddoublesize=8 -Dnvsize=8 -Duvsize=8 -Divsize=8 -Dcharbits=8 -Dalignbytes=8 -Dssizetype=ssize_t -Dsizetype=size_t -Dlseektype=off_t -Dlseeksize=8 -Dtimetype=time_t -Dfpostype=fpos_t -Dfpossize=16 -Dlongdblsize=16 -Ddoublekind=3 -Duseithreads -Dusemultiplicity -Duseperlio -Uebcdic -Dd_PRId64=define -Dd_PRIi64=define -Divdformat='"ld"' -Duvuformat='"lu"' -Duvxformat='"lx"' -DsPRId64='"ld"' -DsPRIi64='"li"' -DsPRIo64='"lo"' -DsPRIu64='"lu"' -DsPRIx64='"lx"' -DsPRIXU64='"lX"' -Dnveformat='"e"' -Dnvgformat='"g"' -DnvEUformat='"E"' -DnvGUformat='"G"' -Dd_PRIfldbl=define -Dd_PRIgldbl=define -Dd_PRIeldbl=define -Dd_PRIFUldbl=define -Dd_PRIGUldbl=define -Dd_PRIEUldbl=define -Dd_SCNfldbl=define -Dd_ldbl_dig=define -DsPRIfldbl='"Lf"' -DsPRIgldbl='"Lg"' -DsPRIeldbl='"Le"' -DsPRIFUldbl='"LF"' -DsPRIGUldbl='"LG"' -DsPRIEUldbl='"LE"' -DsSCNfldbl='"Lf"' -Dtargetarch="${TRIPLE}" -Dsysroot="${_sysroot}" -Dusrinc="${_sysroot}/include" -Dincpth="${_sysroot}/include" -Dlibpth="${_sysroot}/lib /dep/libxcrypt/usr/lib /dep/libxcrypt/usr/lib64 ${_sysroot}/lib64" -Darchname="${TRIPLE}" -Dbyteorder="87654321"
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
    sed -i "s|^doublekind=.*|doublekind='4'|" config.sh
  else
    echo "doublekind='4'" >> config.sh
  fi
  if grep -q ^longdblkind= config.sh >/dev/null; then
    sed -i "s|^longdblkind=.*|longdblkind='2'|" config.sh
  else
    echo "longdblkind='2'" >> config.sh
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
    sed -i "s|^#define DOUBLEKIND[[:space:]].*|#define DOUBLEKIND 4\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "DOUBLEKIND" "4" >> config.h
  fi
  if grep -qE "^#define LONG_DOUBLEKIND([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define LONG_DOUBLEKIND[[:space:]].*|#define LONG_DOUBLEKIND 2\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "LONG_DOUBLEKIND" "2" >> config.h
  fi
  if grep -qE "^#define BYTEORDER([[:space:]]|$)" config.h >/dev/null; then
    sed -i "s|^#define BYTEORDER[[:space:]].*|#define BYTEORDER 0x87654321\t\t/**/|" config.h
  else
    printf '#define %s %s\t\t/**/\n' "BYTEORDER" "0x87654321" >> config.h
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
```

</details>

<!-- /GENERATED -->




## 補足

openssl の Configure が実行するのは HOST の perl です。Autotools ではなく独自の `Configure` スクリプトを使用します（上表の `ConfigureScript` 呼出）。

- `-Dcc` / `-Dld` に `$CC` を渡します。リンクは gcc ドライバと `-fuse-ld=mold` に任せ、`LD` に GNU ld は置きません。
- `-Accflags` / `-Aldflags` は Configure のコンパイラ検査が `CFLAGS` を読まないため明示します。
- クロス時は `-Dusecrosscompile` を付けます（`-Dtargethost` は SSH 先ホスト名なので付けません）。`-Dhostperl` には同じソースを HOST gcc で組んだ `miniperl`（`/tmp/perl-host/miniperl`）を渡します。インストール済み HOST perl はターゲットの `.so` を読み込みます。HOST の perl を `miniperl` として使うとき `.` が `@INC` に無いため `PERL_USE_UNSAFE_INC=1` を付けます。
- Configure の try 実行は行いません。対象 glibc を qemu-user で走らせると高い `-march` で SIGILL になります。C99 はコンパイル成否だけを見て、コンパイラ一貫性検査はコンパイル成功を `Ok` とみなします。型サイズ・アラインメント・printf 書式は `-Dcharbits=8` `-Dalignbytes=8` `-Divdformat='"ld"'` `-Uebcdic` などで明示します。5.42 は `-Dlseeksize` が `config.h` に落ちないため、Configure 後に `config.sh` を補って `./Configure -S` します。
- HOST perl の `libcrypt` は `/dep/libxcrypt-host` を参照します。ホスト依存でマウントされる HOST perl とクロス時の HOST miniperl 向けで、ターゲットの `/dep/libxcrypt` を `LD_LIBRARY_PATH` に載せません。
- LTO は `Configure` スクリプトのプローブ処理と競合するため適用しません。PGO は独自 `Configure` がフラグを管理し注入経路がないため適用しません（`Pgo::None`）。
