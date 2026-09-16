# patch

| 項目 | 値 |
|-|-|
| パッケージ名 | `patch` |
| バージョン | 2.7.6 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/patch.rs`](../../../builder/src/package/definition/patch.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/patch/patch-2.7.6.tar.xz |
| sha3-512 | `60fa793d5f3066e133ebc36ea3a58cfbcad4d49d9773b06a4b162590fc36f1ee187d904791ee7e289bf53b95c3b17798725ee19f5647530c1157fdfb1758c56e` |
| blake3 | `bfa9d0121172f37c35fa0a7a0808a3d54c1538318855476a267e891c9fa6a88b8fa213f508cb32fbc13fa2a3c7499d7990141e9c74577f86f7b9492167d48384` |
| 展開ディレクトリ | `patch-2.7.6` |

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
