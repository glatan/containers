# x86_64-linux-gnu-glibc-headers

| 項目 | 値 |
|-|-|
| パッケージ名 | `x86_64-linux-gnu-glibc-headers` |
| バージョン | 2.44 |
| ライセンス | LGPL-2.1-or-later |
| パッケージイメージ | 公開対象外（ビルド専用中間ステージ） |
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
| [`x86_64-linux-gnu-gcc-stage1`](./x86_64-linux-gnu-gcc-stage1.md) | 16.2.0 | stage1 コンパイラ |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル |

## ビルド設定

ヘッダ、crt、スタブ libc.so。`supports` は HOST（amd64 / v4）のみです。
