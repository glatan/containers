# pcre2

| 項目 | 値 |
|-|-|
| パッケージ名 | `pcre2` |
| バージョン | 10.48 |
| ライセンス | BSD-3-Clause |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/pcre2.rs`](../../../builder/src/package/definition/pcre2.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/PCRE2Project/pcre2/releases/download/pcre2-10.48/pcre2-10.48.tar.bz2 |
| sha3-512 | `445ca5ab94350b9c4307405350758e0c148c92160329b0443159274ce12bae0f437a1d788fb566d7731081553a2a31cc9915055e38e4f97ae17b834d7a1c17e6` |
| blake3 | `5e84ce242f146b8a97ce10e3cf31ac4c2c2b23d67814fa1cf266636d0545f3a50f08634ee1a724793b20cdae76647113f56f13a814328e8c3de48a837168ca45` |
| 展開ディレクトリ | `pcre2-10.48` |

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
| `--enable-pcre2-16` | | 常時 |
| `--enable-pcre2-32` | | 常時 |
| `--enable-jit` | | 常時 |
