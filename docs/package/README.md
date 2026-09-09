# パッケージ

パッケージ定義に共通するビルド方針です。パッケージ毎の設定は [image/packages/](../image/packages/README.md) にあります。

パッケージイメージにはそのパッケージだけを含めます。実行時に必要な他パッケージは `dependencies`、ビルド時だけ必要なものは `build-dependencies` です。`image validate` は実行時依存とパッケージ(`packages/artifacts/`)の ELF を照合します。

## ビルド環境

ビルド環境イメージは `podman run` で使います。参照の正本は [`builder/src/package/env.rs`](../../builder/src/package/env.rs) です。タグ(`base-devel`)は人間向けで、実行は digest ピンです。

更新は `env.rs` の `digest` だけを差し替えます。確認は `containers-builder package env` です。

渡す環境変数は `SOURCE`、`BUILD`、`OUT`、`ARCH`、`VARIANT`、`TRIPLE`、`HOST_TRIPLE`、`CROSS` と、パッケージが `flags()` で要求したビルドフラグです。`CROSS` は常に `{TRIPLE}-` です。クロスかどうかは `TRIPLE` と `HOST_TRIPLE` の比較で判定します。フラグの値は [`builder/src/flags.rs`](../../builder/src/flags.rs) です。いま `CFLAGS` を要求するのは glibc と openssl です。linux-headers は要求しません。
依存パッケージは `/dep/{名前}` に読み取り専用でマウントします(`dependencies` と `build-dependencies` の両方)。

- architecture と variant まで CPU を指定します。カタログとビルド可能集合は [platforms.md](../image/platforms.md) です。
- LTO と PGO は実施するのが基本です。しない場合は [各パッケージのビルド設定](../image/packages/README.md) に理由を書きます。
- 通常は `--network=none` です。設定でネットワークを許可したパッケージだけ例外です。
- いまビルドできるのは amd64 v1〜v4、arm64 の全 variant、riscv64 の全 variant です。ppc64le と s390x は Arch にクロス gcc がありません。

実装は [`builder/src/package/definition/`](../../builder/src/package/definition/) が正本です。文書を足すときは `{name}.md` と `{name}.rs`(ハイフンは `_`)を対で追加します。

ソース tarball の検証は SHA3-512 と BLAKE3(出力 512 bit)です。OCI Image Layout の blob / DiffID は仕様どおり sha256 のままです。
