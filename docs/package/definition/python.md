<!-- GENERATED -->
# python

| 項目 | 値 |
|-|-|
| パッケージ名 | `python` |
| バージョン | 3.14.7 |
| ライセンス | Python-2.0 |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/python.rs`](../../../builder/src/package/definition/python.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://www.python.org/ftp/python/3.14.7/Python-3.14.7.tar.xz |
| sha3-512 | `2da7cc048103dfd9f3681848e13fd3dfa06f25cc9afd76f89f772c1e3a8f14410ba44546754ee177672d0f86fac551bbd26685335830e316ca0e917629934420` |
| blake3 | `4406a4956bb3265c370b011383004cc2dcc94ed2c99b5f3ed67338eeab9e5eaceedabc538098da6772b361383e84ccf0274451296970e8d7c9b4261e9a30de59` |
| 展開ディレクトリ | `Python-3.14.7` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [libffi](./libffi.md) | 3.8.0 | — |
| [zlib](./zlib.md) | 1.3.2 | — |
| [openssl](./openssl.md) | 4.0.2 | — |
| [expat](./expat.md) | 2.8.4 | — |
| [sqlite](./sqlite.md) | 3.53.4 | — |
| [ncurses](./ncurses.md) | 6.6 | — |
| [readline](./readline.md) | 8.3 | — |
| [xz](./xz.md) | 5.8.4 | — |
| [bzip2](./bzip2.md) | 1.0.8 | — |
| [zstd](./zstd.md) | 1.5.7 | — |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [pkgconf](./pkgconf.md) | 2.5.1 | — |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



実行時はパッケージ Platform の `{triple}-glibc` に加え、libffi、zlib、openssl、expat、sqlite、ncurses、readline、xz、bzip2、zstd です。ビルド専用は linux-headers です。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。python は `Bootstrap` を実装せず、`build-env` 合成後に自前環境で組みます。openssl などの実行時依存は python のグラフが引きます。他パッケージがビルド時に実行する python は HOST 成果物をマウントします。

## ビルド設定

実行時依存の `/dep/{name}/usr/include`・`lib{,64}` を `CFLAGS` / `LDFLAGS` / `PKG_CONFIG_PATH` に足してから configure します。`$OUT/usr/bin/python` から `python3` へのシンボリックリンクを作成します。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--enable-shared` | | 常時 |
| `--enable-optimizations` | | ネイティブビルドのみ（`Pgo::Custom`。PROFILE_TASK で生成バイナリを実行する自己完結 PGO のため、クロスでは付けない） |
| `--without-ensurepip` | | 常時 |
