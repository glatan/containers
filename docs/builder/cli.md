# containers-builder CLI

パッケージおよびパッケージイメージのビルドツールの仕様を定めます。
内部実装の構成については [architecture.md](./architecture.md)、生成する Image Layout の OCI 仕様準拠については [oci](../oci.md) を参照してください。

実行バイナリ名は `containers-builder` であり、ソースコードはリポジトリ内の `builder/` ディレクトリで管理します。

## 概要

本ビルドツールは、利用者の目的に応じて次の 2 系統の機能を提供します。

### パッケージ開発者向け機能（本バイナリの主機能）

- ソースコードの取得、パッケージのコンパイル、OCI Image Layout の生成、SPDX SBOM の出力、および公開用アーカイブの作成を一貫して実行します。
- ソフトウェアの複数バージョンの並行管理に対応します。
- 直接依存するクレートのバージョンは `Cargo.toml` で `=` により完全固定します。
- アーカイブ生成には `tar` コマンド、レイヤー圧縮には `zstd` コマンドをそれぞれ外部プロセスとして呼び出します。

### 利用者向け機能

- パッケージの選択、依存解決、および出力イメージの合成は `compose` サブコマンドで行います。合成および出力イメージの設計方針については [パッケージイメージ](../image/README.md) を参照してください。

## コマンド体系

サブコマンドの引数に指定する名前は、`x86_64-linux-gnu-glibc` や `openssl` などのパッケージ名です。コマンドはフラットな構成で（`env` のみサブコマンド群を持ちます）、作業系の動詞はパイプラインステージ（fetch → build → check → pack → compose → clean）に対応します。このほか、コンテナ内で `BuildPlan` を実行する隠しサブコマンド `run-recipe` があります（podman のエントリポイント専用で、利用者向けではありません）。

### 照会

| コマンド | 説明 |
|-|-|
| `ls [--images]` | 定義済みパッケージの一覧を表示する（`--images` で公開対象に絞る） |
| `info {package} -t {arch}/{variant}` | パッケージ定義のメタデータおよび設定内容を表示する |
| `tree {package} -t {arch}/{variant} [--reverse]` | パッケージの依存ツリー（`--reverse` で逆依存）を表示する |
| `targets` | 有効な `arch`/`variant` の一覧を表示する |
| `docs [--check]` | 定義文書（[package/definition/](../package/definition/)）の生成ブロックと索引、および `docs/package/{README,host-tools,toolchain}.md` の名前つき `GENERATED:{name}` ブロックを `PACKAGES` から再生成する（`--check` は差分の検査のみ） |

`info` は `-t` で対象 Platform を指定し（必須）、ターゲットに応じたロール・依存・CFLAGS / LDFLAGS・PGO・recipe（env / `-std` / sysroot / workdir / configure 引数）を表示します。成果物の配置ターゲットが指定ターゲットと異なる場合は `成果物ターゲット` を、対応する全ターゲットを `対応ターゲット` に列挙します。

`tree` は実行時依存・ビルド時依存・ホスト依存を再帰的に表示します。辺とノードに付くタグの意味は次のとおりです。

| タグ | 意味 |
|-|-|
| `[build]` | ビルド時依存の辺 |
| `[host]` | HOST 成果物をマウントして実行するツールの辺 |
| `[seed]` / `[seed:{name}]` | シード相を distro パッケージ（空欄はベースイメージ同梱）が担保する切断辺 |
| `[artifact]` | 常に成果物が必須の切断辺 |
| `[optional]` | あれば使う任意辺 |
| `[bootstrap]` | `Bootstrap` を実装しシード上で組めるノード |
| `{arch}/{variant}` | 異なる Target の成果物を参照する辺 |

`--reverse` はこのパッケージに依存する側をたどり、同じタグを depender 側の意味で付与します。

### 作業

