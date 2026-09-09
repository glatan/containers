# containers

パッケージとパッケージイメージのビルド設定 / ビルドツールを管理するプロジェクトです。
公開単位はパッケージイメージで、GitHub Container Registry に置きます。
準拠仕様は [docs/oci](./docs/oci/README.md) です。

## ディレクトリ構造

```
|- docs          ドキュメント
|- packages      作業ツリー(sources / build / artifacts)
|- builder       ビルドツール(Rust、バイナリ名 containers-builder)
|- oci-layouts   パッケージイメージの Image Layout(packages/artifacts からの派生物)
```

呼び方は [docs](./docs/README.md) の語彙を参照してください。

## ドキュメント

詳しいドキュメントは [docs/](./docs/README.md) にあります。

- [oci](./docs/oci/README.md): 準拠する OCI 仕様と要件
- [package](./docs/package/README.md): パッケージ定義のビルド方針
- [image](./docs/image/README.md): パッケージイメージのモデルと公開方針
- [builder](./docs/builder/README.md): ビルドツールの仕様
