<!-- GENERATED -->
# xz

| 項目 | 値 |
|-|-|
| パッケージ名 | `xz` |
| バージョン | 5.8.4 |
| ライセンス | 0BSD |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/xz.rs`](../../../builder/src/package/definition/xz.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/tukaani-project/xz/releases/download/v5.8.4/xz-5.8.4.tar.xz |
| sha3-512 | `56aff332bbc60da0cb0e075b20cf947f3a6ea2be500a4d05bd27b435c0a40005f90c9a4bed4124c625a374ba0b81473bdf0d7d89cbe381f5f5f36a7687f1e7c4` |
| blake3 | `4776832814549b37914485ee14979b68282d4ddf1c0b679c6f4e055a57666b96fcbdc36febaf229f670d823e253d1abcbbee9dd4163eec27106770a030609c05` |
| 展開ディレクトリ | `xz-5.8.4` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 同梱ソースを連結した複数 MB 入力を `xz -T4` で `-0` / `-6` / `-9e` の 3 レベルで圧縮・解凍 |