| コマンド | 説明 |
|-|-|
| `fetch {package\|--all} [--deps]` | 上流ソースコードを取得して展開する（`--deps` で依存閉包も取得） |
| `build {package\|--all} (-t ...\|--all-targets)` | パッケージをコンパイルし、`packages/artifacts/` に成果物を出力する |
| `check {package\|--all}` | 宣言された依存関係と、パッケージ成果物内 ELF バイナリの動的リンク依存を静的に照合・検証する |
| `pack {package\|--all}` | パッケージ成果物から OCI Image Layout を生成する（SPDX SBOM の出力を含む） |
| `compose {package} -t {arch}/{variant}` | `dependencies` を再帰的に探索し、単一の出力イメージを合成する |
| `index [--registry {prefix}] [--check]` | `oci-layouts/` からレジストリ index artifact を生成する |
| `push {package\|--all} [--registry {prefix}] [--dry-run]` | パッケージイメージをレジストリへ push する（`--all` は index artifact も公開） |
| `sync {config} [--check] [--index {ref}] [--registry {prefix}] [--output {dir}] [--no-push]` | 消費側の設定ファイルに基づき Dockerfile の GENERATED ブロックを同期する |
| `clean {package\|--all} (-t ...\|--all-targets)` | 成果物・ビルド作業ツリー・個別ログを削除する（`--images` で `oci-layouts/{name}-{version}` も対象。`--all --images` で `oci-layouts/index` も対象） |

`--all` を取るコマンドは全件を処理し、失敗を集約して最後に非 0 で終了します（`build` は `--keep-going` 指定時のみ失敗後も継続し、未指定時は最初の失敗で終了します）。

#### build のオプション

- `-t {arch}/{variant}`: 対象ターゲットを指定します（複数回指定可。`targets` コマンドで有効な一覧を確認できます）。
- `--all`: 全パッケージを単一のトポロジカル順でビルドします。この場合 `-t` に対応しないルートはエラーにせずスキップします（`--all-targets` と同じ緩和）。
- `--all-targets`: `supports` が真の architecture / variant をビルド対象とします。最終 gcc / binutils の製品キューはこれより狭く、[toolchain.md](../package/toolchain.md) です。シード環境ではクロスコンパイラが存在しないため ppc64le および s390x を自動的にスキップします。`-t` との同時指定はできません。
- `--force`: 成果物が存在しても対象パッケージを再ビルドします（依存はキャッシュ済みなら再利用します）。
- `--keep-going`: 失敗しても無関係な後続のビルドを継続し、最後に失敗一覧を集約して非 0 で終了します。
- `--no-deps`: 依存をビルドせず対象パッケージのみをビルドします（`--all`・`--keep-going` との同時指定はできません）。
- `--dry-run`: ビルドせず計画のみを表示します（`[cached]` / `[will-build]` / `[skip:unsupported]` / `[skip:no-cross]`）。

`build` の実行時は、対象パッケージをコンパイルする前に、指定した Target ごとの宣言済み依存を先にビルドします。依存グラフの構成と順序付けの規則（`Via` 辺）は [architecture.md](./architecture.md) を参照してください。`--all-targets` はこのグラフを Target ごとに組み、（パッケージ, ターゲット）単位の重複を除いた単一の順序で処理します。非対応の `-t` はエラーです。

イメージ参照、シード向け `pacman` 前置き、ネットワーク許可だけを、合成済みビルド環境の有無で切り替えます。ソースコードが未展開の場合は自動的に `fetch` を実行します。成果物ディレクトリ（`packages/artifacts/`）に既にファイルが存在する場合は、そのターゲットのビルドをスキップします。ビルドに失敗した場合は、不完全な成果物を残さないよう出力ディレクトリを自動的に削除します（最初からビルドを再試行する場合は、該当する成果物ディレクトリを手動で削除します）。`--keep-going` 指定時を除き、最初の失敗でコマンドは終了します。

実コンパイルを開始する直前に `{name} {version} ({arch}/{variant}) をビルドします` という進捗行を出します。

- `podman` の標準出力と標準エラー出力は、端末と `packages/logs/{name}-{version}-{architecture}-{variant}.log` へ同時に書き込みます。失敗してもこの個別ログは削除しません。
- `pack --with-build` および `compose --with-build` も同じ個別ログを書きます。
- `curl` および `tar` による取得は、進捗行と端末への出力のみです。

#### clean

