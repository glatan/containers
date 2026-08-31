# builder

コンテナ/ツールのビルドツール(builder)の仕様書です。

## 概要

- ツールの取得 → ツールのビルド → イメージのビルドまでを行う。
- 複数バージョンに対応する。
- Rust で作成する。
- 依存 crate は必要最小限にする。
  - アーカイブ/圧縮は tar コマンドを利用する。

## コマンド

### tool — ツール管理

| コマンド | 説明 |
|-|-|
| `tool ls` | 一覧 |
| `tool build {?tool}` | ビルド |
| `tool fetch {?tool}` | ツールのソースコードを取得する |
| `tool info {?tool}` | ビルド設定の表示 |

#### tool build のオプション

- `--all-target`: すべての architecture / variant を対象にする
- `--architecture {architecture}`: 特定の architecture を対象にする(variant の指定が必須)
- `--variant {variant}`: 特定の variant を対象にする

#### tool fetch のオプション

- `--all`: すべてのツールを対象にする

### image — イメージ管理

| コマンド | 説明 |
|-|-|
| `image ls` | 一覧 |
| `image build {?image}` | ビルド |
| `image validate {?image}` | ビルド済みのイメージの検証 |

#### image build のオプション

- `--all`: すべてのイメージを対象にする
- `--with-tools`: tool のビルドも実行する

#### image validate のオプション

- `--all`: すべてのイメージを対象にする

## tool の定義

- ビルド設定は Rust コードで管理する。
- tool 間の依存管理も行う。
