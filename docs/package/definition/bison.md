<!-- GENERATED -->
# bison

| 項目 | 値 |
|-|-|
| パッケージ名 | `bison` |
| バージョン | 3.8.2 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/bison.rs`](../../../builder/src/package/definition/bison.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/bison/bison-3.8.2.tar.xz |
| sha3-512 | `8025a9224d91172b8262318bb8df7c6e1dd154407e0743855da383f22c5a0f76c3145687fa93fb0acbd71c26026d34876a05bd16234071638efffa08272df593` |
| blake3 | `9dd90be8df4d0474b941e2ca14ac76d11b7ccb46edb26344b60d866178bbcc9872561d223817a88247e5cc30f573f08f0243eb2f43c8a8928d0c6fc25f532d67` |
| 展開ディレクトリ | `bison-3.8.2` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `bison` で同梱の `.y` 文法（`parse-gram.y`・`examples`・`tests`）を処理 |
