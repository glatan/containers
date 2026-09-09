# openssl

| 項目 | 値 |
|-|-|
| パッケージ名 | `openssl` |
| バージョン | 3.6.4 |
| ライセンス | Apache-2.0 |
| パッケージイメージ | する。[公開一覧](../packages.md) |
| ネットワーク | 許可(クロス時の pacman) |
| 定義 | [`builder/src/package/openssl.rs`](../../../builder/src/package/openssl.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/openssl/openssl/releases/download/openssl-3.6.4/openssl-3.6.4.tar.gz |
| sha3-512 | `ae8c16f7fa9003fe7dac1fc538a8e6805ccafd39beb05a9b3d5a4d0adbf7a3a278de34b04779337c0bea99b98458e485463e6ce2cc71e36dd78f279433664b15` |
| blake3 | `ce6ba56d9f75a9e6965cd98385a85b57aa7d1a001dbb9c638c3ce50d391554c6a4cb4e336bbc22c7d3111b199ac4d893bb66174d23159c9ebcbb0b9aa496a050` |
| 展開ディレクトリ | `openssl-3.6.4` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [glibc](./glibc.md) | 2.44 | `--sysroot=/dep/glibc`、`-B` / `-L` |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.12.109 | `-idirafter /dep/linux-headers/usr/include` |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../../package/README.md) | `package build` |
| `${CROSS}gcc`(例: `aarch64-linux-gnu-gcc`) | クロス時。pacman で導入 |

## ターゲット

`Configure` の第 1 引数は architecture で決まります。`-march` はツールが variant から決めます。表に無い組み合わせは [platforms.md](../platforms.md) にあり、未対応です。

| architecture | variant | Configure | `-march` | 状態 |
|-|-|-|-|-|
| amd64 | v1 | linux-x86_64 | x86-64 | 対応 |
| amd64 | v2 | linux-x86_64 | x86-64-v2 | 対応 |
| amd64 | v3 | linux-x86_64 | x86-64-v3 | 対応 |
| amd64 | v4 | linux-x86_64 | x86-64-v4 | 対応 |
| arm64 | v8 | linux-aarch64 | armv8-a | 対応 |

## ビルドフラグ

| フラグ | 値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--openssldir` | `/etc/ssl` | 常時 |
| `--libdir` | `lib64` | 常時 |
| `enable-lto` | | 常時 |
| `enable-profile-generate` / `enable-profile-use` | | ネイティブのみ。下の「PGO のプロファイル生成」 |
| `make` | `install_sw` | 常時(マニュアルは入れない) |
| `CFLAGS` | `-O2 -march=...` | ツールが付与 |
| `AR` / `RANLIB` / `NM` | `${CROSS}gcc-ar` など | LTO 用 |

LTO は常に有効です。PGO は対象バイナリを実行できるネイティブビルドだけです。クロス(`$CROSS` が付く)はプロファイルを取れないため LTO のみです。

## PGO のプロファイル生成

`make clean` はしません(`.gcda` を残すため)。`Configure` を二度回してから `install_sw` します。

| 手順 | 内容 |
|-|-|
| 1 | `enable-lto enable-profile-generate` でビルドする |
| 2 | 次のワークロードでプロファイルを取る |
| 3 | `enable-lto enable-profile-use` で同じビルドディレクトリを再ビルドする |

ワークロード(`apps/openssl speed -seconds 1`):

| 項目 | 対象 |
|-|-|
| 公開鍵 | `rsa2048`, `ecdsap256` |
| 対称 | `aes-128-cbc`, `aes-256-cbc` |
| ハッシュ | `sha256`, `sha512` |

手順は定義ファイルの `build_script` です。
