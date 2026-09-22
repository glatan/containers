<!-- GENERATED -->
# tar

| 項目 | 値 |
|-|-|
| パッケージ名 | `tar` |
| バージョン | 1.35 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/tar.rs`](../../../builder/src/package/definition/tar.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/tar/tar-1.35.tar.xz |
| sha3-512 | `d075aad9bfe5bde143c87388e3b4995775f0b40016e851d1243cc3b6a59fd65c5eeb8a23624ee41b18999dbd850213cbff7ebfb256e85c3a8441c3aa2cd7747c` |
| blake3 | `55e3853752c56a607ae5dbc9e9190fe5d3f7599b76bb80abe0edba36d712c36bb60b803541ce46293cdf80cb80b8c84fd6940ca9c446fd5db3b04c7ee16664bc` |
| 展開ディレクトリ | `tar-1.35` |

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
| `--disable-acl` | | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `tar` で同梱ソースのアーカイブ作成・一覧・展開を往復 |
