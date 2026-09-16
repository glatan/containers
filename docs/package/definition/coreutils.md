# coreutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `coreutils` |
| バージョン | 9.12 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/coreutils.rs`](../../../builder/src/package/definition/coreutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/coreutils/coreutils-9.12.tar.xz |
| sha3-512 | `d6f1fbd33b86bce8c0eec25875c04192e70a5a946561e2ed5a8fd9a4214e309cb07e0cc4ce1f86b228beeedc4f7fbf1c1218bd5dd3a45a308edd1fefd4e4d82b` |
| blake3 | `420edafa8b231145442d248c00412727651f20435e85dc80fbf18074fc249624c6ff75aaed836074f668767844667fe0ccc3ffa59975cb5bda8e29348dcb2779` |
| 展開ディレクトリ | `coreutils-9.12` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [gmp](./gmp.md) | 6.3.0 | 多倍長演算サポート |

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
