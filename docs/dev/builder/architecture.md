# builder アーキテクチャ

パッケージおよびパッケージイメージのビルドツール（builder）の内部設計と実装構成を定めます。
コマンドの外部仕様については [cli.md](./cli.md) を参照してください。

## 概要

- 実装言語には Rust 1.98.1（edition 2024）を採用します。使用するツールチェーンはリポジトリルートの `rust-toolchain.toml` で固定します。
- 単一のバイナリ（`containers-builder`）で、フラットなサブコマンド（照会: `ls` / `info` / `tree` / `targets` / `docs` / `status`、作業: `fetch` / `build` / `check` / `pack` / `compose` / `restore` / `clean`、配布: `index` / `verify` / `push`、消費: `sync`、環境: `env`）を提供します。ソースコードは `builder/` ディレクトリで管理します。
- コマンド実行はリポジトリのルートディレクトリで行い、作業ディレクトリ（`packages/` および `oci-layouts/`）を相対パスで参照します。

## モジュール構成

| モジュール | 役割 |
|-|-|
| `main.rs` | コマンドライン引数の解析、プロセスの起動、および終了ステータスの管理 |
| `command/` | サブコマンドの実行ロジック（`mod` = 引数解析と選択、`query` = 照会、`work` = 作業、`registry` = `index` / `push`、`sync` = 消費側の同期、`env` = ビルド環境、`docs` = 定義文書の生成ブロック再生成） |
| `registry/` | レジストリ配布・消費側の実装（`index` = index artifact 生成、`skopeo` / `pull` = レジストリ転送と層 pull、`resolve` / `config` / `dockerfile` / `lockfile` = `sync` の解決・設定・書換え・記録、`cache` = XDG キャッシュ、`layout_io` = layout 入出力） |
| `package/describe.rs` | `docs` 生成用の構造情報収集（recipe 評価・フラグ差分・configure 呼出・スクリプト variant の射影） |
| `error.rs` / `process.rs` | 型付きエラー定義と外部コマンドプロセスの実行制御 |
| `package/mod.rs` | `Package` および `Bootstrap` トレイトと共通データ構造の定義 |
| `package/definition/` | 個別パッケージの定義実装と `PACKAGES` 登録一覧 |
| `package/{fetch,build}.rs` | ソースコードの取得・検証とパッケージのコンパイル制御（シード環境および自前環境） |
| `package/build/container.rs` | ビルドのファイルロック、成果物の atomic publish、コンテナ起動引数と plan の組み立て |
| `package/recipe.rs` | 型付きビルド手順（`Recipe` / `Step`）から実行 plan へのコンパイルと bash 描画 |
| `package/runner.rs` / `package/sandbox.rs` | plan を bash へレンダリングせず直接解釈する実行層（コンテナ内 `run-recipe` の実装） |
| `package/env.rs` | シード用 Arch Linux コンテナイメージのダイジェスト固定値の保持 |
| `package/validate.rs` | 宣言された依存関係と ELF バイナリ内の動的リンク（`DT_NEEDED`）の整合性検証 |
| `image.rs` | パッケージ成果物からの Image Layout および SPDX SBOM の生成と、`compose` による出力イメージ合成 |
| `workspace.rs` | `packages/` および `oci-layouts/` の相対パス解決と作業ツリーの管理（成果物、ビルド作業、個別ログ、`bootstrap.log`） |
| `flags.rs` | `Cflag` および `Ldflag` トークンの保持と、環境変数 `CFLAGS` / `LDFLAGS` への合成 |
| `target.rs` | 対象 CPU プラットフォーム（`Target` / variant 列挙型、GNU triple、クロスプレフィックス `CROSS`）の管理 |
| `oci/` | OCI Image Layout のデータモデル構築とファイル配置（`model` / `layout` / `layer`） |
| `sbom.rs` | SPDX 2.3 JSON 形式の SBOM 生成 |
| `util.rs` | OCI 用 SHA-256、ソース検証用 SHA3-512 / BLAKE3（512 bit）、および RFC 3339 タイムスタンプ生成 |

