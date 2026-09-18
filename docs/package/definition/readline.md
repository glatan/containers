# readline

| 項目 | 値 |
|-|-|
| パッケージ名 | `readline` |
| バージョン | 8.3 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/readline.rs`](../../../builder/src/package/definition/readline.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/readline/readline-8.3.tar.gz |
| sha3-512 | `58fed6675aabd02dfab76dd82dddcda75361c96f28782a37883487961352885854dc44fd2868871242c7187ec26837d265c73d131948cffc6eff6238f2d46311` |
| blake3 | `7109f094062bda387a0c16b4875375b96e36437bebbbd8d8f91bb27ba01d687f0e55e7df793b8b1e47dd4d6d4c990d0428a475e748b86defb20881d01f12a6ad` |
| 展開ディレクトリ | `readline-8.3` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [ncurses](./ncurses.md) | 6.6 | 端末制御ライブラリ（`SHLIB_LIBS=-lncursesw`） |

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
| `--with-curses` | | 常時 |
| `SHLIB_LIBS` | `-lncursesw` | 常時（PGO の instrumented / use 両ビルドでも replay） |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `examples/rl`・`examples/rlversion` をビルドして実行 |
