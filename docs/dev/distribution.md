# 公開側の要件と制限

パッケージの配布（`index` / `push`）のために公開側の環境・レジストリが満たすべき要件と、運用上の制限をまとめます。利用側（`sync` / Dockerfile）は [../user/](../user/consume.md) の文書群を参照してください。

## 実行環境の要件

| 要件 | 対象コマンド |
|-|-|
| `skopeo` が PATH 上で動作すること | `push`（`pack`・`index`・`--dry-run` は不要） |
| `pack` 済みの OCI layout が `oci-layouts/` に存在すること | `index` / `push` |

## 手順の要件

- `pack` → `index` → `push` は同一の生成世代で実行する必要があります。Image Config の `created` がビルド時刻を含むため、pack のたびに manifest digest が変わります。`push` はローカル layout の digest と index 内の記録を照合し、一致しない場合は世代ズレとして拒否します。部分更新（1 パッケージだけ再 pack）の場合も index の再生成が必要です。
- `index --registry` と `push --registry` の値は一致させる必要があります。index artifact の JSON には生成時に指定した registry が記録され、消費側はその値からリポジトリを導出するためです。不一致の push はエラーになります。
- `index` というパッケージ名は定義できません。`packages/index` リポジトリとの衝突を避けるための予約名です（テストで保証）。

## レジストリの要件

配布先レジストリは次をすべて満たす必要があります。

- OCI artifact manifest（`artifactType` フィールドと `application/vnd.oci.empty.v1+json` の空 config）の受け入れ。index artifact がこの形式を採用しているため、OCI Image Spec 1.1 への対応が前提となります。
- zstd 圧縮層（`application/vnd.oci.image.layer.v1.tar+zstd`）の受け入れ。
- `skopeo copy` の転送先として `docker://` トランスポートが利用可能であること。

GHCR はこれらを満たします。ただし GHCR 固有の運用要件として、新規パッケージは既定で private になるため、初回 push 後に visibility の変更とリポジトリへのリンクを管理画面で行う必要があります。

## 制限

- タグの不変性は本ツール経由の操作でのみ強制されます。既存タグが同一 digest を指す場合は冪等にスキップし、別 digest を指す場合は上書きせずエラーにします。他クライアントや管理操作に対しても不変性を求める場合は、レジストリ側のイミュータブルタグポリシーの設定が別途必要です。
- 「タグが存在しない」の判定は `skopeo inspect --raw` のエラー文字列（`manifest unknown`、`404` 等）との照合に依存します。skopeo の出力文言が変更された場合は誤判定につながる可能性があります。
- index artifact のタグ `{yyyymmdd}-{rev}` の `rev` は、対象リポジトリの同日タグの最大値 + 1 を `list-tags` から採番します。採番と push の間に別プロセスが同じタグを取得した場合、イミュータブルタグの照合で後続が失敗します。
- 実レジストリへの push/pull の e2e テスト（`builder/tests/registry_e2e.rs`）は `skopeo` と環境変数 `CONTAINERS_E2E_REGISTRY` の両方が揃った環境でのみ実行され、揃わない環境ではスキップされます。