成果物の再ビルドは `artifact_ready` によるスキップがあるため、まず成果物を削除する必要があります。削除はシェルの `rm` ではなくこのコマンドを使います。Target の指定は `build` と同じで、`-t` または `--all-targets` が必須です。`--all` で全パッケージを対象にできます。`--dry-run` で削除対象の一覧だけを表示します。`--images` を付けると `oci-layouts/{name}-{version}` も削除対象に含めます。

削除対象は次のとおりです。

- `packages/artifacts/{name}-{version}-{arch}-{variant}`（と中断残骸の `{...}.tmp`）
- `packages/build/{arch}-{variant}/{name}-{version}`（ビルド作業ツリー）
- `packages/build/{arch}-{variant}/.gen/{name}-{version}`（生成オーバーレイ置き場）
- `packages/logs/{name}-{version}-{arch}-{variant}.log`（個別ログ）

削除対象外は `packages/sources/`（ソース）と compose の出力（`oci-layouts/{name}-{arch}-{date}-{revision}`、`{name}-{arch}` リンク）です。

#### fetch のオプション

- `--all`: 定義済みのすべてのパッケージを対象としてソースコードを取得します。
- `--deps`: 有効依存グラフの閉包ごと取得します。同一 URL は一度だけダウンロードします。

#### pack のオプション

- `--all`: 公開対象となっているすべてのパッケージイメージを生成します（`build-env` およびビルド専用パッケージは対象外、成果物のないエントリは `スキップ(パッケージなし)` を出して継続します）。
- `--with-build`: Image Layout の生成前に、対応するパッケージ成果物のビルドも併せて実行します。
- `--profile {dev|release}`: 圧縮プロファイルを指定します。`release` は公開用アーカイブとして zstd 圧縮レベル 22（`--ultra`）を用い、`dev` はレベル 3（既定値と同等）です。
- `--zstd-level {1-22}`: 圧縮レベルを直接指定します（デフォルトは 3 です。`--profile` との同時指定はできません）。

#### compose のオプション

- `-t {arch}/{variant}`: 合成対象のターゲットを指定します（必須）。なお実行時依存の `{triple}-gcc` 成果物は常に BUILD_ENV（amd64 / v4）に配置されるため、C++ ランタイムに依存するパッケージは現状 BUILD_ENV 以外では合成できません。
- `--with-build`: 合成前に必要なパッケージをビルドします。
- `--profile` / `--zstd-level`: `pack` と同じです。

`build-env` は `compose` / `pack` の対象外であり、`env compose` で合成します。

#### index

`index` は `oci-layouts/` に存在する全パッケージの layout を走査し、パッケージ名・バージョン・platform 別 manifest digest・依存関係・ライセンスを集約した index JSON を OCI artifact として `oci-layouts/index/` に生成します。`--check` は既存の生成物との差分のみを検査します。

- layout が無いパッケージは `スキップ(layout なし)` を出して収録しません。
- 依存エントリの `platform` は `artifact_target` 解決済みの成果物 platform です（例: `{triple}-gcc` の実行時依存は常に amd64 / v4）。
- 収録したパッケージの依存先が index に存在しない、または依存が要求する platform の manifest を持たない場合はエラーです。
- `build-env`（`compose_root`）は `env compose` の合成 layout を自己完結イメージ（依存空）として収録します。

#### push

`push` は `skopeo copy --all oci:{dir} docker://{ref}` でパッケージイメージを公開します。`--all` は index artifact を `{registry}/packages/index:{yyyymmdd}-{rev}`（同日タグの最大 rev + 1）で最後に公開します。レジストリ前置詞の既定は `ghcr.io` で、`--registry` または `CONTAINERS_REGISTRY` で上書きします。

- タグはイミュータブルとして扱います。既存タグが同一 digest を指す場合は冪等にスキップし、別の digest を指す場合は上書きせずエラーにします。
- push 前に `oci-layouts/index` のエントリとローカル layout の digest を照合し、世代ズレ（pack 後に index を再生成していない等）を拒否します。
- push 後の digest は `--digestfile` で検証し、期待値と一致しない場合はエラーにします。
- `--dry-run` は対象参照の一覧のみを表示します。

#### sync

