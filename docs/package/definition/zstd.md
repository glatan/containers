# zstd

| 項目 | 値 |
|-|-|
| パッケージ名 | `zstd` |
| バージョン | 1.5.7 |
| ライセンス | BSD-3-Clause OR GPL-2.0-only |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/zstd.rs`](../../../builder/src/package/definition/zstd.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/facebook/zstd/releases/download/v1.5.7/zstd-1.5.7.tar.gz |
| sha3-512 | `9fcbd59a61d336c34cd46ff6f335787848d37e8c6d0a4aef468755ef4d31fcce7368ce35650282b31d28303bc985470a97b06110b8fb050b0e72eba66afabca8` |
| blake3 | `730dca31244abd219e995f03a55d95b2cfb4b3e16cda055a79fa6f30a4f0e1db8febef4887bffd4b8c19d6926fca83c0ae68e6e5abf535c0c0a85193342d8568` |
| 展開ディレクトリ | `zstd-1.5.7` |

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
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。Makefile を直接呼び、クロスは `CC` / `TRIPLE` に任せます。

Autotools の `configure` スクリプトではなく、Makefile を直接呼び出します。

| 変数 | 設定値 | 条件 |
|-|-|-|
| `PREFIX` | `/usr` | 常時 |
| `LIBDIR` | `/usr/lib` | 常時 |
| `HAVE_ZLIB` | `0` | 常時 |
| `HAVE_LZMA` | `0` | 常時 |
| `HAVE_LZ4` | `0` | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 同梱ソースを連結した複数 MB 入力を `zstd -b{1,3,9,19} -T0` でベンチマーク |

CLI の .gz / .xz / .lz4 連携は無効化し、シード環境のライブラリを自動検出させません。`liblz4` 等を実行時依存として引き込まないためです。
