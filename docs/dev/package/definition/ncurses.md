<!-- GENERATED -->
# ncurses

| 項目 | 値 |
|-|-|
| パッケージ名 | `ncurses` |
| バージョン | 6.6 |
| ライセンス | MIT-open-group |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| 定義 | [`builder/src/package/definition/ncurses.rs`](../../../../builder/src/package/definition/ncurses.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/ncurses/ncurses-6.6.tar.gz |
| sha3-512 | `c569ffea33791685cd04890e85fc457f7be7bb62b6a478341f9e17ccc2ef248edf084ac4078ef1ea08e43b7aca045c38e49054bde51fa30cd34eebf89269672d` |
| blake3 | `fbec55697a01f99b9cc3f25be55e73ae7091f4c53e5d81a1ea15734c4e5b72385c811def3c51272818645313ef1007cebb8edebdc6cec4a5feab9607a7c6636b` |
| 展開ディレクトリ | `ncurses-6.6` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::Lto`（フルフラグのまま configure。configure が CFLAGS を Makefile へ焼く型） |
| `-std` 付与 | `-std=gnu17`（CFLAGS 末尾） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（out-of-tree） |

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
| `--with-shared` |  | 常時 |
| `--without-cxx` |  | 常時 |
| `--enable-widec` |  | 常時 |
| `--enable-pc-files` |  | 常時 |
| `--with-pkg-config-libdir` | `/usr/lib/pkgconfig` | 常時 |
| `--without-ada` |  | 常時 |
| `--without-debug` |  | 常時 |
| `--with-versioned-syms` |  | 常時 |
| `--with-build-cc` | `x86_64-linux-gnu-gcc` | arm64, ppc64le, riscv64, s390x |
| `--with-build-cpp` | `x86_64-linux-gnu-gcc` | arm64, ppc64le, riscv64, s390x |
| `--with-build-cflags` | `'-O2 -std=gnu17'` | arm64, ppc64le, riscv64, s390x |
| `--with-build-cppflags` | `` | arm64, ppc64le, riscv64, s390x |
| `--with-build-ldflags` | `` | arm64, ppc64le, riscv64, s390x |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 2 系統に分かれます。

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
autotools_configure --prefix=/usr --with-shared --without-cxx --enable-widec --enable-pc-files --with-pkg-config-libdir=/usr/lib/pkgconfig --without-ada --without-debug --with-versioned-syms
make -j"$(nproc)"
make install DESTDIR="$OUT"
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
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
autotools_configure --prefix=/usr --with-shared --without-cxx --enable-widec --enable-pc-files --with-pkg-config-libdir=/usr/lib/pkgconfig --without-ada --without-debug --with-versioned-syms --with-build-cc=x86_64-linux-gnu-gcc --with-build-cpp=x86_64-linux-gnu-gcc --with-build-cflags='-O2 -std=gnu17' --with-build-cppflags= --with-build-ldflags=
make -j"$(nproc)"
make install DESTDIR="$OUT"
```

</details>

<!-- /GENERATED -->




## 補足

`--without-cxx` で C++ バインディング（`libncurses++w`）を組みません。`libstdc++.so.6` / `libgcc_s.so.1` への実行時依存を避けるためです。

Wide-character 版ライブラリ（`libncursesw.so`）へリンクする非 wide 互換シンボリックリンクを作成します。

```text
/usr/lib/libncurses.so -> libncursesw.so
/usr/lib/libcurses.so  -> libncursesw.so
```

PGO は適用しません。`tic` は libncurses の消費者経路を駆動しないため、意味のあるプロファイルを採取できません（`Pgo::None`）。
