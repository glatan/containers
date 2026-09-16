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

- パッケージの選択、依存解決、および出力イメージの合成は `compose` サブコマンドで行います。合成および出力イメージの設計方針については [パッケージイメージ](../image/README.md) を参照してください。

## コマンド体系

サブコマンドの引数に指定する名前は、`x86_64-linux-gnu-glibc` や `openssl` などのパッケージ名です。コマンドはフラットな構成で、動詞がパイプラインステージ（fetch → build → check → pack → compose → clean）を表します。

### 照会

| コマンド | 説明 |
|-|-|
| `ls [--images]` | 定義済みパッケージの一覧を表示する（`--images` で配布対象に絞る） |
| `info {package} -t {arch}/{variant}` | パッケージ定義のメタデータおよび設定内容を表示する |
| `tree {package} -t {arch}/{variant} [--reverse]` | パッケージの依存ツリー（`--reverse` で逆依存）を表示する |
| `targets` | 有効な `arch`/`variant` の一覧を表示する |

`info` は `-t` で対象 Platform を指定し（必須）、ターゲットに応じたロール・依存・CFLAGS / LDFLAGS・PGO・recipe（env / `-std` / sysroot / workdir / configure 引数）を表示します。成果物の配置ターゲットが指定ターゲットと異なる場合は `成果物ターゲット` を、対応する全ターゲットを `対応ターゲット` に列挙します。

`tree` は実行時依存とビルド時依存を再帰的に表示します。`[build]` はビルド時依存の辺、`[bootstrap]` は `BOOTSTRAP` / プレフィックス付き toolchain に属しシード上で組めるノード、異なる Target の成果物を参照する辺には `{arch}/{variant}` を表示します。`--reverse` はこのパッケージに依存する側をたどり、同じタグを depender 側の意味で付与します。

### 作業

| コマンド | 説明 |
|-|-|
| `fetch {package\|--all} [--deps]` | 上流ソースコードを取得して展開する（`--deps` で依存閉包も取得） |
| `build {package\|--all} (-t ...\|--all-targets)` | パッケージをコンパイルし、`packages/artifacts/` に成果物を出力する |
| `check {package\|--all}` | 宣言された依存関係と、パッケージ成果物内 ELF バイナリの動的リンク依存を静的に照合・検証する |
| `pack {package\|--all}` | パッケージ成果物から OCI Image Layout を生成する（SPDX SBOM の出力を含む） |
| `compose {package} -t {arch}/{variant}` | `dependencies` を再帰的に探索し、単一の出力イメージを合成する |
| `clean {package\|--all} (-t ...\|--all-targets)` | 成果物・ビルド作業ツリー・個別ログを削除する（`--images` で `oci-layouts/` も対象） |

`--all` を取るコマンドは全件を処理し、失敗を集約して最後に非 0 で終了します（`build` は `--keep-going` 指定時のみ失敗後も継続し、未指定時は最初の失敗で終了します）。

#### build のオプション

- `-t {arch}/{variant}`: 対象ターゲットを指定します（複数回指定可。`targets` コマンドで有効な一覧を確認できます）。
- `--all`: 全パッケージを単一のトポロジカル順でビルドします。この場合 `-t` に対応しないルートはエラーにせずスキップします（`--all-targets` と同じ緩和）。
- `--all-targets`: `supports` が真の architecture / variant をビルド対象とします。最終 gcc / binutils の製品キューはこれより狭く、[toolchain.md](../package/toolchain.md) です。シード環境ではクロスコンパイラが存在しないため ppc64le および s390x を自動的にスキップします。`-t` との同時指定はできません。
- `--force`: 成果物が存在しても対象パッケージを再ビルドします（依存はキャッシュ済みなら再利用します）。
- `--keep-going`: 失敗しても無関係な後続のビルドを継続し、最後に失敗一覧を集約して非 0 で終了します。
- `--no-deps`: 依存をビルドせず対象パッケージのみをビルドします（`--all`・`--keep-going` との同時指定はできません）。
- `--dry-run`: ビルドせず計画のみを表示します（`[cached]` / `[will-build]` / `[skip:unsupported]` / `[skip:no-cross]`）。

`build` の実行時は、対象パッケージをコンパイルする前に、指定した Target ごとの `dependencies` および `build-dependencies` をトポロジカル順で先にビルドします。`--all-targets` はこのグラフを Target ごとに組み、（パッケージ, ターゲット）単位の重複を除いた単一の順序で処理します。非対応の `-t` はエラーです。プレフィックス付き toolchain は `BOOTSTRAP` 配列の外でも Bootstrap グラフを使い、binutils と `{triple}-glibc` の循環を避けます。イメージ参照、シード向け `pacman` 前置き、ネットワーク許可だけを、合成済みビルド環境の有無で切り替えます。依存グラフに循環がある場合はエラーとして処理を中断します。ソースコードが未展開の場合は自動的に `fetch` を実行します。成果物ディレクトリ（`packages/artifacts/`）に既にファイルが存在する場合は、そのターゲットのビルドをスキップします。ビルドに失敗した場合は、不完全な成果物を残さないよう出力ディレクトリを自動的に削除します（最初からビルドを再試行する場合は、該当する成果物ディレクトリを手動で削除します）。`--keep-going` 指定時を除き、最初の失敗でコマンドは終了します。

実コンパイルを開始する直前に「をビルドします」と進捗を出し、`podman` の標準出力と標準エラー出力を端末と `packages/logs/{name}-{version}-{architecture}-{variant}.log` へ同時に書き込みます。失敗してもこの個別ログは削除しません。`pack --with-build` および `compose --with-build` も同じ個別ログを書きます。`curl` および `tar` による取得は進捗行と端末への出力のままです。

