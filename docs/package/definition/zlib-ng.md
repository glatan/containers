# zlib-ng

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib-ng` |
| バージョン | 2.3.3 |
| ライセンス | Zlib |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が cmake を導入） |
| 定義 | [`builder/src/package/definition/zlib_ng.rs`](../../../builder/src/package/definition/zlib_ng.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/zlib-ng/zlib-ng/archive/refs/tags/2.3.3.tar.gz |
| sha3-512 | `b045a4d3358f73db5566951944186293b9a99c5bc9d395866bc0a539617b9584b1499d3d0e9a8943284a7b10a829902dd37e1d6ec55823e6203b000644ba0170` |
| blake3 | `103cd4e5aa69bb2d0383a242f231bccf64c8a79c21156e7c6187db3cebc1367dc975216cfa092b7d37cdfb303e1fc9cdb2b63d6ef59ce6dc662fbe89286ada42` |
| 展開ディレクトリ | `zlib-ng-2.3.3` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

openssl からは依存されません。標準 zlib の `libz.so` とは配置パスを分離して共存可能にします。

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が cmake を導入） |

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
