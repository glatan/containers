# builder

パッケージとパッケージイメージのビルドツールの仕様です。
実装の構成は [implementation.md](./implementation.md) を参照してください。
生成する Layout の OCI 準拠は [oci](../oci/README.md) を参照してください。

バイナリ名は `containers-builder` です。リポジトリ内のディレクトリは `builder/` です。

## 概要

ビルドツールは役割で 2 種類に分けます。

### パッケージ開発者向け(本バイナリ)

- ソース取得 → パッケージのビルド → パッケージイメージの Layout 生成 → SBOM → 公開。
- 複数バージョンに対応する。
- 依存 crate は必要最小限にする。直接依存のバージョンは `Cargo.toml` で `=` 固定する。
  - アーカイブは tar、レイヤー圧縮は zstd コマンドを利用する。

### ユーザー向け

- パッケージの選択・依存解決・出力イメージの生成を行う。
- 実装はパッケージが増えてから開始する。方針は [パッケージイメージ](../image/README.md) の出力イメージです。
- 本リポジトリ内の別バイナリとして実装する。
- 依存情報は公開済みパッケージイメージの SBOM / アノテーションから取得する。

## コマンド

引数に取る名前はパッケージ名です(`glibc` など)。

### package — パッケージ定義の操作

| コマンド | 説明 |
|-|-|
| `package ls` | 一覧 |
| `package build {?package}` | パッケージを `packages/artifacts/` にビルドする |
| `package fetch {?package}` | ソースを取得する |
| `package info {?package}` | パッケージ定義の表示 |
| `package env` | ビルド環境イメージの参照(digest ピン) |

#### package build のオプション

- `--all-target`: カタログ上のすべての architecture / variant を対象にする(未対応はスキップ)。`--architecture` / `--variant` と同時には指定しない
- `--architecture {architecture}`: 特定の architecture を対象にする(variant の指定が必須)
- `--variant {variant}`: 特定の variant を対象にする

#### package fetch のオプション

- `--all`: すべてのパッケージ定義を対象にする

### image — パッケージイメージ

| コマンド | 説明 |
|-|-|
| `image ls` | 公開対象の一覧 |
| `image build {?package}` | パッケージから Layout を生成する(SBOM を含む) |
| `image validate {?package}` | 宣言依存とパッケージ内 ELF の動的リンクを静的に検証する |

`image validate` は `oci-layouts/` を見ません。`packages/artifacts/` を見ます。

#### image build のオプション

- `--all`: 公開対象のパッケージイメージすべて
- `--with-packages`: package のビルドも実行する
- `--publish`: 公開用に zstd レベル 22 で圧縮する
- `--zstd-level {1-22}`: 圧縮レベルを直接指定する(指定なしは 3。`--publish` と同時指定は不可)

#### image validate のオプション

- `--all`: 公開対象すべて(ビルド専用は含めない)

## パッケージ定義

- 実装の正本は Rust の `Package` trait です。パッケージ毎のコードは `builder/src/package/definition/{name}.rs`、文書は [image/packages/](../image/packages/README.md) です。
- 共通のビルド環境は [package](../package/README.md) です。
- 依存はパッケージイメージには含めません。実行時依存は `dependencies`、ビルド時だけ必要なものは `build-dependencies` です。
- SPDX SBOM を Layout 内の `sbom.spdx.json` に書く。attestation としての公開は未実装です。
