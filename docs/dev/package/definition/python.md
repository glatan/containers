<!-- GENERATED -->
# python

| 項目 | 値 |
|-|-|
| パッケージ名 | `python` |
| バージョン | 3.15.0 |
| ライセンス | Python-2.0 |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | 現行条件では適用対象なし（`Pgo::Custom`、レシピ内の個別手順） |
| host 実行 | 依存として常に BUILD_ENV 側で実行 |
| 定義 | [`builder/src/package/definition/python.rs`](../../../../builder/src/package/definition/python.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://www.python.org/ftp/python/3.15.0/Python-3.15.0.tar.xz |
| sha3-512 | `b9ec570f6e23d402f5b445a2dbda7c71611aa217c41c086fe2e2a4ace36e972e65c97d54f6387b8dd0857c4e8fdc35bd1fbc57f3de3db54b3dc49698f4641e5d` |
| blake3 | `1254fe6e327eaba1572549b1f8d1a1a4cf07d87733411542868cf9bebce6199572072a521e6f261f96f6b054ebac934a88237b955efc7548a8343c4e1d68a574` |
| 展開ディレクトリ | `Python-3.15.0` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [libffi](./libffi.md) | 3.8.0 | — |
| [zlib](./zlib.md) | 1.3.2 | — |
| [openssl](./openssl.md) | 4.0.3 | — |
| [expat](./expat.md) | 2.9.0 | — |
| [sqlite](./sqlite.md) | 3.54.0 | — |
| [ncurses](./ncurses.md) | 6.6 | — |
| [readline](./readline.md) | 8.3 | — |
| [xz](./xz.md) | 5.8.4 | — |
| [bzip2](./bzip2.md) | 1.0.8 | — |
| [zstd](./zstd.md) | 1.5.7 | — |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.56 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [pkgconf](./pkgconf.md) | 3.0.8 | — |
| [mold](./mold.md) | 3.0.0 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::Autotools`（configure のみ `-flto*`・LTO 系 `-Werror=*`・`-f{,no-}hardened` を `sanitize_flags` で除去。`make`/`make install` に `CFLAGS`/`CXXFLAGS`/`LDFLAGS` の変数上書きを自動付与） |
| `-std` 付与 | `-std=gnu17`（CFLAGS 末尾） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（`$SOURCE` をコピーして in-source） |

configure には `autotools_configure` が `--build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}"` を自動付与します。

`make install` には `DESTDIR="$OUT"` を自動付与します。

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--enable-shared` |  | 常時 |
| `--without-ensurepip` |  | 常時 |
| `--with-build-python` | `/dep/python/usr/bin/python3` | arm64, ppc64le, riscv64, s390x |
| `--enable-optimizations` |  | amd64 |

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
for _dep in libffi zlib openssl expat sqlite ncurses readline xz bzip2 zstd; do
  if [ -d /dep/${_dep}/usr/include ]; then
    export CPPFLAGS="${CPPFLAGS:-} -I/dep/${_dep}/usr/include"
    export CFLAGS="${CFLAGS} -I/dep/${_dep}/usr/include"
  fi
  if [ -d /dep/${_dep}/usr/lib64 ]; then
    export LDFLAGS="${LDFLAGS} -L/dep/${_dep}/usr/lib64"
    export PKG_CONFIG_PATH="/dep/${_dep}/usr/lib64/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
    export LD_LIBRARY_PATH="/dep/${_dep}/usr/lib64${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
  else
    if [ -d /dep/${_dep}/usr/lib ]; then
      export LDFLAGS="${LDFLAGS} -L/dep/${_dep}/usr/lib"
      export PKG_CONFIG_PATH="/dep/${_dep}/usr/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
      export LD_LIBRARY_PATH="/dep/${_dep}/usr/lib${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
    fi
  fi
done
export CFLAGS="${CFLAGS} -DOPENSSL_NO_SSL3 -DOPENSSL_NO_SSL3_METHOD -DOPENSSL_NO_TLS1_METHOD -DOPENSSL_NO_TLS1_1_METHOD -DOPENSSL_NO_TLS1_2_METHOD"
autotools_configure --prefix=/usr --enable-shared --enable-optimizations --without-ensurepip
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
make install CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
ln -sf python3 "$OUT"/usr/bin/python
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
```

</details>

<!-- /GENERATED -->



実行時はパッケージ Platform の `{triple}-glibc` に加え、libffi、zlib、openssl、expat、sqlite、ncurses、readline、xz、bzip2、zstd です。ビルド専用は linux-headers です。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。python は `Bootstrap` を実装せず、`build-env` 合成後に自前環境で組みます。openssl などの実行時依存は python のグラフが引きます。他パッケージがビルド時に実行する python は HOST 成果物をマウントします。


## 補足

実行時依存の `/dep/{name}/usr/include`・`lib{,64}` を `CFLAGS` / `LDFLAGS` / `PKG_CONFIG_PATH` に足してから configure します。`$OUT/usr/bin/python` から `python3` へのシンボリックリンクを作成します。`--enable-optimizations` はネイティブビルドのみです（PROFILE_TASK で生成バイナリを実行する自己完結 PGO のため、クロスでは付けません）。
