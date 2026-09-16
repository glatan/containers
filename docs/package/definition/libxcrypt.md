# libxcrypt

| 項目 | 値 |
|-|-|
| パッケージ名 | `libxcrypt` |
| バージョン | 4.5.2 |
| ライセンス | LGPL-2.1-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/libxcrypt.rs`](../../../builder/src/package/definition/libxcrypt.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/besser82/libxcrypt/releases/download/v4.5.2/libxcrypt-4.5.2.tar.xz |
| sha3-512 | `7c5eeccd7d7706048d73109937aba4624d9641f22965b154ba23ec8373c6c2bc939d182704c214f7a1a8d7600f5d0f61720d4e4a57913ab500d050d647a3ffe8` |
| blake3 | `8294e7735d240876f4c4c38a1370e1e9e9e2da4856fcfcb81168ea20ab83f33d1cc591380e2f0c00eb45ff4ac999b47c54d0d20b05b29757b1d147559227a9c9` |
| 展開ディレクトリ | `libxcrypt-4.5.2` |

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
| `--disable-werror` | | 常時 |
