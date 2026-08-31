# image

OCI イメージに関するドキュメントです。

distroless のように必要なツールのみをイメージに含め、ベースとなるディストロを持ちません。

## 種別

runtime / builder の2種類に大別します。

- runtime: アプリケーション/スクリプトの実行に必要なツールだけを含むイメージ
- builder: ビルドに必要なツールのみを含むイメージ

## 命名規則

```
{runtime / builder}-{tool}:{version}
```

- glibc のみを含むイメージ: `runtime-glibc:2.44`
- Go のコンパイラを含むイメージ: `builder-go:1.27.0`

## ビルド方法

docker build, podman(buildah) build は利用しません。
レイヤーの tarball 生成、メタデータ生成まで独自で実施します。

## アーキテクチャ

対象アーキテクチャの一覧は [architectures.md](./architectures.md) を参照してください。

## ライセンス

イメージ自体のライセンスは未定です。

## 未定

- glibc + openssl のように複数のツールを含むイメージの命名・管理方法
  - openssl だけにして、暗黙に glibc への依存にしたくない
  - `use glibc, openssl;` のようなことができれば理想
  - ツール A とツール B のバージョンを指定したイメージを OCI 上でうまく管理する方法が未確定

## イメージ一覧

- [runtime](./runtime/README.md)
- [builder](./builder/README.md)
