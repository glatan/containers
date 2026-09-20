# make

| 項目 | 値 |
|-|-|
| パッケージ名 | `make` |
| バージョン | 4.4.1 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/make.rs`](../../../builder/src/package/definition/make.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/make/make-4.4.1.tar.gz |
| sha3-512 | `a5ad233e908b20121b3db5e39dd228e3f62af3b8d0ca5391dff86640ad82a5c08cc58bbc260c786495315ad5299cc160ba6188d5aafc82051453c0779f929ed6` |
| blake3 | `a7d8aee97b7e9a525ef561afa84eea0d929f246e3aafa420231c0602151cf9eb034f36820f07eb9c0a8afa7ab1f9fd5c41eed9f0db7d0397d7caf503846302ad` |
| 展開ディレクトリ | `make-4.4.1` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 生成した Makefile を `make` で実行（`all`・`clean`・`-n`） |
