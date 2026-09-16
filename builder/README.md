# builder

パッケージおよびパッケージイメージのビルドツールです。実行バイナリ名は `containers-builder` です。

- コマンド仕様: [docs/builder/README.md](../docs/builder/README.md)
- 内部実装構成: [docs/builder/implementation.md](../docs/builder/implementation.md)
- OCI 仕様準拠: [docs/oci/README.md](../docs/oci/README.md)
- 用語の定義: [docs/README.md](../docs/README.md)

## 実行方法

作業ディレクトリ（`packages/` および `oci-layouts/`）を相対パスで参照するため、リポジトリのルートディレクトリで実行します。

```bash
cargo run --manifest-path builder/Cargo.toml -- {subcommand}
```
