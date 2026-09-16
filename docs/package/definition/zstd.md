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
| sha3-512 | `3f24bf7c770c8f94676443cfa11075ffcb3a70e7e1f486a4e392cae6a0d2fc41071d79730f576e3e5898cfd145cbeae11ff9203a9876f1e8feaeeb0429f6dd85` |
| blake3 | `b18c64cfb92d6e3c091931326cbb0bf7817eb253b811fb2d86a076717f903740e53a2a46c37c227eb058444a956ae53d532f146a782b1bdf1e8784c6ef4931a2` |
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
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

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

CLI の .gz / .xz / .lz4 連携は無効化し、シード環境のライブラリを自動検出させません。`liblz4` 等を実行時依存として引き込まないためです。
