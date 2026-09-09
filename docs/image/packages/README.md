# パッケージのビルド設定

各 `{name}.md` がそのパッケージ定義の文書です。実装の正本は [`builder/src/package/definition/`](../../../builder/src/package/definition/) です。
共通のビルド環境・環境変数は [package](../../package/README.md) を参照してください。

LTO と PGO は実施するのが基本です。しない場合は各 `{name}.md` に理由を書きます。
ソースの checksum は SHA3-512 と BLAKE3(出力 512 bit)です。
実行時依存は `dependencies`、ビルド時だけ必要なものは `build-dependencies` です。

公開するパッケージイメージの一覧は [../packages.md](../packages.md) です。
