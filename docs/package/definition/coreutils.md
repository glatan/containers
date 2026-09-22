<!-- GENERATED -->
# coreutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `coreutils` |
| バージョン | 9.12 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/coreutils.rs`](../../../builder/src/package/definition/coreutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/coreutils/coreutils-9.12.tar.xz |
| sha3-512 | `d6f1fbd33b86bce8c0eec25875c04192e70a5a946561e2ed5a8fd9a4214e309cb07e0cc4ce1f86b228beeedc4f7fbf1c1218bd5dd3a45a308edd1fefd4e4d82b` |
| blake3 | `420edafa8b231145442d248c00412727651f20435e85dc80fbf18074fc249624c6ff75aaed836074f668767844667fe0ccc3ffa59975cb5bda8e29348dcb2779` |
| 展開ディレクトリ | `coreutils-9.12` |

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
| `--disable-xattr` | | 常時 |
| `--without-selinux` | | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `ls -lR`・`sort`・`cp -a`・`wc`・`cat`・`md5sum` パイプラインで同梱ソースを処理 |
