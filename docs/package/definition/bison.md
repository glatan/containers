# bison

| 項目 | 値 |
|-|-|
| パッケージ名 | `bison` |
| バージョン | 3.8.2 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/bison.rs`](../../../builder/src/package/definition/bison.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/bison/bison-3.8.2.tar.xz |
| sha3-512 | `8025a9224d91172b8262318bb8df7c6e1dd154407e0743855da383f22c5a0f76c3145687fa93fb0acbd71c26026d34876a05bd16234071638efffa08272df593` |
| blake3 | `9dd90be8df4d0474b941e2ca14ac76d11b7ccb46edb26344b60d866178bbcc9872561d223817a88247e5cc30f573f08f0243eb2f43c8a8928d0c6fc25f532d67` |
| 展開ディレクトリ | `bison-3.8.2` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [m4](./m4.md) | 1.4.21 | マクロプロセッサ |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `bison` で同梱の `.y` 文法（`parse-gram.y`・`examples`・`tests`）を処理 |
