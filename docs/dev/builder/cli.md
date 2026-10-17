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

サブコマンドの引数に指定する名前は、`x86_64-linux-gnu-glibc` や `openssl` などのパッケージ名です。コマンドはフラットな構成で（`env` のみサブコマンド群を持ちます）、作業系の動詞はパイプラインステージ（fetch → build → check → pack → compose → clean、逆方向の復元は `restore`）に対応します。このほか、コンテナ内で `BuildPlan` を実行する隠しサブコマンド `run-recipe` があります（podman のエントリポイント専用で、利用者向けではありません）。

### 照会

| コマンド | 説明 |
|-|-|
| `ls [--images]` | 定義済みパッケージの一覧を表示する（`--images` で公開対象に絞る） |
| `info {package} -t {arch}/{variant}` | パッケージ定義のメタデータおよび設定内容を表示する |
| `tree {package} -t {arch}/{variant} [--reverse]` | パッケージの依存ツリー（`--reverse` で逆依存）を表示する |
| `targets` | 有効な `arch`/`variant` の一覧を表示する |
| `docs [--check]` | 定義文書（[package/definition/](../package/definition/)）の生成ブロックと索引、および `docs/dev/package/{README,host-tools,toolchain}.md` の名前つき `GENERATED:{name}` ブロックを `PACKAGES` から再生成する（`--check` は差分の検査のみ） |

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
| `fetch {package\|--all-packages} [--deps]` | 上流ソースコードを取得して展開する（`--deps` で依存閉包も取得） |
| `build {package\|--all-packages} (-t ...\|--all-targets)` | パッケージをコンパイルし、`packages/artifacts/` に成果物を出力する |
| `check {package\|--all-packages}` | 宣言された依存関係と、パッケージ成果物内 ELF バイナリの動的リンク依存を静的に照合・検証する |
| `pack {package\|--all-packages} [-t {arch}/{variant}...] [--replace] [--remove-platform {arch}/{variant}...]` | パッケージ成果物から OCI Image Layout を生成する（既定は既存 layout の他 platform を保持する増分 merge。SPDX SBOM の出力を含む） |
| `compose {package} -t {arch}/{variant}` | `dependencies` を再帰的に探索し、単一の出力イメージを合成する |
| `index --registry {prefix} [--check] [--all-issues]` | `oci-layouts/` からレジストリ index artifact を生成する。前置詞は `--registry` または `CONTAINERS_REGISTRY` が必須（`--all-issues` で個別パッケージの失敗を集約報告） |
| `verify {package\|--all-packages}` | OCI layout が参照する全 blob の実在と digest を検査する（push 前の健全性検査） |
| `status {package\|--all-packages} [--registry {prefix}] [--local]` | artifacts/layout/index/レジストリの状態を一覧する（`--local` でリモート問い合わせを省略。リモート照会は前置詞が必須） |
| `push {package\|--all-packages} --registry {prefix} [--dry-run] [--force-replace] [--replace-platform {arch}/{variant}...] [--with-index]` | パッケージイメージをレジストリへ push する。前置詞は `--registry` または `CONTAINERS_REGISTRY` が必須（`--all-packages` は index artifact も公開） |
| `sync {config} [--check] [--index {ref}] [--registry {prefix}] [--output {dir}] [--no-push] [--profile {dev|release}] [--zstd-level {1-22}]` | 消費側の設定ファイルに基づき Dockerfile の GENERATED ブロックを同期する |
| `clean {package\|--all-packages} (--packages-build\|--packages-artifacts\|--packages-logs\|--oci-layouts\|--oci-layouts-index)...` | 指定したディレクトリだけを削除する。フラグは一つ以上必須で、併用は和集合になる |
| `restore {package\|--all-packages} [-t {arch}/{variant}...] [--from-registry] [--registry {prefix}] [--force]` | layout（またはレジストリ）の blob から成果物を `packages/artifacts/` へ復元する（`--from-registry` は前置詞が必須） |

`--all-packages` を取るコマンドは全件を処理し、失敗を集約して最後に非 0 で終了します（`build` は `--keep-going` 指定時のみ失敗後も継続し、未指定時は最初の失敗で終了します）。

#### build のオプション

