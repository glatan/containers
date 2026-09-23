# 配布と利用の要件・制約

パッケージの配布（`index` / `push`）と利用（`sync`）に要求される環境・ツール・レジストリの条件と、各方式が持つ制約をまとめます。手順そのものは [consume.md](./consume.md) と [builder/cli.md](../builder/cli.md) を参照してください。

## 公開側の要件

公開側とは、パッケージを `pack` でビルドし、index を生成してレジストリへ `push` する側（本リポジトリの運用者）を指します。

### 必須ツール

| ツール | 用途 |
|-|-|
| `containers-builder` | pack / index / push の実行 |
| `skopeo` | レジストリへの転送。`push` では必須。TLS・認証・プロキシはすべて skopeo に委譲する |

`pack` と `index`（`--check` を含む）、`push --dry-run` は skopeo なしで動作します。skopeo が必要になるのは実際にレジストリへ読み書きする時点からです。

### 世代の一致

`pack` → `index` → `push` は同一の生成世代で実行します。Image Config の `created` がビルド時刻を含むため、pack のたびに manifest digest とネスト index の digest が変わります。`push` はローカル layout の digest と index 内の `indexDigest` を照合し、一致しない場合は世代ズレとして拒否します。部分更新（1 パッケージだけ再 pack）した場合も index の再生成が必要です。

### イミュータブルタグの強制範囲

タグの不変性はクライアント側（`push` / `sync`）でのみ強制します。`push` は既存タグが同一 digest を指す場合は冪等にスキップし、別 digest を指す場合は上書きせずエラーにします。レジストリ側にイミュータブルタグのポリシーを設定していない限り、他のクライアントや管理操作による上書きは防げません。

なお、「タグが存在しない」判定は `skopeo inspect --raw` のエラー文字列（`manifest unknown`、`name unknown`、`404` 等）との照合で行います。404 系以外の失敗（認証・ネットワーク障害など）は「存在しない」とはみなさず、push を中断してエラーを返します。

### レジストリ側の要件

配布先レジストリには次の受容性が必要です。

- OCI artifact manifest（`artifactType` フィールドと `application/vnd.oci.empty.v1+json` の空 config）の受容。index artifact がこの形式のため、OCI Image Spec 1.1 対応が前提となります
- zstd 圧縮層（`application/vnd.oci.image.layer.v1.tar+zstd`）の受容
- `skopeo copy` の push 先として `docker://` トランスポートが使えること

GHCR はこれらを満たします。ただし GHCR では新規パッケージが既定で private になるため、初回 push 後にパッケージの visibility 変更とリポジトリへのリンクを管理画面で行う必要があります。visibility や retention はツールの管轄外です。

### index の registry 自己記述

index artifact は生成時の `--registry`（既定 `ghcr.io`）を JSON 内の `registry` フィールドに記録します。消費側はこの値からパッケージのリポジトリを導出するため、`push` 時に index の `registry` と実際の push 先が一致しない場合はエラーになります。別前置詞へ公開する場合は `index --registry` で生成し直してください。

また `index` は `packages/` 名前空間の予約名であり、同名のパッケージは定義できません（テストで保証）。

### index タグの採番

index artifact は `{yyyymmdd}-{rev}` のタグで公開します。`rev` は `skopeo list-tags` で同日タグの最大値 + 1 を採番するため、同日の再公開は `-0`、`-1` と進みます。採番と push の間に別プロセスが同じタグを取った場合、イミュータブルタグの照合で後続が失敗します。

## 消費側の要件

消費側とは、`containers.json` を持つリポジトリで `sync` を実行し、Dockerfile をビルドする側を指します。

### sync の実行環境

| ツール | 条件 |
|-|-|
| `containers-builder` | sync の実行本体 |
| `skopeo` | index artifact の pull、`list-tags`、mode `compose` の層 pull・合成イメージ push。mode によらず必須 |
| `tar` + `zstd` | mode `compose` の層展開（`tar --zstd` が動くこと）。mode `block` だけなら不要 |
| ネットワーク接続 | `sync` は毎回 index をレジストリから引くため、`--check` でも接続が必要です。オフラインでは動作しません |

mode `compose` が設定に 1 件でも含まれる場合、`sync` は開始時に `tar` と `zstd` の存在を検査します。

### 設定ファイルの制約

`containers.json` には次の検査があります（違反はエラー）。

- `schemaVersion` は `1` 固定
- `images` は 1 件以上必須
- `name` は一意。`dockerfile` パスの重複も拒否（同一 Dockerfile を複数エントリで共有すると、後のエントリが先のエントリの生成ブロックを上書きして消すため）
- `platform` は `Target` に定義された `arch/variant`（例: `amd64/v4`）
- `packages` は 1 件以上必須。トップレベル依存のみでよく、推移依存は index から解決します
- `dockerfile` の実在を検査

### index 参照の解決順

使用する index 参照は `--index` > `CONTAINERS_INDEX` > 設定ファイルの `index` > 内蔵 pin の順に解決します。

`"auto"` を指定すると `skopeo list-tags` で `{8桁の日付}-{数値rev}` 形式のタグだけに絞り、最新を採用します。形式外のタグ（`latest` 等）は候補に含めません。

index の pull はタグではなく解決済み digest への pin で行います。タグ解決と pull の間にタグが差し替わっても古い index を取り込みません（TOCTOU 対策）。解決結果は `containers.lock.json` の `index` に `{repo}@sha256:…` 形式で記録されます。

