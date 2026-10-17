<!-- GENERATED -->
# openssl

| 項目 | 値 |
|-|-|
| パッケージ名 | `openssl` |
| バージョン | 4.0.3 |
| ライセンス | Apache-2.0 |
| 分類 | カタログ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Custom`、レシピ内の個別手順） |
| 定義 | [`builder/src/package/definition/openssl.rs`](../../../../builder/src/package/definition/openssl.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/openssl/openssl/releases/download/openssl-4.0.3/openssl-4.0.3.tar.gz |
| sha3-512 | `11583faf5718638acdca493ca593d382302fab032db7ec1c6adedb07d073394e891ddf236591070f3fb5d48fe7fad149ee990d87397c4c3a6f95748db80d20cc` |
| blake3 | `92a6d5e2e74a989c1405de8a7770c7fb469ddcf9c9ed74b768d51c274af6487b5816f7dd05f15714baf55b17bb948175891ecfc24da8117fb5527462795abe2b` |
| 展開ディレクトリ | `openssl-4.0.3` |

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
| [perl](./perl.md) | 5.44.0 | — |
| [mold](./mold.md) | 3.0.0 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（out-of-tree） |

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `linux-aarch64` |  | arm64 |
| `--prefix` | `/usr` | 常時 |
| `--openssldir` | `/etc/ssl` | 常時 |
| `--libdir` | `lib64` | 常時 |
| `linux-ppc64le` |  | ppc64le |
| `linux64-riscv64` |  | riscv64 |
| `linux-x86_64` |  | amd64 |
| `linux64-s390x` |  | s390x |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 5 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
"${SOURCE}/Configure" linux-x86_64 --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
export PGODIR=/tmp/pgo
rm -rf "${PGODIR}"
mkdir -p "${PGODIR}"
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic" LDFLAGS="$LDFLAGS -fprofile-generate"
export LD_LIBRARY_PATH="${PWD}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export OPENSSL_MODULES="${PWD}/providers"
export OPENSSL_CONF=/dev/null
(
  ./apps/openssl speed -seconds 1 rsa2048 ecdsap256 ed25519
  ./apps/openssl speed -seconds 1 -evp aes-128-gcm -evp aes-256-gcm -evp chacha20-poly1305 -evp sha256 -evp sha512
  ./apps/openssl speed -seconds 1 aes-128-cbc aes-256-cbc sha256 sha512
) || true
make clean
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="$LDFLAGS -fprofile-use"
make install_sw DESTDIR="${OUT}"
```

</details>

<details>
<summary>arm64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
"${SOURCE}/Configure" linux-aarch64 --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
make -j"$(nproc)"
make install_sw DESTDIR="${OUT}"
```

</details>

<details>
<summary>ppc64le（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
"${SOURCE}/Configure" linux-ppc64le --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
make -j"$(nproc)"
make install_sw DESTDIR="${OUT}"
```

</details>

<details>
<summary>riscv64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
"${SOURCE}/Configure" linux64-riscv64 --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
make -j"$(nproc)"
make install_sw DESTDIR="${OUT}"
```

</details>

<details>
<summary>s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
"${SOURCE}/Configure" linux64-s390x --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
make -j"$(nproc)"
make install_sw DESTDIR="${OUT}"
```

</details>

<!-- /GENERATED -->



## ターゲット

`Configure` スクリプトの第 1 引数は architecture から決めます。CPU 指定フラグは builder が variant から決めます（ppc64le は `-mcpu=`、それ以外は `-march=`）。対象 Platform は `Target::all()` です。variant ごとの CPU 名は [targets.md](../../../user/targets.md) と `Target::march()` です。

## LTO / PGO 方針

LTO は常時有効とします。一方、PGO は生成バイナリを実行可能なネイティブビルド時のみ適用します。クロスビルド環境（生成コードの triple がホスト triple と異なる場合）ではプロファイル採取を実行できないため、LTO のみ適用します。分岐は `Pgo::Custom` ポリシーとして `recipe(ctx)` が `ctx.cross()` で静的に解決します。

## PGO のプロファイル生成手順

`Configure` は最初の 1 回のみ実行します。LTO フラグは builder が合成する `CFLAGS` および `LDFLAGS` を経由して指定し、PGO 用フラグは `make` の引数に追加します。

| 手順 | 処理内容 |
|-|-|
| 1 | `Configure` の実行後、`-fprofile-generate` を付与してビルドする |
| 2 | ビルドディレクトリを `LD_LIBRARY_PATH` に、providers ディレクトリを `OPENSSL_MODULES` に入れて（`OPENSSL_CONF=/dev/null` で設定読み込みを回避）、下表のワークロードを実行してプロファイルを採取する |
| 3 | `make clean` で中間生成物を除去し、同一ビルドディレクトリ内で `-fprofile-use` を付与して再ビルドする |

ワークロード（`apps/openssl speed -seconds 1`）:

| 項目 | 対象アルゴリズム |
|-|-|
| 公開鍵 | `rsa2048` `ecdsap256` `ed25519` |
| EVP（AEAD） | `aes-128-gcm` `aes-256-gcm` `chacha20-poly1305` `sha256` `sha512` |
| 暗号 | `aes-128-cbc` `aes-256-cbc` |
| ダイジェスト | `sha256` `sha512` |
