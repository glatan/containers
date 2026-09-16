# builder

パッケージおよびパッケージイメージのビルドツールの仕様を定めます。
内部実装の構成については [implementation.md](./implementation.md)、生成する Image Layout の OCI 仕様準拠については [oci](../oci/README.md) を参照してください。

実行バイナリ名は `containers-builder` であり、ソースコードはリポジトリ内の `builder/` ディレクトリで管理します。

## 概要

本ビルドツールは、利用者の目的に応じて次の 2 系統の機能を提供します。

### パッケージ開発者向け機能（本バイナリの主機能）

- ソースコードの取得、パッケージのコンパイル、OCI Image Layout の生成、SPDX SBOM の出力、および公開用アーカイブの作成を一貫して実行します。
- ソフトウェアの複数バージョンの並行管理に対応します。
- 依存クレートは必要最小限に抑え、直接依存するクレートのバージョンは `Cargo.toml` において `=` で完全固定します。
- アーカイブ生成には `tar` コマンド、レイヤー圧縮には `zstd` コマンドをそれぞれ外部プロセスとして呼び出します。

### 利用者向け機能

- パッケージの選択、依存解決、および出力イメージの合成は `image compose` サブコマンドで行います。合成および出力イメージの設計方針については [パッケージイメージ](../image/README.md) を参照してください。

## コマンド体系

サブコマンドの引数に指定する名前は、`x86_64-linux-gnu-glibc` や `openssl` などのパッケージ名です。

### package — パッケージ定義の操作

| コマンド | 説明 |
|-|-|
| `package ls` | 定義済みパッケージの一覧を表示する |
| `package build {?package}` | パッケージをコンパイルし、`packages/artifacts/` に成果物を出力する |
| `package bootstrap` | `BOOTSTRAP` の依存閉包を HOST（amd64 / v4）上で一度にコンパイルする |
| `package fetch {?package}` | 上流ソースコードを取得して展開する |
| `package clean {?package}` | 成果物・ビルド作業ツリー・個別ログを削除する |
| `package tree {package}` | パッケージの依存ツリーを表示する |
| `package bootstrap-path {?package}` | 他パッケージを組める最小のシード相ビルド集合を出力する |
| `package info {?package}` | パッケージ定義のメタデータおよび設定内容を表示する |

`package info` は `--architecture` + `--variant` で対象 Platform を指定でき、ターゲットに応じたロール・依存・CFLAGS / LDFLAGS・PGO・recipe（env / `-std` / sysroot / workdir / configure 引数）を表示します。未指定時は `Target::BUILD_ENV`（amd64 / v4 ネイティブ）として表示します。
| `package env` | 合成済みビルド環境イメージの参照先を表示する（未合成時はシードのダイジェスト固定値を表示） |

#### package build のオプション

- `--all-target`: `supports` が真の architecture / variant をビルド対象とします。最終 gcc / binutils の製品キューはこれより狭く、[toolchain.md](../package/toolchain.md) です。シード環境ではクロスコンパイラが存在しないため ppc64le および s390x を自動的にスキップします。`--architecture` や `--variant` との同時指定はできません。
- `--architecture {architecture}`: 特定の architecture のみを対象とします（`--variant` の指定が必須です）。
- `--variant {variant}`: 特定の variant のみを対象とします。

`package build` の実行時は、対象パッケージをコンパイルする前に、指定した Target ごとの `dependencies` および `build-dependencies` をトポロジカル順で先にビルドします。`--all-target` はこのグラフを Target ごとに組みます。非対応の `--architecture` はエラーです。プレフィックス付き toolchain は `BOOTSTRAP` 配列の外でも Bootstrap グラフを使い、binutils と `{triple}-glibc` の循環を避けます。イメージ参照、シード向け `pacman` 前置き、ネットワーク許可だけを、合成済みビルド環境の有無で切り替えます。依存グラフに循環がある場合はエラーとして処理を中断します。ソースコードが未展開の場合は自動的に `package fetch` を実行します。成果物ディレクトリ（`packages/artifacts/`）に既にファイルが存在する場合は、そのターゲットのビルドをスキップします。ビルドに失敗した場合は、不完全な成果物を残さないよう出力ディレクトリを自動的に削除します（最初からビルドを再試行する場合は、該当する成果物ディレクトリを手動で削除します）。最初の失敗でコマンドは終了します。

実コンパイルを開始する直前に「をビルドします」と進捗を出し、`podman` の標準出力と標準エラー出力を端末と `packages/logs/{name}-{version}-{architecture}-{variant}.log` へ同時に書き込みます。失敗してもこの個別ログは削除しません。`image build --with-packages` も同じ個別ログを書きます。`curl` および `tar` による取得は進捗行と端末への出力のままです。

#### package clean

