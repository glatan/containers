<!-- GENERATED -->
# sqlite

| 項目 | 値 |
|-|-|
| パッケージ名 | `sqlite` |
| バージョン | 3.53.4 |
| ライセンス | blessing |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/sqlite.rs`](../../../builder/src/package/definition/sqlite.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://www.sqlite.org/2026/sqlite-autoconf-3530400.tar.gz |
| sha3-512 | `2b9595cc6142a529462577b1d120d20a06390402efe97fd88f0a49ec915835853339775a8ff2eaee25c3d46b372724fb77a11cf4490d34439d10b198e59d3380` |
| blake3 | `e74b1d27f5053bd29ee7e930ad71d99126b59e4f1a45493440c72d2228e32412adad771dccb35bc3ed5990e3412e7bb9e50539bfd7805d978fbdb7e84259b2a6` |
| 展開ディレクトリ | `sqlite-autoconf-3530400` |

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
| `--enable-readline` | | 常時 |
| `CPPFLAGS` | `-DSQLITE_ENABLE_COLUMN_METADATA -DSQLITE_ENABLE_FTS3 -DSQLITE_ENABLE_FTS4 -DSQLITE_ENABLE_FTS5 -DSQLITE_ENABLE_RTREE` を追加 | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `sqlite3` でテーブル作成・INSERT / SELECT / JOIN / UPDATE / `VACUUM` を実行 |
