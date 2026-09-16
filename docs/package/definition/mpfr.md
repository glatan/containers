# mpfr

| 項目 | 値 |
|-|-|
| パッケージ名 | `mpfr` |
| バージョン | 4.2.2 |
| ライセンス | LGPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/mpfr.rs`](../../../builder/src/package/definition/mpfr.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/mpfr/mpfr-4.2.2.tar.xz |
| sha3-512 | `92d8b3f8b16cdb1130e59717e2c8aa8b249070bd34675000af73327b60043358b4238f94cf9c37b97fa8323ec823d518977746afb92573558ce1d40d6121b406` |
| blake3 | `11d59d061ef8db588650bc7dc5172594a6e5aad013994801c6f63011a62b191dd94685e03abb7e145f97b51c40f75ae11795c1ab6112fc4cf82d638bb68c0588` |
| 展開ディレクトリ | `mpfr-4.2.2` |

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
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
