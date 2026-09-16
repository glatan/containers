# パッケージ

パッケージ定義に共通するビルド方針です。段階ごとのコマンドは [build-flow.md](./build-flow.md)、個別設定は [definition/](./definition/)、Host tools の共通前処理は [host-tools.md](./host-tools.md)、プレフィックス付きツールチェーンは [toolchain.md](./toolchain.md) です。公開対象の分類は [image/packages.md](../image/packages.md) です。

`PACKAGES` に載るパッケージは [definition/](./definition/) に `{パッケージ名}.md` を 1 ファイル置きます。索引は [definition/README.md](./definition/README.md) です。実装は [`builder/src/package/definition/`](../../builder/src/package/definition/) です。ハイフンはモジュール名ではアンダースコア `_` です。プレフィックス付きツールチェーンの実装は `toolchain.rs` にまとめ、文書はパッケージ名のまま分けます。

パッケージイメージには各パッケージ固有のファイルのみを含めます。実行時に必要となる他のパッケージは `dependencies`、ビルド時のみ必要な依存パッケージは `build-dependencies` として分離して宣言します。`image validate` コマンドは、この実行時依存の宣言と、パッケージ成果物（`packages/artifacts/`）に含まれる ELF バイナリの動的リンク情報を静的に照合します。

## ビルド環境の構成

ビルド環境は特定のディストリビューションそのものではなく、HOST（amd64 / v4）向けパッケージを合成した単一の出力イメージです。その正本は `image compose build-env` によって合成された成果物（`oci-layouts/build-env-amd64`）です。現在のビルド環境の参照先は `containers-builder package env` コマンドで確認できます。合成済みイメージが存在しないブートストラップ初期段階においては、シード環境として [`builder/src/package/env.rs`](../../builder/src/package/env.rs) でダイジェストを固定した Arch Linux `base-devel` イメージを使用します。

各パッケージのビルドスクリプトには、環境変数として `SOURCE`、`BUILD`、`OUT`、`ARCH`、`VARIANT`、`TRIPLE`、`HOST_TRIPLE`、`PACKAGE_HOST`、`CROSS`、および `CFLAGS` / `LDFLAGS` が渡されます。`CFLAGS` および `LDFLAGS` は、パッケージ定義が返したフラグトークンから builder が合成します。`TRIPLE` はコード生成側の GNU triple（プレフィックス付き gcc ではパッケージ名の triple、Host tools ではパッケージ Platform の triple）です。`PACKAGE_HOST` は成果物が動く Platform の triple、`HOST_TRIPLE` はビルドマシン（常に `x86_64-linux-gnu`）です。

`CC`、`CXX`、`AR`、`RANLIB`、`NM` などのツールチェーン環境変数は builder が制御します。シード上で HOST 向けに組むあいだは `CC=gcc` です。自前 gcc のあと、および合成イメージ上では、対象 Platform の `CROSS` 接頭辞を付けます。amd64 では `x86_64-linux-gnu-gcc`、arm64 では `aarch64-linux-gnu-gcc` です。

フラグトークンの正本は [`builder/src/flags.rs`](../../builder/src/flags.rs) です。builder は `CFLAGS` に指定された `-flto` を `LDFLAGS` へ暗黙に複製せず、両者で独立してフラグを管理します。基本最適化レベルには `-O2` を指定し、CPU 最適化オプションは対象 variant から一意に決定します（ppc64le は `-mcpu=`、それ以外は `-march=` を使用し、ホスト環境に依存する `native` は指定しません）。`FormatSecurity` トークンは `-Wformat` と `-Wformat-security` の 2 つのコンパイラフラグへ展開されます。PGO のプロファイル採取および適用手順は各パッケージのビルドスクリプト内で定義します。`-O3`、`-g`、`-Wall`、`-fexceptions` はカタログ用コンパイラフラグとしては付与しません。

種別ごとの例外は、glibc と最終 gcc は [toolchain.md](./toolchain.md)、Host tools は [host-tools.md](./host-tools.md)、カタログと qemu は各 `{パッケージ名}.md` です。

依存パッケージは `/dep/{パッケージ名}` に読み取り専用でマウントされます（`dependencies` と `build-dependencies` の双方が対象です）。リンカ環境変数 `LD` には gcc を入れません。共有ライブラリのリンクは gcc ドライバと `-fuse-ld=mold` に任せます。libtool が `LD` を GNU ld と誤認しないようにするためです。

- architecture および variant 単位で CPU プラットフォームを指定します。対象プラットフォームの一覧は [platforms.md](../image/platforms.md) です。
- LTO および PGO の適用方針はパッケージの種別に応じて分かれます。カタログの共有ライブラリでは各ビルド設定で個別に宣言し、Host tools の共通前処理やプレフィックス付き最終 gcc・qemu の例外設定については各設計文書に記載しています。
- 自前で合成したビルド環境イメージ上では `--network=none` を指定してネットワークを遮断します。シード環境上でのビルド時に限り、pacman による依存解決や、カタログのクロス gcc および cmake 導入のためにネットワークアクセスを許可します。
- シード環境には ppc64le および s390x 向けのクロス gcc が存在しないため、`--all-target` 指定時はそれらのターゲットをスキップします。自前の合成イメージが存在する場合はスキップせずビルドします。

## ブートストラップと定義の管理

`Package` トレイト自体にはブートストラップ用のメソッドを設けず、ブートストラップ処理は [`Bootstrap`](../../builder/src/package/mod.rs) トレイトおよび `BOOTSTRAP` 登録一覧として分離して定義しています。`BOOTSTRAP` に登録されたパッケージは、合成済みビルド環境イメージが存在しない初期段階において、シード環境（Arch Linux）と pacman を用いてビルドします。それ以外のパッケージは、合成済みビルド環境が存在する場合はそれをベースとし、未合成の場合はシード環境へフォールバックしてビルドします。

ビルド手順は `recipe(&self, ctx)` メソッドが返す型付きの [`Recipe`](../../builder/src/package/recipe.rs)（`Step::Configure` / `Make` / `MakeInstall` / `Install` / `Symlink` 等）として記述します。定型の autotools パッケージは `Recipe::autotools(&[...])` で configure 引数のみを列挙します。`ctx`（`BuildContext`）は `target`・`$TRIPLE`・`$HOST_TRIPLE`・クロス判定を保持するため、クロス限定の configure 引数は Rust 側の条件分岐で静的に解決します。型では表現しきれない手順は `Step::Shell` または `build_script()`（`Recipe::Shell`）に逃がします。いずれの場合も configure、make、`DESTDIR` を用いたインストール手順のみを記述し、pacman コマンドなどのホスト環境を変更する処理は含めません。

上流ソースコードのアーカイブ検証には SHA3-512 および BLAKE3（出力 512 bit）の双方を使用します。生成される OCI Image Layout 内の blob ダイジェストおよび DiffID については、OCI 仕様に従い SHA-256 のまま算出します。
