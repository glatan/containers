<!-- GENERATED -->
# bzip2

| 項目 | 値 |
|-|-|
| パッケージ名 | `bzip2` |
| バージョン | 1.0.8 |
| ライセンス | bzip2-1.0.6 |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Profiled`） |
| 定義 | [`builder/src/package/definition/bzip2.rs`](../../../../builder/src/package/definition/bzip2.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz |
| sha3-512 | `8192e0e7a92c083354d91e114f4e1a0571d56cc106f58a11c92b4028a1bb1ecad2e00892bb81cc1d2d2dd7371e35beebab2e92e2c83ef9ccb8a2490a97a94f52` |
| blake3 | `97af3f520629c65fe41292f77e6ca798fe594d7987bfb2aebe7c6fcdc7ab5ed25e9035d582b0a344afcd3ae378c3f3f999d66594bd1dd88cfcabf143cd70b8ef` |
| 展開ディレクトリ | `bzip2-1.0.8` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |

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
| ビルドディレクトリ | `$BUILD`（`$SOURCE` をコピーして in-source） |
| PGO テンプレート | `make`（instrumented make → workload → clean → use make → install。ネイティブのみ） |

`make install` には `DESTDIR="$OUT"` を自動付与します。

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 2 系統に分かれます。

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
sed -i -e 's/$(CC) -shared/$(CC) $(LDFLAGS) -shared/' -e 's/$(CFLAGS) -o bzip2-shared/$(CFLAGS) $(LDFLAGS) -o bzip2-shared/' Makefile-libbz2_so
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
sed -i -e 's/$(CC) -shared/$(CC) $(LDFLAGS) -shared/' -e 's/$(CFLAGS) -o bzip2-shared/$(CFLAGS) $(LDFLAGS) -o bzip2-shared/' Makefile-libbz2_so
make -f Makefile-libbz2_so CC="$CC" CFLAGS="$CFLAGS -fpic -fPIC -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" CXXFLAGS="${CXXFLAGS:-}"
make CC="$CC" AR="${AR:-${TRIPLE}-ar}" RANLIB="${RANLIB:-${TRIPLE}-ranlib}" CFLAGS="$CFLAGS -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" libbz2.a bzip2 bzip2recover CXXFLAGS="${CXXFLAGS:-}"
make install PREFIX="$OUT/usr" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
mkdir -p "${OUT}/usr/lib"
cp -a libbz2.so* "${OUT}/usr/lib/"
```

</details>

<details>
<summary>arm64, ppc64le, riscv64, s390x（クロス・Canadian）</summary>

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
sed -i -e 's/$(CC) -shared/$(CC) $(LDFLAGS) -shared/' -e 's/$(CFLAGS) -o bzip2-shared/$(CFLAGS) $(LDFLAGS) -o bzip2-shared/' Makefile-libbz2_so
make -f Makefile-libbz2_so CC="$CC" CFLAGS="$CFLAGS -fpic -fPIC -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" CXXFLAGS="${CXXFLAGS:-}"
make CC="$CC" AR="${AR:-${TRIPLE}-ar}" RANLIB="${RANLIB:-${TRIPLE}-ranlib}" CFLAGS="$CFLAGS -Wall -Winline -D_FILE_OFFSET_BITS=64" LDFLAGS="$LDFLAGS" libbz2.a bzip2 bzip2recover CXXFLAGS="${CXXFLAGS:-}"
make install PREFIX="$OUT/usr" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
mkdir -p "${OUT}/usr/lib"
cp -a libbz2.so* "${OUT}/usr/lib/"
```

</details>

<!-- /GENERATED -->




## 補足

Autotools の `configure` ではなく、ソースツリーを作業ディレクトリへコピーしたのち共有ライブラリ用 Makefile（`Makefile-libbz2_so`）を用いてビルドします。`make install` 時のインストール先は `PREFIX=$OUT/usr` です。

PGO はネイティブビルドのみ適用します（`Pgo` ポリシーはメタデータ表を参照）。同梱ソースを連結した複数 MB 入力を `bzip2` で `-1` / `-6` / `-9` の 3 レベルで圧縮・解凍し `bzip2recover` も実行します。パッケージ固有の `CFLAGS`（`-fpic -fPIC -D_FILE_OFFSET_BITS=64` 等）と make 引数は両フェーズで replay します。
