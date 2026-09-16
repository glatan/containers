# flex

| 項目 | 値 |
|-|-|
| パッケージ名 | `flex` |
| バージョン | 2.6.4 |
| ライセンス | BSD-2-Clause |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/flex.rs`](../../../builder/src/package/definition/flex.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/westes/flex/files/981163/flex-2.6.4.tar.gz |
| sha3-512 | `eac223beda4c1629dccfd29666eab9f510e05f5b606587496364ae59622c3bab8104a9ac52f4bb06098a628a4d4fba0c59f5a9d851d65cbc94ed0a660475e162` |
| blake3 | `d78b714ac38bd9de7f9b44a078efed82e96ed43e7cf9cd33944a7f379a2d09a4d024e6d3a3accb71ae72f36b4f028328f551c6271a231cdd3c307b542cdca143` |
| 展開ディレクトリ | `flex-2.6.4` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [m4](./m4.md) | 1.4.21 | マクロプロセッサ |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル（シード環境では builder が `gcc` `make` `m4` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
