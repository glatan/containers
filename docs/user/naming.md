# 命名規則とタグポリシー

## レジストリパス

パッケージイメージは、以下の URL 形式で GitHub Container Registry（GHCR）に公開します。

```
ghcr.io/{org}/packages/{package}:{version}
```

パス内の `packages` は、パッケージイメージを配置するための名前空間を表します。`{package}` はパッケージ名です。

- openssl: `ghcr.io/{org}/packages/openssl:4.0.3`
- x86_64 の実行時 libc: `ghcr.io/{org}/packages/x86_64-linux-gnu-glibc:2.44`

無印の `glibc` というパッケージ名はありません。タグ名に architecture や variant は含めません。対象 CPU プラットフォームの差異は、マルチプラットフォーム Image Index 内の `platform` フィールドによって表現します。

## 関連アーティファクト

レジストリにはパッケージイメージのほか、以下を配置します。

| パス | 内容 | タグ |
|-|-|-|
| `ghcr.io/{org}/packages/index` | 全パッケージの name→version→platform→manifest digest→依存関係を集約した index artifact（`artifactType: application/vnd.containers.index.v1+json`） | `{yyyymmdd}-{rev}`（イミュータブル） |
| `ghcr.io/{org}/images/{name}` | `sync` の mode `compose` が生成する合成ベースイメージ | `{yyyymmdd}-{rev}` |

`index` は `packages/` 名前空間の予約名であり、同名のパッケージは定義できません。

## タグポリシー

- パッケージイメージのタグはバージョン番号のみを公開し、`latest` やメジャーバージョン追従のようなストリームタグは作成しません（index・合成イメージの `{yyyymmdd}-{rev}` タグは上表のとおりです）。
- 公開後のタグはイミュータブル（不変）として扱い、同一タグに対する再プッシュは行いません。ただし不変性の強制は `push` / `sync` のクライアント側のみで行われるため、他クライアントや管理操作による上書きを防ぐにはレジストリ側のイミュータブルタグポリシーが別途必要です。確実な固定にはタグではなく digest 参照を使います。
- digest でイメージを参照する際、Image Index のダイジェストは内容（バージョン）を固定しますが、platform の選択はコンテナランタイム側のマッチング処理に依存します。`platform.variant` 付きの manifest を確実に選択するには、platform 別 manifest のダイジェストで指定します（`sync` が生成する pin はこの形式です。背景は [limitations.md](./limitations.md)）。
