# xz

| 項目 | 値 |
|-|-|
| パッケージ名 | `xz` |
| バージョン | 5.8.4 |
| ライセンス | 0BSD AND LGPL-2.1-or-later AND GPL-2.0-or-later AND GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/xz.rs`](../../../builder/src/package/definition/xz.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/tukaani-project/xz/releases/download/v5.8.4/xz-5.8.4.tar.xz |
| sha3-512 | `56aff332bbc60da0cb0e075b20cf947f3a6ea2be500a4d05bd27b435c0a40005f90c9a4bed4124c625a374ba0b81473bdf0d7d89cbe381f5f5f36a7687f1e7c4` |
| blake3 | `4776832814549b37914485ee14979b68282d4ddf1c0b679c6f4e055a57666b96fcbdc36febaf229f670d823e253d1abcbbee9dd4163eec27106770a030609c05` |
| 展開ディレクトリ | `xz-5.8.4` |

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
| `--disable-static` | | 常時 |
