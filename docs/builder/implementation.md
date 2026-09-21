# 実装構成

パッケージおよびパッケージイメージのビルドツール（builder）の内部設計と実装構成を定めます。
コマンドの外部仕様については [README.md](./README.md) を参照してください。

## 概要

- 実装言語には Rust 1.98.1（edition 2024）を採用します。使用するツールチェーンはリポジトリルートの `rust-toolchain.toml` で固定します。
- 単一のバイナリ（`containers-builder`）で、フラットなサブコマンド（照会: `ls` / `info` / `tree` / `targets`、作業: `fetch` / `build` / `check` / `pack` / `compose` / `clean`、環境: `env`）を提供します。ソースコードは `builder/` ディレクトリで管理します。
- コマンド実行はリポジトリのルートディレクトリで行い、作業ディレクトリ（`packages/` および `oci-layouts/`）を相対パスで参照します。

## モジュール構成

| モジュール | 役割 |
|-|-|
| `main.rs` | コマンドライン引数の解析、プロセスの起動、および終了ステータスの管理 |
| `command/` | サブコマンドの実行ロジック（`mod` = 引数解析と選択、`query` = 照会、`work` = 作業、`env` = ビルド環境） |
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
OCI 仕様準拠のテスト群は `tests/oci_spec.rs`、CLI 仕様のテスト群は `tests/cli.rs` に配置しています。

## 依存クレート

直接依存クレートは、サプライチェーンの安全性を保つため `builder/Cargo.toml` において `=` によりバージョンを完全固定します。

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
| `zstd` | レイヤー tarball の圧縮（`--single-thread`。通常ビルドはレベル 3、公開用ビルドはレベル 22 であり、20〜22 は `--ultra` を付与） |
| `podman` | 隔離されたビルド環境コンテナ上でのパッケージコンパイル |
| `/bin/bash` | 描画したビルドスクリプトの構文検査（`bash -n`）および golden・照会・デバッグ用途のスクリプト描画。実実行は外部コマンドではなく、コンテナ内の `run-recipe` runner が plan を直接解釈します |
| `readelf` | ELF バイナリの動的リンク依存（`DT_NEEDED`）の解析（`check`） |

## データフロー

```
fetch            ソース取得 + SHA3-512 / BLAKE3 検証 + 展開（--deps で依存閉包）
    ↓                 packages/sources/{package}/
env bootstrap    build-env の依存閉包を HOST 上で一度歩き、失敗しても後続を続ける
    ↓                 packages/artifacts/ および packages/logs/bootstrap.log
build            依存関係をトポロジカル順に取得・ビルドしたのち、podman 上でビルドスクリプトを実行
    ↓                 packages/artifacts/{package}-{version}-{arch}-{variant}/
                      packages/logs/{package}-{version}-{arch}-{variant}.log
check            宣言された依存関係とパッケージ内 ELF の NEEDED を静的照合
pack             パッケージ成果物をレイヤー化して Image Layout を生成 + SPDX SBOM 出力
    ↓                 oci-layouts/{package}-{version}/
compose          dependencies の推移閉包を単一の出力イメージへ合成
    ↓                 oci-layouts/{name}-{arch}-{date}-{revision}/
clean            成果物・ビルド作業ツリー・個別ログを削除（再ビルド前の掃除、--images で oci-layouts/ も対象）
ls / info / tree / targets / env plan  照会系（tree --reverse は逆依存、env plan はシード相の最小集合）
```

ビルドスクリプトへ渡す環境変数と `/dep` へのマウントは [package/README.md](../package/README.md) です。
`build` は依存パッケージを先にビルドします。ルートが `Bootstrap` を実装している場合、合成済みビルド環境の有無にかかわらずグラフには `Bootstrap` の依存を使います。`own`（合成済みイメージの使用、`pacman` 前置きの有無、ネットワーク遮断）だけを `build_env_ready()` で決めます。成果物ディレクトリが既に存在し空でない場合はそのターゲットのコンパイルをスキップし、ビルド途中で失敗した成果物は不整合を防ぐため自動的に削除します。`podman` の出力は `packages/logs/` の個別ファイルへ tee し、失敗してもログは残します。

コンテナ内で plan を解釈する runner は、ビルドツール自身を `/tmp/runner` にマウントして起動します。マウントするバイナリは `CONTAINERS_BUILDER_RUNNER` 環境変数の明示指定を最優先とし、次に cargo 規約パス `target/x86_64-unknown-linux-musl/<profile>/containers-builder` の musl 静的ビルドを探し、見つからなければ実行中のバイナリを使います。musl 静的ビルドはコンテナ側 libc に依存しないため推奨であり、`rustup target add x86_64-unknown-linux-musl` ののち `cargo build --target x86_64-unknown-linux-musl` で生成します。フォールバック時は、コンテナ側 glibc がホストより古いと runner が起動しない旨の警告を一度だけ出力します。実際にマウントした runner のパスは個別ログの `runner:` 行に記録します。

