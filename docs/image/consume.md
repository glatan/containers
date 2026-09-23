# パッケージイメージの利用（消費側）

公開済みのパッケージイメージを消費側リポジトリの Dockerfile から利用する手順を定めます。
同期は `containers-builder sync` が行い、レジストリへの転送は `skopeo` に委譲します。

## 設定ファイル

消費側リポジトリに `containers.json` を置きます。1 ファイルで複数イメージを定義できます。

```json
{
  "schemaVersion": 1,
  "index": "ghcr.io/{org}/packages/index:20260923-0",
  "images": [
    {
      "name": "nginx",
      "dockerfile": "docker/nginx/Dockerfile",
      "platform": "amd64/v4",
      "mode": "block",
      "packages": ["openssl", "nginx"]
    },
    {
      "name": "laravel",
      "dockerfile": "docker/laravel/Dockerfile",
      "platform": "amd64/v4",
      "mode": "compose",
      "packages": ["php", "composer"]
    }
  ]
}
```

| フィールド | 説明 |
|-|-|
| `schemaVersion` | `1` 固定 |
| `index` | index artifact の参照。`"auto"` は `list-tags` で最新の `{yyyymmdd}-{rev}` タグに解決する。`--index` / `CONTAINERS_INDEX` で上書き可 |
| `images[].name` | イメージ識別子。mode `compose` のリポジトリ名 `{registry}/images/{name}` と lockfile のキーに使う |
| `images[].dockerfile` | 対象 Dockerfile（設定ファイルからの相対パス） |
| `images[].platform` | `amd64/v4` 形式の解決 platform |
| `images[].mode` | `block` または `compose` |
| `images[].packages` | トップレベル依存のパッケージ名。推移依存は index から自動解決する |
| `images[].repository` | mode `compose` の push 先を上書きする場合のリポジトリ（既定 `{registry}/images/{name}`） |

`packages` に指定するのはトップレベル依存だけでよく、推移依存（`openssl` の `glibc` など）は index に記録された依存情報からツールが解決します。

## mode: block（GENERATED ブロック）

`sync` が index を解決し、Dockerfile 内の生成ブロックを pinned FROM + COPY 連結に書き換えます。

```dockerfile
# BEGIN GENERATED:containers platform=amd64/v4 index=sha256:…
FROM --platform=linux/amd64/v4 ghcr.io/{org}/packages/x86_64-linux-gnu-glibc@sha256:… AS pkg-x86_64-linux-gnu-glibc
FROM --platform=linux/amd64/v4 ghcr.io/{org}/packages/openssl@sha256:… AS pkg-openssl
FROM --platform=linux/amd64/v4 ghcr.io/{org}/packages/nginx@sha256:… AS pkg-nginx
FROM scratch AS containers-packages
COPY --from=pkg-x86_64-linux-gnu-glibc / /
COPY --from=pkg-openssl / /
COPY --from=pkg-nginx / /
# END GENERATED

FROM containers-packages
# … ユーザー記述 …
```

- 各 FROM は **platform 別 manifest の digest pin** です。`amd64/v1` から `v4` や `power9`、`zEC12`、`rva23u64` のような variant はランタイムの platform マッチングでは区別できないため、digest が真値となります。`--platform` 修飾は containerd/platforms が解釈できる arch/variant（amd64 `v1`–`v4`、arm64 `v8`/`v9`）のみ付けます。
- 同一パッケージが複数 platform で解決される場合、stage 名に `-{arch}-{variant}` を付けて衝突を回避します。
- **パス競合は検査しません**。mode `block` は blob を pull しないため、パッケージ間のファイル衝突は COPY が後勝ちで静かに上書きします。競合検査が必要なら `compose` を使ってください。
- `FROM scratch` には `RUN` が使えないため `bin → usr/bin` 等の usrmerge symlink は生成できません。`{triple}-glibc` が `bin`/`sbin` symlink を同梱しており、glibc を含む閉包では自動的に充足されます。

## mode: compose（合成ベースイメージ）

`sync` が各パッケージの層を pull・展開・合成して `{registry}/images/{name}:{yyyymmdd}-{rev}` へ push し、Dockerfile には pin 済みの `ARG` のみを書き込みます。

```dockerfile
# BEGIN GENERATED:containers index=sha256:…
ARG CONTAINERS_BASE=ghcr.io/{org}/images/laravel@sha256:…
# END GENERATED
FROM ${CONTAINERS_BASE}
# … ユーザー記述 …
```

- `ARG` は最初の `FROM` より前に宣言する必要があるため、ブロックは Dockerfile 先頭付近に配置します。既存ブロックが最初の FROM より後にある場合はエラーにします。
- usrmerge・層順序・パス競合検査は `containers-builder` 側（`image::assemble` と同じ経路）で処理します。
- `--no-push` で push を省略しローカル layout のみ生成できます（ARG は `oci:` パスを指します）。
- 解決結果が lockfile と一致しリモートにイメージが残っていれば再合成・再 push を省略します。

## キャッシュとロックファイル

- 展開済みパッケージツリーは `$XDG_CACHE_HOME/containers-builder/trees/{manifestDigest}` に置き、manifest digest で共有します。
- pull した index artifact は `…/index/{digest}` に置きます。
- 解決結果は `containers.lock.json`（設定ファイルと同じディレクトリ）に記録します。`sync --check` は再解決結果と lockfile・各 Dockerfile の差分を比較し、差があれば失敗します。

## 前提と制約

- レジストリ操作には `skopeo` が必要です（TLS・認証・プロキシは skopeo に委譲）。
- mode `compose` の層展開には `tar` と `zstd` が必要です。
- zstd 圧縮層の展開には Docker 23.0 以降、または BuildKit 経由のビルドが必要です（従来の graphdriver では解凍できない場合があります）。
- `sync` の実行には index artifact の pull が必要なため、レジストリへのネットワーク接続が必須です。

要件・制約の網羅的な一覧は [requirements.md](./requirements.md) を参照してください。
