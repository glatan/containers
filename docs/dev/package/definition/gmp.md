<!-- GENERATED -->
# gmp

| 項目 | 値 |
|-|-|
| パッケージ名 | `gmp` |
| バージョン | 6.3.0 |
| ライセンス | LGPL-3.0-or-later |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Profiled`） |
| 定義 | [`builder/src/package/definition/gmp.rs`](../../../../builder/src/package/definition/gmp.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz |
| sha3-512 | `b459a8c258273ff76a2627a64627739d233064e5a9a48de68a544c921a1fe4d617af48528de7b67778f30754d793d645a9ecf9c38a71578bd4f0ab6febf591de` |
| blake3 | `fffe4996713928ae19331c8ef39129e46d3bf5b7182820656fd4639435cd83a40d7c77aa7c571938ae3f25a58bfdfa44b6a710f2a7e002ab11edf8389cd6928b` |
| 展開ディレクトリ | `gmp-6.3.0` |

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
| env プリリュード | `Env::Lto`（フルフラグのまま configure。configure が CFLAGS を Makefile へ焼く型） |
| `-std` 付与 | `-std=gnu17`（CFLAGS 末尾） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（out-of-tree） |
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
| `ABI` | `64` | amd64, arm64, s390x |
| `ABI` | `mode64` | ppc64le |
| `ABI` | `standard` | riscv64 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 4 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
export PGODIR=/tmp/pgo
rm -rf "$PGODIR"
mkdir -p "$PGODIR"
export _PGO_CFLAGS="${CFLAGS:-}" _PGO_CXXFLAGS="${CXXFLAGS:-}" _PGO_LDFLAGS="${LDFLAGS:-}"
export CFLAGS="$_PGO_CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-generate"
autotools_configure --prefix=/usr ABI=64
make -j"$(nproc)"
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
(
  make check
) || true
make distclean || make clean
export CFLAGS="$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
autotools_configure --prefix=/usr ABI=64
make -j"$(nproc)"
make install DESTDIR="$OUT"
```

</details>

<details>
<summary>arm64, s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export BUILD_CC="$CC_FOR_BUILD"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
autotools_configure --prefix=/usr ABI=64
make -j"$(nproc)"
make install DESTDIR="$OUT"
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
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
autotools_configure --prefix=/usr ABI=mode64
make -j"$(nproc)"
make install DESTDIR="$OUT"
```

</details>

<details>
<summary>riscv64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export BUILD_CC="$CC_FOR_BUILD"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
autotools_configure --prefix=/usr ABI=standard
make -j"$(nproc)"
make install DESTDIR="$OUT"
```

</details>

<!-- /GENERATED -->




## 補足

PGO はネイティブビルドのみ適用します（`Pgo` ポリシーはメタデータ表を参照）。`make check`。`Env::Lto` のため `-fprofile-generate` / `-fprofile-use` は configure 時の `CFLAGS` へ注入します。
