# s390x-linux-gnu-gcc-stage2

| 項目 | 値 |
|-|-|
| パッケージ名 | `s390x-linux-gnu-gcc-stage2` |
| バージョン | 16.2.0 |
| ライセンス | GPL-3.0-or-later WITH GCC-exception-3.1 |
| パッケージイメージ | 公開対象外（ビルド専用中間ステージ） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/toolchain.rs`](../../../builder/src/package/definition/toolchain.rs) |
| 共通方針 | [toolchain.md](../toolchain.md) |

## ソース

gcc 16.2.0。URL とハッシュは [toolchain.md](../toolchain.md) です。

## 依存

### dependencies

なし。

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | カーネルヘッダ |
| [`s390x-linux-gnu-binutils`](./s390x-linux-gnu-binutils.md) | 2.47 | アセンブラ / リンカ |
| [s390x-linux-gnu-glibc-headers](./s390x-linux-gnu-glibc-headers.md) | 2.44 | スタブ sysroot |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル |

## ビルド設定

C のみ、共有 libgcc_s。プレフィックスは `/usr/lib/cross/s390x-linux-gnu/stage2/`。`supports` は HOST（amd64 / v4）のみです。gcc フラグ `--with-arch=z900`。
