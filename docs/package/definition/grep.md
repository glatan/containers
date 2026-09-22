<!-- GENERATED -->
# grep

| 項目 | 値 |
|-|-|
| パッケージ名 | `grep` |
| バージョン | 3.12 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/grep.rs`](../../../builder/src/package/definition/grep.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/grep/grep-3.12.tar.xz |
| sha3-512 | `226b3cd6aeddf50e0f01581c0829033d6a75e4d9ab101252dabbd3fbfc17f4f6c0ce9ec9381bbd00ef6c902f0c7773a24358bf63e37f3112221652d060ef6e1e` |
| blake3 | `e15c2e03ff6e9e7e8fb7c726539e10b6ff17ecfc3d6961f7419e948d31043f4f953d1acd1988861931ee051d8764723d82bb4c01cd5ee2890d2e9ff7d0e88c16` |
| 展開ディレクトリ | `grep-3.12` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `grep -r -E` で同梱ソースを正規表現検索 |
