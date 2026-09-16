# grep

| 項目 | 値 |
|-|-|
| パッケージ名 | `grep` |
| バージョン | 3.12 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/grep.rs`](../../../builder/src/package/definition/grep.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/grep/grep-3.12.tar.xz |
| sha3-512 | `226b3cd6aeddf50e0f01581c0829033d6a75e4d9ab101252dabbd3fbfc17f4f6c0ce9ec9381bbd00ef6c902f0c7773a24358bf63e37f3112221652d060ef6e1e` |
| blake3 | `e15c2e03ff6e9e7e8fb7c726539e10b6ff17ecfc3d6961f7419e948d31043f4f953d1acd1988861931ee051d8764723d82bb4c01cd5ee2890d2e9ff7d0e88c16` |
| 展開ディレクトリ | `grep-3.12` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [pcre2](./pcre2.md) | 10.48 | 正規表現処理（Perl 互換正規表現） |

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
