<!-- GENERATED -->
# zlib

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib` |
| バージョン | 1.3.2 |
| ライセンス | Zlib |
| 分類 | カタログ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Custom`、レシピ内の個別手順） |
| 定義 | [`builder/src/package/definition/zlib.rs`](../../../builder/src/package/definition/zlib.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.gz |
| sha3-512 | `860936cb0ddea209f9c14219dbf05efbef7ea83abe93a25bf39dd591a8f075ab5ad0c8225b1455def844d831f2d7b8f645704ebe387e630ad3075f765925dcc7` |
| blake3 | `ea79f8ae7c98d38ea360b70449fbb0052da4ff4a7201f03387113e5dab8543793587a6ee51d8307b5203aa03bf0a0053fca792f4c99d95c48aecc4b49359dc1b` |
| 展開ディレクトリ | `zlib-1.3.2` |

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
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（`$SOURCE` をコピーして in-source） |

`make install` には `DESTDIR="$OUT"` を自動付与します。

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

### `./configure`（1回目の呼出）

- 対象: arm64, ppc64le, riscv64, s390x
- env: `CHOST="${TRIPLE}"`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | arm64, ppc64le, riscv64, s390x |
| `--libdir` | `/usr/lib64` | arm64, ppc64le, riscv64, s390x |
| `--shared` |  | arm64, ppc64le, riscv64, s390x |

### `./configure`（2回目の呼出）

- 対象: amd64

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | amd64 |
| `--libdir` | `/usr/lib64` | amd64 |
| `--shared` |  | amd64 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 2 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
PGO=/tmp/zlib-pgo
rm -rf "$PGO"
mkdir -p "$PGO"
./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-generate=$PGO -fprofile-update=atomic" LDFLAGS="$LDFLAGS -fprofile-generate"
export LD_LIBRARY_PATH="${PWD}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
(
  for _i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do
    cat ./*.c ./*.h
  done > /tmp/zlib-pgo-input
  for _level in 1 6 9; do
    ./minigzip -c "-$_level" < /tmp/zlib-pgo-input | ./minigzip -d -c > /dev/null
  done
  ./example > /dev/null
) || true
make distclean
./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-use=$PGO -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="$LDFLAGS -fprofile-use"
make install DESTDIR="$OUT"
```

</details>

<details>
<summary>arm64, ppc64le, riscv64, s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
PGO=/tmp/zlib-pgo
rm -rf "$PGO"
mkdir -p "$PGO"
CHOST="${TRIPLE}" ./configure --prefix=/usr --libdir=/usr/lib64 --shared
make -j"$(nproc)"
make install DESTDIR="$OUT"
```

</details>

<!-- /GENERATED -->



openssl からは依存されません。

## PGO ワークロード

PGO は生成バイナリを実行可能なネイティブビルド時のみ適用します（`Pgo::Custom`）。クロスビルドではプロファイル採取を実行できないため、`recipe(ctx)` が `ctx.cross()` で分岐して素の `make`（`CHOST=$TRIPLE` 付き `configure`）に切り替えます。ネイティブでは `-fprofile-generate` でビルド後、同梱ソース（`.c` / `.h` を 16 回連結した複数 MB 入力）を `minigzip` で `-1` / `-6` / `-9` の 3 レベルで圧縮・解凍し、`example` も実行して `make distclean` のうえ `-fprofile-use` で再ビルドします。
