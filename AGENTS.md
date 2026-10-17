# AGENTS.md

## ルール

- シェルコマンド（`rm` 等）でのファイル削除は禁止。

## パイプライン運用

- `oci-layouts/` がローカル正本。`packages/artifacts/` は中間生成物。
- `pack` は既定で増分 merge（既存 platform を保持）。platform の追加・再封入は `pack {pkg} -t {arch}/{variant}`、除去は `--remove-platform`、全再生成は `--replace`。
- 成果物の復元は `restore`（`--from-registry` で layout ごとレジストリから復元）。
- push 前は `verify` で blob 整合性、`status --local` で状態を確認。`index` の再生成は pack 後に必須。
- `pack` の既定 profile は release（zstd 22）。開発用途は `--profile dev`。

## シェル注意

- この環境のシェルは `$` 変数展開が不安定な場合がある。変数を含む破壊的コマンドは避け、実行前に小さく検証すること。
- ワークスペースは UNC 経由（`\\?\UNC\wsl.localhost\...`）で、
  `edit`/`write` ツールによる変更が WSL 側へ反映されない・遅延するケースがある。
- 変更後はシェル側（`grep`/`sed -n`）で内容を必ず確認すること。
- phantom 状態（変更が効かない・テストが旧動作のまま）では、read ツールと
  シェル出力を突き合わせる。
- 大きな書き換えは `.local/` 配下の Python スクリプト経由が安全。
