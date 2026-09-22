<!-- GENERATED -->
# diffutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `diffutils` |
| バージョン | 3.12 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/diffutils.rs`](../../../builder/src/package/definition/diffutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/diffutils/diffutils-3.12.tar.xz |
| sha3-512 | `24fde52f69eebea2c5807bf3ab532dd5334ec6714203e9dbeca6cd71f7b31abc0fc1ebe8fa20a17aca8bac41b7b66344c092310bfe5af240fbf45f2e4e47e6fb` |
| blake3 | `363fb378c85505b43f91d6e7c7382d571f2f7b47e4d334c18d522baf55599d18a186e566dcd9cdcf7f3580edf8381ddfc7fd7c0b1100e20ee6ba6dd7f3e6e80c` |
| 展開ディレクトリ | `diffutils-3.12` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `diff -u` / `diff -r` で同梱ソースのコピーと比較 |
