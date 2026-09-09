# ca-certificates

| 項目 | 値 |
|-|-|
| パッケージ名 | `ca-certificates` |
| バージョン | 2026.07.22 |
| ライセンス | MPL-2.0 |
| パッケージイメージ | する。[公開一覧](../packages.md) |
| ネットワーク | しない |
| 定義 | [`builder/src/package/definition/ca_certificates.rs`](../../../builder/src/package/definition/ca_certificates.rs) |

## ソース

Mozilla の CA バンドルを [certifi](https://github.com/certifi/python-certifi) のリリース tarball でピンします。Python は使いません。`cacert.pem` だけをインストールします。

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

なし。

openssl からは依存しません。

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../../package/README.md) | `package build` |

## ターゲット

ファイル集合は architecture / variant で変わりません。ビルドはフラグ解決を通るため、`-march` 未対応の組み合わせはスキップされます。表に無い組み合わせは [platforms.md](../platforms.md) にあり、未対応です。

| architecture | variant | `-march` | 状態 |
|-|-|-|-|
| amd64 | v1 | x86-64 | 対応 |
| amd64 | v2 | x86-64-v2 | 対応 |
| amd64 | v3 | x86-64-v3 | 対応 |
| amd64 | v4 | x86-64-v4 | 対応 |
| arm64 | v8 | armv8-a | 対応 |
| arm64 | v8.1 | armv8.1-a | 対応 |
| arm64 | v8.2 | armv8.2-a | 対応 |
| arm64 | v8.3 | armv8.3-a | 対応 |
| arm64 | v8.4 | armv8.4-a | 対応 |
| arm64 | v8.5 | armv8.5-a | 対応 |
| arm64 | v8.6 | armv8.6-a | 対応 |
| arm64 | v8.7 | armv8.7-a | 対応 |
| arm64 | v8.8 | armv8.8-a | 対応 |
| arm64 | v8.9 | armv8.9-a | 対応 |
| arm64 | v9 | armv9-a | 対応 |
| arm64 | v9.1 | armv9.1-a | 対応 |
| arm64 | v9.2 | armv9.2-a | 対応 |
| arm64 | v9.3 | armv9.3-a | 対応 |
| arm64 | v9.4 | armv9.4-a | 対応 |
| arm64 | v9.5 | armv9.5-a | 対応 |
| riscv64 | rva20u64 | rva20u64 | 対応 |
| riscv64 | rva22u64 | rva22u64 | 対応 |
| riscv64 | rva23u64 | rva23u64 | 対応 |

## ビルドフラグ

| フラグ | 値 | 条件 |
|-|-|-|
| LTO | しない | 証明書ファイルのみ |
| PGO | しない | 証明書ファイルのみ |

成果物は `/etc/ssl/certs/ca-certificates.crt` と、`/etc/ssl/certs/ca-bundle.crt`・`/etc/ssl/cert.pem` のシンボリックリンクです。手順は定義ファイルの `build_script` です。
