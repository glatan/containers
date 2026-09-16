# zlib

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib` |
| バージョン | 1.3.2 |
| ライセンス | Zlib |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（コンパイラ環境変数は builder が設定） |
| 定義 | [`builder/src/package/definition/zlib.rs`](../../../builder/src/package/definition/zlib.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.gz |
| sha3-512 | `860936cb0ddea209f9c14219dbf05efbef7ea83abe93a25bf39dd591a8f075ab5ad0c8225b1455def844d831f2d7b8f645704ebe387e630ad3075f765925dcc7` |
| blake3 | `ea79f8ae7c98d38ea360b70449fbb0052da4ff4a7201f03387113e5dab8543793587a6ee51d8307b5203aa03bf0a0053fca792f4c99d95c48aecc4b49359dc1b` |
| 展開ディレクトリ | `zlib-1.3.2` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

openssl からは依存されません。

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル（`CC` は builder が渡す） |

## ターゲット

`CFLAGS` の CPU 指定フラグは、builder が architecture および variant から決めます（ppc64le は `-mcpu=`、それ以外は `-march=`）。対象 Platform は `Target::all()` です。variant ごとの CPU 名は [platforms.md](../../image/platforms.md) です。

## ビルドフラグ

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--libdir` | `/usr/lib64` | 常時 |
| `--shared` | | 常時 |
| `flags()` の Cflag | openssl と同等だが `FormatSecurity` は付与しない | 書式付き出力処理がほぼ存在しないため |
| `flags()` の Ldflag | openssl と同等 | 依存関係グラフの末端ライブラリであるため |
| LTO | 適用する | builder のフラグ指定による |
| PGO | 適用しない | プロファイル手順を設けていないため |
