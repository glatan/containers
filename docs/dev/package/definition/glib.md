<!-- GENERATED -->
# glib

| 項目 | 値 |
|-|-|
| パッケージ名 | `glib` |
| バージョン | 2.90.0 |
| ライセンス | LGPL-2.1-or-later |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| 定義 | [`builder/src/package/definition/glib.rs`](../../../../builder/src/package/definition/glib.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://download.gnome.org/sources/glib/2.90/glib-2.90.0.tar.xz |
| sha3-512 | `c896df2b43beb87254843d355e2fc389d9d62e01a49c58e078f3736286c0f0f16eedcfb611b19edb13ebf38ec8714ab1639b7b52b1ad02301eb1365cddaa205d` |
| blake3 | `0cd33e9b83b0fbc9b8396e99a59504909ced4e2f233f445e6127dacbcf17c0d00cc425186477da98e7f37cacfc8a27f8957f28b6b1cbd94ba1e98deef2570866` |
| 展開ディレクトリ | `glib-2.90.0` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [libffi](./libffi.md) | 3.8.0 | — |
| [pcre2](./pcre2.md) | 10.48 | — |
| [zlib](./zlib.md) | 1.3.2 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [meson](./meson.md) | 1.12.0 | — |
| [ninja](./ninja.md) | 1.13.2 | — |
| [python](./python.md) | 3.14.7 | — |
| [pkgconf](./pkgconf.md) | 2.5.1 | — |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::Lto`（フルフラグのまま configure。configure が CFLAGS を Makefile へ焼く型） |
| `-std` 付与 | `-std=gnu17`（CFLAGS 末尾） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（out-of-tree） |

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--wrap-mode` | `nofallback` | 常時 |
| `-Dlibmount` | `disabled` | 常時 |
| `-Dnls` | `disabled` | 常時 |
| `-Dtests` | `false` | 常時 |
| `-Dsysprof` | `disabled` | 常時 |
| `-Dintrospection` | `disabled` | 常時 |
| `--cross-file` |  | arm64, ppc64le, riscv64, s390x |
| `"${BUILD}/meson-cross.ini"` |  | arm64, ppc64le, riscv64, s390x |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 5 系統に分かれます。

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
meson setup "${SOURCE}" --prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled
meson compile -C . -j $(nproc)
meson install -C . --destdir "${OUT}"
```

</details>

<details>
<summary>arm64（クロス・Canadian）</summary>

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
cat > meson-cross.ini <<EOF
[binaries]
c = '$CC'
cpp = '${CXX:-$CC}'
ar = '${AR:-${TRIPLE}-ar}'
strip = 'aarch64-linux-gnu-strip'
pkg-config = 'pkg-config'
[host_machine]
system = 'linux'
cpu_family = 'aarch64'
cpu = 'aarch64'
endian = 'little'
EOF
meson setup "${SOURCE}" --prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled --cross-file "${BUILD}/meson-cross.ini"
meson compile -C . -j $(nproc)
meson install -C . --destdir "${OUT}"
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
cat > meson-cross.ini <<EOF
[binaries]
c = '$CC'
cpp = '${CXX:-$CC}'
ar = '${AR:-${TRIPLE}-ar}'
strip = 'powerpc64le-linux-gnu-strip'
pkg-config = 'pkg-config'
[host_machine]
system = 'linux'
cpu_family = 'ppc64'
cpu = 'ppc64le'
endian = 'little'
EOF
meson setup "${SOURCE}" --prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled --cross-file "${BUILD}/meson-cross.ini"
meson compile -C . -j $(nproc)
meson install -C . --destdir "${OUT}"
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
cat > meson-cross.ini <<EOF
[binaries]
c = '$CC'
cpp = '${CXX:-$CC}'
ar = '${AR:-${TRIPLE}-ar}'
strip = 'riscv64-linux-gnu-strip'
pkg-config = 'pkg-config'
[host_machine]
system = 'linux'
cpu_family = 'riscv64'
cpu = 'riscv64'
endian = 'little'
EOF
meson setup "${SOURCE}" --prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled --cross-file "${BUILD}/meson-cross.ini"
meson compile -C . -j $(nproc)
meson install -C . --destdir "${OUT}"
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
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
cat > meson-cross.ini <<EOF
[binaries]
c = '$CC'
cpp = '${CXX:-$CC}'
ar = '${AR:-${TRIPLE}-ar}'
strip = 's390x-linux-gnu-strip'
pkg-config = 'pkg-config'
[host_machine]
system = 'linux'
cpu_family = 's390x'
cpu = 's390x'
endian = 'little'
EOF
meson setup "${SOURCE}" --prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled --cross-file "${BUILD}/meson-cross.ini"
meson compile -C . -j $(nproc)
meson install -C . --destdir "${OUT}"
```

</details>

<!-- /GENERATED -->




## 補足

クロス時は meson の `--cross-file` を付けます。meson 実行前にホスト用 glibc の `--sysroot` と linux-headers の `-idirafter` を付与します。`-Dsysprof=disabled` は `glib-2.0.pc` が外部の `sysprof-capture-4` を要求しないようにするため、`--wrap-mode=nofallback` はサブプロジェクトの自動取得を禁止するためです。LTO は共通前処理を適用しないため既定フラグの `-flto` を維持します。

PGO は適用しません。meson ビルドのため PGO テンプレートの対象外で、現状は個別手順を設けていません（`Pgo::None`）。
