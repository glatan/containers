# パッケージイメージ

distroless のように必要なパッケージのみを含め、ベースとなるディストロを持ちません。
パッケージのファイル集合を、実行エントリのない OCI Image(パッケージイメージ)として配ります。
生成する Layout がどの OCI 仕様に準拠するかは [oci](../oci/README.md) を参照してください。

## モデル

パッケージ 1 つを 1 つのパッケージイメージとして配信します。1 パッケージ = 1 レイヤーです。

- パッケージイメージにはそのパッケージだけを含めます。
- 実行時に必要な他パッケージは `dependencies`、ビルド時だけ必要なものは `build-dependencies` です。
- ビルド用と実行用というパッケージイメージの種別は置きません(ツール名の builder とは無関係です)。

## 出力イメージ

レジストリへ載せるのは個別のパッケージイメージです。すべての依存のすべてのバージョンの組み合わせを配ることはしません。

出力イメージは利用側が作ります。指定したパッケージに加え、その `dependencies` を辿って必要なパッケージイメージが入ります。`build-dependencies` は入りません。同じパスに複数パッケージがファイルを持つ場合は失敗です。

ツールは未実装です。パッケージが増えてから詳細を詰めます。

## 関連ドキュメント

- [naming.md](./naming.md) — レジストリパスとタグポリシー
- [platforms.md](./platforms.md) — architecture / variant の一覧
- [packages.md](./packages.md) — 公開するパッケージイメージの一覧
- [packages/](./packages/) — パッケージ毎のビルド設定
- ビルド方法は docker / podman build を使わず、builder がレイヤー tarball とメタデータを生成します([builder](../builder/README.md))

## ライセンス

パッケージイメージのライセンスは、含まれるパッケージのライセンスのうち最も制約の強いものを採用します。

- すべて permissive ライセンスのみの構成: Apache-2.0(MIT のみの構成は MIT)
- LGPL を含む構成: LGPL
- AGPL / GPL 本体を含むパッケージは提供しません。

同梱パッケージの正確なライセンスは SPDX SBOM です。いまは Layout 内の `sbom.spdx.json` です。digest への attestation(referrers)は未実装です。

利用側が pin する値は層によって違います。

- DiffID: 未圧縮 tar。圧縮レベルが変わっても同じ
- layer digest: zstd レベルで変わる
- image digest(index / manifest): `created` を含むためビルド毎に変わり得る

## 検証

`image validate` はパッケージイメージの Layout ではなく、パッケージ(`packages/artifacts/`)の ELF と `dependencies` を静的に見ます。
実行確認の方法は出力イメージのツールを入れるときに定義します。

## 未定

- musl の提供