`lib.rs` では外部および統合テスト向けに各モジュールを公開します。
OCI 仕様準拠のテスト群は `tests/oci_spec.rs`（本体は `tests/oci_spec/`）、CLI 仕様のテスト群は `tests/cli.rs` に配置しています。スクリプト描画・plan の検証は `tests/golden_scripts.rs`・`tests/golden_plans.rs`・`tests/render_scripts.rs`（fixture は `tests/golden/`・`tests/golden-plans/`・`tests/support/`）です。

## 依存クレート

直接依存クレートは、サプライチェーンの安全性を保つため `builder/Cargo.toml` で `=` によりバージョンを完全固定します。

| クレート | 用途 |
|-|-|
| `clap` | コマンドライン引数の解析と CLI 構造の定義 |
| `sha2` | OCI blob ダイジェストおよび DiffID 用の SHA-256 算出 |
| `sha3` | パッケージソースコード検証用の SHA3-512 算出 |
| `blake3` | パッケージソースコード検証用の BLAKE3（出力 512 bit）算出 |
| `serde` / `serde_json` | OCI メタデータおよび SPDX SBOM の JSON シリアライズ / デシリアライズ |
| `thiserror` | 内部処理に応じた型付きエラーの定義 |
| `tempfile` | tar/zstd 処理および staging layout 用の一時作業ディレクトリ管理 |
| `jiff` | RFC 3339 形式に準拠した作成時刻の算出 |
| `fd-lock` | 同一パッケージ・ターゲットの同時ビルドを直列化するファイルロック |

開発依存（`[dev-dependencies]`）として、CLI 統合テスト用に `assert_cmd` / `predicates` を使用します。

## 外部コマンドの利用

| コマンド | 用途 |
|-|-|
| `curl` | 上流ソースコードの取得（`fetch`） |
| `tar` | ソースコードの展開、およびレイヤー tarball の生成（`--sort=name` などを指定してビルドの再現性を確保） |
| `zstd` | レイヤー tarball の圧縮（`--single-thread`。`pack` の既定はレベル 22、`compose` と `--profile dev` はレベル 3。20〜22 は `--ultra` を付与） |
| `podman` | 隔離されたビルド環境コンテナ上でのパッケージコンパイル |
| `/bin/bash` | 描画したビルドスクリプトの構文検査（`bash -n`）および golden・照会・デバッグ用途のスクリプト描画。実実行は外部コマンドではなく、コンテナ内の `run-recipe` runner が plan を直接解釈します |
| `readelf` | ELF バイナリの動的リンク依存（`DT_NEEDED`）の解析（`check`） |

## データフロー

```
fetch            ソース取得 + SHA3-512 / BLAKE3 検証 + 展開（--deps で依存閉包）
    ↓                 packages/sources/{package}/
env bootstrap    build-env の依存閉包を HOST 上で一度たどり、失敗しても後続を続ける
    ↓                 packages/artifacts/ および packages/logs/bootstrap.log
build            依存関係をトポロジカル順に取得・ビルドしたのち、podman 上でビルドスクリプトを実行
    ↓                 packages/artifacts/{package}-{version}-{arch}-{variant}/
                      packages/logs/{package}-{version}-{arch}-{variant}.log
check            宣言された依存関係とパッケージ内 ELF の NEEDED を静的照合
pack             パッケージ成果物をレイヤー化して Image Layout を生成 + SPDX SBOM 出力
    ↓                 oci-layouts/{package}-{version}/
compose          dependencies の推移閉包を単一の出力イメージへ合成
    ↓                 oci-layouts/{name}-{arch}-{date}-{revision}/
index            oci-layouts/ から index artifact を生成
    ↓                 oci-layouts/index/
push             パッケージイメージと index を skopeo 経由で公開
    ↓                 {registry}/packages/{name}:{version}、{registry}/packages/index:{yyyymmdd}-{rev}
sync             消費側の containers.json から Dockerfile 生成ブロックと lockfile を同期
    ↓                 containers.lock.json、キャッシュ $XDG_CACHE_HOME/containers-builder/
clean            指定したディレクトリだけを削除する（--packages-build、--packages-artifacts、--packages-logs、--oci-layouts、--oci-layouts-index。少なくとも一つ必須）
ls / info / tree / targets / env plan  照会系（tree --reverse は逆依存、env plan はシード相の最小集合）
docs             定義文書（docs/dev/package/definition/）の生成ブロックと索引、および docs/dev/package/{README,host-tools,toolchain}.md の名前つき GENERATED:{name} ブロックを PACKAGES から再生成（--check で差分検査）
```

