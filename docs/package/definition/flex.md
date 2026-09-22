<!-- GENERATED -->
# flex

| 項目 | 値 |
|-|-|
| パッケージ名 | `flex` |
| バージョン | 2.6.4 |
| ライセンス | BSD-2-Clause |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/flex.rs`](../../../builder/src/package/definition/flex.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/westes/flex/files/981163/flex-2.6.4.tar.gz |
| sha3-512 | `eac223beda4c1629dccfd29666eab9f510e05f5b606587496364ae59622c3bab8104a9ac52f4bb06098a628a4d4fba0c59f5a9d851d65cbc94ed0a660475e162` |
| blake3 | `d78b714ac38bd9de7f9b44a078efed82e96ed43e7cf9cd33944a7f379a2d09a4d024e6d3a3accb71ae72f36b4f028328f551c6271a231cdd3c307b542cdca143` |
| 展開ディレクトリ | `flex-2.6.4` |

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
| `-std` | `gnu11` | gnulib の `lib/malloc.c` が K&R 宣言のため gnu17 で壊れる |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `flex` で同梱の `.l` ファイルをスキャナ生成 |
