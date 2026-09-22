# ドキュメント

このディレクトリには、プロジェクト全体の仕様および設計文書を格納します。
本ドキュメント群のライセンスは GNU Free Documentation License 1.3 です（[LICENSE](./LICENSE) を参照）。

## 語彙

パッケージ名は GNU triple（`x86_64-linux-gnu-gcc`）です。成果物ディレクトリと OCI `platform` は architecture / variant（`amd64` / `v4`）です。対応は [targets.md](./targets.md) です。

**成果物と配信**

| 呼称 | 指すもの | 配置・参照 |
|-|-|-|
| パッケージ定義 | `Package` トレイトの実装コード | `builder/src/package/definition/` |
| ソース | 展開済みの上流ソースツリー | `packages/sources/{name}/` |
| パッケージ | `build` が出力する未圧縮の成果物ファイル集合 | `packages/artifacts/{name}-{version}-{arch}-{variant}/` |
| パッケージイメージの Layout | ローカルに生成された OCI Image Layout | `oci-layouts/{name}-{version}/` |
| パッケージイメージ | 成果物の公開単位（エントリポイントを持たない OCI Image） | `ghcr.io/{org}/packages/{name}:{version}` |
| 出力イメージ | `compose` により `dependencies` の推移閉包を束ねて合成したイメージ | [image/README.md](./image/README.md) |

**環境とツール**

| 呼称 | 指すもの | 配置・参照 |
|-|-|-|
| builder | パッケージおよびイメージのビルドツール（バイナリ名は `containers-builder`） | `builder/` |
| HOST | ビルドマシン。常に amd64 / v4。コード上の名前は `Target::BUILD_ENV`。環境変数は `HOST_TRIPLE=x86_64-linux-gnu` | [package/build-flow.md](./package/build-flow.md) |
| ビルド環境イメージ | パッケージをコンパイルするための基準環境。正本は `env compose` の合成結果であり、合成前はシードを使用 | [package/README.md](./package/README.md) |
| シード | ビルド環境イメージの合成前に使用する Arch Linux `base-devel`（ダイジェスト固定） | [`builder/src/package/env.rs`](../builder/src/package/env.rs) |

**対象と分類**

| 呼称 | 指すもの | 配置・参照 |
|-|-|-|
| `Target::all()` | カタログと Host tools（qemu 以外）の既定の対象 Platform | [targets.md](./targets.md) |
| architecture / variant | 対象とする CPU プラットフォーム（OCI platform） | [targets.md](./targets.md) |
| カタログ | 公開する `openssl`、`zlib`、`zlib-ng`、`ca-certificates` の 4 本。対象は `Target::all()`。実行時 libc は `{triple}-glibc` | [image/published.md](./image/published.md) |
| Host tools | gcc が引くライブラリ、PATH コマンド、ビルド時に HOST 成果物をマウントして実行する処理系。qemu 以外は `Target::all()` 向け成果物も出す。qemu は amd64 / v4 のみ | [package/host-tools.md](./package/host-tools.md) |
| プレフィックス付きツールチェーン | `{triple}-binutils`、`{triple}-glibc`、`{triple}-gcc`、およびその中間ステージ。無印の `glibc` は無い | [package/toolchain.md](./package/toolchain.md) |

レジストリパスに含まれる `packages` はパッケージイメージを配置するための名前空間であり、ローカル作業ディレクトリの `packages/artifacts/` を指すものではありません。

## 正本

`PACKAGES` に載るパッケージは、[package/definition/](./package/definition/) に `{パッケージ名}.md` を 1 ファイルずつ置きます。ファイルを種類ごとに統合しません。

| 文書 | 正本とする内容 |
|-|-|
| [package/definition/README.md](./package/definition/README.md) | ビルド対象すべての索引 |
| [image/published.md](./image/published.md) | 公開対象の分類 |
| [package/build-flow.md](./package/build-flow.md) | 段階とコマンド |
| [package/toolchain.md](./package/toolchain.md) | ツールチェーンのステージ、製品キュー、共通ソース |
| [package/host-tools.md](./package/host-tools.md) | Host tools の対象 Platform と共通前処理 |
| [package/README.md](./package/README.md) | パッケージ定義に共通するビルド環境 |
| [targets.md](./targets.md) | `Target::all()` |
| [builder/cli.md](./builder/cli.md) | CLI |
| [builder/architecture.md](./builder/architecture.md) | 内部構成 |

このディレクトリの内容は以下のとおりです。

| パス | 内容 |
|-|-|
| [oci.md](./oci.md) | 生成する Image Layout が準拠する OCI 仕様と、実装が満たすべき要件 |
| [targets.md](./targets.md) | 対象 CPU プラットフォーム（`Target::all()`）の一覧 |
| [image](./image/README.md) | パッケージイメージのモデル、命名規則、および公開方針 |
| [package](./package/README.md) | パッケージ定義と共通ビルド方針 |
| [builder](./builder/cli.md) | ビルドツールのコマンド仕様と内部構成 |
