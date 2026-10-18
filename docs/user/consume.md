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
      "name": "devtools",
      "dockerfile": "docker/devtools/Dockerfile",
      "platform": "amd64/v4",
      "mode": "block",
      "packages": ["bash", "coreutils", "grep", "sed", "gawk"]
    },
    {
      "name": "tls",
      "dockerfile": "docker/tls/Dockerfile",
      "platform": "amd64/v4",
      "mode": "compose",
      "packages": ["openssl", "ca-certificates", "zlib"]
    }
  ]
}
```

| フィールド | 説明 |
|-|-|
| `schemaVersion` | `1` 固定 |
| `index` | index artifact の参照。`"auto"` は `list-tags` で最新の `{yyyymmdd}-{rev}` タグに解決する。解決順序は `--index` > `CONTAINERS_INDEX` > このフィールド > 内蔵 pin の順。`"auto"` が参照する `{registry}/packages/index` の前置詞は `--registry` または `CONTAINERS_REGISTRY` が必須（既定値なし） |
| `images[].name` | イメージ識別子。mode `compose` のリポジトリ名 `{registry}/images/{name}` と lockfile のキーに使う |
| `images[].dockerfile` | 対象 Dockerfile（設定ファイルからの相対パス） |
| `images[].platform` | `amd64/v4` 形式の解決 platform |
| `images[].mode` | `block` または `compose` |
| `images[].packages` | トップレベル依存のパッケージ名。推移依存は index から自動解決する |
| `images[].repository` | mode `compose` の push 先を上書きする場合のリポジトリ（既定 `{registry}/images/{name}`） |

`packages` に指定するのはトップレベル依存だけでよく、推移依存（`openssl` の `{triple}-glibc` など）は index に記録された依存情報からツールが解決します。

パッケージのバージョンは index artifact が記録した解決対象で決まり、パッケージ単位で選択はできません。バージョンの固定・更新は `index` 参照（タグまたは digest）の差し替えで行います。解決済みの index digest は `containers.lock.json` に記録されます。

## sync の実行

設定ファイルのパスを引数に取ります。

```bash
containers-builder sync containers.json [--check] [--index {ref}] [--registry {prefix}] [--output {dir}] [--no-push] [--profile {dev|release}] [--zstd-level {1-22}]
```

- `--check`: 再解決・再生成した内容と差分があれば失敗します（CI 向け。書き込みません）
- `--index`: 使用する index 参照を上書きします
- `--registry`: `"index": "auto"` が参照する `{registry}/packages/index` の前置詞です。`"auto"` のときは `--registry` または `CONTAINERS_REGISTRY` が必須で、既定値はありません。生成ブロック内の FROM 参照や mode `compose` の push 先は、このフラグではなく index artifact に記録された `registry` フィールドから導出されます
- `--output`: mode `compose` の合成 layout の出力先を変更します（既定はキャッシュの `composed/{name}-{tag}`）
- `--no-push`: mode `compose` で push を省略しローカル layout のみ生成します
- `--profile` / `--zstd-level`: mode `compose` の合成イメージの圧縮レベルを指定します

## mode: block（GENERATED ブロック）

`sync` が index を解決し、Dockerfile 内の生成ブロックを pinned FROM + COPY 連結に書き換えます。

```dockerfile
# BEGIN GENERATED:containers platform=amd64/v4 index=sha256:…
FROM --platform=linux/amd64/v4 ghcr.io/{org}/packages/x86_64-linux-gnu-glibc@sha256:… AS pkg-x86_64-linux-gnu-glibc
FROM --platform=linux/amd64/v4 ghcr.io/{org}/packages/bash@sha256:… AS pkg-bash
FROM --platform=linux/amd64/v4 ghcr.io/{org}/packages/coreutils@sha256:… AS pkg-coreutils
FROM scratch AS containers-packages
COPY --from=pkg-x86_64-linux-gnu-glibc / /
COPY --from=pkg-bash / /
COPY --from=pkg-coreutils / /
# END GENERATED

FROM containers-packages
# … ユーザー記述 …
```

- 各 FROM は platform 別 manifest の digest pin です。`amd64/v1` から `v4` や `power9`、`zEC12`、`rva23u64` のような variant はランタイムの platform マッチングでは区別できないため、digest が真値となります。`--platform` 修飾は containerd/platforms が解釈できる arch/variant（amd64 `v1`–`v4`、arm64 `v8`/`v9`）のみ付与します。
- 同一パッケージが複数 platform で解決される場合、stage 名に `-{arch}-{variant}` を付けて衝突を回避します。
- パス競合は検査しません。mode `block` は blob を pull しないため、パッケージ間のファイル衝突は COPY が後勝ちで警告なく上書きされます。競合の検査が必要な場合は mode `compose` を使用します。
- `FROM scratch` には `RUN` が使えないため `bin → usr/bin` 等の usrmerge symlink は生成できません。`{triple}-glibc` が `bin`/`sbin` symlink を同梱しており、glibc を含む閉包では自動的に充足されます。

## mode: compose（合成ベースイメージ）

`sync` が各パッケージの層を pull・展開・合成して `{registry}/images/{name}:{yyyymmdd}-{rev}` へ push し、Dockerfile には pin 済みの `ARG` のみを書き込みます。

```dockerfile
# BEGIN GENERATED:containers index=sha256:…
ARG CONTAINERS_BASE=ghcr.io/{org}/images/tls@sha256:…
# END GENERATED
FROM ${CONTAINERS_BASE}
# … ユーザー記述 …
```

- `ARG` は最初の `FROM` より前に宣言する必要があるため、ブロックは Dockerfile 先頭付近に配置します。既存ブロックが最初の FROM より後にある場合はエラーにします。
- usrmerge の適用、層順序の決定、パス競合の検査は `containers-builder` が行います。
- `--no-push` で push を省略しローカル layout のみ生成できます。ただし ARG は `oci:{ローカルパス}`（skopeo の転送形式）を指すため、そのままでは `docker build` の `FROM` 参照として使えません。ローカル生成物の検査用途です。
- 解決結果が lockfile と一致しリモートにイメージが残っていれば再合成・再 push を省略します。

## キャッシュとロックファイル

- 展開済みパッケージツリーは `$XDG_CACHE_HOME/containers-builder/trees/{manifestDigest}` に置き、manifest digest で共有します。`$XDG_CACHE_HOME` 未設定時は `~/.cache/containers-builder` を使います。
- pull した index artifact は `…/index/{digest}`、mode `compose` の合成 layout は `…/composed/{name}-{tag}`（`--output` で変更可）に置きます。
- 解決結果は `containers.lock.json`（設定ファイルと同じディレクトリ）に記録します。`sync --check` は再解決結果と lockfile・各 Dockerfile の差分を比較し、差があれば失敗します。生成ブロックと `containers.lock.json` はリポジトリへコミットし、CI でドリフトを検出する運用を想定しています。

## 前提と制約

- レジストリ操作には `skopeo` が必要です（TLS・認証・プロキシは skopeo に委譲）。
- mode `compose` の層展開には `tar` と `zstd` が必要です。
- zstd 圧縮層の展開には Docker 23.0 以降、または BuildKit 経由のビルドが必要です（従来の graphdriver では解凍できない場合があります）。
- `sync` の実行にはリモートの manifest digest 照合を行うため、レジストリへのネットワーク接続が必要です。

環境要件は [requirements.md](./requirements.md)、機能制限の一覧は [limitations.md](./limitations.md) を参照してください。