ビルドスクリプトへ渡す環境変数と `/dep` へのマウントは [package/README.md](../package/README.md) です。

### 依存グラフとスキップ

`build` は依存パッケージを先にビルドします。グラフは `Package` の宣言辺のみから構成され、`Via::Ordered` 辺だけがビルド順に参加します（Ordered 部分グラフは常に DAG で、未宣言の循環はエラー）。循環を断つ辺は `Via::Seed` / `Via::Artifact` / `Via::Optional` で明示し、シード相（`is_seed` = `Bootstrap::seed` の BUILD_ENV ビルド）では `Via::Seed` 辺が distro パッケージで担保され、それ以外では成果物を要求します。`own`（合成済みイメージの使用、`pacman` 前置きの有無、ネットワーク遮断）だけを `build_env_ready()` で決めます。成果物ディレクトリが既に存在し空でない場合はそのターゲットのコンパイルをスキップし、ビルド途中で失敗した成果物は不整合を防ぐため自動的に削除します。`podman` の出力は `packages/logs/` の個別ファイルへ tee し、失敗してもログは残します。

### runner

コンテナ内で plan を解釈する runner は、ビルドツール自身を `/tmp/runner` にマウントして起動します。マウントするバイナリは次の優先順で選びます。

1. `CONTAINERS_BUILDER_RUNNER` 環境変数の明示指定
2. cargo 規約パス `target/x86_64-unknown-linux-musl/<profile>/containers-builder` の musl 静的ビルド
3. 実行中のバイナリ（フォールバック）

musl 静的ビルドはコンテナ側 libc に依存しないため推奨します（手順は [builder/README.md](../../../builder/README.md)）。フォールバック時は、コンテナ側 glibc がホストより古いと runner が起動しない旨の警告を一度だけ出力します。実際にマウントした runner のパスは個別ログの `runner:` 行に記録します。

runner の意味論は bash 描画スクリプトと揃えています。

- `$(cmd)`・`$(cmd || true)` は失敗時も部分 stdout を保持します（裸の代入 `X=$(f)` のみ errexit で中断）。
- `ln -sf t dir/` は実在ディレクトリ配下にリンクします。
- `cp -a` 相当のコピーは symlink・mode・mtime（ディレクトリ含む）を保持します。owner 保持だけは、コンテナ内で一様に root のため意図的に省略しています。
- glob は `*`・`?`・`[...]` クラスと dotfiles 規則に従います。
- `cd` 後の `PWD` は正規化済みパスです。
- `if cmd` の条件コマンド出力はログへ継承します。

### パッケージの種別と追加

- ビルド専用のパッケージ定義（`build_only`）は、中間成果物としてのみ扱い、パッケージイメージ化は行いません。これらは他のパッケージの `build-dependencies` にのみ現れます。
- `Bootstrap` トレイトを実装したパッケージは、シード環境（Arch Linux イメージ）上で直接ビルドできます。判定は `Package::as_bootstrap()` で行い、登録一覧は持ちません。`env bootstrap` の対象は `build-env` の依存閉包から導出します。
- 自前で合成したビルド環境イメージ上では `--network=none` を指定して完全に隔離された状態でビルドします。一方、シード環境では `Package::pacman()` によるパッケージ導入や、カタログのクロス gcc 導入のためにネットワークアクセスを許可します。

新規パッケージを定義する場合は次の手順です。

1. `builder/src/package/definition/{name}.rs` に `Package` トレイトを実装します。
2. `builder/src/package/definition/mod.rs` の一覧へ登録します。
3. `flags(target)` メソッドを実装します（フラグを要求しない場合は空の `Flag` を返します）。

