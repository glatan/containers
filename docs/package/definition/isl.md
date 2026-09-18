# isl

| 項目 | 値 |
|-|-|
| パッケージ名 | `isl` |
| バージョン | 0.26 |
| ライセンス | MIT |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/isl.rs`](../../../builder/src/package/definition/isl.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://libisl.sourceforge.io/isl-0.26.tar.xz |
| sha3-512 | `4b31f4a9a343cc15319359ad41ef8d4176bcf2f713ea1fb3b2e5a275a6bc6d109943521ef181fab5c21f0ba581f08e753b4e7c71f9a296a559dc71f77754f1aa` |
| blake3 | `a27da5d097f4e105d3a63c5e81d26c2b00cc35a4a3bf62dd2a49335a0f20ce7f57a3e1152dac76aa4b16d8a4205040f108562aa9647e2d6d27717ed533af673f` |
| 展開ディレクトリ | `isl-0.26` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [gmp](./gmp.md) | 6.3.0 | 整数集合の演算処理 |

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
| `--with-gmp-prefix` | オーバーレイの gmp 成果物 | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `make check` |
