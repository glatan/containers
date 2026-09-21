# AGENTS.md

## ルール

- シェルコマンド（`rm` 等）でのファイル削除は禁止。成果物・ビルド作業ツリー・ログの削除は `containers-builder clean` を使うこと。

## ビルドツール

- `builder/` は Rust 製の `containers-builder`。`cd builder && cargo test` / `cargo build` で検証する。
- 実ビルドでコンテナへマウントする runner は musl 静的ビルドを推奨:
  `rustup target add x86_64-unknown-linux-musl` ののち `cargo build --target x86_64-unknown-linux-musl`。
  無い場合はホストバイナリにフォールバックする（コンテナ側 glibc がホストより古いと起動しない）。
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

削除対象: `packages/artifacts/{name}-{version}-{arch}-{variant}`（と中断残骸の `{...}.tmp`）、`packages/build/{arch}-{variant}/{name}-{version}`、`packages/logs/{name}-{version}-{arch}-{variant}.log`。`--images` 指定時は `oci-layouts/{name}-{version}` も対象。`packages/sources/` は削除しない。

## シェル注意

- この環境のシェルは `$` 変数展開が不安定な場合がある。変数を含む破壊的コマンドは避け、実行前に小さく検証すること。
- ワークスペースは UNC 経由（`\\?\UNC\wsl.localhost\...`）で、エージェントの
  `edit`/`write` ツールによる変更が WSL 側へ反映されない・遅延するケースがある。
  変更後はシェル側（`grep`/`sed -n`）で内容を必ず確認すること。phantom 状態
  （変更が効かない・テストが旧動作のまま）では read ツールとシェル出力を
  突き合わせる。大きな書き換えは `.local/` 配下の Python スクリプト経由が安全。
