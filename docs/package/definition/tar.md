# tar

| 項目 | 値 |
|-|-|
| パッケージ名 | `tar` |
| バージョン | 1.35 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/tar.rs`](../../../builder/src/package/definition/tar.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/tar/tar-1.35.tar.xz |
| sha3-512 | `d075aad9bfe5bde143c87388e3b4995775f0b40016e851d1243cc3b6a59fd65c5eeb8a23624ee41b18999dbd850213cbff7ebfb256e85c3a8441c3aa2cd7747c` |
| blake3 | `55e3853752c56a607ae5dbc9e9190fe5d3f7599b76bb80abe0edba36d712c36bb60b803541ce46293cdf80cb80b8c84fd6940ca9c446fd5db3b04c7ee16664bc` |
| 展開ディレクトリ | `tar-1.35` |

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
| `--disable-acl` | | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `tar` で同梱ソースのアーカイブ作成・一覧・展開を往復 |
