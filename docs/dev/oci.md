# OCI 仕様準拠

builder が生成するパッケージイメージおよび Image Layout は、以下の仕様に準拠します。

| 仕様 | バージョン | 適用範囲 |
|-|-|-|
| [OCI Image Specification](https://github.com/opencontainers/image-spec/tree/v1.1.1) | v1.1.1 | Image Layout、Descriptor、Image Index、Image Manifest、Image Config |
| [SPDX](https://spdx.github.io/spdx-spec/v2.3/) | 2.3 | Image Layout 内の `sbom.spdx.json`（OCI Image Specification の対象外だが、追加ファイルとして許容される） |

以下の仕様は現時点では対象外とします。

- OCI Distribution Specification（レジストリへのプッシュ処理。公開運用の自動化フェーズで対応します）
- OCI Runtime Specification（実行用コンテナバンドル。パッケージイメージはエントリポイントを持たないため対象外です）

仕様の SHOULD 要件は可能な範囲で満たし、MUST および MUST NOT 要件はテスト群によって遵守を保証します。下表の各 ID は、テストコード（`builder/tests/oci_spec.rs`）のテストケースと対応しています。

## 要件とテスト対応


### Image Layout

| ID | 要件 | 対応テスト |
|-|-|-|
| `layout-files` | `oci-layout`、`index.json`、および `blobs/` ディレクトリが存在すること | `layout_has_required_entries` |
| `layout-version` | `oci-layout` が JSON オブジェクトであり、`imageLayoutVersion` が `1.0.0` であること | `oci_layout_version_is_1_0_0` |
| `layout-index` | `index.json` が有効な Image Index であること | `index_is_image_index` |
| `layout-blob-digest` | `blobs/<alg>/<encoded>` の内容がダイジェスト `<alg>:<encoded>` と一致すること | `blob_bytes_match_digest` |
| `layout-blob-name` | blob ファイル名の文字集合が Descriptor のダイジェスト文法に従っていること（SHA-256 は 64 桁の小文字 16 進数） | `blob_names_match_digest_grammar` |
| `layout-unknown-files` | レイアウト内に未知の追加ファイルが存在してもエラーとしてはならないこと | 運用上 `sbom.spdx.json` を配置する（仕様の対象外として許容） |

なお `index.json` は、containers/image の `oci:` トランスポートとの互換性のため、platform 別 manifest を直接列挙せず、`org.opencontainers.image.ref.name` 注釈付きの単一 descriptor を介してネストした Image Index blob を指します（buildx 形式）。複数エントリの `index.json` は `skopeo copy oci:` で `ErrMoreThanOneImage` により拒否されるためです。

### Descriptor

| ID | 要件 | 対応テスト |
|-|-|-|
| `desc-fields` | Descriptor に `mediaType`、`digest`、`size` の必須フィールドが含まれること | `descriptors_have_required_fields` |
| `desc-digest` | `digest` が `sha256:` と 64 桁の小文字 16 進数で構成されること | `descriptors_use_sha256_digest` |
| `desc-size` | `size` が参照先 blob のバイト長と正確に一致すること | `descriptor_size_matches_blob` |

### Image Index

| ID | 要件 | 対応テスト |
|-|-|-|
| `index-schema` | `schemaVersion` が `2` であること | `index_is_image_index` |
| `index-media-type` | `mediaType` を記述する場合、`application/vnd.oci.image.index.v1+json` であること | `index_is_image_index` |
| `index-manifests` | `manifests` 配列の各要素が Descriptor であること | `descriptors_have_required_fields` |
| `index-platform` | `platform` を記述する場合、`architecture` と `os` が必須であること（`variant` は任意の文字列） | `index_platform_required_fields` |
| `index-variant` | Platform Variants 表にない組み合わせも実装定義（implementation-defined）として許容され、Image Index として有効であること | `implementation_defined_variant_is_accepted` |

### Image Manifest

| ID | 要件 | 対応テスト |
|-|-|-|
| `manifest-schema` | `schemaVersion` が `2` であること | `manifest_required_fields` |
| `manifest-media-type` | `mediaType` を記述する場合、`application/vnd.oci.image.manifest.v1+json` であること | `manifest_required_fields` |
| `manifest-config` | `config` が Descriptor であり、`mediaType` が `application/vnd.oci.image.config.v1+json` であること | `manifest_required_fields` |
| `manifest-layers` | `layers` が Descriptor の配列であり、先頭がベース層、media type が `application/vnd.oci.image.layer.v1.tar+zstd` であること | `manifest_required_fields` |
| `manifest-annotations` | アノテーションを付与する場合、キーと値が文字列のマップであること | `manifest_annotations` |

### Image Config

| ID | 要件 | 対応テスト |
|-|-|-|
| `config-arch-os` | `architecture` と `os` が必須フィールドであること | `config_required_fields` |
| `config-variant` | `variant` は任意であり、仕様の Platform Variants 表の値を使用することが推奨されること（SHOULD） | `config_required_fields` |
| `config-rootfs` | `rootfs.type` が `layers` であり、`rootfs.diff_ids` が必須であること | `config_required_fields` |
| `config-diff-id` | DiffID が未圧縮 tar アーカイブのダイジェストであること | `diff_id_is_uncompressed_tar` |
| `config-created` | `created` を記述する場合、RFC 3339 形式に準拠していること | `config_created_is_rfc3339` |
| `config-matches-index` | Image Config の `architecture`、`os`、`variant` が Image Index の `platform` と一致すること | `config_platform_matches_index` |

### 本プロジェクト独自の実装方針

| ID | 方針 | 対応テスト |
|-|-|-|
| `policy-targets` | 対象プラットフォームの一覧（[targets.md](../user/targets.md)）が [`builder/src/target.rs`](../../builder/src/target.rs) の `Target::all()` と一致すること | `documented_platforms_match_targets` |
| `policy-rebuild` | 再ビルド時に不要となった未参照 blob を残存させないこと（仕様の MAY よりも厳格に削除） | `rebuild_replaces_stale_blobs` |
| `policy-escape` | JSON 内の特殊文字を適切にエスケープすること | `json_special_characters_are_escaped` |
| `policy-reproducible` | 同一の入力ファイルかつ同一の圧縮レベルであれば、生成される layer digest と DiffID が完全に一致すること | `same_input_same_layer_digest` |
| `policy-preserve` | Image Layout の生成に失敗した場合、既存の layout ディレクトリを破損させずに保護すること | `missing_layer_dir_is_rejected_before_replace`, `failed_layer_after_staging_preserves_existing_layout` |
| `policy-zstd-publish` | 通常ビルド（レベル 3）と公開ビルド（レベル 22）で DiffID は同一を保ち、layer digest のみが変化すること | `publish_level_changes_blob_but_not_diff_id` |
| `policy-zstd-level` | zstd 圧縮レベルは 1〜22 の範囲とし、通常ビルドは 3、公開ビルドは 22 とすること（`--profile release` と `--zstd-level` は同時指定不可） | `zstd_level_must_be_1_to_22` |

## 発行アノテーション・メディアタイプ一覧

GHCR 公開時に付与するメタデータの一覧です。アノテーションの置き場所は
manifest のトップレベル `annotations`（複数 platform イメージでは
ネスト Image Index の `annotations` にも同内容を付与）です。

| キー / 値 | 置き場所 | 意味 |
|-|-|-|
| `org.opencontainers.image.title` | manifest / index | イメージ名（パッケージ名、`{name}-{arch}` 等） |
| `org.opencontainers.image.version` | manifest / index | バージョンまたは日付-リビジョンタグ |
| `org.opencontainers.image.description` | manifest / index | パッケージページの説明文（GHCR は multi-arch で index の値のみ参照） |
| `org.opencontainers.image.source` | manifest / index | `https://github.com/glatan/containers`。GHCR のリポジトリ自動連携と初回公開時の権限継承に必要 |
| `org.opencontainers.image.revision` | manifest / index | パッケージング時点の HEAD コミット（git 利用不可時は省略） |
| `org.opencontainers.image.licenses` | manifest（パッケージ・index artifact） | SPDX 識別子。compose 系は複数ライセンス混在のため付与しない |
| `dev.containers.dependencies` | manifest（パッケージのみ） | platform 評価済み実行時依存（`name version` カンマ区切り）。独自キー |
| `org.opencontainers.image.ref.name` | index.json / ネスト index 内 descriptor | タグ参照名（containers/image との互換規約） |
| `io.github.containers.compression.zstd` | ネスト index 内 manifest descriptor | skopeo が push 時に付与する zstd 注釈を先回りして付け、digest 一致を保証 |

| メディアタイプ / 値 | 用途 |
|-|-|
| `application/vnd.oci.image.index.v1+json` | Image Index |
| `application/vnd.oci.image.manifest.v1+json` | Image Manifest |
| `application/vnd.oci.image.config.v1+json` | Image Config |
| `application/vnd.oci.image.layer.v1.tar+zstd` | レイヤー blob（zstd 圧縮 tar） |
| `application/vnd.oci.empty.v1+json` | index artifact の空 config |
| `application/vnd.containers.index.v1+json` | index artifact の artifactType および index JSON blob（独自） |
| `unknown/unknown` platform | 実行プラットフォームを持たない index descriptor（buildx 規約） |
| `imageLayoutVersion: "1.0.0"` | oci-layout ファイル |

`ImageManifest.artifactType` は通常イメージでは省略し、index artifact
のみ `vnd.containers.index` を設定します。また `ExecConfig::runnable()` は、
compose 系 runnable イメージにおける環境変数、実行コマンド、および作業ディレクトリを設定します。

## レイヤー圧縮

OCI Image Specification が規定しているのは、レイヤーのエンコーディングを示す media type のみであり、圧縮率、圧縮レベル、辞書の有無などは定義されていません。

ランタイム実装におけるサポート要件として、`application/vnd.oci.image.layer.v1.tar` および `...tar+gzip` は MUST、`...tar+zstd` は SHOULD と位置づけられています。本プロジェクトでは、レイヤーを `application/vnd.oci.image.layer.v1.tar+zstd` として出力します。DiffID には常に未圧縮 tar アーカイブのダイジェストを使用します。

zstd の圧縮レベル設定は仕様外の実装方針です。ビルドの再現性を確保するため、マルチスレッド圧縮は行わず `--single-thread` を指定します。

- `pack` の既定は `release`（レベル 22、公開用高圧縮率、`--ultra`）です。ローカルでの動作確認には `--profile dev`（レベル 3、処理速度優先）を指定します。
- `compose`・`sync`・`env compose` の既定は `dev`（レベル 3）です。
- `--zstd-level` オプションで直接レベルを指定することも可能ですが、`--profile` との同時指定はできません。

## 圧縮レベルの違いと成果物の正本

圧縮レベルを変更すると layer blob のダイジェストは変化しますが、未圧縮 tar のハッシュ値である DiffID は同一のまま保持されます。したがって、レジストリ上においてレベル 3 とレベル 22 の blob を共有することはできません。

ローカルの正本は `oci-layouts/`（`pack` による確定成果物）であり、`packages/artifacts/` はビルドの出力先および `restore` の復元先となる中間ストレージです。既定の増分 merge は既存 platform を保持したまま対象 platform のみを差し替えるため、同一 layout 内に圧縮プロファイルの異なる blob が併存し得ます。全 platform を同一プロファイルに揃える場合は `--replace` で全再生成します。単一パッケージを指定して全再生成する場合は、対応する全 platform の成果物が必要です。`--all-packages` と併用した場合は、成果物が存在しない platform について警告を表示したうえで、該当 platform を layout から除外して処理を継続します。

## プラットフォームと variant の扱い

`platform.variant` は OPTIONAL な文字列フィールドです。仕様書の [Platform Variants](https://github.com/opencontainers/image-spec/blob/v1.1.1/image-index.md#platform-variants) 表に記載されていない値であっても、仕様上は implementation-defined（実装定義）として扱われ、Image Index や Image Config として不正ではありません。

対応プラットフォーム（architecture および variant）の一覧は [targets.md](../user/targets.md) に記載しています。この一覧は [`builder/src/target.rs`](../../builder/src/target.rs) の `Target::all()` と一致させ、不一致はテスト `documented_platforms_match_targets` によって検出します。

下表の status は OCI 仕様に対する分類を示します。仕様書の表に明記されている組み合わせ（`…` で省略表記された系列を含む）を `listed` とし、仕様書の表には記載がないものの実装定義として許容される組み合わせを `defined` とします。

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

一部のコンテナランタイムは、一般的に使われる組み合わせ以外のプラットフォームを自動選択できない場合があります。そのため、利用者側でイメージを指定する際は、予期しないプラットフォームの選択を防ぐために platform 別 manifest のダイジェストで固定します（Image Index のダイジェストでは、manifest の選択がランタイム側の platform マッチング処理に依存するためです）。
