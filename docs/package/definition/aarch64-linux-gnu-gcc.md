# aarch64-linux-gnu-gcc

| 項目 | 値 |
|-|-|
| パッケージ名 | `aarch64-linux-gnu-gcc` |
| バージョン | 16.2.0 |
| ライセンス | GPL-3.0-or-later WITH GCC-exception-3.1 |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/toolchain.rs`](../../../builder/src/package/definition/toolchain.rs) |
| 共通方針 | [toolchain.md](../toolchain.md) |

## ソース

gcc 16.2.0。URL とハッシュは [toolchain.md](../toolchain.md) です。

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [`aarch64-linux-gnu-glibc`](./aarch64-linux-gnu-glibc.md) | 2.44 | 実行時 libc。成果物には焼き付けない |
| [gmp](./gmp.md) | 6.3.0 | 多倍長整数 |
| [mpfr](./mpfr.md) | 4.2.2 | 多倍長浮動小数 |
| [mpc](./mpc.md) | 1.4.1 | 複素数 |
| [isl](./isl.md) | 0.26 | 整数集合 |
| [zlib](./zlib.md) | 1.3.2 | 圧縮 |
| [zstd](./zstd.md) | 1.5.7 | 圧縮 |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | カーネルヘッダ |
| [`aarch64-linux-gnu-binutils`](./aarch64-linux-gnu-binutils.md) | 2.47 | アセンブラ / リンカ |
| [`aarch64-linux-gnu-glibc`](./aarch64-linux-gnu-glibc.md) | 2.44 | sysroot |
| [bison](./bison.md) | 3.8.2 | パーサ生成 |
| [flex](./flex.md) | 2.6.4 | 字句解析 |
| [perl](./perl.md) | 5.42.3 | テスト / 生成 |
| [texinfo](./texinfo.md) | 7.3 | info |
| [m4](./m4.md) | 1.4.21 | マクロ |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル |

## ビルド設定

C/C++、`--disable-bootstrap`。gcc フラグ `--with-arch=armv8-a`。PGO と sanitizer は [toolchain.md](../toolchain.md) です。configure は `--build=$HOST_TRIPLE --host=$PACKAGE_HOST --target=aarch64-linux-gnu` です。

`supports` は `Target::all()` です。製品キューは [toolchain.md](../toolchain.md)。このパッケージでは HOST と、architecture `arm64` の variant（v8 から v9.5）です。
