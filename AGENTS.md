# AGENTS.md

## ルール

- シェルコマンド（`rm` 等）でのファイル削除は禁止。成果物・ビルド作業ツリー・ログの削除は `containers-builder clean` を使うこと。

## ビルドツール

- `builder/` は Rust 製の `containers-builder`。`cd builder && cargo test` / `cargo build` で検証する。
- パッケージ成果物は `packages/artifacts/`、作業ツリーは `packages/build/`、ログは `packages/logs/`。
- 成果物が存在すると `artifact_ready` によりビルドがスキップされる。再ビルド前は `clean` で削除する。

## clean

```
containers-builder clean <name> --all-targets
containers-builder clean <name> -t amd64/v4
containers-builder clean --all --all-targets
containers-builder clean <name> --all-targets --dry-run
containers-builder clean <name> --all-targets --images
```

削除対象: `packages/artifacts/{name}-{version}-{arch}-{variant}`、`packages/build/{arch}-{variant}/{name}-{version}`、`packages/logs/{name}-{version}-{arch}-{variant}.log`。`--images` 指定時は `oci-layouts/{name}-{version}` も対象。`packages/sources/` は削除しない。

## シェル注意

- この環境のシェルは `$` 変数展開が不安定な場合がある。変数を含む破壊的コマンドは避け、実行前に小さく検証すること。
