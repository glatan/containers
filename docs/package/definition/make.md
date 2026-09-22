<!-- GENERATED -->
# make

| 項目 | 値 |
|-|-|
| パッケージ名 | `make` |
| バージョン | 4.4.1 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/make.rs`](../../../builder/src/package/definition/make.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/make/make-4.4.1.tar.gz |
| sha3-512 | `a5ad233e908b20121b3db5e39dd228e3f62af3b8d0ca5391dff86640ad82a5c08cc58bbc260c786495315ad5299cc160ba6188d5aafc82051453c0779f929ed6` |
| blake3 | `a7d8aee97b7e9a525ef561afa84eea0d929f246e3aafa420231c0602151cf9eb034f36820f07eb9c0a8afa7ab1f9fd5c41eed9f0db7d0397d7caf503846302ad` |
| 展開ディレクトリ | `make-4.4.1` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 生成した Makefile を `make` で実行（`all`・`clean`・`-n`） |
