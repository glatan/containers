# OCI 準拠

builder が生成するパッケージイメージは、次の仕様に準拠します。

| 仕様 | 版 | 適用範囲 |
|-|-|-|
| [OCI Image Specification](https://github.com/opencontainers/image-spec/tree/v1.1.1) | **v1.1.1** | Image Layout、Descriptor、Image Index、Image Manifest、Image Config |
| [SPDX](https://spdx.github.io/spdx-spec/v2.3/) | 2.3 | レイアウト隣の `sbom.spdx.json`(Image Spec の対象外。追加ファイルとして許容される) |

次は現時点の対象外です。

- OCI Distribution Spec(レジストリへの push。公開運用フェーズで扱う)
- OCI Runtime Spec(実行用 bundle。パッケージイメージは実行エントリを持たない)

仕様の SHOULD は可能な範囲で満たします。MUST / MUST NOT はテストで固定します。
テスト名は下表の ID と対応します(`builder/tests/oci_spec.rs`)。

## 要件とテスト

出典はいずれも Image Spec v1.1.1 です。

### Image Layout

| ID | 要件 | テスト |
|-|-|-|
| `layout-files` | `oci-layout`、`index.json`、`blobs/` が存在する | `layout_has_required_entries` |
| `layout-version` | `oci-layout` は JSON オブジェクトで `imageLayoutVersion` は `1.0.0` | `oci_layout_version_is_1_0_0` |
| `layout-index` | `index.json` は Image Index である | `index_is_image_index` |
| `layout-blob-digest` | `blobs/<alg>/<encoded>` の内容は digest `<alg>:<encoded>` と一致する | `blob_bytes_match_digest` |
| `layout-blob-name` | blob 名の文字集合は Descriptor の digest 文法に従う(sha256 は 64 桁の小文字 hex) | `blob_names_match_digest_grammar` |
| `layout-unknown-files` | 追加ファイルがあってもレイアウトとして不正ではない | 運用上 `sbom.spdx.json` を隣に置く。生成は Image Spec 外 |

### Descriptor

| ID | 要件 | テスト |
|-|-|-|
| `desc-fields` | descriptor は `mediaType`、`digest`、`size` を持つ | `descriptors_have_required_fields` |
| `desc-digest` | `digest` は `sha256:` + 64 桁の小文字 hex | `descriptors_use_sha256_digest` |
| `desc-size` | `size` は参照 blob のバイト数と一致する | `descriptor_size_matches_blob` |

### Image Index

| ID | 要件 | テスト |
|-|-|-|
| `index-schema` | `schemaVersion` は `2` | `index_is_image_index` |
| `index-media-type` | `mediaType` を書く場合は `application/vnd.oci.image.index.v1+json` | `index_is_image_index` |
| `index-manifests` | `manifests` 配列の各要素は descriptor である | `descriptors_have_required_fields` |
| `index-platform` | platform を書く場合 `architecture` と `os` は必須。`variant` は任意の文字列 | `index_platform_required_fields` |
| `index-variant` | Platform Variants 表に無い組み合わせも implementation-defined であり、index として不正ではない | `implementation_defined_variant_is_accepted` |

### Image Manifest

| ID | 要件 | テスト |
|-|-|-|
| `manifest-schema` | `schemaVersion` は `2` | `manifest_required_fields` |
| `manifest-media-type` | `mediaType` を書く場合は `application/vnd.oci.image.manifest.v1+json` | `manifest_required_fields` |
| `manifest-config` | `config` は descriptor。`mediaType` は `application/vnd.oci.image.config.v1+json` | `manifest_required_fields` |
| `manifest-layers` | `layers` は descriptor の配列。先頭が base。media type は `application/vnd.oci.image.layer.v1.tar+zstd` | `manifest_required_fields` |
| `manifest-annotations` | アノテーションを付ける場合は key/value の文字列マップ | `manifest_annotations` |

### Image Config

| ID | 要件 | テスト |
|-|-|-|
| `config-arch-os` | `architecture` と `os` は必須 | `config_required_fields` |
| `config-variant` | `variant` は任意。Platform Variants 表の値を使う(SHOULD) | `config_required_fields` |
| `config-rootfs` | `rootfs.type` は `layers`。`rootfs.diff_ids` は必須 | `config_required_fields` |
| `config-diff-id` | DiffID は未圧縮 tar の digest | `diff_id_is_uncompressed_tar` |
| `config-created` | `created` を書く場合は RFC 3339 | `config_created_is_rfc3339` |
| `config-matches-index` | config の architecture / os / variant は index の platform と一致する | `config_platform_matches_index` |

### 本プロジェクトの方針

| ID | 方針 | テスト |
|-|-|-|
| `policy-targets` | 対象プラットフォーム一覧はドキュメントと `Target::all()` が一致する | `documented_platforms_match_targets` |
| `policy-rebuild` | 再ビルドで未参照 blob を残さない(仕様の MAY より厳しい) | `rebuild_replaces_stale_blobs` |
| `policy-escape` | JSON の特殊文字をエスケープする | `json_special_characters_are_escaped` |
| `policy-reproducible` | 同一入力・同一圧縮レベルなら layer digest と DiffID が一致する | `same_input_same_layer_digest` |
| `policy-preserve` | 生成失敗時に既存 layout を残す | `missing_layer_dir_is_rejected_before_replace`, `failed_layer_after_staging_preserves_existing_layout` |
| `policy-zstd-publish` | 通常 3 と公開 22 で DiffID は同じ、layer digest は異なる | `publish_level_changes_blob_but_not_diff_id` |
| `policy-zstd-level` | zstd レベルは 1〜22。通常 3、公開 22。`--publish` と `--zstd-level` は同時指定しない | `zstd_level_must_be_1_to_22` |

## レイヤー圧縮

Image Spec が定めるのはエンコーディングを示す media type だけです。圧縮率・圧縮レベル・辞書の有無は定義されていません。

実装(ランタイム)は `application/vnd.oci.image.layer.v1.tar` と `...tar+gzip` を MUST、`...tar+zstd` を SHOULD で扱います。
本プロジェクトはレイヤーを `application/vnd.oci.image.layer.v1.tar+zstd` で出します。DiffID は常に未圧縮 tar の digest です。

zstd のレベルは仕様外の実装選択です。再現性のため `--single-thread` を使います。

- 通常の `image build` はレベル **3**(速さ優先)
- `image build --publish` はレベル **22**(公開用。`--ultra`)
- `--zstd-level` で直接指定できるが、`--publish` と同時には使わない

## 圧縮の違いと正本

圧縮レベルが違うと **layer blob の digest は変わる** が、**DiffID(未圧縮 tar)は同じ**です。
レジストリ上で L3 と L22 の blob を共有することはできません。

正本はパッケージ(`packages/artifacts/`)です。`oci-layouts/` はそこから都度生成する派生物で、L3 と L22 を並べて置かず、公開時に `--publish` で上書きして push します。
ローカル確認は L3、GHCR には L22 だけを載せます。

## プラットフォームと variant

`platform.variant` は OPTIONAL な文字列です。
[Platform Variants](https://github.com/opencontainers/image-spec/blob/v1.1.1/image-index.md#platform-variants) に無い値は implementation-defined であり、index / config としては不正ではありません。

本プロジェクトの一覧と、表に載る組み合わせ・載らない組み合わせの区別は [platforms.md](../image/platforms.md) を参照してください。
ランタイムが未知の variant を自動選択しない場合があるため、利用側は image digest で pin します。
