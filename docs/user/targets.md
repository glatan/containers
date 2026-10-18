# 対象プラットフォーム

`containers.json` の `platform` に書ける architecture と variant の一覧です。`images[].platform` はこの表の `architecture/variant` です（[requirements.md](./requirements.md)）。

この表は、「表にある platform は、すべてのパッケージで公開されている」を意味しません。公開範囲はパッケージごとに異なり、[published.md](./published.md) にあります。

| architecture | variant |
|-|-|
| arm64 | v8 |
| arm64 | v8.1 |
| arm64 | v8.2 |
| arm64 | v8.3 |
| arm64 | v8.4 |
| arm64 | v8.5 |
| arm64 | v8.6 |
| arm64 | v8.7 |
| arm64 | v8.8 |
| arm64 | v8.9 |
| arm64 | v9 |
| arm64 | v9.1 |
| arm64 | v9.2 |
| arm64 | v9.3 |
| arm64 | v9.4 |
| arm64 | v9.5 |
| ppc64le | power8 |
| ppc64le | power9 |
| ppc64le | power10 |
| ppc64le | power11 |
| riscv64 | rva20u64 |
| riscv64 | rva22u64 |
| riscv64 | rva23u64 |
| amd64 | v1 |
| amd64 | v2 |
| amd64 | v3 |
| amd64 | v4 |
| s390x | z900 |
| s390x | z990 |
| s390x | z9-ec |
| s390x | z10 |
| s390x | z196 |
| s390x | zEC12 |
| s390x | z13 |
| s390x | z14 |
| s390x | z15 |
| s390x | z16 |
| s390x | z17 |

## プラットフォームの固定

一部のコンテナランタイムは、一般的に普及している組み合わせ以外のプラットフォームを自動選択できない場合があります。Image Index のダイジェストで参照しても platform の選択はランタイム側の platform マッチング処理に依存するため、利用者側でイメージを指定する際は platform 別 manifest のダイジェストで固定します（`sync` の生成 pin はこの形式です。背景は [oci.md](../dev/oci.md)）。
