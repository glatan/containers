# builder

パッケージとパッケージイメージのビルドツールです。バイナリ名は `containers-builder` です。

- 仕様: [docs/builder/README.md](../docs/builder/README.md)
- 実装: [docs/builder/implementation.md](../docs/builder/implementation.md)
- OCI 準拠: [docs/oci/README.md](../docs/oci/README.md)
- 語彙: [docs/README.md](../docs/README.md)

## 実行方法

リポジトリルートで実行します(`packages/` と `oci-layouts/` を相対パスで参照するため)。

```
cargo run --manifest-path builder/Cargo.toml -- {subcommand}
```
