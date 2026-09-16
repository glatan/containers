# 命名規則とタグポリシー

## レジストリパス

パッケージイメージは、以下の URL 形式で GitHub Container Registry（GHCR）に公開します。

```
ghcr.io/{org}/packages/{package}:{version}
```

パス内の `packages` は、パッケージイメージを配置するための名前空間を表します。`{package}` は `PACKAGES` のパッケージ名です。

- openssl: `ghcr.io/{org}/packages/openssl:4.0.2`
- x86_64 の実行時 libc: `ghcr.io/{org}/packages/x86_64-linux-gnu-glibc:2.44`

無印の `glibc` というパッケージ名はありません。タグ名に architecture や variant は含めません。対象 CPU プラットフォームの差異は、マルチプラットフォーム Image Index 内の `platform` フィールドによって表現します。

## タグポリシー

- バージョン番号を表すタグのみを公開し、`latest` やメジャーバージョン追従のようなストリームタグは作成しません。
- 公開後のタグはイミュータブル（不変）として扱い、同一タグに対する再プッシュは行いません。
- 利用者はイメージを参照する際、Image Index のダイジェスト（image digest）を用いてバージョンを固定します。コンテナランタイムによっては、仕様書で事前定義されていない `platform.variant` を自動で適切に選択できない場合があるためです。
