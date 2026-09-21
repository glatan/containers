# python

| 項目 | 値 |
|-|-|
| パッケージ名 | `python` |
| バージョン | 3.14.7 |
| ライセンス | Python-2.0 |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/python.rs`](../../../builder/src/package/definition/python.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://www.python.org/ftp/python/3.14.7/Python-3.14.7.tar.xz |
| sha3-512 | `2da7cc048103dfd9f3681848e13fd3dfa06f25cc9afd76f89f772c1e3a8f14410ba44546754ee177672d0f86fac551bbd26685335830e316ca0e917629934420` |
| blake3 | `4406a4956bb3265c370b011383004cc2dcc94ed2c99b5f3ed67338eeab9e5eaceedabc538098da6772b361383e84ccf0274451296970e8d7c9b4261e9a30de59` |
| 展開ディレクトリ | `Python-3.14.7` |

## 依存

実行時はパッケージ Platform の `{triple}-glibc` に加え、libffi、zlib、openssl、expat、sqlite、ncurses、readline、xz、bzip2、zstd です。ビルド専用は linux-headers です。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。python は `Bootstrap` を実装せず、`build-env` 合成後に自前環境で組みます。openssl などの実行時依存は python のグラフが引きます。他パッケージがビルド時に実行する python は HOST 成果物をマウントします。

## ビルド設定

実行時依存の `/dep/{name}/usr/include`・`lib{,64}` を `CFLAGS` / `LDFLAGS` / `PKG_CONFIG_PATH` に足してから configure します。`$OUT/usr/bin/python` から `python3` へのシンボリックリンクを作成します。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--enable-shared` | | 常時 |
| `--enable-optimizations` | | ネイティブビルドのみ（`Pgo::Custom`。PROFILE_TASK で生成バイナリを実行する自己完結 PGO のため、クロスでは付けない） |
| `--without-ensurepip` | | 常時 |
