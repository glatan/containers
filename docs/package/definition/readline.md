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
| sha3-512 | `6ba1e8ee6e7e00e0fe3606f5c04db12aa311b714b1975eecab05e0ae2b1e4a39b369fef3f42656910385bf5fb04b901fc46d2cb77ef58eb8d5df8f9f694e1e82` |
| blake3 | `9ca0529d332612d9b626786a3458db41f71587d6cf120fcbf9cb7be9ceadbf1d293d0c2e3995f5431665a343c5b525d886295551f357ff3307ec93ceee47c9e0` |
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
| `SHLIB_LIBS` | `-lncursesw` | 常時 |
