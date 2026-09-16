# expat

| 項目 | 値 |
|-|-|
| パッケージ名 | `expat` |
| バージョン | 2.8.4 |
| ライセンス | MIT |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/expat.rs`](../../../builder/src/package/definition/expat.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/libexpat/libexpat/releases/download/R_2_8_4/expat-2.8.4.tar.xz |
| sha3-512 | `056d6b25263d95d7375e2fc14f22f161722558c7e695ca3fccc73eb2842bf921e2791fcc1104fd52b0e057dd3be00795af698af5c70a9d8a257b5d6123ac75cc` |
| blake3 | `1311a1adc8da8ffd10ca65db96be3abe9e7e9d83561d899bd0dbdab17ab599dbddde8e8e3a1a6b9a3223c184a3bf1e71d767750418b304be8b22e8e209348f54` |
| 展開ディレクトリ | `expat-2.8.4` |

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

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
