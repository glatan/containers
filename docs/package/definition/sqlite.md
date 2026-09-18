# sqlite

| 項目 | 値 |
|-|-|
| パッケージ名 | `sqlite` |
| バージョン | 3.53.4 |
| ライセンス | blessing |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/sqlite.rs`](../../../builder/src/package/definition/sqlite.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://www.sqlite.org/2026/sqlite-autoconf-3530400.tar.gz |
| sha3-512 | `2b9595cc6142a529462577b1d120d20a06390402efe97fd88f0a49ec915835853339775a8ff2eaee25c3d46b372724fb77a11cf4490d34439d10b198e59d3380` |
| blake3 | `e74b1d27f5053bd29ee7e930ad71d99126b59e4f1a45493440c72d2228e32412adad771dccb35bc3ed5990e3412e7bb9e50539bfd7805d978fbdb7e84259b2a6` |
| 展開ディレクトリ | `sqlite-autoconf-3530400` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [readline](./readline.md) | 8.3 | 対話型シェル向けコマンドライン編集機能 |
| [zlib](./zlib.md) | 1.3.2 | 圧縮サポート |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--enable-readline` | | 常時 |
| `CPPFLAGS` | `-DSQLITE_ENABLE_COLUMN_METADATA -DSQLITE_ENABLE_FTS3 -DSQLITE_ENABLE_FTS4 -DSQLITE_ENABLE_FTS5 -DSQLITE_ENABLE_RTREE` を追加 | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `sqlite3` でテーブル作成・INSERT / SELECT / JOIN / UPDATE / `VACUUM` を実行 |
