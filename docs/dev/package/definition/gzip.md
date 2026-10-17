<!-- GENERATED -->
# gzip

| 項目 | 値 |
|-|-|
| パッケージ名 | `gzip` |
| バージョン | 1.15 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Profiled`） |
| 定義 | [`builder/src/package/definition/gzip.rs`](../../../../builder/src/package/definition/gzip.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gzip/gzip-1.15.tar.xz |
| sha3-512 | `a9b7f473b0baa272f67f18ac4de788f5457231cde528f9b1f287db9c870bdf67857210bc444fe3f8502cd3b76f44e3fec3544da3aec42b0a665dea3e890b219a` |
| blake3 | `ed2b2743048530c90a8b93b32bac1f9e06b6fc0988c36ca39d94e10ffb5379d11bab254fe1a13f97a6950991b8d817f4186cada139291aa69c5cf560b72d91e6` |
| 展開ディレクトリ | `gzip-1.15` |

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
| PGO テンプレート | `autotools`（configure → instrumented make → workload → distclean → 再 configure → use make → install。ネイティブのみ） |

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
sed -i "/gzip\.h/i #include <signal.h>" gzip.c
autotools_configure --prefix=/usr
export _PGO_CFLAGS="${CFLAGS:-}" _PGO_CXXFLAGS="${CXXFLAGS:-}" _PGO_LDFLAGS="${LDFLAGS:-}"
export CFLAGS="$_PGO_CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-generate"
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
(
  for _i in 1 2 3 4 5 6 7 8; do
    cat "$SOURCE"/*.c "$SOURCE"/*.h 2> /dev/null
  done > /tmp/pgo-input
  ./gzip -9 -c /tmp/pgo-input | ./gzip -d -c > /dev/null
  ./gzip -1 -c /tmp/pgo-input | ./gzip -d -c > /dev/null
) || true
make distclean || make clean
export CFLAGS="$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
autotools_configure --prefix=/usr
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
make install CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
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
sed -i "/gzip\.h/i #include <signal.h>" gzip.c
autotools_configure --prefix=/usr
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
make install CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
```

</details>

<!-- /GENERATED -->




## 補足

PGO はネイティブビルドのみ適用します（`Pgo` ポリシーはメタデータ表を参照）。同梱ソースを連結した複数 MB 入力を `gzip` で `-1` / `-9` の 2 レベルで圧縮・解凍します。
