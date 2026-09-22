# 対象プラットフォーム

本プロジェクトのパッケージイメージが対象とする CPU プラットフォーム（architecture および variant）の一覧です。この表が `Target::all()` です。カタログと Host tools（qemu 以外）の `supports` はこの集合です。

qemu、中間 gcc ステージ、`build-env` は HOST（amd64 / v4）のみです。`{triple}-glibc` はその architecture の variant だけです。最終 gcc / binutils の `supports` と製品キューは [toolchain.md](./package/toolchain.md) です。実装順は HOST → 各 architecture の基底 variant → 残り variant です。

本ドキュメントの表は、[`builder/src/target.rs`](../builder/src/target.rs) の `Target::all()` と常に一致させます。内容の不整合はテスト `documented_platforms_match_targets` によって自動検出されます。

表中の `status` は、[Image Spec v1.1.1 の Platform Variants](https://github.com/opencontainers/image-spec/blob/v1.1.1/image-index.md#platform-variants) に対する分類を示します。

- `listed`: 仕様書の表に明記されている（または `…` で省略表記された系列に含まれる）組み合わせです。
- `defined`: 仕様書の表には掲載されていませんが、仕様上 implementation-defined（実装定義）として許容されており、OCI 準拠として不正ではない組み合わせです。

`--all-targets` は各パッケージについて `supports` が真の Platform を組みます。製品キューより広い `supports` を持つパッケージでは、運用上の製品範囲は [toolchain.md](./package/toolchain.md) に従います。

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

## プラットフォームの固定

一部のコンテナランタイムは、一般的に普及している組み合わせ以外のプラットフォームを自動選択できない場合があります。そのため、利用者側でイメージを指定する際は Image Index のダイジェスト（image digest）でプラットフォームを固定します（背景は [oci.md](./oci.md)）。