`Bootstrap` を実装するのは x86_64 ツールチェーン、POSIX PATH、cmake、mold、linux-headers です。他の Host tools は `env bootstrap` の対象に依存グラフ経由で入ります。

### 定義文書の生成

各パッケージのビルド設定は [`docs/dev/package/definition/{name}.md`](../package/definition/) に記述します。1 パッケージ 1 ファイルです。各ファイルの先頭部（タイトル・メタデータ・ソース・依存表）と索引表は `<!-- GENERATED -->` マーカーで囲まれた生成ブロックであり、`containers-builder docs` が `PACKAGES` から再生成します（`docs --check` で差分検査）。手書きの補足はマーカーの外側に書きます。索引は [definition/README.md](../package/definition/README.md) です。Host tools の共通前処理については [host-tools.md](../package/host-tools.md)、プレフィックス付きツールチェーンのステージ順については [toolchain.md](../package/toolchain.md) を参照してください。

モジュール名はパッケージ名のハイフンをアンダースコアに置き換えた名前です（`linux_headers.rs` は `linux-headers`、`x86_64_linux_gnu_binutils.rs` は `x86_64-linux-gnu-binutils` に対応します）。プレフィックス付きパッケージも 1 パッケージ 1 ファイルです。ステージ共通の実装（レシピ・依存表・バージョン）は `definition/` 直下の `binutils.rs` / `gcc.rs` / `gcc_stage1.rs` / `gcc_stage2.rs` / `glibc.rs` / `glibc_headers.rs`（共有部品は `stages.rs`）に置き、各タプルのリーフは `impl Package` / `impl Bootstrap` で共有実装へ委譲します。

## クロスビルドの構成

ビルド環境イメージは amd64 / v4 上で動作します。非 amd64 プラットフォーム向けパッケージは、イメージ内に配置した `{triple}-gcc` を用いてクロスコンパイルします。PGO は生成バイナリを実行可能なネイティブビルドにのみ適用するため、クロスビルドではプロファイル実行（QEMU 利用を含む）を行いません。なお、自前イメージ合成前のシード期間に限り、Arch Linux 上で pacman を用いてクロス gcc を導入します。

- カタログパッケージのビルド時は、対象 architecture の GNU triple を `TRIPLE` に渡し、クロス接頭辞 `CROSS` は常に `{TRIPLE}-` とします（amd64 の場合は `x86_64-linux-gnu-`）。ネイティブビルドかクロスビルドかの判定は、`TRIPLE` と `HOST_TRIPLE` の一致によって行います。
- Host tools のコード生成側 GNU triple はパッケージ Platform の triple です（vendor 名の `pc` は含めません）。他パッケージがビルド時に実行する python / cmake / meson / ninja / perl および C ツールチェーンは `host_dependencies` 辺として宣言され、HOST 成果物を `dep:false` でマウントします。依存辺の `Via` が充足方法（Ordered / Seed / Artifact / Optional）を表し、`Via::Ordered` 辺のみがビルド順に参加するため、暗黙の依存は存在せず循環はすべて切断辺で明示されます。
- クロスビルドした `{triple}-glibc` には、クロス gcc が要求するディレクトリ構造に合わせて `usr/{triple}/{include,lib,lib64}` を置きます。Host tools とカタログの `--sysroot` はこの `usr/{triple}` を優先します。
- 実行時 libc は `{triple}-glibc` です。
- `linux-headers` の配置規則については [linux-headers.md](../package/definition/linux-headers.md) を参照してください。
- 最終 gcc の sanitizer 設定および PGO の適用方針については [toolchain.md](../package/toolchain.md) を参照してください。
- qemu における pkg-config の設定と `--sysroot` の扱いについては [qemu.md](../package/definition/qemu.md) を参照してください。

## レイヤーとメタデータの生成

