<!-- GENERATED -->
# findutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `findutils` |
| バージョン | 4.11.0 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/findutils.rs`](../../../builder/src/package/definition/findutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/findutils/findutils-4.11.0.tar.xz |
| sha3-512 | `0ffbc8f9fd502abc230fe2df5ebc68aa46297317a9c32eb024cdb044bf018c90bb6dbfeee030dfb5052cea60f8ecb062011b4554d63ea81ca76944406834d13c` |
| blake3 | `a043f0e36d43c1acbb773fb71bb05342c11f777a34cec2a4ed699c5e578281cd4672a4d409352085f08c1f5c587ddc995dd8322243d4c3a77bac293bad47782f` |
| 展開ディレクトリ | `findutils-4.11.0` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `find` / `xargs` で同梱ソースを走査（`-name`・`-size`・`-newer`・`-print0`） |
