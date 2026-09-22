<!-- GENERATED -->
# patch

| 項目 | 値 |
|-|-|
| パッケージ名 | `patch` |
| バージョン | 2.8 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/patch.rs`](../../../builder/src/package/definition/patch.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/patch/patch-2.8.tar.xz |
| sha3-512 | `d18e96a61a152e03a84d36384931fa3ce01f36240414b2af3bbd5c7ba02a665251cb8300ad2e4d9f7387238e0a717294928f8a30343c7a8647a80e66f543f9b5` |
| blake3 | `d7dadf0ac720de7560f140a9021928bc0f8089182121c0e65c8bec115cf311240545c913ea3be61b067c31dd4787ac6b2fe7da2d18b1f4047b1ff65a9e55fa60` |
| 展開ディレクトリ | `patch-2.8` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 同梱ソースへの `diff -u` 出力を `./src/patch` で dry-run / 適用 |
