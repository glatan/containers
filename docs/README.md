# ドキュメント

このディレクトリはプロジェクトの仕様と設計を置きます。
ドキュメントは GNU Free Documentation License 1.3 です([LICENSE](./LICENSE))。

## 語彙

| 呼称 | 指すもの | パス |
|-|-|-|
| パッケージ定義 | `Package` trait と各パッケージの実装 | `builder/src/package/` |
| ソース | 展開した上流ソース | `packages/sources/{name}/` |
| パッケージ | `package build` が出力する未圧縮のファイル集合 | `packages/artifacts/{name}-{version}-{arch}-{variant}/` |
| パッケージイメージの Layout | ローカルの OCI Image Layout | `oci-layouts/{name}-{version}/` |
| パッケージイメージ | 公開単位(実行エントリのない OCI Image) | `ghcr.io/{org}/packages/{name}:{version}` |
| builder | ツール。ディレクトリ名。バイナリは `containers-builder` | `builder/` |
| ビルド環境イメージ | パッケージをコンパイルする Arch イメージ | [`builder/src/package/env.rs`](../builder/src/package/env.rs) |
| architecture / variant | CPU(OCI platform) | [image/platforms.md](./image/platforms.md) |
| 出力イメージ | 未実装。複数パッケージを束ねるユーザー向けイメージ | |

レジストリパスの `packages` はパッケージイメージの名前空間であり、`packages/artifacts/` の置き場ではありません。

`docs/image/` はパッケージイメージのモデルです。Image Spec との対応は [oci](./oci/README.md) です。

| ディレクトリ | 内容 |
|-|-|
| [oci](./oci/README.md) | 生成する Layout が準拠する OCI 仕様と、実装が満たす要件 |
| [image](./image/README.md) | パッケージイメージのモデル、命名、プラットフォーム、公開対象。ビルド設定は [image/packages/](./image/packages/README.md) |
| [package](./package/README.md) | パッケージ定義に共通するビルド方針 |
| [builder](./builder/README.md) | 開発者向けビルドツールのコマンド仕様と実装構成 |
