# bzip2

| 項目 | 値 |
|-|-|
| パッケージ名 | `bzip2` |
| バージョン | 1.0.8 |
| ライセンス | bzip2-1.0.6 |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/bzip2.rs`](../../../builder/src/package/definition/bzip2.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz |
| sha3-512 | `8192e0e7a92c083354d91e114f4e1a0571d56cc106f58a11c92b4028a1bb1ecad2e00892bb81cc1d2d2dd7371e35beebab2e92e2c83ef9ccb8a2490a97a94f52` |
| blake3 | `97af3f520629c65fe41292f77e6ca798fe594d7987bfb2aebe7c6fcdc7ab5ed25e9035d582b0a344afcd3ae378c3f3f999d66594bd1dd88cfcabf143cd70b8ef` |
| 展開ディレクトリ | `bzip2-1.0.8` |

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
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。Makefile を直接呼び、クロスは `CC` に任せます。

ソースツリーを作業ディレクトリへコピーしたのち、共有ライブラリ用 Makefile を用いてビルドします。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `Makefile-libbz2_so` | | 共有ライブラリのビルド |
| `PREFIX` | `$OUT/usr` | `make install` 時のインストール先プレフィックス |
