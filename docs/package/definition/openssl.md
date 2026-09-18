# openssl

| 項目 | 値 |
|-|-|
| パッケージ名 | `openssl` |
| バージョン | 4.0.2 |
| ライセンス | Apache-2.0 |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（コンパイラ環境変数は builder が設定） |
| 定義 | [`builder/src/package/definition/openssl.rs`](../../../builder/src/package/definition/openssl.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/openssl/openssl/releases/download/openssl-4.0.2/openssl-4.0.2.tar.gz |
| sha3-512 | `a7d86c8609e98dea12cab18f22724439a1fda43ef9a45a6440a201a6d9e616fe29dcf83dbffbc5c303cc5f43a1c92f5d222adb29f03a30b1e0b578af0a47dc4b` |
| blake3 | `ef5af5011e1ee875521ed63331cf35d16e82167042b37f361a630b10f5bf2f999fad90fcc972bc02a0d83e2972af48040140d4e78279311b46a7c1557c206d7a` |
| 展開ディレクトリ | `openssl-4.0.2` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter` によるカーネルヘッダ参照 |
| [perl](./perl.md) | 5.42.3 | Configure。成果物は HOST の perl をマウントする |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（`CC` は builder が渡す） |

## ターゲット

`Configure` スクリプトの第 1 引数は architecture から決めます。CPU 指定フラグは builder が variant から決めます（ppc64le は `-mcpu=`、それ以外は `-march=`）。対象 Platform は `Target::all()` です。variant ごとの CPU 名は [platforms.md](../../image/platforms.md) と `Target::march()` です。

| architecture | Configure |
|-|-|
| amd64 | `linux-x86_64` |
| arm64 | `linux-aarch64` |
| riscv64 | `linux64-riscv64` |
| ppc64le | `linux-ppc64le` |
| s390x | `linux64-s390x` |

## ビルドフラグ

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--openssldir` | `/etc/ssl` | 常時 |
| `--libdir` | `lib64` | 常時 |
| LTO | 適用する | builder の `Cflag::Lto` / `Ldflag::Lto` で指定（Configure の `enable-lto` は 3.6 系で未提供） |
| PGO | ネイティブビルドのみ適用（`Pgo::Custom`） | `make` の `CFLAGS` / `LDFLAGS` に `-fprofile-generate` / `-fprofile-use` を付与 |
| `make` ターゲット | `install_sw` | 常時（マニュアルファイルはインストールしない） |
| `flags()` | `tool_flags(target)`（既定セット。[README.md](../README.md) の既定フラグ表） | 個別追加・除外はなし |
| `AR` / `RANLIB` / `NM` | ネイティブは `gcc-ar` 等、クロスは `${CROSS}gcc-ar` 等 | LTO 最適化オブジェクトの処理に必要 |

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