- 1 パッケージを 1 レイヤーとして扱います。`packages/artifacts/` の未圧縮成果物を tar 形式でまとめ、zstd で圧縮して blob を生成します。
- 未圧縮 tar のダイジェストである DiffID をレイヤー内容の恒久的な識別子として使用します。zstd の圧縮レベルは layer digest にのみ影響し、DiffID には影響しません。
- `pack` の既定はレベル 22（`--profile release`）、`compose` の既定はレベル 3（`--profile dev`）です。既定の merge は既存 platform の manifest を保持するため、同一 layout 内で圧縮レベルの異なる platform が併存します。全 platform を同じレベルに揃えるときは `pack --replace` です。
- Layout の生成時は、一時的な staging ディレクトリで構造を構築したのち、不可分（アトミック）に配置を置き換えます。これにより、生成途中で処理が失敗した場合でも既存の `oci-layouts/` が保護されます。
- レイヤーのハッシュ計算はストリーミング方式で行い、アーカイブ全体をメモリ上に展開しません。
- SPDX SBOM は Layout ディレクトリ内の `sbom.spdx.json` に配置します。OCI Image Specification ではレイアウト内の未知の追加ファイルを許容しているため、この配置は仕様に適合します。なお、referrers による attestation としての公開は現時点では未実装です。

## コンテナ内ステージングの規約

- 生成オーバーレイ（la-overlay / pc-overlay / 各種ラッパ）はホスト側で
  `packages/build/{target}/.gen/{pkg}/` に生成し、コンテナの `/tmp` 配下へ
  read-only でマウントします。ビルドツリー（`/build` マウント）配下には
  置きません。openssl の `make clean` のように配下のシンボリックリンクを
  全削除する手順があるためです。
- dep 成果物の `.la` ファイルのうち、cc1/lto1 が実行時リンクする
  ライブラリ（gmp / mpfr / mpc / isl）は la-overlay に staging しません。
  `installed=no` の dep `.la` が `dependency_libs` に入ると libtool が
  install 時に relink し、relink 出力と同名の `.so` が lto1 の読み込み
  中に truncate され、衝突するためです。`.so` / `.a` のシンボリックリンクは
  残るため `-l` による解決は維持されます。
- dep glibc の `libc.so` リンカスクリプトは `/lib/libc.so.6` 等の絶対パスを
  参照するため、コンテナ内で `/lib` を glibc のライブラリディレクトリへの
  シンボリックリンクとして生成します（ターゲット glibc を優先し、
  Canadian cross ではホスト glibc にフォールバック）。
- `ld` はリンカスクリプトの実パスが sysroot 配下にある場合のみ、
  スクリプト内の絶対パスへ sysroot を適用します。このため shadow
  sysroot の lib ディレクトリは `/dep` への symlink ではなく実コピーです。

## マルチターゲット検証の状況

2026-09-21 時点で次のカタログビルドが完了しています。

- amd64 / v4（BUILD_ENV ベースライン）
- amd64 / v1（x86-64 baseline への回帰。Canadian gcc を含む）
- arm64 / v8（カタログ 40 件 + toolchain）
- riscv64 / rva20u64（同規模）

`check --all-packages` による動的依存検証は全件パスしています。ppc64le / s390x は
bootstrap 全体の検証として別途整理するため本範囲外です。生成バイナリを
実行できないクロスビルドでは PGO を適用しません。

## 現状の制約事項

- 対象とする architecture / variant の一覧は [targets.md](../../user/targets.md) および `targets` コマンドを参照してください。シード環境ではクロス gcc が提供されていないため、`--all-targets` 指定時は ppc64le および s390x をスキップします。合成済み `build-env` が利用可能な状態ではスキップせずにビルドを実行します。
- `pack --with-build` は `Target::all()` 上でパッケージビルドを連鎖的に実行し、定義の存在しないターゲットはビルドをスキップして警告を出力します。`compose --with-build` は `-t` で指定したターゲットのみを対象とし、非対応の場合はスキップではなくエラーになります。
- `compose` の `-t` は任意ターゲットを受理しますが、実行時依存の `{triple}-gcc` 成果物は `artifact_target` により常に BUILD_ENV（amd64 / v4）へ配置される一方、`compose_layer_dirs` はこの正規化を適用しません。そのため C++ ランタイム依存を持つパッケージは BUILD_ENV 以外では構造的に合成できません（既知の制約）。
