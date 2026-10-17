# 公開側の要件と制限

パッケージの配布（`index` / `push`）のために公開側の環境・レジストリが満たすべき要件と、運用上の制限をまとめます。利用側（`sync` / Dockerfile）は [../user/](../user/consume.md) の文書群を参照してください。

## 実行環境の要件

| 要件 | 対象コマンド |
|-|-|
| `skopeo` が PATH 上で動作すること | `push`（`pack`・`index`・`--dry-run` は不要） |
| `pack` 済みの OCI layout が `oci-layouts/` に存在すること | `index` / `push` |

## 手順の要件

- `pack` → `index` → `push` は同一の生成世代で実行する必要があります。`push` はローカル layout の digest と index 内の記録を照合し、一致しない場合は世代の不整合として処理を拒否します。1 パッケージのみを再 pack した場合でも index の再生成が必要です。なお、pack が生成する digest 自体は同一コミット上では決定的です。Image Config の `created` は `SOURCE_DATE_EPOCH` に基づき既定で epoch 0 に固定されます。ただし、`org.opencontainers.image.revision` アノテーションには HEAD コミットを埋め込むため、成果物が同一であってもコミットが異なれば digest は変化します。
- レジストリ前置詞に既定値はありません。`index` と `push` は `--registry` または `CONTAINERS_REGISTRY` が必須で、両者の値は一致させる必要があります。index artifact の JSON には生成時に指定した registry が記録され、消費側はその値からリポジトリを導出するためです。不一致の push はエラーになります。
- `index` というパッケージ名は定義できません。`packages/index` リポジトリとの衝突を避けるための予約名です（テストで保証）。

## レジストリの要件

配布先レジストリは次をすべて満たす必要があります。

- OCI artifact manifest（`artifactType` フィールドと `application/vnd.oci.empty.v1+json` の空 config）の受け入れ。index artifact がこの形式を採用しているため、OCI Image Spec 1.1 への対応が前提となります。
- zstd 圧縮層（`application/vnd.oci.image.layer.v1.tar+zstd`）の受け入れ。
- `skopeo copy` の転送先として `docker://` トランスポートが利用可能であること。

GHCR はこれらを満たします。GHCR 固有の運用要件として、新規パッケージは既定で private になるため、初回 push 後に visibility の変更を管理画面で行う必要があります（一度 public にしたパッケージは private へ戻せません）。リポジトリへのリンクは `org.opencontainers.image.source` アノテーションにより自動で行われます。パッケージがリポジトリのアクセス権を継承するのはリンクが初回公開より前にある場合のみで、後から管理画面でリンクした場合は継承されません。

## 制限

- タグの不変性は、本ツール経由の操作でのみ強制されます。既存タグが同一 digest を指す場合は冪等にスキップします。パッケージイメージにおいてリモートの platform 集合がローカルの部分集合であり、かつ共通 platform の内容が一致する場合に限り、platform の追加として上書きを許可します。共通 platform の内容一致は、次のいずれかを満たすことと定義します。
  1. manifest digest が完全に一致する。
  2. config と layer の digest が一致し、annotations が additive 適合である。
  3. digest が異なる場合でも、config が `created` を除いて一致し、layer の個数および media type が一致し、`rootfs.diff_ids` が等しい（同一ペイロードの再エンコード。圧縮形式の変更は不適合）。
  ここで annotations の additive 適合とは、リモートに存在する全 annotation がローカルにも同値で存在することを指します。ローカル側への新規アノテーション追記はメタデータ追記として許容され、`org.opencontainers.image.revision` のみ値の差異や一方の欠落を許容します。
  既存 platform の削除や内容変更を検知した場合はエラーとなります。正当な再ビルドによって生じたペイロード変更を公開する場合は `push --replace-platform <arch>/<variant>` で指定 platform のみ置換を許可します（他の platform は通常どおり検証されます）。意図した全体置換を行う場合は `push --force-replace` で additive 判定をスキップします。なお、外部クライアントや管理操作に対しても不変性を担保したい場合は、レジストリ側でイミュータブルタグポリシーを別途設定する必要があります。
- 「タグが存在しない」の判定は `skopeo inspect --raw` のエラー文字列（`manifest unknown`、`404` 等）との照合に依存します。skopeo の出力文言が変更された場合は誤判定につながる可能性があります。
- index artifact のタグ `{yyyymmdd}-{rev}` の `rev` は、対象リポジトリの同日タグの最大値 + 1 を `list-tags` から採番します。採番と push の間に別プロセスが同じタグを取得した場合、イミュータブルタグの照合で後続が失敗します。
- 実レジストリへの push/pull の e2e テスト（`builder/tests/registry_e2e.rs`）は `skopeo` と環境変数 `CONTAINERS_E2E_REGISTRY` の両方が揃った環境でのみ実行され、揃わない環境ではスキップされます。