- `-t {arch}/{variant}`: 対象ターゲットを指定します（複数回指定可。`targets` コマンドで有効な一覧を確認できます）。
- `--all-packages`: 全パッケージを単一のトポロジカル順でビルドします。この場合 `-t` に対応しないルートはエラーにせずスキップします（`--all-targets` と同じ緩和）。
- `--all-targets`: `supports` が真の architecture / variant をビルド対象とします。最終 gcc / binutils の製品キューはこれより狭く、[toolchain.md](../package/toolchain.md) に記載しています。シード環境ではクロスコンパイラが存在しないため ppc64le および s390x を自動的にスキップします。`-t` との同時指定はできません。
- `--force`: 成果物が存在しても対象パッケージを再ビルドします（依存はキャッシュ済みなら再利用します）。
- `--keep-going`: 失敗しても無関係な後続のビルドを継続し、最後に失敗一覧を集約して非 0 で終了します。
- `--no-deps`: 依存をビルドせず対象パッケージのみをビルドします（`--all-packages`・`--keep-going` との同時指定はできません）。
- `--dry-run`: ビルドせず計画のみを表示します（`[cached]` / `[will-build]` / `[skip:unsupported]` / `[skip:no-cross]`）。

`build` の実行時は、対象パッケージをコンパイルする前に、指定した Target ごとの宣言済み依存を先にビルドします。依存グラフの構成と順序付けの規則（`Via` 辺）は [architecture.md](./architecture.md) を参照してください。`--all-targets` はこのグラフを Target ごとに組み、（パッケージ, ターゲット）単位の重複を除いた単一の順序で処理します。非対応の `-t` はエラーです。

イメージ参照、シード向け `pacman` 前置き、ネットワーク許可だけを、合成済みビルド環境の有無で切り替えます。ソースコードが未展開の場合は自動的に `fetch` を実行します。成果物ディレクトリ（`packages/artifacts/`）に既にファイルが存在する場合は、そのターゲットのビルドをスキップします。ビルドに失敗した場合は、不完全な成果物を残さないよう出力ディレクトリを自動的に削除します（最初からビルドを再試行する場合は、該当する成果物ディレクトリを手動で削除します）。`--keep-going` 指定時を除き、最初の失敗でコマンドは終了します。

実コンパイルを開始する直前に `{name} {version} ({arch}/{variant}) をビルドします` という進捗行を出します。

- `podman` の標準出力と標準エラー出力は、端末と `packages/logs/{name}-{version}-{architecture}-{variant}.log` へ同時に書き込みます。失敗してもこの個別ログは削除しません。
- `pack --with-build` および `compose --with-build` も同じ個別ログを書きます。
- `curl` および `tar` による取得は、進捗行と端末への出力のみです。

#### clean

`clean` は削除対象のディレクトリ種別をフラグで指定します。フラグは一つ以上が必須で、複数指定したときはその和集合を削除します。`--all-packages` は対象パッケージを全件に広げ、`--all-targets` は対象 platform を全件に広げます。いずれのオプションも選択範囲を広げるだけであり、削除対象とするディレクトリ種別を自動で増やすことはありません。`--dry-run` は削除を実行せず、削除予定のパスを `削除予定:` と表示します（この表示処理中はロックを保持しません）。

ターゲット（platform）単位で削除対象を指定するフラグは `--packages-build`、`--packages-artifacts`、`--packages-logs` の 3 つです。作業ツリーを削除するのは `--packages-build`、公開済みの成果物ディレクトリを削除するのは `--packages-artifacts`、個別ログファイルを削除するのは `--packages-logs` です。これらを指定する場合は `-t` または `--all-targets` が必須です。`--all-targets` は各パッケージが対応している platform のみを対象としますが、`-t` で明示した platform は対応の有無にかかわらず削除対象とします。

`--packages-build` は次を削除します。公開済みの成果物ディレクトリは残します。

- `packages/build/<arch>-<variant>/<name>-<version>`
- `packages/build/<arch>-<variant>/.gen/<name>-<version>`
- 成果物ディレクトリの兄弟ディレクトリである `{artifact}.tmp`（展開・ビルド途中の成果物）および `{artifact}.restore-bak`（`restore --force` による退避先）

