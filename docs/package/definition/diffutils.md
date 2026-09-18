# diffutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `diffutils` |
| バージョン | 3.12 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/diffutils.rs`](../../../builder/src/package/definition/diffutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/diffutils/diffutils-3.12.tar.xz |
| sha3-512 | `24fde52f69eebea2c5807bf3ab532dd5334ec6714203e9dbeca6cd71f7b31abc0fc1ebe8fa20a17aca8bac41b7b66344c092310bfe5af240fbf45f2e4e47e6fb` |
| blake3 | `363fb378c85505b43f91d6e7c7382d571f2f7b47e4d334c18d522baf55599d18a186e566dcd9cdcf7f3580edf8381ddfc7fd7c0b1100e20ee6ba6dd7f3e6e80c` |
| 展開ディレクトリ | `diffutils-3.12` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `diff -u` / `diff -r` で同梱ソースのコピーと比較 |
