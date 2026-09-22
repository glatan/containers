<!-- GENERATED -->
# sed

| 項目 | 値 |
|-|-|
| パッケージ名 | `sed` |
| バージョン | 4.10 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/sed.rs`](../../../builder/src/package/definition/sed.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/sed/sed-4.10.tar.xz |
| sha3-512 | `a5ae1b5ce201013c51e4f6b8fd132d86f63087a5d5f0908d3cd8d6c03b9a49f67c24ce24d759599dda81e88b3b6a9d40a70e124e24ad53ce5fdee3864308a579` |
| blake3 | `b48115aadef05df0018c3081bc5d7d2d55bf455b0679c6bae665e8498df38be2ea4771112f4fb48783952aa8aef8f8bd184129228e26e95755ba247b97c60ea1` |
| 展開ディレクトリ | `sed-4.10` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `sed` で同梱ソースを置換・範囲出力・`y` 変換 |
