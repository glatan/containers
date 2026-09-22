<!-- GENERATED -->
# isl

| 項目 | 値 |
|-|-|
| パッケージ名 | `isl` |
| バージョン | 0.26 |
| ライセンス | MIT |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/isl.rs`](../../../builder/src/package/definition/isl.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://libisl.sourceforge.io/isl-0.26.tar.xz |
| sha3-512 | `4b31f4a9a343cc15319359ad41ef8d4176bcf2f713ea1fb3b2e5a275a6bc6d109943521ef181fab5c21f0ba581f08e753b4e7c71f9a296a559dc71f77754f1aa` |
| blake3 | `a27da5d097f4e105d3a63c5e81d26c2b00cc35a4a3bf62dd2a49335a0f20ce7f57a3e1152dac76aa4b16d8a4205040f108562aa9647e2d6d27717ed533af673f` |
| 展開ディレクトリ | `isl-0.26` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [gmp](./gmp.md) | 6.3.0 | — |

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
| `--with-gmp-prefix` | オーバーレイの gmp 成果物 | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `make check` |
