# プラットフォーム

パッケージイメージが対象にする CPU の一覧です。
カタログとしては、ビルドできない場合を除きすべて必要です。
OCI 上の意味は [oci](../oci/README.md) を参照してください。

表は `builder` の `TARGETS` と一致させます。ずれはテスト `documented_platforms_match_targets` で検出します。

`status` は [Image Spec v1.1.1 の Platform Variants](https://github.com/opencontainers/image-spec/blob/v1.1.1/image-index.md#platform-variants) に対する位置づけです。

- `listed` — 表に載る(または `…` で示される系列の)組み合わせ
- `defined` — 表に無いが、仕様上は implementation-defined であり不正ではない

いま `package build` できるのは **amd64 v1〜v4 と arm64 v8** だけです。他は `--all-target` 時にスキップします。

| architecture | variant | status |
|-|-|-|
| arm64 | v8 | listed |
| arm64 | v8.1 | listed |
| arm64 | v8.2 | listed |
| arm64 | v8.3 | listed |
| arm64 | v8.4 | listed |
| arm64 | v8.5 | listed |
| arm64 | v8.6 | listed |
| arm64 | v8.7 | listed |
| arm64 | v8.8 | listed |
| arm64 | v8.9 | listed |
| arm64 | v9 | listed |
| arm64 | v9.1 | listed |
| arm64 | v9.2 | listed |
| arm64 | v9.3 | listed |
| arm64 | v9.4 | listed |
| arm64 | v9.5 | listed |
| ppc64le | power8 | listed |
| ppc64le | power9 | listed |
| ppc64le | power10 | listed |
| ppc64le | power11 | listed |
| riscv64 | rva20u64 | listed |
| riscv64 | rva22u64 | listed |
| riscv64 | rva23u64 | listed |
| amd64 | v1 | listed |
| amd64 | v2 | listed |
| amd64 | v3 | listed |
| amd64 | v4 | listed |
| s390x | z900 | defined |
| s390x | z990 | defined |
| s390x | z9-ec | defined |
| s390x | z10 | defined |
| s390x | z196 | defined |
| s390x | zEC12 | defined |
| s390x | z13 | defined |
| s390x | z14 | defined |
| s390x | z15 | defined |
| s390x | z16 | defined |
| s390x | z17 | defined |

一部ランタイムはよく使う組み合わせ以外を自動選択しないことがあります。
利用側は image digest で pin します。
