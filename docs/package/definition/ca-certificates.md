# ca-certificates

| 項目 | 値 |
|-|-|
| パッケージ名 | `ca-certificates` |
| バージョン | 2026.07.22 |
| ライセンス | MPL-2.0 |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/ca_certificates.rs`](../../../builder/src/package/definition/ca_certificates.rs) |

## ソース

Mozilla のルート CA 証明書バンドルとして、[certifi](https://github.com/certifi/python-certifi) のリリースターボールを固定して使用します。Python ランタイムは使用せず、アーカイブから `cacert.pem` のみを抽出してインストールします。

| 項目 | 値 |
|-|-|
| URL | https://github.com/certifi/python-certifi/archive/refs/tags/2026.07.22.tar.gz |
| sha3-512 | `1a708f7616f0e6e55159740c21a6af6d7207f40a39c3d04d1faccb848e544b2275c19160835c08f7a1127cd876242945b11147b6e1dceebe7db8cacbec7abbd7` |
| blake3 | `6c932fab86221709499401ce0ceddc997ad9b91d2f6c95e73493f18b90ddae45070a7a97681a866cce09f61c490d66e9e4ff2b2122f88918743ec98c34c63ad7` |
| 展開ディレクトリ | `python-certifi-2026.07.22` |

## 依存

### dependencies

なし。

### build-dependencies

なし（openssl からも本パッケージには依存しません）。

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるファイル配置 |

## ターゲット

成果物に含まれるファイル集合は architecture / variant に依存しないため、`flags()` は空のフラグを返します。`--all-targets` は `Target::all()` を組みます。

## ビルドフラグ

| フラグ | 設定値 | 条件 |
|-|-|-|
| `flags()` | 空 | 静的証明書ファイルのみ |
| LTO | 適用しない | コンパイルを伴わないため |
| PGO | 適用しない | コンパイルを伴わないため |

成果物として `/etc/ssl/certs/ca-certificates.crt` を配置し、`/etc/ssl/certs/ca-bundle.crt` および `/etc/ssl/cert.pem` からのシンボリックリンクを作成します。具体的な配置手順は定義ファイルの `recipe` メソッド（`Step::Install` と `Step::Symlink`）に記述しています。