### Dockerfile の生成ブロック

`sync` が書き換えるのは `# BEGIN GENERATED:containers … # END GENERATED` のブロックだけです。ブロックは Dockerfile 内に 1 つだけ存在でき、BEGIN/END の対応が崩れている場合や複数ある場合はエラーになります。ブロックが無い Dockerfile には、最初の `FROM` の直前（`FROM` が無ければ末尾）に挿入します。

生成されるステージ名 `pkg-*` と `containers-packages` はその Dockerfile 内で予約されます。ユーザー記述のステージ名との衝突はツール側で検査していないため、これらの接頭辞をユーザーステージ名に使わないでください（衝突すると `docker build` が失敗します）。

## mode: block の要件と制約

### ビルド環境

生成ブロックの `FROM` はパッケージイメージを直接参照するため、ビルド環境が zstd 圧縮層を展開できる必要があります。

- Docker 23.0 以降（従来の graphdriver でも zstd 層を解凍可能）
- それ以前でも BuildKit 経由のビルドであればイメージストアに依存せず展開できます。Docker 23.0 以降は `docker build` が既定で BuildKit を使います
- containerd image store を使う環境でも利用できます

プライベート公開の場合、`docker build` がパッケージイメージを pull できるよう、ビルド環境側でもレジストリ認証（`docker login ghcr.io` 相当）が必要です。認証情報は BuildKit のデーモン側ではなくクライアント側の docker config から渡されます。

### 検査しないもの

mode `block` は blob を pull しないため、パッケージ間のファイルパス競合を検査できません。`COPY --from=` の連結は後勝ちで静かに上書きします。競合検査は mode `compose`（`check_path_conflicts` 経由）でのみ行われます。

### usrmerge の扱い

`FROM scratch` には `RUN` が使えないため、生成ブロックでは `bin → usr/bin` のような symlink を作れません。`{triple}-glibc` が `bin`/`sbin` の symlink を同梱しており、glibc を含む閉包では自動的に充足されます。glibc を含まない閉包（`ca-certificates` 単独など）を mode `block` で使う場合、生成イメージは `/bin`、`/sbin` の symlink を持ちません。mode `compose` では合成側が欠落を検出して usrmerge 層を補いますが、mode `block` にはこのフォールバックがありません。

## mode: compose の要件と制約

### ARG の宣言位置

生成ブロックの `ARG CONTAINERS_BASE=…` は最初の `FROM` より前に宣言する必要があります（`FROM ${CONTAINERS_BASE}` で参照するため）。既存ブロックが最初の `FROM` より後にある場合、`sync` はエラーにします。

### pin の対象

Dockerfile に書き込む pin は合成イメージのネスト index digest ではなく、選択 platform の子 manifest digest です。index digest で pin すると containerd/platforms の platform マッチングが走り、`amd64/v4` などの variant 付き manifest が既定 platform で解決不能になるためです。

### 再利用と採番

解決結果（パッケージ・platform・digest の組）が lockfile と一致し、かつリモートに合成イメージが残っていれば再合成・再 push を省略します。新たに push する場合のタグは `{yyyymmdd}-{rev}` で、rev は対象リポジトリの同日タグの最大値 + 1 を `list-tags` から採番します。

`--no-push` を指定すると push を省略し、ローカルの OCI layout のみ生成します。この場合 ARG は `oci:{ローカルパス}` を指し、他環境からは参照できないため検証用途に限ります。

## プラットフォーム解決の制約

パッケージの参照は常に digest pin が真値です。`--platform` 修飾は、containerd/platforms が正規化できる `linux/amd64/v1`–`v4` と `linux/arm64/v8`・`v9` にのみ付けます。`ppc64le/powerN`、`riscv64/rvaN`、`s390x/zN`（`zEC12` のような大文字を含むものを含む）や `arm64` の `v8.x`/`v9.x` 系は正規化表に無く、ランタイムの platform 選択に委ねられないためです。

依存閉包の解決は index に記録された依存エントリの platform（`artifact_target` 解決済み）に従います。要求した platform と成果物 platform が異なる依存（arm64 向けパッケージの実行時依存が amd64/v4 の成果物を指す等）は、そのまま別 platform の manifest を引きます。依存先が要求 platform の manifest を持たない場合は解決エラーです。循環依存もエラーとして検出します。

## キャッシュとロックファイルの制約

展開済みツリーは `$XDG_CACHE_HOME/containers-builder/trees/{manifestDigest}/rootfs`、pull した index は `…/index/{digest}`、mode `compose` の合成 layout は `…/composed/` に置きます。キャッシュに GC はなく、digest 単位で蓄積し続けるため、長期運用では利用側での定期的な削除が前提です。

`containers.lock.json` は設定ファイルと同じディレクトリに生成され、`sync --check` の比較対象になります。lockfile の `schemaVersion` は index のスキーマ版とは独立して管理されます。

## 検証済み・未検証の範囲

ユニットテストと統合テストはすべてローカルで完結します。レジストリへの実 push/pull/sync の e2e テスト（`builder/tests/registry_e2e.rs`）は `skopeo` の存在と環境変数 `CONTAINERS_E2E_REGISTRY` の両方が揃った環境でのみ実行されます。開発環境に skopeo が無い場合、この経路は未検証のままです。
