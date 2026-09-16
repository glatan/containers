# findutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `findutils` |
| バージョン | 4.11.0 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/findutils.rs`](../../../builder/src/package/definition/findutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/findutils/findutils-4.11.0.tar.xz |
| sha3-512 | `0ffbc8f9fd502abc230fe2df5ebc68aa46297317a9c32eb024cdb044bf018c90bb6dbfeee030dfb5052cea60f8ecb062011b4554d63ea81ca76944406834d13c` |
| blake3 | `a043f0e36d43c1acbb773fb71bb05342c11f777a34cec2a4ed699c5e578281cd4672a4d409352085f08c1f5c587ddc995dd8322243d4c3a77bac293bad47782f` |
| 展開ディレクトリ | `findutils-4.11.0` |

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

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
