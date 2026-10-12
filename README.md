# containers

本プロジェクトは、パッケージおよびパッケージイメージのビルド設定と、そのビルドツールを管理します。
成果物の公開単位はパッケージイメージであり、GitHub Container Registry（GHCR）へ配信します。
準拠する OCI 仕様および実装要件については [docs/dev/oci.md](./docs/dev/oci.md) を参照してください。

## ディレクトリ構造

```
|- docs          ドキュメント（設計・仕様・運用方針）
|- packages      作業ツリー（sources / build / artifacts / logs）
|- builder       ビルドツール（Rust 実装、バイナリ名 containers-builder）
|- oci-layouts   パッケージイメージの Image Layout（packages/artifacts からの派生物）
```

プロジェクト内で使用する用語の定義については、[docs/dev/README.md](./docs/dev/README.md) の語彙一覧を参照してください。

## ドキュメント

ドキュメントは読者ごとに分離しています。パッケージイメージの利用者向けは [docs/user/](./docs/user/)、本リポジトリの開発者向けは [docs/dev/](./docs/dev/) に格納しています（入口は [docs/README.md](./docs/README.md)）。

- [user/consume.md](./docs/user/consume.md): パッケージイメージの利用手順（containers.json / sync）
- [dev/oci.md](./docs/dev/oci.md): 準拠する OCI 仕様と実装要件
- [dev/package](./docs/dev/package/README.md): パッケージ定義とビルド方針（個別設定は [definition](./docs/dev/package/definition/)）
- [dev/image](./docs/dev/image/README.md): パッケージイメージのモデルと配信方針
- [dev/builder](./docs/dev/builder/cli.md): ビルドツールのコマンド仕様と内部構成