成果物の再ビルドは `artifact_ready` によるスキップがあるため、まず成果物を削除する必要があります。削除はシェルの `rm` ではなくこのコマンドを使います。Target の指定は `package build` と同じで、`--all-target` または `--architecture` + `--variant` が必須です。`--all` で全パッケージを対象にできます。`--dry-run` で削除対象の一覧だけを表示します。

対象は 3 種類です。`packages/artifacts/{name}-{version}-{arch}-{variant}`、`packages/build/{arch}-{variant}/{name}-{version}`、`packages/logs/{name}-{version}-{arch}-{variant}.log`。ソース（`packages/sources/`）は削除しません。

#### package tree

`package tree {package}` は実行時依存とビルド時依存を再帰的に表示します。Target は `--architecture` + `--variant` で指定し、省略時は HOST（amd64 / v4）です。`[build]` はビルド専用依存、`[bootstrap]` は `BOOTSTRAP` / プレフィックス付き toolchain に属しシード上で組めるノード、異なる Target の成果物を参照する辺には `{arch}/{variant}` を表示します。

#### package bootstrap-path

`package bootstrap-path`（引数なし）は、「これだけシード（Arch base-devel）上で組めば他のパッケージは合成済みビルド環境でビルドできる」最小集合を、`build-env` の依存閉包からトポロジカル順で出力します。`package bootstrap-path {package}` にパッケージを渡した場合、そのパッケージが `BOOTSTRAP` / toolchain に属するならそのパッケージ自身のシード閉包（最小部分集合）を、属さないなら build-env のシード集合に続けて、合成環境でビルドする残りの依存を出力します。

#### package bootstrap

`package bootstrap` は `--architecture` を取りません。対象は常に HOST（`Target::BUILD_ENV`、amd64 / v4）です。

対象は `BOOTSTRAP` に載るパッケージ（x86_64 ツールチェーン、POSIX PATH、cmake、mold）の依存閉包です。他 GNU triple の toolchain は配列にありません。linux-headers と、最終 gcc が引く gmp などはグラフ経由で入ります。python は `build-env` 合成後に組みます。閉包は一度だけトポロジカル順に歩きます。あるパッケージが失敗しても後続のうち依存が揃っているものは実行し、失敗した依存を持つノードは理由を出してスキップします。1 件以上失敗した場合は非 0 で終了し、エラーに失敗件数と統合ログのパスを含めます。

フローの図は [build-flow.md](../package/build-flow.md) です。

統合ログは実行のたびに `packages/logs/bootstrap.log` を作り直します。進捗行と各 `podman` 出力を時系列で追記します。通常の `package build` はこのファイルを開きません。

`package bootstrap` は `image compose build-env` を呼びません。自前のビルド環境イメージが必要な場合は、bootstrap の次に compose を実行します。

#### package fetch のオプション

- `--all`: 定義済みのすべてのパッケージを対象としてソースコードを取得します。

### image — パッケージイメージの操作

| コマンド | 説明 |
|-|-|
| `image ls` | 公開対象パッケージイメージの一覧を表示する |
| `image build {?package}` | パッケージ成果物から OCI Image Layout を生成する（SPDX SBOM の出力を含む） |
| `image compose {package}` | `dependencies` を再帰的に探索し、単一の出力イメージを合成する |
| `image validate {?package}` | 宣言された依存関係と、パッケージ成果物内 ELF バイナリの動的リンク依存を静的に照合・検証する |

`image validate` は生成済みの `oci-layouts/` ではなく、`packages/artifacts/` に出力された未圧縮パッケージ成果物の ELF バイナリを直接検査します。

#### image build のオプション

- `--all`: 公開対象となっているすべてのパッケージイメージを生成します。
- `--with-packages`: Image Layout の生成前に、対応するパッケージ成果物のビルドも併せて実行します。
- `--publish`: 公開用アーカイブとして、zstd 圧縮レベル 22（`--ultra`）を用いてレイヤーを圧縮します。
- `--zstd-level {1-22}`: 圧縮レベルを直接指定します（デフォルトは 3 です。`--publish` との同時指定はできません）。

#### image validate のオプション

- `--all`: 公開対象の全パッケージイメージを対象として検証を実行します（ビルド専用パッケージは除外します）。

## パッケージ定義の規則

- 実装の正本は Rust の `Package` トレイトです。パッケージごとの定義コードは `builder/src/package/definition/{name}.rs` に配置し、仕様文書は [package/definition/](../package/definition/) にまとめます。
- 共通のビルド環境方針については [package](../package/README.md) を参照してください。
- パッケージイメージには各パッケージ固有のファイルのみを含め、依存パッケージのファイルは含めません。実行時依存は `dependencies`、ビルド時のみ必要な依存は `build-dependencies` として分離して宣言します。
- SPDX SBOM は Image Layout 内の `sbom.spdx.json` に出力します。なお、OCI attestation（referrers）としての外部公開は現時点では未実装です。
