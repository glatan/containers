<!-- GENERATED -->
# zlib

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib` |
| バージョン | 1.3.2 |
| ライセンス | Zlib |
| 分類 | カタログ |
| 定義 | [`builder/src/package/definition/zlib.rs`](../../../builder/src/package/definition/zlib.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/madler/zlib/releases/download/v1.3.2/zlib-1.3.2.tar.gz |
| sha3-512 | `860936cb0ddea209f9c14219dbf05efbef7ea83abe93a25bf39dd591a8f075ab5ad0c8225b1455def844d831f2d7b8f645704ebe387e630ad3075f765925dcc7` |
| blake3 | `ea79f8ae7c98d38ea360b70449fbb0052da4ff4a7201f03387113e5dab8543793587a6ee51d8307b5203aa03bf0a0053fca792f4c99d95c48aecc4b49359dc1b` |
| 展開ディレクトリ | `zlib-1.3.2` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



openssl からは依存されません。

## ターゲット

`CFLAGS` の CPU 指定フラグは、builder が architecture および variant から決めます（ppc64le は `-mcpu=`、それ以外は `-march=`）。対象 Platform は `Target::all()` です。variant ごとの CPU 名は [platforms.md](../../image/platforms.md) です。

## ビルドフラグ

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--libdir` | `/usr/lib64` | 常時 |
| `--shared` | | 常時 |
| `flags()` | `tool_flags(target)`（既定セット。[README.md](../README.md) の既定フラグ表） | 個別追加・除外はなし |
| `flags()` の Ldflag | openssl と同等 | 依存関係グラフの末端ライブラリであるため |
| LTO | 適用する | builder のフラグ指定による |
| PGO | ネイティブビルドのみ適用 | `make` の `CFLAGS` / `LDFLAGS` に `-fprofile-generate` / `-fprofile-use` を付与 |

PGO は生成バイナリを実行可能なネイティブビルド時のみ適用します（`Pgo::Custom`）。クロスビルドではプロファイル採取を実行できないため、`recipe(ctx)` が `ctx.cross()` で分岐して素の `make`（`CHOST=$TRIPLE` 付き `configure`）に切り替えます。ネイティブでは `-fprofile-generate` でビルド後、同梱ソース（`.c` / `.h` を 16 回連結した複数 MB 入力）を `minigzip` で `-1` / `-6` / `-9` の 3 レベルで圧縮・解凍し、`example` も実行して `make distclean` のうえ `-fprofile-use` で再ビルドします。
