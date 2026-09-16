# gzip

| 項目 | 値 |
|-|-|
| パッケージ名 | `gzip` |
| バージョン | 1.14 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/gzip.rs`](../../../builder/src/package/definition/gzip.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gzip/gzip-1.14.tar.xz |
| sha3-512 | `6e7b2e6c5d87a64d2adde0dbd81e0262d638c60088dc5a61be5a0e64b8fa1ed9452bc2a7c3c29667ddb37fac8f8dc35ef16e10154f7a6a57ec8de6aa24ee4162` |
| blake3 | `4d318e6b8ff830c98b98314bcafb54d540ab940553ef1ad5e0743b21a36a473be1c98b3671dae64e436526b39c2359f623d66dae1a003067c3a5ed9e0c0d0617` |
| 展開ディレクトリ | `gzip-1.14` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
