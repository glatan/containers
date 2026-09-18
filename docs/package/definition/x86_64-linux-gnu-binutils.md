# x86_64-linux-gnu-binutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `x86_64-linux-gnu-binutils` |
| バージョン | 2.47 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/x86_64_linux_gnu_binutils.rs`](../../../builder/src/package/definition/x86_64_linux_gnu_binutils.rs) |
| 共通方針 | [toolchain.md](../toolchain.md) |

## ソース

binutils 2.47。URL とハッシュは [toolchain.md](../toolchain.md) です。

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [`x86_64-linux-gnu-glibc`](./x86_64-linux-gnu-glibc.md) | 2.44 | 実行時 libc。seed では Bootstrap グラフの実行時依存は空（glibc より先に組む） |
| [zlib](./zlib.md) | 1.3.2 | 圧縮 |
| [zstd](./zstd.md) | 1.5.7 | 圧縮 |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | カーネルヘッダ |
| [zlib](./zlib.md) | 1.3.2 | seed 完了後 |
| [zstd](./zstd.md) | 1.5.7 | seed 完了後 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル |

## ビルド設定

configure は `--build=$HOST_TRIPLE --host=$PACKAGE_HOST --target=x86_64-linux-gnu` です。ステージ順とプレフィックスは [toolchain.md](../toolchain.md) です。ネイティブ（`$PACKAGE_HOST = $HOST_TRIPLE`）では PGO を適用します（`Pgo::Custom`。instrumented ツリーで `as` / `ld` / `objdump` を駆動してプロファイルを採取）。クロスでは適用しません。

`supports` は `Target::all()` です。製品キューは [toolchain.md](../toolchain.md)。このパッケージでは HOST と、architecture `amd64` の variant（v1, v2, v3, v4）です。
