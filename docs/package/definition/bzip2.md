<!-- GENERATED -->
# bzip2

| 項目 | 値 |
|-|-|
| パッケージ名 | `bzip2` |
| バージョン | 1.0.8 |
| ライセンス | bzip2-1.0.6 |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/bzip2.rs`](../../../builder/src/package/definition/bzip2.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz |
| sha3-512 | `8192e0e7a92c083354d91e114f4e1a0571d56cc106f58a11c92b4028a1bb1ecad2e00892bb81cc1d2d2dd7371e35beebab2e92e2c83ef9ccb8a2490a97a94f52` |
| blake3 | `97af3f520629c65fe41292f77e6ca798fe594d7987bfb2aebe7c6fcdc7ab5ed25e9035d582b0a344afcd3ae378c3f3f999d66594bd1dd88cfcabf143cd70b8ef` |
| 展開ディレクトリ | `bzip2-1.0.8` |

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



## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。Makefile を直接呼び、クロスは `CC` に任せます。

ソースツリーを作業ディレクトリへコピーしたのち、共有ライブラリ用 Makefile を用いてビルドします。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `Makefile-libbz2_so` | | 共有ライブラリのビルド |
| `PREFIX` | `$OUT/usr` | `make install` 時のインストール先プレフィックス |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 同梱ソースを連結した複数 MB 入力を `bzip2` で `-1` / `-6` / `-9` の 3 レベルで圧縮・解凍し `bzip2recover` も実行。パッケージ固有の `CFLAGS`（`-fpic -fPIC -D_FILE_OFFSET_BITS=64` 等）と make 引数は両フェーズで replay |
