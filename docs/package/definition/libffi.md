# libffi

| 項目 | 値 |
|-|-|
| パッケージ名 | `libffi` |
| バージョン | 3.8.0 |
| ライセンス | MIT |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/libffi.rs`](../../../builder/src/package/definition/libffi.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/libffi/libffi/releases/download/v3.8.0/libffi-3.8.0.tar.gz |
| sha3-512 | `adad40a2b8be7e69762ba21e0d6d1a51da4ab55340ee2924cb9e74d292ee0fbcf9b9f52bc096c6d6ae285c39ca02fb12508add63e92c505efb0f7490483022ad` |
| blake3 | `df92f5720298710e22ffbfbb7d9a9969e90b1f95c01e02e6356a56de162b57ea6585c6fedb65ca37cf816e9d8f12dbadf706f75e266b04cdaab06ee4c270bb87` |
| 展開ディレクトリ | `libffi-3.8.0` |

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

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
