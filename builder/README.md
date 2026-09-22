# builder

パッケージおよびパッケージイメージのビルドツールです。実行バイナリ名は `containers-builder` です。

## 実行方法

作業ディレクトリ（`packages/` および `oci-layouts/`）を相対パスで参照するため、リポジトリのルートディレクトリで実行します。

```bash
cargo run --manifest-path builder/Cargo.toml -- {subcommand}
```

## 検証

```bash
cd builder
cargo test
cargo build
```

## runner のビルド

実ビルドでは、コンテナ内でビルド plan を実行する runner（`run-recipe` サブコマンドのエントリポイント）としてビルドツール自身をマウントします。コンテナ側 libc に依存しない musl 静的ビルドを推奨します:

```bash
rustup target add x86_64-unknown-linux-musl
cargo build --target x86_64-unknown-linux-musl
```

musl 静的ビルドが無い場合はホストバイナリへフォールバックしますが、コンテナ側 glibc がホストより古いと runner が起動しません（詳細は [docs/builder/architecture.md](../docs/builder/architecture.md)）。

## ドキュメント

- コマンド仕様: [docs/builder/cli.md](../docs/builder/cli.md)
- 内部構成: [docs/builder/architecture.md](../docs/builder/architecture.md)
- OCI 仕様準拠: [docs/oci.md](../docs/oci.md)
- 用語の定義: [docs/README.md](../docs/README.md)
