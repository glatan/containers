# 実装構成

builder(パッケージとパッケージイメージのビルドツール)の構成です。
コマンド仕様は [README.md](./README.md) を参照してください。

## 概要

- Rust 1.98.1(edition 2024)で実装します。toolchain はリポジトリルートの `rust-toolchain.toml` で固定します。
- バイナリ 1 つ(`containers-builder`)で、`package` / `image` の 2 系統のサブコマンドを提供します。ディレクトリ名は `builder/` です。
- 実行はリポジトリルートで行い、作業ディレクトリ(`packages/`・`oci-layouts/`)を相対パスで参照します。

## モジュール

| モジュール | 役割 |
|-|-|
| `main.rs` | CLI の定義と exit code の処理 |
| `error.rs` / `process.rs` | 型付きエラーと外部コマンド実行 |
| `package/mod.rs` | `Package` trait、共通 struct、`package` サブコマンド |
| `package/definition/` | パッケージ定義と一覧 |
| `package/{fetch,build}.rs` | `package fetch / build` の共通ワークフロー |
| `package/env.rs` | ビルド環境イメージ(`archlinux:base-devel` の digest ピン) |
| `image.rs` | `image ls / build / validate` |
| `oci/` | OCI Image Layout の生成(`model` / `layout` / `layer`) |
| `validate.rs` | 宣言依存と NEEDED の整合性検証(対象はパッケージ) |
| `sbom.rs` | SPDX 2.3 JSON の生成 |
| `util.rs` | OCI 用 sha256、ソース検証用 sha3-512 / blake3(512 bit)、RFC 3339 タイムスタンプ |

`lib.rs` は各モジュールの公開です。
OCI 準拠のテストは `tests/oci_spec.rs`、CLI 契約は `tests/cli.rs` にあります。

## 依存 crate

直接依存は `builder/Cargo.toml` で `=` によりバージョンを完全固定します。

| crate | 用途 |
|-|-|
| `clap` | CLI 定義 |
| `sha2` | OCI blob / DiffID の sha256 |
| `sha3` | パッケージソースの SHA3-512 |
| `blake3` | パッケージソースの BLAKE3(出力 512 bit) |
| `serde` / `serde_json` | OCI / SPDX の JSON |
| `thiserror` | 型付きエラー |
| `tempfile` | tar/zstd と staging layout の一時ディレクトリ |
| `jiff` | RFC 3339 の作成時刻 |

## 外部コマンド

| コマンド | 用途 |
|-|-|
| `curl` | ソースの取得(package fetch) |
| `tar` | ソースの展開、レイヤー tarball の生成(`--sort=name` 等で再現性を確保) |
| `zstd` | レイヤーの圧縮(`--single-thread`。通常はレベル 3、`--publish` は 22。20〜22 は `--ultra`) |
| `podman` | ビルド環境イメージ上でのパッケージビルド |
| `/bin/bash` | パッケージ毎のビルドスクリプトの実行 |
| `readelf` | 動的リンク(NEEDED)の解析(validate) |

## データフロー

```
package fetch      ソース取得 + sha3-512 / blake3 検証 + 展開
    ↓                 packages/sources/{package}/
package build      podman run ビルド環境イメージでビルドスクリプトを実行
    ↓                 packages/artifacts/{package}-{version}-{arch}-{variant}/
image build        パッケージをレイヤーとして Image Layout を生成 + SBOM
    ↓                 oci-layouts/{package}-{version}/
image validate     宣言依存とパッケージ内 NEEDED の整合性チェック
```

ビルドスクリプトには環境変数で `SOURCE`(ソース)、`BUILD`(ビルドディレクトリ)、`OUT`(パッケージの出力先)、`ARCH` / `VARIANT` / `CFLAGS` が渡されます。
依存パッケージは `/dep/{パッケージ名}` に読み取り専用でマウントされます(`dependencies` と `build-dependencies`。glibc の sysroot 等に使用)。
LTO と PGO は実施するのが基本です。しない場合は各パッケージのビルド設定に理由を書きます。

ビルド専用のパッケージ定義(`build_only`)はパッケージイメージにしません。`build-dependencies` にだけ現れます。
通常は `--network=none` でビルドしますが、ビルド時にネットワークが必要なパッケージ(`network`)は例外的に許可します。

新規パッケージ定義は `package/definition/{name}.rs` に `Package` を実装し、`package/definition/mod.rs` の一覧へ登録します。
文書は [`docs/image/packages/{name}.md`](../image/packages/README.md) にビルド設定を書きます。
モジュール名の `linux_headers.rs` はパッケージ名 `linux-headers` に対応します。

## クロスビルド

ビルドホストは amd64 で、非 amd64 のターゲットはビルド環境イメージ上にクロスツールチェーンを pacman で導入してビルドします(例: `aarch64-linux-gnu-gcc`)。

- ツールは architecture から target triple(`TRIPLE`)とクロス接頭辞(`CROSS`)を計算し、スクリプトに渡します。
- クロスビルドした glibc には、クロス gcc の期待するディレクトリ構造に合わせて `usr/{triple}/{include,lib,lib64}` のシンボリックリンクを作成します。
- クロス gcc の `-L` には sysroot が適用されないため、`libc.so` のリンカスクリプトを相対パスに書き換え、依存パッケージ側では `-B` / `-L` で自前 glibc のディレクトリを優先させます。

## レイヤーとメタデータ

- 1 パッケージ = 1 レイヤー。`packages/artifacts/` を tar+zstd して blob 化します。
- DiffID(未圧縮 tar の digest)がレイヤー内容の識別子です。zstd のレベルは layer digest にだけ影響します。
- 通常はレベル 3、`--publish` は 22。`oci-layouts/` はパッケージからの派生物なので、圧縮レベル違いを両方残しません。
- Layout は staging ディレクトリに完成させてから置換し、失敗時に既存 `oci-layouts/` を残します。
- レイヤーのハッシュ計算はストリーミングで行い、メモリに全体を保持しません。
- SBOM は Layout 内の `sbom.spdx.json` です。referrers への attestation は未実装です。Image Layout は未知ファイルをエラーにしてはならないため、この配置は仕様に反しません。

## 現状の制約

- ビルドできる architecture / variant は amd64 v1〜v4、arm64 の全 variant、riscv64 の全 variant です。ppc64le / s390x は Arch にクロス gcc が無く、`--all-target` 時にスキップされます。
- `--with-packages` はパッケージビルドを連鎖実行するのみで、無いターゲットはスキップして警告します。
