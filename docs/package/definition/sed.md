# sed

| 項目 | 値 |
|-|-|
| パッケージ名 | `sed` |
| バージョン | 4.10 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/sed.rs`](../../../builder/src/package/definition/sed.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/sed/sed-4.10.tar.xz |
| sha3-512 | `a5ae1b5ce201013c51e4f6b8fd132d86f63087a5d5f0908d3cd8d6c03b9a49f67c24ce24d759599dda81e88b3b6a9d40a70e124e24ad53ce5fdee3864308a579` |
| blake3 | `b48115aadef05df0018c3081bc5d7d2d55bf455b0679c6bae665e8498df38be2ea4771112f4fb48783952aa8aef8f8bd184129228e26e95755ba247b97c60ea1` |
| 展開ディレクトリ | `sed-4.10` |

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