runner の意味論は bash 描画スクリプトと揃えています。`$(cmd)`・`$(cmd || true)` は失敗時も部分 stdout を保持し（裸の代入 `X=$(f)` のみ errexit で中断）、`ln -sf t dir/` は実在ディレクトリ配下にリンクし、`cp -a` 相当のコピーは symlink・mode・mtime（ディレクトリ含む）を保持し、glob は `*`・`?`・`[...]` クラスと dotfiles 規則、`cd` 後の `PWD` は正規化済みパス、`if cmd` の条件コマンド出力はログへ継承します。`cp -a` の owner 保持だけはコンテナ内で一様に root のため意図的に省略しています。
LTO や PGO の適用方針はパッケージの種別ごとに異なります。個別の指定内容は [各パッケージのビルド設定](../package/definition/) を参照してください。

ビルド専用のパッケージ定義（`build_only`）は、中間成果物としてのみ扱い、パッケージイメージ化は行いません。これらは他のパッケージの `build-dependencies` にのみ現れます。
`Bootstrap` トレイトを実装したパッケージは、シード環境（Arch Linux イメージ）上で直接ビルドできます。判定は `Package::as_bootstrap()` で行い、登録一覧は持ちません。`env bootstrap` の対象は `build-env` の依存閉包から導出します。
自前で合成したビルド環境イメージ上では `--network=none` を指定して完全に隔離された状態でビルドします。一方、シード環境では `Package::pacman()` によるパッケージ導入や、カタログのクロス gcc 導入のためにネットワークアクセスを許可します。

新規パッケージを定義する場合は、`builder/src/package/definition/{name}.rs` に `Package` トレイトを実装し、`builder/src/package/definition/mod.rs` の一覧へ登録します。`Bootstrap` を実装するのは x86_64 ツールチェーン、POSIX PATH、cmake、mold、linux-headers です。他の Host tools は `env bootstrap` の対象に依存グラフ経由で入ります。`flags(target)` メソッドの実装は必須であり、フラグを要求しない場合は空の `Flag` を返します。
文書としては、[`docs/package/definition/{name}.md`](../package/definition/) にビルド設定を記述します。1 パッケージ 1 ファイルです。索引は [definition/README.md](../package/definition/README.md) です。Host tools の共通前処理については [host-tools.md](../package/host-tools.md)、プレフィックス付きツールチェーンのステージ順については [toolchain.md](../package/toolchain.md) を参照してください。モジュール名の `linux_headers.rs` はパッケージ名 `linux-headers` に対応します。プレフィックス付きパッケージも 1 パッケージ 1 ファイルで、`x86_64_linux_gnu_binutils.rs` が `x86_64-linux-gnu-binutils` に対応します。ステージ共通の実装（レシピ・依存表・バージョン）は `definition/` 直下の `binutils.rs` / `gcc.rs` / `gcc_stage1.rs` / `gcc_stage2.rs` / `glibc.rs` / `glibc_headers.rs`（共有部品は `stages.rs`）に置き、各タプルのリーフは `impl Package` / `impl Bootstrap` で共有実装へ委譲します。

## クロスビルドの構成

ビルド環境イメージは amd64 / v4 上で動作します。非 amd64 プラットフォーム向けパッケージは、イメージ内に配置した `{triple}-gcc` を用いてクロスコンパイルします。PGO は生成バイナリを実行可能なネイティブビルドにのみ適用するため、クロスビルドではプロファイル実行（QEMU 利用を含む）を行いません。なお、自前イメージ合成前のシード期間に限り、Arch Linux 上で pacman を用いてクロス gcc を導入します。

- カタログパッケージのビルド時は、対象 architecture の GNU triple を `TRIPLE` に渡し、クロス接頭辞 `CROSS` は常に `{TRIPLE}-` とします（amd64 の場合は `x86_64-linux-gnu-`）。ネイティブビルドかクロスビルドかの判定は、`TRIPLE` と `HOST_TRIPLE` の一致によって行います。
- Host tools のコード生成側 GNU triple はパッケージ Platform の triple です（vendor 名の `pc` は含めません）。他パッケージがビルド時に実行する python / cmake / meson / ninja / perl は HOST 成果物をマウントします。
- クロスビルドした `{triple}-glibc` には、クロス gcc が要求するディレクトリ構造に合わせて `usr/{triple}/{include,lib,lib64}` を置きます。Host tools とカタログの `--sysroot` はこの `usr/{triple}` を優先します。
- 実行時 libc は `{triple}-glibc` です。
- `linux-headers` の配置規則については [linux-headers.md](../package/definition/linux-headers.md) を参照してください。
- 最終 gcc の sanitizer 設定および PGO の適用方針については [toolchain.md](../package/toolchain.md) を参照してください。
- qemu における pkg-config の設定と `--sysroot` の扱いについては [qemu.md](../package/definition/qemu.md) を参照してください。