`--packages-artifacts` は公開済みの成果物ディレクトリ `packages/artifacts/<name>-<version>-<arch>-<variant>` のみを削除します。`.locks`、`.tmp`、`.restore-bak` は残します。

`--packages-logs` は個別ログファイル `packages/logs/<name>-<version>-<arch>-<variant>.log` のみを削除します。`packages/logs/bootstrap.log` は残します。

OCI Layout ディレクトリを削除するフラグは `--oci-layouts` と `--oci-layouts-index` です。Layout は platform 単位ではなくパッケージ単位で管理されるため、この 2 つのみを指定する場合は `-t` や `--all-targets` を指定できません。ターゲット指定が必要なフラグ（`--packages-build` 等）と併用した場合は、`-t` が該当フラグの絞り込みのみに適用され、`--oci-layouts` は該当パッケージの Layout ディレクトリ全体を削除します。

`--oci-layouts` は `oci-layouts/<name>-<version>` を削除します。`oci-layouts/index`、compose による Layout（`oci-layouts/{name}-{arch}` および `{name}-{arch}-{date}-{revision}`）、`oci-layouts/.locks`、`oci-layouts/.tmp` は残します。

`--oci-layouts-index` は `oci-layouts/index` を削除します。`--all-packages` と同時にのみ指定できます。

上流ソースコードを展開した `packages/sources/` を削除するフラグはありません。

排他制御として、`build`、`pack`、`restore`、`compose` は `packages/artifacts/.locks/active.lock` を共有ロックで保持します。`--packages-artifacts`、`--oci-layouts`、`--oci-layouts-index` の各処理はこのロックを排他で取得できなければコマンド全体を拒否し、削除を行いません。`--packages-build` と `--packages-logs` は `active.lock` を取得しませんが、処理中のターゲットの flock が取得できない場合は `実行中のためスキップ` と表示して削除を保留し、コマンド自体は成功とします。一方、`--packages-artifacts` は対象ターゲットの flock が 1 つでも取得できない場合に、安全のためコマンド全体を拒否します。同様に、`--oci-layouts` および `--oci-layouts-index` は対象 Layout の出力ロックが取得できない場合にコマンド全体を拒否します。

ビルド作業ツリーのみを削除する場合は、次のコマンドを実行します。

```
containers-builder clean --all-packages --all-targets --packages-build
```

ビルド作業ツリーに加えて成果物と個別ログも削除する場合は、次のコマンドを実行します（`oci-layouts/` は保持されます）。

```
containers-builder clean --all-packages --all-targets --packages-build --packages-artifacts --packages-logs
```

#### fetch のオプション

- `--all-packages`: 定義済みのすべてのパッケージを対象としてソースコードを取得します。
- `--deps`: 有効依存グラフの閉包ごと取得します。同一 URL は一度だけダウンロードします。

#### pack のオプション

- `--all-packages`: 公開対象となっているすべてのパッケージイメージを生成します（`build-env` およびビルド専用パッケージは対象外、成果物のないエントリは `スキップ(パッケージなし)` を出して継続します）。
- `-t {arch}/{variant}`: 封入対象の platform を絞ります（複数回指定可）。指定 platform の成果物が無い場合はエラーです。既定（`-t` 省略）は成果物が揃う全 platform です。
- `--replace`: 既存 layout を全再生成します（`-t`・`--remove-platform` との同時指定はできません）。単一パッケージで指定した場合は、対応する全 platform の成果物が必要です。成果物が不足している platform があるとエラーとなり、layout は変更されません。`--all-packages` と併用した場合は、成果物が不足している platform について警告を表示したうえで layout の構成対象から外し、他のパッケージの pack は継続します。
- `--remove-platform {arch}/{variant}`: 既存 layout から指定した platform を除去します（複数回指定可）。`-t` と併用すると成果物の封入と指定 platform の除去を同時に実行します。単独で指定した場合は platform の除去のみを行い、新たな成果物は封入しません。なお、最後の 1 platform を除去しようとするとエラーになります（layout 全体を削除する場合は `clean --oci-layouts` を使用します）。
- `--with-build`: Image Layout の生成前に、対応するパッケージ成果物のビルドも併せて実行します（`-t` 指定時はそのターゲットのみをビルドします）。
- `--profile {dev|release}`: 圧縮プロファイルを指定します。`release` は公開用アーカイブとして zstd 圧縮レベル 22（`--ultra`）を用い、`dev` はレベル 3 を用います。配布用 layout を生成するコマンドであるため、pack の既定値は `release` です。開発用途には `--profile dev` を指定します。
- `--zstd-level {1-22}`: 圧縮レベルを直接指定します（`--profile` より優先されます。`--profile` との同時指定はできません）。

