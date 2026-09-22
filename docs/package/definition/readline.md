<!-- GENERATED -->
# readline

| 項目 | 値 |
|-|-|
| パッケージ名 | `readline` |
| バージョン | 8.3 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/readline.rs`](../../../builder/src/package/definition/readline.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/readline/readline-8.3.tar.gz |
| sha3-512 | `58fed6675aabd02dfab76dd82dddcda75361c96f28782a37883487961352885854dc44fd2868871242c7187ec26837d265c73d131948cffc6eff6238f2d46311` |
| blake3 | `7109f094062bda387a0c16b4875375b96e36437bebbbd8d8f91bb27ba01d687f0e55e7df793b8b1e47dd4d6d4c990d0428a475e748b86defb20881d01f12a6ad` |
| 展開ディレクトリ | `readline-8.3` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [ncurses](./ncurses.md) | 6.6 | — |

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
<!-- /GENERATED -->



## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--with-curses` | | 常時 |
| `SHLIB_LIBS` | `-lncursesw` | 常時（PGO の instrumented / use 両ビルドでも replay） |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `examples/rl`・`examples/rlversion` をビルドして実行 |