## レイヤーとメタデータの生成

- 1 パッケージを 1 レイヤーとして扱います。`packages/artifacts/` の未圧縮成果物を tar 形式でまとめ、zstd で圧縮して blob を生成します。
- 未圧縮 tar のダイジェストである DiffID をレイヤー内容の恒久的な識別子として使用します。zstd の圧縮レベルは layer digest にのみ影響し、DiffID には影響しません。
- 通常の開発ビルドではレベル 3（速度優先）、公開用ビルド（`--profile release`）ではレベル 22（圧縮率優先）を使用します。`oci-layouts/` はパッケージ成果物からの派生物であるため、圧縮レベルの異なる layout を同一ディレクトリに併存させず、目的に応じて上書き生成します。
- Layout の生成時は、一時的な staging ディレクトリで構造を構築したのち、不可分（アトミック）に配置を置き換えます。これにより、生成途中で処理が失敗した場合でも既存の `oci-layouts/` が保護されます。
- レイヤーのハッシュ計算はストリーミング方式で行い、アーカイブ全体をメモリ上に展開しません。
- SPDX SBOM は Layout ディレクトリ内の `sbom.spdx.json` に配置します。OCI Image Specification ではレイアウト内の未知の追加ファイルを許容しているため、この配置は仕様に適合します。なお、referrers による attestation としての公開は現時点では未実装です。

## コンテナ内ステージングの規約

- 生成オーバーレイ（la-overlay / pc-overlay / 各種ラッパ）はホスト側で
  packages/build/{target}/.gen/{pkg}/ に生成し、コンテナの /tmp 配下へ
  read-only でマウントします。ビルドツリー（/build マウント）配下には
  置きません。openssl の make clean のように配下のシンボリックリンクを
  全削除する手順があるためです。
- dep 成果物の .la ファイルのうち、cc1/lto1 が実行時リンクする
  ライブラリ（gmp / mpfr / mpc / isl）は la-overlay に staging しません。
  installed=no の dep .la が dependency_libs に入ると libtool が
  install 時に relink し、relink 出力と同名の .so を lto1 が読み込み
  中に truncate されて衝突するためです。.so / .a のシンボリックリンクは
  残るため -l による解決は維持されます。
- dep glibc の libc.so リンカスクリプトは /lib/libc.so.6 等の絶対パスを
  参照するため、コンテナ内で /lib を glibc のライブラリディレクトリへの
  シンボリックリンクとして生成します（ターゲット glibc を優先し、
  Canadian cross ではホスト glibc にフォールバック）。
- ld はリンカスクリプトの実パスが sysroot 配下にある場合のみ、
  スクリプト内の絶対パスへ sysroot を適用します。このため shadow
  sysroot の lib ディレクトリは /dep への symlink ではなく実コピーです。

## マルチターゲット検証の状況

2026-09-21 時点で次のカタログビルドを完走しています。

- amd64 / v4（BUILD_ENV ベースライン）
- amd64 / v1（x86-64 baseline への回帰。Canadian gcc を含む）
- arm64 / v8（カタログ 40 件 + toolchain）
- riscv64 / rva20u64（同規模）

check --all による動的依存検証は全件グリーンです。ppc64le / s390x は
bootstrap 全体の検証として別途整理するため本範囲外です。生成バイナリを
実行できないクロスビルドでは PGO を適用しません。

## 現状の制約事項

- 対象とする architecture / variant の一覧は [platforms.md](../image/platforms.md) および `targets` コマンドを参照してください。シード環境ではクロス gcc が提供されていないため、`--all-targets` 指定時は ppc64le および s390x をスキップします。合成済み `build-env` が利用可能な状態ではスキップせずにビルドを実行します。
- `pack --with-build` は `Target::all()` 上でパッケージビルドを連鎖的に実行し、定義の存在しないターゲットはビルドをスキップして警告を出力します。`compose --with-build` は `-t` で指定したターゲットのみを対象とし、非対応の場合はスキップではなくエラーになります。
- `compose` の `-t` は任意ターゲットを受理しますが、実行時依存の `{triple}-gcc` 成果物は `artifact_target` により常に BUILD_ENV（amd64 / v4）へ配置される一方、`compose_layer_dirs` はこの正規化を適用しません。そのため C++ ランタイム依存を持つパッケージは BUILD_ENV 以外では構造的に合成できません（既知の制約）。
