# tar

| 項目 | 値 |
|-|-|
| パッケージ名 | `tar` |
| バージョン | 1.35 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/tar.rs`](../../../builder/src/package/definition/tar.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/tar/tar-1.35.tar.xz |
| sha3-512 | `6ba10e401b44cb7246ee95d8eb1a147e4529d8e7c10b754877ee54b7ca5796df3fef8c353381a943a4192b67f1ee0fbfb1e779a1cf64560d2bbfdc0349ca4e1c` |
| blake3 | `9eb03d8d6978430f878a99a770337c76cc7823f03b5ee0689b9148d88e894c25987d60f588c8008064a59bb5f73d8495c6a51d45dc716869a2d2fb4ea778b7e2` |
| 展開ディレクトリ | `tar-1.35` |

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
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
