# パッケージイメージ

本プロジェクトは、distroless コンテナと同様に必要なファイルのみを含め、基盤となる特定のディストリビューションに依存しません。
パッケージに属するファイル集合を、エントリポイントを持たない最小構成の OCI Image（パッケージイメージ）として配信します。
生成する Image Layout が準拠する OCI 仕様については [oci](../oci/README.md) を参照してください。

## 配信モデル

パッケージ 1 つにつき、1 つのパッケージイメージを配信します。1 つのパッケージは 1 つのレイヤーに対応します。

- 各パッケージイメージには、そのパッケージ自身のファイルのみを含めます。
- 実行時に必要となる他のパッケージは `dependencies`、ビルド時のみ必要な依存パッケージは `build-dependencies` として分離して宣言します。
- パッケージイメージ自体に「ビルド用」「実行用」といった種別は設けません（ツール名である builder とパッケージイメージの役割は独立しています）。

## 出力イメージの合成

レジストリで公開するのは個別のパッケージイメージのみであり、すべての依存関係やバージョンの組み合わせを事前に結合したイメージを網羅的に配信することはしません。

実際の実行環境となる出力イメージは、`containers-builder compose` コマンドにより必要に応じて合成します。指定したパッケージを起点として `dependencies` を再帰的に探索し、依存するすべてのパッケージイメージを束ねた出力イメージを生成します。このとき `build-dependencies` は出力イメージに含めません。複数のパッケージ間で同一パスのファイルが存在する場合は、ファイル競合エラーとして処理を中断します。合成時のプラットフォームは `-t` で指定します（現状、実行時依存の `{triple}-gcc` 成果物が BUILD_ENV にしか配置されない構造上の制約から、機能するのは amd64 / v4 です）。タグ形式は `{name}-{arch}:{yyyymmdd-revision}` です。なお、イメージの利用側ツールも本バイナリと同一の合成ロジックを使用します。

個別のパッケージイメージの Image Config には実行時設定を含めず空としていますが、合成された出力イメージに限り `PATH` 環境変数および `/bin/bash` を設定します。

## 関連ドキュメント

- [naming.md](./naming.md) — レジストリパスとタグポリシー
- [platforms.md](./platforms.md) — architecture / variant の一覧
- [packages.md](./packages.md) — 公開するパッケージイメージの一覧
- [../package/build-flow.md](../package/build-flow.md) — ビルドの段階とコマンド
- [../package/definition/](../package/definition/) — パッケージごとのビルド設定
- [builder](../builder/README.md) — ビルドツールの仕様（docker や podman build は使用せず、builder がレイヤー tarball とメタデータを直接生成します）

## ライセンスの扱い

各パッケージイメージのライセンスは、対象ソフトウェア本来のライセンスをそのまま継承します（`Package::license()` の定義が SBOM に反映されます）。たとえば gcc は `GPL-3.0-or-later WITH GCC-exception-3.1`、glibc は `LGPL-2.1-or-later` です。

合成後の出力イメージには複数のライセンスが併存することになります。これらのライセンスはすべて SBOM に列挙し、合成結果全体に対して特定の代表ライセンスを独自に割り当てることはしません。

各パッケージの正確なライセンス情報は SPDX SBOM で管理します。現行の実装では Image Layout 内の `sbom.spdx.json` に配置しており、イメージダイジェストへの attestation（referrers）としての付与は未実装です。

## ダイジェストによるバージョンの固定

利用者が固定（pin）すべき対象値は、参照するレイヤーや目的によって異なります。

- DiffID: 未圧縮 tar アーカイブのハッシュ値です。圧縮レベルを変更しても同一の値が保たれます。
- layer digest: zstd の圧縮レベルによって変化します。
- image digest（Image Index または Image Manifest）: 作成時刻（`created`）を含むため、ビルドを実行するたびに変化します。

## 整合性の検証

`check` コマンドは、生成後の Image Layout ではなく、未圧縮のパッケージ成果物（`packages/artifacts/`）に含まれる ELF バイナリと、宣言された `dependencies` の整合性を静的に検証します。libc の SONAME（`libc.so.6` など）については、`{triple}-glibc` の依存宣言によって充足されているかを確認します。それ以外の SONAME は提供パッケージへマッピングして照合します。`libstdc++.so` / `libgcc_s.so` / `libatomic.so` / `libgomp.so` / `libgfortran.so` / `libquadmath.so` は `{triple}-gcc`、`libcrypt.so` / `libowcrypt.so` / `libxcrypt.so` は `libxcrypt`、`liblzma.so` は `xz`、`libbz2.so` は `bzip2`、`libexpat.so` は `expat`、`libsqlite3.so` は `sqlite` に対応します。`libcrypt.so` は glibc に含まれず `libxcrypt` が提供します。マッピングの無い SONAME は検証エラーになります。

## 今後の検討課題

- musl libc を採用したパッケージイメージの提供
