# 命名とタグ

## レジストリパス

```
ghcr.io/{org}/packages/{package}:{version}
```

ここでの `packages` はパッケージイメージの名前空間です。

- glibc のパッケージイメージ: `ghcr.io/{org}/packages/glibc:2.44`
- Go のパッケージイメージ: `ghcr.io/{org}/packages/go:1.27.0`

architecture / variant はタグに含めません。マルチプラットフォーム index の `platform` で表します。

## タグポリシー

- バージョンタグのみを公開します(`latest` や stream タグは作成しません)。
- 公開後のタグはイミュータブルです(同一タグへの再プッシュは行いません)。
- 利用側は image digest(index)で pin します。未知の `platform.variant` をランタイムが選ばないことがあるためです。
