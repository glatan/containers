<!-- GENERATED -->
# zlib-ng

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib-ng` |
| バージョン | 2.3.3 |
| ライセンス | Zlib |
| 分類 | カタログ |
| 定義 | [`builder/src/package/definition/zlib_ng.rs`](../../../builder/src/package/definition/zlib_ng.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/zlib-ng/zlib-ng/archive/refs/tags/2.3.3.tar.gz |
| sha3-512 | `b045a4d3358f73db5566951944186293b9a99c5bc9d395866bc0a539617b9584b1499d3d0e9a8943284a7b10a829902dd37e1d6ec55823e6203b000644ba0170` |
| blake3 | `103cd4e5aa69bb2d0383a242f231bccf64c8a79c21156e7c6187db3cebc1367dc975216cfa092b7d37cdfb303e1fc9cdb2b63d6ef59ce6dc662fbe89286ada42` |
| 展開ディレクトリ | `zlib-ng-2.3.3` |

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
| [cmake](./cmake.md) | 4.4.3 | — |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



openssl からは依存されません。標準 zlib の `libz.so` とは配置パスを分離して共存可能にします。

## ターゲット

`CFLAGS` の CPU 指定フラグは、builder が architecture および variant から決めます（ppc64le は `-mcpu=`、それ以外は `-march=`）。対象 Platform は `Target::all()` です。variant ごとの CPU 名は [platforms.md](../../image/platforms.md) です。

## ビルドフラグ

| フラグ | 設定値 | 条件 |
|-|-|-|
| `CMAKE_INSTALL_PREFIX` | `/usr` | 常時 |
| `CMAKE_INSTALL_LIBDIR` | `lib64` | 常時 |
| `CMAKE_SYSTEM_NAME` | `Linux` | `$TRIPLE` が `$HOST_TRIPLE` と異なるとき |
| `ZLIB_COMPAT` | `OFF` | 常時（標準 zlib との競合を防ぐため `libz.so` は出力しない） |
| `BUILD_TESTING` | `OFF`（PGO instrumented ツリーでは `ON`） | 常時（テスト用バイナリのビルドを無効化） |
| `WITH_GTEST` | `OFF` | 常時 |
| `flags()` | `tool_flags(target)`（既定セット。[README.md](../README.md) の既定フラグ表） | 個別追加・除外はなし |
| LTO | 適用する | builder のフラグ指定による |
| PGO | ネイティブビルドのみ適用（`Pgo::Custom`） | 別ツリー `$BUILD/gen` で `-fprofile-generate` の instrumented ビルドを組み、同梱ソースを 16 回連結した複数 MB 入力を `minideflate` で `-1` / `-6` / `-9` の 3 レベルで圧縮・解凍してプロファイルを採取。`$BUILD` で `-fprofile-use` の成果物を組む |
