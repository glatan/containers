# build-env

| 項目 | 値 |
|-|-|
| パッケージ名 | `build-env` |
| バージョン | 1 |
| ライセンス | CC0-1.0（合成後の出力イメージは構成パッケージの全ライセンスを SBOM に列挙） |
| パッケージイメージ | 合成出力イメージの基準環境（個別ソースコードはなし） |
| 定義 | [`builder/src/package/definition/build_env.rs`](../../../builder/src/package/definition/build_env.rs) |

`env compose` コマンドが、実行時依存パッケージをトポロジカル順に並べてレイヤー化し、ベースとなるビルド環境イメージを合成します。本パッケージ固有の単体ソースコードは存在しません。全体の段階は [build-flow.md](../build-flow.md) が正本です。

最初の合成に入れるのは x86_64 の gcc / binutils / glibc、PATH 用 POSIX、および mold です。他 triple の `{triple}-glibc` と qemu / python / cmake は載せません。

生成されるイメージのタグ形式は `{name}-{arch}:{yyyymmdd-revision}`（例: `build-env-amd64:20260910-0`）です。対象プラットフォームは amd64 / v4 であり、生成された Image Layout はローカルの `oci-layouts/build-env-amd64` に配置されます。

個別のパッケージイメージとは異なり、この合成出力イメージに限り Image Config 内に `PATH` 環境変数と `/bin/bash` の実行設定を保持します。
