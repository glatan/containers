# x86_64-linux-gnu-glibc

| 項目 | 値 |
|-|-|
| パッケージ名 | `x86_64-linux-gnu-glibc` |
| バージョン | 2.44 |
| ライセンス | LGPL-2.1-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/toolchain.rs`](../../../builder/src/package/definition/toolchain.rs) |
| 共通方針 | [toolchain.md](../toolchain.md) |

## ソース

glibc 2.44。URL とハッシュは [toolchain.md](../toolchain.md) です。

## 依存

### dependencies

なし。

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | カーネルヘッダ |
| [`x86_64-linux-gnu-binutils`](./x86_64-linux-gnu-binutils.md) | 2.47 | アセンブラ / リンカ |
| [`x86_64-linux-gnu-gcc-stage2`](./x86_64-linux-gnu-gcc-stage2.md) | 16.2.0 | stage2 コンパイラ |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル |

## ビルド設定

実行時 libc です。`supports` は architecture `amd64` の variant（v1, v2, v3, v4）です。
