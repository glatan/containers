# containers

OCI コンテナのビルド設定 / ビルドツールを管理するプロジェクトです。
成果物は OCI Image で、GitHub Container Registry に公開します。

## ディレクトリ構造

```
|- docs      ドキュメント
|- tools     コンテナに含めるツール(packages / sources / build はビルド成果物)
|- builder   コンテナ/ツールのビルドツール(Rust)
|- images    イメージのビルド結果
```

## ドキュメント

詳しいドキュメントは [docs/](./docs/) にあります。

- [tool](./docs/tool/README.md): コンテナに含めるツールのビルド設定
- [image](./docs/image/README.md): OCI イメージの種別・命名規則・ビルド方法
- [builder](./docs/builder/README.md): ビルドツールの仕様
