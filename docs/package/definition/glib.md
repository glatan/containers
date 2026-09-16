# glib

| 項目 | 値 |
|-|-|
| パッケージ名 | `glib` |
| バージョン | 2.90.0 |
| ライセンス | LGPL-2.1-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/glib.rs`](../../../builder/src/package/definition/glib.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://download.gnome.org/sources/glib/2.90/glib-2.90.0.tar.xz |
| sha3-512 | `c896df2b43beb87254843d355e2fc389d9d62e01a49c58e078f3736286c0f0f16eedcfb611b19edb13ebf38ec8714ab1639b7b52b1ad02301eb1365cddaa205d` |
| blake3 | `0cd33e9b83b0fbc9b8396e99a59504909ced4e2f233f445e6127dacbcf17c0d00cc425186477da98e7f37cacfc8a27f8957f28b6b1cbd94ba1e98deef2570866` |
| 展開ディレクトリ | `glib-2.90.0` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [libffi](./libffi.md) | 3.8.0 | 外部関数インターフェース（FFI） |
| [pcre2](./pcre2.md) | 10.48 | 正規表現処理 |
| [zlib](./zlib.md) | 1.3.2 | データ圧縮処理 |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（実行する meson / ninja / pkgconf / python は HOST 成果物をマウント） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。クロス時は meson の `--cross-file` を付けます。

meson 実行前に、ホスト用 glibc の `--sysroot` オプションおよび linux-headers の `-idirafter` オプションを付与します。

| フラグ | 設定値 | 条件 |
|-|-|-|
| meson `--prefix` | `/usr` | 常時 |
| `-Dlibmount` | `disabled` | 常時 |
| `-Dnls` | `disabled` | 常時 |
| `-Dtests` | `false` | 常時 |
| `-Dsysprof` | `disabled` | 常時（`glib-2.0.pc` が外部の `sysprof-capture-4` を要求しないようにする） |
| LTO | 適用する | 共通前処理を適用しないため、既定フラグの `-flto` を維持 |
