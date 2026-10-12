# ドキュメント

このディレクトリには、読者ごとに分離したドキュメントを格納します。
本ドキュメント群のライセンスは GNU Free Documentation License 1.3 です（[LICENSE](./LICENSE) を参照）。

## 利用者向け（[user/](./user/)）

公開済みパッケージイメージを利用する側（消費側リポジトリで `containers-builder sync` を実行し、Dockerfile をビルドする側）向けの文書です。

| パス | 内容 |
|-|-|
| [user/consume.md](./user/consume.md) | `containers.json` と sync の利用手順 |
| [user/requirements.md](./user/requirements.md) | 利用側の環境要件 |
| [user/limitations.md](./user/limitations.md) | 利用側の機能制限 |
| [user/naming.md](./user/naming.md) | レジストリパスとタグポリシー |
| [user/published.md](./user/published.md) | 公開するパッケージイメージの一覧 |
| [user/targets.md](./user/targets.md) | 対象 CPU プラットフォームの一覧 |

## 開発者向け（[dev/](./dev/)）

本リポジトリでパッケージ定義・builder・公開運用を開発・保守する側向けの文書です。

| パス | 内容 |
|-|-|
| [dev/README.md](./dev/README.md) | 語彙と正本文書の一覧 |
| [dev/distribution.md](./dev/distribution.md) | 公開側（index / push）の要件と制限 |
| [dev/oci.md](./dev/oci.md) | 生成する Image Layout が準拠する OCI 仕様 |
| [dev/image/](./dev/image/README.md) | パッケージイメージのモデルと配信方針 |
| [dev/package/](./dev/package/README.md) | パッケージ定義と共通ビルド方針 |
| [dev/builder/](./dev/builder/cli.md) | ビルドツールのコマンド仕様と内部構成 |
