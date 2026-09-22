<!-- GENERATED -->
# gawk

| 項目 | 値 |
|-|-|
| パッケージ名 | `gawk` |
| バージョン | 5.4.1 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/gawk.rs`](../../../builder/src/package/definition/gawk.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gawk/gawk-5.4.1.tar.xz |
| sha3-512 | `886ec44952ec9f11ff0eeeeb113479d85d8763a91c83f55f5df7c7e96e71f4f9f20c37f478de6914ed456ff9c1cbc9c19b1a381c5959976d0cfa9b085677a60a` |
| blake3 | `d2f64ff7d253202c500da1d6c8b33b5f61693385d81cb5126ff546940872e3bcca174d16646dccd8a64da59a5c92123c265a4176d8efbafa414f8dc61bfdd81b` |
| 展開ディレクトリ | `gawk-5.4.1` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [gmp](./gmp.md) | 6.3.0 | — |
| [mpfr](./mpfr.md) | 4.2.2 | — |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `gawk` スクリプトで同梱ソースを処理（フィールド集計・連想配列・`asort`） |