`sync` は消費側リポジトリの設定ファイル（`containers.json`）を読み、各イメージ定義に対応する Dockerfile の `# BEGIN GENERATED:containers … # END GENERATED` ブロックを index 解決済みの pin 参照で書き換えます。解決結果は設定ファイルと同じディレクトリの `containers.lock.json` に記録します。詳細は [image/consume.md](../image/consume.md) を参照してください。

- `--check`: 再解決・再生成した内容と差分があれば失敗します（CI 向け。書き込みません）。
- `--index`: 使用する index 参照。`"auto"` は `skopeo list-tags` で最新の `{yyyymmdd}-{rev}` タグに解決します。既定は設定ファイルの `index` → `CONTAINERS_INDEX` → 内蔵 pin の順です。
- `--no-push`: mode `compose` で合成イメージを push せずローカル layout のみ生成します。

#### check

`check` は生成済みの `oci-layouts/` ではなく、`packages/artifacts/` に出力された未圧縮パッケージ成果物の ELF バイナリを直接検査します。`--all` では公開対象の全パッケージイメージを対象とします（ビルド専用パッケージは除外します）。`--all` 指定時、成果物のないエントリは `スキップ(パッケージなし)` を出して継続します。

### env — ビルド環境の管理

| コマンド | 説明 |
|-|-|
| `env status` | ビルド環境イメージの `正本:` 参照（合成済みなら layout の canonical path、未合成なら `未合成`）を表示する。シードの digest 固定値は常に併記する |
| `env bootstrap` | `build-env` の依存閉包を HOST（amd64 / v4）上で一度にコンパイルする |
| `env plan [{package}]` | 他パッケージを組める最小のシード相ビルド集合を出力する |
| `env compose` | ビルド環境イメージを合成する（`--profile` / `--zstd-level` は `pack` と同じ） |

#### env plan

`env plan`（引数なし）は、「これだけシード（Arch base-devel）上で組めば他のパッケージは合成済みビルド環境でビルドできる」最小集合を、`build-env` の依存閉包からトポロジカル順で出力します。`env plan {package}` にパッケージを渡した場合、そのパッケージが `Bootstrap` を実装しているならそのパッケージ自身のシード閉包（最小部分集合）を、実装していないなら build-env のシード集合に続けて、合成環境でビルドする残りのパッケージを出力します。指定パッケージがシード集合に既に含まれる場合は、出力の `Phase 3`（合成済みビルド環境でビルド）には現れません。

#### env bootstrap

`env bootstrap` はターゲット指定を取りません。対象は常に HOST（`Target::BUILD_ENV`、amd64 / v4）です。

対象は `build-env` の依存閉包（x86_64 ツールチェーン、POSIX PATH、cmake、mold、linux-headers とそれらの推移的依存）です。他 GNU triple の toolchain は閉包に含まれません。python は `build-env` 合成後に組みます。閉包は一度だけトポロジカル順にたどります。あるパッケージが失敗しても後続のうち依存が揃っているものは実行し、失敗した依存を持つノードは理由を出してスキップします。1 件以上失敗した場合は非 0 で終了し、エラーに失敗件数と統合ログのパスを含めます。

フローの図は [build-flow.md](../package/build-flow.md) です。

統合ログは実行のたびに `packages/logs/bootstrap.log` を作り直します。進捗行と各 `podman` 出力を時系列で追記します。通常の `build` はこのファイルを開きません。

`env bootstrap` は `env compose` を呼びません。自前のビルド環境イメージが必要な場合は、bootstrap の次に `env compose` を実行します。

## パッケージ定義の規則

- 実装の正本は Rust の `Package` トレイトです。パッケージごとの定義コードは `builder/src/package/definition/{name}.rs` に配置し、仕様文書は [package/definition/](../package/definition/) にまとめます。
- 共通のビルド環境方針については [package](../package/README.md) を参照してください。
- パッケージイメージには各パッケージ固有のファイルのみを含め、依存パッケージのファイルは含めません。実行時依存は `dependencies`、ターゲット向け `-I` / `-L` 入力となるビルド時依存は `build-dependencies`、ビルド中に HOST 側で実行するツールは `host-dependencies` として分離して宣言します。
- SPDX SBOM は Image Layout 内の `sbom.spdx.json` に出力します。なお、OCI attestation（referrers）としての外部公開は現時点では未実装です。
