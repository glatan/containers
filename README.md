# containers

本プロジェクトは、パッケージおよびパッケージイメージのビルド設定と、そのビルドツールを管理します。
成果物の公開単位はパッケージイメージであり、GitHub Container Registry（GHCR）へ配信します。
準拠する OCI 仕様および実装要件については [docs/oci.md](./docs/oci.md) を参照してください。

## ディレクトリ構造

```
|- docs          ドキュメント（設計・仕様・運用方針）
|- packages      作業ツリー（sources / build / artifacts / logs）
|- builder       ビルドツール（Rust 実装、バイナリ名 containers-builder）
|- oci-layouts   パッケージイメージの Image Layout（packages/artifacts からの派生物）
```

プロジェクト内で使用する用語の定義については、[docs/README.md](./docs/README.md) の語彙一覧を参照してください。

## ドキュメント

プロジェクトの各領域に関する詳細な設計文書は [docs/](./docs/README.md) に格納しています。

- [oci.md](./docs/oci.md): 準拠する OCI 仕様と実装要件
- [package](./docs/package/README.md): パッケージ定義とビルド方針（個別設定は [definition](./docs/package/definition/)）
- [image](./docs/image/README.md): パッケージイメージのモデルと配信方針
- [builder](./docs/builder/cli.md): ビルドツールのコマンド仕様と内部構成
