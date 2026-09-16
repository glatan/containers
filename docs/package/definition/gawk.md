# gawk

| 項目 | 値 |
|-|-|
| パッケージ名 | `gawk` |
| バージョン | 5.4.1 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/gawk.rs`](../../../builder/src/package/definition/gawk.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gawk/gawk-5.4.1.tar.xz |
| sha3-512 | `886ec44952ec9f11ff0eeeeb113479d85d8763a91c83f55f5df7c7e96e71f4f9f20c37f478de6914ed456ff9c1cbc9c19b1a381c5959976d0cfa9b085677a60a` |
| blake3 | `d2f64ff7d253202c500da1d6c8b33b5f61693385d81cb5126ff546940872e3bcca174d16646dccd8a64da59a5c92123c265a4176d8efbafa414f8dc61bfdd81b` |
| 展開ディレクトリ | `gawk-5.4.1` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [gmp](./gmp.md) | 6.3.0 | 多倍長演算サポート |
| [mpfr](./mpfr.md) | 4.2.2 | 多倍長浮動小数点演算サポート |
| [readline](./readline.md) | 8.3 | コマンドライン行編集 |

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