#### clean

成果物の再ビルドは `artifact_ready` によるスキップがあるため、まず成果物を削除する必要があります。削除はシェルの `rm` ではなくこのコマンドを使います。Target の指定は `build` と同じで、`-t` または `--all-targets` が必須です。`--all` で全パッケージを対象にできます。`--dry-run` で削除対象の一覧だけを表示します。`--images` を付けると `oci-layouts/{name}-{version}` も削除対象に含めます。

対象は 3 種類です。`packages/artifacts/{name}-{version}-{arch}-{variant}`、`packages/build/{arch}-{variant}/{name}-{version}`、`packages/logs/{name}-{version}-{arch}-{variant}.log`。ソース（`packages/sources/`）は削除しません。

#### fetch のオプション

- `--all`: 定義済みのすべてのパッケージを対象としてソースコードを取得します。
- `--deps`: 有効依存グラフの閉包ごと取得します。同一 URL は一度だけダウンロードします。

#### pack のオプション

- `--all`: 公開対象となっているすべてのパッケージイメージを生成します（`build-env` およびビルド専用パッケージは対象外、成果物のないエントリは `スキップ(パッケージなし)` を出して継続します）。
- `--with-build`: Image Layout の生成前に、対応するパッケージ成果物のビルドも併せて実行します。
- `--profile release`: 公開用アーカイブとして、zstd 圧縮レベル 22（`--ultra`）を用いてレイヤーを圧縮します。
- `--zstd-level {1-22}`: 圧縮レベルを直接指定します（デフォルトは 3 です。`--profile` との同時指定はできません）。

#### compose のオプション

- `-t {arch}/{variant}`: 合成対象のターゲットを指定します（必須）。なお実行時依存の `{triple}-gcc` 成果物は常に BUILD_ENV（amd64 / v4）に配置されるため、C++ ランタイムに依存するパッケージは現状 BUILD_ENV 以外では合成できません。
- `--with-build`: 合成前に必要なパッケージをビルドします。
- `--profile` / `--zstd-level`: `pack` と同じです。

`build-env` は `compose` / `pack` の対象外であり、`env compose` で合成します。

`check` は生成済みの `oci-layouts/` ではなく、`packages/artifacts/` に出力された未圧縮パッケージ成果物の ELF バイナリを直接検査します。`--all` では公開対象の全パッケージイメージを対象とします（ビルド専用パッケージは除外します）。

### env — ビルド環境の管理

| コマンド | 説明 |
|-|-|
| `env status` | ビルド環境イメージの `正本:` 参照（合成済みなら layout の canonical path、未合成なら `未合成`）を表示する。シードの digest 固定値は常に併記する |
| `env bootstrap` | `BOOTSTRAP` の依存閉包を HOST（amd64 / v4）上で一度にコンパイルする |
| `env plan {?package}` | 他パッケージを組める最小のシード相ビルド集合を出力する |
| `env compose` | ビルド環境イメージを合成する |

#### env plan

`env plan`（引数なし）は、「これだけシード（Arch base-devel）上で組めば他のパッケージは合成済みビルド環境でビルドできる」最小集合を、`build-env` の依存閉包からトポロジカル順で出力します。`env plan {package}` にパッケージを渡した場合、そのパッケージが `BOOTSTRAP` / toolchain に属するならそのパッケージ自身のシード閉包（最小部分集合）を、属さないなら build-env のシード集合に続けて、合成環境でビルドする残りのパッケージ（指定パッケージ自身を含む）を出力します。

#### env bootstrap

`env bootstrap` はターゲット指定を取りません。対象は常に HOST（`Target::BUILD_ENV`、amd64 / v4）です。

対象は `BOOTSTRAP` に載るパッケージ（x86_64 ツールチェーン、POSIX PATH、cmake、mold）の依存閉包です。他 GNU triple の toolchain は配列にありません。linux-headers と、最終 gcc が引く gmp などはグラフ経由で入ります。python は `build-env` 合成後に組みます。閉包は一度だけトポロジカル順に歩きます。あるパッケージが失敗しても後続のうち依存が揃っているものは実行し、失敗した依存を持つノードは理由を出してスキップします。1 件以上失敗した場合は非 0 で終了し、エラーに失敗件数と統合ログのパスを含めます。

フローの図は [build-flow.md](../package/build-flow.md) です。

統合ログは実行のたびに `packages/logs/bootstrap.log` を作り直します。進捗行と各 `podman` 出力を時系列で追記します。通常の `build` はこのファイルを開きません。

`env bootstrap` は `env compose` を呼びません。自前のビルド環境イメージが必要な場合は、bootstrap の次に `env compose` を実行します。

## パッケージ定義の規則

- 実装の正本は Rust の `Package` トレイトです。パッケージごとの定義コードは `builder/src/package/definition/{name}.rs` に配置し、仕様文書は [package/definition/](../package/definition/) にまとめます。
- 共通のビルド環境方針については [package](../package/README.md) を参照してください。
- パッケージイメージには各パッケージ固有のファイルのみを含め、依存パッケージのファイルは含めません。実行時依存は `dependencies`、ビルド時のみ必要な依存は `build-dependencies` として分離して宣言します。
- SPDX SBOM は Image Layout 内の `sbom.spdx.json` に出力します。なお、OCI attestation（referrers）としての外部公開は現時点では未実装です。
