# 利用側の環境要件

公開済みパッケージイメージを利用する（`containers-builder sync` の実行と Dockerfile のビルド）ために、環境と利用側リポジトリが満たすべき条件をまとめます。手順は [consume.md](./consume.md) を参照してください。公開側（`pack` / `index` / `push`）の要件は開発者向けの [../dev/distribution.md](../dev/distribution.md) です。

## sync 実行環境の要件

| 項目 | 適用条件と役割 |
|-|-|
| `containers-builder` | 必須。sync コマンドの実行本体 |
| `skopeo` | 必須。index artifact の pull、`list-tags`、mode `compose` の層 pull および push に使用 |
| `tar` と `zstd` | mode `compose` を 1 件でも含む場合に必須（`tar --zstd` の実行）。mode `block` のみでは不要 |
| レジストリへのネットワーク接続 | 必須。リモートの manifest digest 照合を行うため、`--check` であっても接続が必要（ローカルの `oci:` 参照を除く。キャッシュ済みであれば index の再 pull は省略されます） |
| レジストリ認証（読み取り） | パッケージや index が private 公開の場合に必須。sync 時の `skopeo` による pull および inspect で使用（`skopeo login` または docker config を参照） |
| レジストリ認証（書き込み） | mode `compose` で push を行う場合に必須。合成イメージの push 先に対する書き込み権限（`repository` フィールドで変更した場合はそのリポジトリへの push 権限が必要で、`--no-push` 指定時は不要） |

## containers.json の要件

設定ファイルは次をすべて満たす必要があります（違反時はエラーとして処理を中断します）。

- `schemaVersion` は `1` であること。
- `images` 配列は 1 件以上の定義を含むこと。
- `images[].name` は設定内で一意であり、レジストリ命名規則（小文字英数字と `_`、`-` などの区切り文字）に従っていること（mode `compose` のリポジトリ名 `{index.registry}/images/{name}` に使用されるため）。
- `images[].dockerfile` は実在するファイルへのパスであり、設定内のエントリ間で重複しないこと（同一 Dockerfile を複数エントリで共有すると、後のエントリが先の生成ブロックを上書きするため拒否します）。
- `images[].platform` は対象一覧（[targets.md](./targets.md)）に存在する `arch/variant` であること。
- `images[].packages` は 1 件以上のパッケージ名を指定すること（トップレベル依存のみを記述し、推移依存は index から自動解決されます）。
- `images[].repository` は任意項目であり、指定した場合は mode `compose` の push 先を上書きします（mode `block` では無視されます）。

## index 参照の要件

- index 参照は `--index` > `CONTAINERS_INDEX` > 設定ファイルの `index` > 内蔵 pin の順に解決し、いずれかの指定が存在すること。
- `"auto"` を指定した場合、`{8桁の日付}-{数値rev}` 形式のタグのみを対象に最新を採用します。形式外のタグは候補から除外されます。
- index の pull はタグではなく解決済み digest への pin で行われ、タグ差し替えに伴う不整合（TOCTOU）を防ぎます。

## Dockerfile の要件

- `# BEGIN GENERATED:containers` と `# END GENERATED` のブロックは、1 つの Dockerfile 内に 1 つだけ配置すること（対応が崩れている場合や複数存在する場合はエラーとなります）。
- ブロックが存在しない Dockerfile に対しては、最初の `FROM` の直前（`FROM` が無ければ末尾）に自動挿入されます。
- mode `compose` のブロックは、最初の `FROM` より前に配置されている必要があります（`ARG` 宣言位置の制約。後に置かれた既存ブロックはエラーとなります）。
- 生成されるステージ名 `pkg-*`・`containers-packages`、および mode `compose` の生成 `ARG CONTAINERS_BASE` は予約名です。ユーザーステージ名やユーザー宣言の ARG でこれらの名前を使用すると、生成された pin が上書きされたりビルドが失敗したりするため、ユーザー記述側での使用は避けます。

## Dockerfile ビルド環境の要件

- ビルド環境が zstd 圧縮層を展開できること。生成されるイメージ（mode `block` が参照する個別パッケージイメージ、および mode `compose` の合成ベースイメージ）はいずれも zstd 圧縮層を含むため、両 mode で必要です。Docker 23.0 以降（`docker build` は既定で BuildKit を使用）、BuildKit 経由のビルド、または containerd image store のいずれかを利用します。
- 生成ブロックが参照するイメージが private 公開の場合、ビルド環境側でもレジストリ認証が必要です（mode `block` では個別パッケージイメージ、mode `compose` では合成ベースイメージを pull するため）。認証情報はクライアント側の docker config から BuildKit に渡されます。
