# riscv64-linux-gnu-glibc-headers

| 項目 | 値 |
|-|-|
| パッケージ名 | `riscv64-linux-gnu-glibc-headers` |
| バージョン | 2.44 |
| ライセンス | LGPL-2.1-or-later |
| パッケージイメージ | 公開対象外（ビルド専用中間ステージ） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/riscv64_linux_gnu_glibc_headers.rs`](../../../builder/src/package/definition/riscv64_linux_gnu_glibc_headers.rs) |
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
| [`riscv64-linux-gnu-binutils`](./riscv64-linux-gnu-binutils.md) | 2.47 | アセンブラ / リンカ |
| [`riscv64-linux-gnu-gcc-stage1`](./riscv64-linux-gnu-gcc-stage1.md) | 16.2.0 | stage1 コンパイラ |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル。bison / python / m4 は host tool として optional mount（glibc→python→openssl→glibc の循環回避のため依存宣言には書かない） |

## ビルド設定

ヘッダ、crt、スタブ libc.so。`supports` は HOST（amd64 / v4）のみです。