`pack` は既定で既存 layout の指定外 platform の manifest を digest 不変のまま保持します（増分 merge）。そのため、特定 platform の追加や更新の際に既存 platform の manifest を再生成する必要はありません。merge 対象の既存 layout が破損している場合はエラーとなり、`--replace` による全再生成を促します。また、同一パッケージに対する同時実行を防ぐため、`oci-layouts/.locks/{name}-{version}.lock` の flock により layout 単位で処理を直列化します。

#### restore

`restore` は `oci-layouts/{name}-{version}` の層 blob（`tar+zstd`）を `packages/artifacts/{name}-{version}-{arch}-{variant}` へ展開し、依存解決や増分 pack の入力となる成果物を復元します。build と同じ `packages/artifacts/.locks/` のロックを取り、`{artifact}.tmp` → rename で原子的に公開します。既存の成果物がある場合はスキップし、`--force` で上書きします。

- `-t {arch}/{variant}`: 復元する platform を絞ります（複数回可。既定は layout が持つ全 platform）。
- `--from-registry`: ローカルに layout が無い場合に `skopeo copy --all docker://{registry}/packages/{name}:{version}` で `oci-layouts/` へ layout 自体を先に復元します（`skopeo` が必要です。`--all` は全 platform を取得します）。`--all-packages` では layout が無いパッケージをスキップします。
- `--registry {prefix}`: `--from-registry` が参照するレジストリ前置詞です。`--registry` または `CONTAINERS_REGISTRY` が必須で、既定値はありません。

`build-env`（合成イメージ）とビルド専用パッケージは対象外です。

#### compose のオプション

- `-t {arch}/{variant}`: 合成対象のターゲットを指定します（必須）。なお実行時依存の `{triple}-gcc` 成果物は常に BUILD_ENV（amd64 / v4）に配置されるため、C++ ランタイムに依存するパッケージは現状 BUILD_ENV 以外では合成できません。
- `--with-build`: 合成前に必要なパッケージをビルドします。
- `--profile` / `--zstd-level`: 指定形式は `pack` と同じですが、既定値は `dev`（zstd 3）です。

`build-env` は `build` / `compose` / `pack` の対象外であり、`env compose` で合成します。

#### index

`index` は `oci-layouts/` に存在する全パッケージの layout を走査し、パッケージ名・バージョン・platform 別 manifest digest・依存関係・ライセンスを集約した index JSON を OCI artifact として `oci-layouts/index/` に生成します。`--check` は既存の生成物との差分のみを検査します。

- layout が無いパッケージは `スキップ(layout なし)` を出して収録しません。
- 依存エントリの `platform` は `artifact_target` 解決済みの成果物 platform です（例: `{triple}-gcc` の実行時依存は常に amd64 / v4）。
- 収録したパッケージの依存先が index に存在しない、または依存が要求する platform の manifest を持たない場合はエラーです。
- `build-env`（`compose_root`）は `env compose` の合成 layout を自己完結イメージ（依存空）として収録します。

#### push

`push` は `skopeo copy --all oci:{dir} docker://{ref}` でパッケージイメージを公開します。既存タグが同一 digest を指す場合は冪等にスキップし、新規 platform の追加のみを含む差分（リモート ⊆ ローカル）であれば additive 更新として上書きを許可します。一方、既存 platform の削除や層内容の変更が含まれる場合は push を拒否します。`--all-packages` では additive 条件に適合しないパッケージを集約して報告し、適合したパッケージのみを順次 push します。強制的にタグを置換する場合は `--force-replace` を指定します。また、正当な再ビルドにより成果物が更新された特定 platform のみ置換を許可し、他の platform は通常どおり検証する場合は `--replace-platform {arch}/{variant}` を指定します。`--with-index` は単一パッケージの push 後に index artifact も併せて公開します（単体 push では index に記録された digest と実際の layout とに乖離が生じるためです）。`--all-packages` は全パッケージの push 完了後、末尾で index artifact を `{registry}/packages/index:{yyyymmdd}-{rev}`（同日タグの最大 rev + 1）として公開します。レジストリ前置詞に既定値はなく、`--registry` または `CONTAINERS_REGISTRY` の指定が必須です。skopeo が使用する一時ディレクトリは `oci-layouts/.tmp` に固定されます。

