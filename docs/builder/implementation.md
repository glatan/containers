# 実装構成

パッケージおよびパッケージイメージのビルドツール（builder）の内部設計と実装構成を定めます。
コマンドの外部仕様については [README.md](./README.md) を参照してください。

## 概要

- 実装言語には Rust 1.98.1（edition 2024）を採用します。使用するツールチェーンはリポジトリルートの `rust-toolchain.toml` で固定します。
- 単一のバイナリ（`containers-builder`）で、`package` および `image` の 2 系統のサブコマンドを提供します。ソースコードは `builder/` ディレクトリで管理します。
- コマンド実行はリポジトリのルートディレクトリで行い、作業ディレクトリ（`packages/` および `oci-layouts/`）を相対パスで参照します。

## モジュール構成

| モジュール | 役割 |
|-|-|
| `main.rs` | コマンドライン引数の解析、プロセスの起動、および終了ステータスの管理 |
| `command/` | サブコマンド（`package` / `image`）の実行ロジック |
| `error.rs` / `process.rs` | 型付きエラー定義と外部コマンドプロセスの実行制御 |
| `package/mod.rs` | `Package` および `Bootstrap` トレイトと共通データ構造の定義 |
| `package/definition/` | 個別パッケージの定義実装、`PACKAGES`、および `BOOTSTRAP` の登録一覧 |
| `package/{fetch,build}.rs` | ソースコードの取得・検証とパッケージのコンパイル制御（シード環境および自前環境） |
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

## 外部コマンドの利用

| コマンド | 用途 |
|-|-|
| `curl` | 上流ソースコードの取得（`package fetch`） |
| `tar` | ソースコードの展開、およびレイヤー tarball の生成（`--sort=name` などを指定してビルドの再現性を確保） |
| `zstd` | レイヤー tarball の圧縮（`--single-thread`。通常ビルドはレベル 3、公開用ビルドはレベル 22 であり、20〜22 は `--ultra` を付与） |
| `podman` | 隔離されたビルド環境コンテナ上でのパッケージコンパイル |
| `/bin/bash` | パッケージごとのビルドスクリプトの実行 |
| `readelf` | ELF バイナリの動的リンク依存（`DT_NEEDED`）の解析（`image validate`） |

## データフロー

```
package fetch      ソース取得 + SHA3-512 / BLAKE3 検証 + 展開
    ↓                 packages/sources/{package}/
package bootstrap  BOOTSTRAP 閉包を HOST 上で一度歩き、失敗しても後続を続ける
    ↓                 packages/artifacts/ および packages/logs/bootstrap.log
package build      依存関係をトポロジカル順に取得・ビルドしたのち、podman 上でビルドスクリプトを実行
    ↓                 packages/artifacts/{package}-{version}-{arch}-{variant}/
                      packages/logs/{package}-{version}-{arch}-{variant}.log
image build        パッケージ成果物をレイヤー化して Image Layout を生成 + SPDX SBOM 出力
    ↓                 oci-layouts/{package}-{version}/
image compose     dependencies の推移閉包を単一の出力イメージへ合成
    ↓                 oci-layouts/{name}-{arch}-{date}-{revision}/
image validate     宣言された依存関係とパッケージ内 ELF の NEEDED を静的照合
package clean      成果物・ビルド作業ツリー・個別ログを削除（再ビルド前の掃除）
package tree       依存ツリーを表示
package bootstrap-path  シード相で組む最小集合（build-env 依存閉包）を出力
```

ビルドスクリプトへ渡す環境変数と `/dep` へのマウントは [package/README.md](../package/README.md) です。
`package build` は依存パッケージを先にビルドします。ルートが `BOOTSTRAP` に載っている場合、合成済みビルド環境の有無にかかわらずグラフには `Bootstrap` の依存を使います。`own`（合成済みイメージの使用、`pacman` 前置きの有無、ネットワーク遮断）だけを `build_env_ready()` で決めます。成果物ディレクトリが既に存在し空でない場合はそのターゲットのコンパイルをスキップし、ビルド途中で失敗した成果物は不整合を防ぐため自動的に削除します。`podman` の出力は `packages/logs/` の個別ファイルへ tee し、失敗してもログは残します。
LTO や PGO の適用方針はパッケージの種別ごとに異なります。個別の指定内容は [各パッケージのビルド設定](../package/definition/) を参照してください。

