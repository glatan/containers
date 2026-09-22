<!-- GENERATED -->
# expat

| 項目 | 値 |
|-|-|
| パッケージ名 | `expat` |
| バージョン | 2.8.4 |
| ライセンス | MIT |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/expat.rs`](../../../builder/src/package/definition/expat.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/libexpat/libexpat/releases/download/R_2_8_4/expat-2.8.4.tar.xz |
| sha3-512 | `056d6b25263d95d7375e2fc14f22f161722558c7e695ca3fccc73eb2842bf921e2791fcc1104fd52b0e057dd3be00795af698af5c70a9d8a257b5d6123ac75cc` |
| blake3 | `1311a1adc8da8ffd10ca65db96be3abe9e7e9d83561d899bd0dbdab17ab599dbddde8e8e3a1a6b9a3223c184a3bf1e71d767750418b304be8b22e8e209348f54` |
| 展開ディレクトリ | `expat-2.8.4` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `xmlwf` で同梱 XML（`doc/xmlwf.xml` 他）をパース |