- タグは原則として不変（イミュータブル）として扱います。既存タグが同一 digest を指す場合は冪等にスキップします。digest が異なる場合でも、リモートの全 platform がローカルに含まれ、かつ共通する各 platform の内容が一致しているときに限り、platform の追加として上書きを許可します。それ以外の場合は上書きせずエラーとします。内容の一致と判定されるのは、次のいずれかの条件を満たす場合です。
  1. manifest digest が完全に一致する。
  2. config および各 layer の digest が一致し、annotations が additive 条件に適合する。
  3. digest が異なる場合でも、config が `created` を除いて一致し、layer の個数および media type が一致し、`rootfs.diff_ids` が等しい（同一ペイロードの再エンコード。圧縮形式の変更は不適合）。
  ここで annotations の additive 適合とは、リモートに存在する全 annotation がローカルにも同値で保持されている状態を指します（ローカル側への新規追加は許容され、`org.opencontainers.image.revision` のみ値の差異や有無が許容されます）。
- push 前に `oci-layouts/index` のエントリとローカル layout の digest を照合し、世代ズレ（pack 後に index を再生成していない等）を拒否します。
- push 後の digest は `--digestfile` で検証し、期待値と一致しない場合はエラーにします。
- `--dry-run` は対象参照の一覧のみを表示します。

#### sync

`sync` は消費側リポジトリの設定ファイル（`containers.json`）を読み、各イメージ定義に対応する Dockerfile の `# BEGIN GENERATED:containers … # END GENERATED` ブロックを index 解決済みの pin 参照で書き換えます。解決結果は設定ファイルと同じディレクトリの `containers.lock.json` に記録します。詳細は [user/consume.md](../../user/consume.md) を参照してください。

- `--check`: 再解決・再生成した内容と差分があれば失敗します（CI 向け。書き込みません）。
- `--index`: 使用する index 参照を指定します。`"auto"` は `skopeo list-tags` で最新の `{yyyymmdd}-{rev}` タグに解決します。index 参照は `--index` > `CONTAINERS_INDEX` > 設定ファイルの `index` > 内蔵 pin の順に解決します。
- `--registry`: `"auto"` が参照する `{registry}/packages/index` の前置詞です。`"auto"` のときは `--registry` または `CONTAINERS_REGISTRY` が必須で、既定値はありません。生成ブロック内の FROM 参照や compose の push 先は、このフラグではなく index artifact の `registry` フィールドから導出します。
- `--output`: mode `compose` の合成 layout の出力先を変更します（既定はキャッシュの `composed/{name}-{tag}`）。
- `--no-push`: mode `compose` で合成イメージを push せずローカル layout のみ生成します。
- `--profile` / `--zstd-level`: mode `compose` の合成イメージの圧縮レベルを指定します（指定形式は `pack` と同じ。既定値は `dev`）。

#### check

`check` は生成済みの `oci-layouts/` ではなく、`packages/artifacts/` に出力された未圧縮パッケージ成果物の ELF バイナリを直接検査します。`--all-packages` では公開対象の全パッケージイメージを対象とします（ビルド専用パッケージは除外します）。`--all-packages` 指定時、成果物のないエントリは `スキップ(パッケージなし)` を出して継続します。

### env — ビルド環境の管理

| コマンド | 説明 |
|-|-|
| `env status` | ビルド環境イメージの `正本:` 参照（合成済みなら layout の canonical path、未合成なら `未合成`）を表示する。シードの digest 固定値は常に併記する |
| `env bootstrap` | `build-env` の依存閉包を HOST（amd64 / v4）上で一度にコンパイルする |
| `env plan [{package}]` | 他パッケージを組める最小のシード相ビルド集合を出力する |
| `env compose` | ビルド環境イメージを合成する（`--profile` / `--zstd-level` の形式は `pack` と同じ、既定値は `dev`） |

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
