<!-- GENERATED -->
# build-env

| 項目 | 値 |
|-|-|
| パッケージ名 | `build-env` |
| バージョン | 1 |
| ライセンス | CC0-1.0 |
| 分類 | build-env（合成ルート） |
| 定義 | [`builder/src/package/definition/build_env.rs`](../../../builder/src/package/definition/build_env.rs) |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | — |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | — |
| [mold](./mold.md) | 2.42.1 | — |
| [bash](./bash.md) | 5.3 | — |
| [coreutils](./coreutils.md) | 9.12 | — |
| [make](./make.md) | 4.4.1 | — |
| [sed](./sed.md) | 4.10 | — |
| [grep](./grep.md) | 3.12 | — |
| [gawk](./gawk.md) | 5.4.1 | — |
| [findutils](./findutils.md) | 4.11.0 | — |
| [tar](./tar.md) | 1.35 | — |
| [gzip](./gzip.md) | 1.14 | — |
| [xz](./xz.md) | 5.8.4 | — |
| [diffutils](./diffutils.md) | 3.12 | — |
| [patch](./patch.md) | 2.8 | — |
| [m4](./m4.md) | 1.4.21 | — |

### build-dependencies

なし。

### host-dependencies

なし。

<!-- /GENERATED -->



`env compose` コマンドが、実行時依存パッケージをトポロジカル順に並べてレイヤー化し、ベースとなるビルド環境イメージを合成します。本パッケージ固有の単体ソースコードは存在しません。合成後の出力イメージは構成パッケージの全ライセンスを SBOM に列挙します。全体の段階は [build-flow.md](../build-flow.md) が正本です。

最初の合成に入れるのは x86_64 の gcc / binutils / glibc、PATH 用 POSIX、および mold です。他 triple の `{triple}-glibc` と qemu / python / cmake は載せません。

生成されるイメージのタグ形式は `{name}-{arch}:{yyyymmdd-revision}`（例: `build-env-amd64:20260910-0`）です。対象プラットフォームは amd64 / v4 であり、生成された Image Layout はローカルの `oci-layouts/build-env-amd64` に配置されます。

個別のパッケージイメージとは異なり、この合成出力イメージに限り Image Config 内に `PATH` 環境変数と `/bin/bash` の実行設定を保持します。
