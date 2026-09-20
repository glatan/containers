# patch

| 項目 | 値 |
|-|-|
| パッケージ名 | `patch` |
| バージョン | 2.8 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/patch.rs`](../../../builder/src/package/definition/patch.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/patch/patch-2.8.tar.xz |
| sha3-512 | `d18e96a61a152e03a84d36384931fa3ce01f36240414b2af3bbd5c7ba02a665251cb8300ad2e4d9f7387238e0a717294928f8a30343c7a8647a80e66f543f9b5` |
| blake3 | `d7dadf0ac720de7560f140a9021928bc0f8089182121c0e65c8bec115cf311240545c913ea3be61b067c31dd4787ac6b2fe7da2d18b1f4047b1ff65a9e55fa60` |
| 展開ディレクトリ | `patch-2.8` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 同梱ソースへの `diff -u` 出力を `./src/patch` で dry-run / 適用 |
