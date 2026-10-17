<!-- GENERATED -->
# zstd

| 項目 | 値 |
|-|-|
| パッケージ名 | `zstd` |
| バージョン | 1.5.7 |
| ライセンス | BSD-3-Clause OR GPL-2.0-only |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Profiled`） |
| 定義 | [`builder/src/package/definition/zstd.rs`](../../../../builder/src/package/definition/zstd.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/facebook/zstd/releases/download/v1.5.7/zstd-1.5.7.tar.gz |
| sha3-512 | `9fcbd59a61d336c34cd46ff6f335787848d37e8c6d0a4aef468755ef4d31fcce7368ce35650282b31d28303bc985470a97b06110b8fb050b0e72eba66afabca8` |
| blake3 | `730dca31244abd219e995f03a55d95b2cfb4b3e16cda055a79fa6f30a4f0e1db8febef4887bffd4b8c19d6926fca83c0ae68e6e5abf535c0c0a85193342d8568` |
| 展開ディレクトリ | `zstd-1.5.7` |

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
| env プリリュード | `Env::None`（前処理なし） |
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
make -j"$(nproc)" PREFIX=/usr LIBDIR=/usr/lib HAVE_ZLIB=0 HAVE_LZMA=0 HAVE_LZ4=0
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
(
  for _i in 1 2 3 4 5 6 7 8; do
    cat ./lib/*.c ./lib/*/*.c ./lib/*/*.h ./programs/*.c 2> /dev/null
  done > /tmp/pgo-input
  for _level in 1 3 9 19; do
    ./programs/zstd -b"$_level" -T0 /tmp/pgo-input
  done
) || true
make clean || make distclean
export CFLAGS="$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
make -j"$(nproc)" PREFIX=/usr LIBDIR=/usr/lib HAVE_ZLIB=0 HAVE_LZMA=0 HAVE_LZ4=0
make install PREFIX=/usr LIBDIR=/usr/lib DESTDIR="$OUT"
```

</details>

<details>
<summary>arm64, ppc64le, riscv64, s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
make -j"$(nproc)" PREFIX=/usr LIBDIR=/usr/lib HAVE_ZLIB=0 HAVE_LZMA=0 HAVE_LZ4=0
make install PREFIX=/usr LIBDIR=/usr/lib DESTDIR="$OUT"
```

</details>

<!-- /GENERATED -->




## 補足

Autotools の `configure` スクリプトではなく Makefile を直接呼び出します（クロスは `CC` / `TRIPLE` に任せます）。CLI の .gz / .xz / .lz4 連携は無効化し、シード環境のライブラリを自動検出させません。`liblz4` 等を実行時依存として引き込まないためです。

PGO はネイティブビルドのみ適用します（`Pgo` ポリシーはメタデータ表を参照）。同梱ソースを連結した複数 MB 入力を `zstd -b{1,3,9,19} -T0` でベンチマークします。