ビルド専用のパッケージ定義（`build_only`）は、中間成果物としてのみ扱い、パッケージイメージ化は行いません。これらは他のパッケージの `build-dependencies` にのみ現れます。
`BOOTSTRAP` 一覧に登録されたパッケージは、シード環境（Arch Linux イメージ）上で直接ビルドできます。`Package` トレイトには `bootstrap()` メソッドを設けず、`Bootstrap` トレイトによりブートストラップ処理を分離して実装します。
自前で合成したビルド環境イメージ上では `--network=none` を指定して完全に隔離された状態でビルドします。一方、シード環境では `Bootstrap::pacman()` によるパッケージ導入や、カタログのクロス gcc 導入のためにネットワークアクセスを許可します。

新規パッケージを定義する場合は、`builder/src/package/definition/{name}.rs` に `Package` トレイトを実装し、`builder/src/package/definition/mod.rs` の一覧へ登録します。`BOOTSTRAP` に載せるのは x86_64 ツールチェーンと POSIX PATH、cmake、mold です。他の Host tools は配列に無く、依存グラフ経由で入ります。プレフィックス付きツールチェーンは配列の外でも Bootstrap グラフを使います。`flags(target)` メソッドの実装は必須であり、フラグを要求しない場合は空の `Flag` を返します。
文書としては、[`docs/package/definition/{name}.md`](../package/definition/) にビルド設定を記述します。1 パッケージ 1 ファイルです。索引は [definition/README.md](../package/definition/README.md) です。Host tools の共通前処理については [host-tools.md](../package/host-tools.md)、プレフィックス付きツールチェーンのステージ順については [toolchain.md](../package/toolchain.md) を参照してください。モジュール名の `linux_headers.rs` はパッケージ名 `linux-headers` に対応します。プレフィックス付きパッケージの実装は `toolchain.rs` にまとめ、文書ファイル名はパッケージ名です。

## クロスビルドの構成

ビルド環境イメージは amd64 / v4 上で動作します。非 amd64 プラットフォーム向けパッケージは、イメージ内に配置した `{triple}-gcc` を用いてクロスコンパイルし、PGO のプロファイル実行には QEMU を利用します。なお、自前イメージ合成前のシード期間に限り、Arch Linux 上で pacman を用いてクロス gcc を導入します。

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
- 通常の開発ビルドではレベル 3（速度優先）、公開用ビルド（`--publish`）ではレベル 22（圧縮率優先）を使用します。`oci-layouts/` はパッケージ成果物からの派生物であるため、圧縮レベルの異なる layout を同一ディレクトリに併存させず、目的に応じて上書き生成します。
- Layout の生成時は、一時的な staging ディレクトリで構造を構築したのち、不可分（アトミック）に配置を置き換えます。これにより、生成途中で処理が失敗した場合でも既存の `oci-layouts/` が保護されます。
- レイヤーのハッシュ計算はストリーミング方式で行い、アーカイブ全体をメモリ上に展開しません。
- SPDX SBOM は Layout ディレクトリ内の `sbom.spdx.json` に配置します。OCI Image Specification ではレイアウト内の未知の追加ファイルを許容しているため、この配置は仕様に適合します。なお、referrers による attestation としての公開は現時点では未実装です。

## 現状の制約事項

- 対象とする architecture / variant の一覧は [platforms.md](../image/platforms.md) を参照してください。シード環境ではクロス gcc が提供されていないため、`--all-target` 指定時は ppc64le および s390x をスキップします。合成済み `build-env` が利用可能な状態ではスキップせずにビルドを実行します。
- `--with-packages` オプションはパッケージビルドを連鎖的に実行する機能ですが、定義の存在しないターゲットについてはビルドをスキップして警告を出力します。
