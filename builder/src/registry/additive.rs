//! push の additive 判定。
//!
//! リモートタグが既に存在する場合、トップレベル digest の不一致は
//! 「上書き禁止」ではなく「platform 集合の差分」の可能性がある。
//! リモートの platform 集合がローカルの部分集合であり、かつ共通
//! platform の manifest 内容が一致する場合に限り push を許容する
//! （ローカルの platform 追加は digest が変わっても安全）。
//!
//! manifest の内容一致は次のいずれかとする:
//! - manifest digest が完全一致
//! - config・layers の digest が一致し、かつ annotations が
//!   additive 適合（リモートの annotation がローカルの部分集合。
//!   ローカル側の追加はメタデータ追記として許容し、値の変更・
//!   キーの削除は拒否する。`org.opencontainers.image.revision` は
//!   git HEAD 連動のため値差・欠落を両方向に許容）
//! - 上記が不成立でも、config の意味内容（`created` を正規化して
//!   比較）と layer の media_type が一致し diff_ids が等しい場合は
//!   「同一ペイロードの再エンコード」として許容する
//!   （zstd 圧縮レベル変更や created のエポック化で blob digest が
//!   変わるが、展開後の実態は同一）

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::error::{Error, Result};
use crate::oci::{
    ANNOTATION_REVISION, Descriptor, Digest, ImageIndex, ImageManifest, MEDIA_INDEX, MEDIA_MANIFEST,
};
use crate::target::Target;
use crate::util;

use super::{digest_ref, layout_io, skopeo};

/// remote タグの生 manifest/index バイト列が local layout と
/// additive 適合（remote ⊆ local）かを検査する。
/// `allow_replace` に含まれる platform は内容不一致を許容する
/// （正当なリビルドの置き換え用。platform 削除は許容しない）。
/// 適合しない場合は差分箇所を列挙した Err を返す。
pub fn check(
    layout: &Path,
    image: &layout_io::PublishedImage,
    repository: &str,
    remote_raw: &[u8],
    allow_replace: &[Target],
) -> Result<()> {
    #[derive(serde::Deserialize)]
    struct MediaProbe {
        #[serde(rename = "mediaType", default)]
        media_type: String,
    }
    let probe: MediaProbe = serde_json::from_slice(remote_raw).map_err(|source| {
        Error::message(format!(
            "リモートタグの manifest/index パースに失敗: {source}"
        ))
    })?;
    match probe.media_type.as_str() {
        MEDIA_INDEX => check_index(layout, image, repository, remote_raw, allow_replace),
        MEDIA_MANIFEST => {
            // タグが manifest を直に指す（flat 単一 platform）場合は
            // platform 名が取れないため、local のいずれかの manifest と
            // 内容一致するかで判定する。
            let remote: ImageManifest = serde_json::from_slice(remote_raw).map_err(|source| {
                Error::message(format!("リモート manifest のパースに失敗: {source}"))
            })?;
            for entry in &image.manifests {
                let local = read_local_manifest(layout, &entry.digest)?;
                if content_equal(&remote, &local) {
                    return Ok(());
                }
            }
            let remote_digest = Digest::from_hex(util::sha256_hex(remote_raw));
            // platform 指定の置き換え許容: flat manifest は platform を
            // 持たないため remote config の architecture/variant から
            // 解決する。ローカルがその platform を持たなければ push は
            // platform 削除になるため拒否する。
            if !allow_replace.is_empty()
                && let Some(target) = flat_remote_target(repository, &remote_digest)?
            {
                if !allow_replace.contains(&target) {
                    // 対象外 platform → 以降の semantic 判定へ
                } else if image.manifests.iter().any(|entry| entry.target == target) {
                    return Ok(());
                } else {
                    return Err(Error::message(format!(
                        "リモートが持つ {}/{} をローカルが削除します（platform 削除には明示的な置き換えが必要です）",
                        target.architecture(),
                        target.variant()
                    )));
                }
            }
            // digest レベルで一致する platform が無い場合、再エンコード
            // （圧縮レベル・created の違いで同一ペイロード）の可能性を
            // config 比較で確認する。manifest だけで判別できる不適合は
            // config 取得の往復を省くため先に落とす。
            let mut has_candidate = false;
            for entry in &image.manifests {
                let local = read_local_manifest(layout, &entry.digest)?;
                if semantic_prelim(&remote, &local) {
                    has_candidate = true;
                    break;
                }
            }
            if !has_candidate {
                return Err(Error::message(
                    "リモートの manifest はローカルのどの platform とも一致しません（上書きしません）",
                ));
            }
            let remote_config = remote_config(repository, &remote_digest)?;
            for entry in &image.manifests {
                let local = read_local_manifest(layout, &entry.digest)?;
                let local_config = read_config_value(layout, &local.config.digest)?;
                if semantic_equal(&remote, &local, &remote_config, &local_config) {
                    return Ok(());
                }
            }
            Err(Error::message(
                "リモートの manifest はローカルのどの platform とも一致しません（上書きしません）",
            ))
        }
        other => Err(Error::message(format!(
            "リモートタグが予期しない形式です（{other}）。上書きしません"
        ))),
    }
}

/// リモートが manifest list（ネスト index）の場合の platform 単位比較。
fn check_index(
    layout: &Path,
    image: &layout_io::PublishedImage,
    repository: &str,
    remote_raw: &[u8],
    allow_replace: &[Target],
) -> Result<()> {
    let remote: ImageIndex = serde_json::from_slice(remote_raw)
        .map_err(|source| Error::message(format!("リモート index のパースに失敗: {source}")))?;
    for descriptor in &remote.manifests {
        let target = remote_platform(descriptor)?;
        // allow_replace より先に存在確認: ローカルが platform を持た
        // なければ push は削除になるため置き換え許容でも拒否する。
        let Some(local) = image.manifests.iter().find(|entry| entry.target == target) else {
            return Err(Error::message(format!(
                "リモートが持つ {}/{} をローカルが削除します（platform 削除には明示的な置き換えが必要です）",
                target.architecture(),
                target.variant()
            )));
        };
        // 指定 platform の置き換え許容（正当なリビルドで変わった
        // ペイロードの公開用）。他 platform は通常どおり検証される
        // ため、全体 push で結果的に platform 粒度の置換になる。
        if allow_replace.contains(&target) {
            continue;
        }
        if local.digest == descriptor.digest {
            continue;
        }
        // digest 不一致は revision 注釈の差分の可能性がある。
        // manifest blob を digest 参照で取得して内容を比較する。
        let raw = skopeo::remote_raw(&digest_ref(repository, &descriptor.digest))?;
        let remote_manifest: ImageManifest = serde_json::from_slice(&raw).map_err(|source| {
            Error::message(format!("リモート manifest blob のパースに失敗: {source}"))
        })?;
        let local_manifest = read_local_manifest(layout, &local.digest)?;
        if content_equal(&remote_manifest, &local_manifest) {
            continue;
        }
        // 再エンコード（圧縮レベル・created の違いで同一ペイロード）
        // の可能性を config 比較で確認する。manifest だけで判別
        // できる不適合は config 取得の往復を省くため先に落とす。
        let reasons = if semantic_prelim(&remote_manifest, &local_manifest) {
            let remote_config = remote_config(repository, &descriptor.digest)?;
            let local_config = read_config_value(layout, &local_manifest.config.digest)?;
            config_diffs(&remote_config, &local_config)
        } else {
            content_diffs(&remote_manifest, &local_manifest)
        };
        // 空 = 再エンコードのみの差分（ペイロード同一）で適合。
        if !reasons.is_empty() {
            return Err(Error::message(format!(
                "{}/{} の内容がリモートと異なります: {}",
                target.architecture(),
                target.variant(),
                reasons.join(", ")
            )));
        }
    }
    Ok(())
}

/// remote index の descriptor から対応する `Target` を解決する。
fn remote_platform(descriptor: &Descriptor) -> Result<Target> {
    if descriptor.media_type != MEDIA_MANIFEST {
        return Err(Error::message(format!(
            "リモート index が manifest 以外のエントリを含みます: {}",
            descriptor.media_type
        )));
    }
    let platform = descriptor.platform.as_ref().ok_or_else(|| {
        Error::message("リモート index の manifest に platform がありません".to_string())
    })?;
    let variant = platform.variant.as_deref().unwrap_or_default();
    Target::lookup(&platform.architecture, variant).ok_or_else(|| {
        Error::message(format!(
            "リモート index に未知の platform があります: {}/{}",
            platform.architecture, variant
        ))
    })
}

/// local manifest blob を読む。`published()` は manifest blob の実在を
/// 保証しないため、欠損・破損はここで Err として伝播する。
fn read_local_manifest(layout: &Path, digest: &Digest) -> Result<ImageManifest> {
    layout_io::read_blob(layout, digest)
}

/// local config blob を JSON 値として読む。
fn read_config_value(layout: &Path, digest: &Digest) -> Result<serde_json::Value> {
    layout_io::read_blob(layout, digest)
}

/// リモート manifest の config JSON を取得する。`manifest_digest` は
/// manifest blob の digest（`repo@sha256:...` で参照可能）。
fn remote_config(repository: &str, manifest_digest: &Digest) -> Result<serde_json::Value> {
    let raw = skopeo::remote_config(&digest_ref(repository, manifest_digest))?;
    serde_json::from_slice(&raw)
        .map_err(|source| Error::message(format!("リモート config のパースに失敗: {source}")))
}

/// flat（manifest 直参照）リモートの platform を config blob の
/// architecture/variant から解決する。platform を特定できない
/// config は None（semantic 判定へ進む）。
fn flat_remote_target(repository: &str, manifest_digest: &Digest) -> Result<Option<Target>> {
    let config = remote_config(repository, manifest_digest)?;
    let Some(architecture) = config.get("architecture").and_then(|v| v.as_str()) else {
        return Ok(None);
    };
    let variant = config
        .get("variant")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    Ok(Target::lookup(architecture, variant))
}

/// manifest の内容一致: config/layers の digest が一致し、annotations
/// が additive 適合（リモート ⊆ ローカル、revision は値差を許容）。
fn content_equal(remote: &ImageManifest, local: &ImageManifest) -> bool {
    content_diffs(remote, local).is_empty()
}

/// content_equal の差分箇所を列挙する（空なら一致）。
fn content_diffs(remote: &ImageManifest, local: &ImageManifest) -> Vec<String> {
    let mut diffs = Vec::new();
    if remote.config.media_type != local.config.media_type {
        diffs.push("config media_type".to_string());
    }
    if remote.config.digest != local.config.digest {
        diffs.push("config digest".to_string());
    }
    if remote.layers.len() != local.layers.len() {
        diffs.push(format!(
            "layer 個数 {} vs {}",
            remote.layers.len(),
            local.layers.len()
        ));
    } else {
        for (index, (remote, local)) in remote.layers.iter().zip(&local.layers).enumerate() {
            if remote.media_type != local.media_type {
                diffs.push(format!("layer[{index}] media_type"));
            }
            if remote.digest != local.digest {
                diffs.push(format!("layer[{index}] digest"));
            }
        }
    }
    diffs.extend(annotation_diffs(&remote.annotations, &local.annotations));
    if remote.artifact_type != local.artifact_type {
        diffs.push("artifact_type".to_string());
    }
    diffs
}

/// ペイロード同一性の判定（digest 不一致 manifest の第2段）。
///
/// blob digest が違っても「同一ペイロードの再エンコード」はあり得る:
/// - config の `created`: 再現性確保でエポック固定化したため差が出る
/// - layer の圧縮 blob: zstd レベル変更で digest が変わるが
///   `rootfs.diff_ids`（展開後 tar の digest）が一致すれば実態は同一
///
/// config は `created` を除いた全フィールドの一致を要求する
/// （architecture/os/variant/env/cmd/diff_ids が全て同じ）。
/// layer は個数と media_type の一致を要求する（tar+gzip→tar+zstd の
/// ような形式変更は許容しない）。annotations と artifact_type の
/// 要件は digest 比較と同じ。
fn semantic_equal(
    remote: &ImageManifest,
    local: &ImageManifest,
    remote_config: &serde_json::Value,
    local_config: &serde_json::Value,
) -> bool {
    semantic_prelim(remote, local)
        && normalize_config(remote_config) == normalize_config(local_config)
}

/// manifest フィールドだけで判別できるペイロード不適合の事前チェック
/// （config の往復取得を省くための早期拒否）。
fn semantic_prelim(remote: &ImageManifest, local: &ImageManifest) -> bool {
    remote.layers.len() == local.layers.len()
        && remote
            .layers
            .iter()
            .zip(&local.layers)
            .all(|(remote, local)| remote.media_type == local.media_type)
        && annotations_compatible(&remote.annotations, &local.annotations)
        && remote.artifact_type == local.artifact_type
}

/// `created` タイムスタンプを除いた config の正規化。
fn normalize_config(config: &serde_json::Value) -> serde_json::Value {
    let mut config = config.clone();
    if let Some(object) = config.as_object_mut() {
        object.remove("created");
    }
    config
}

/// normalize 後の config の差分キーを列挙する（空ならペイロード同一）。
/// `rootfs.diff_ids` の不一致はペイロード変更を意味するため明示する。
fn config_diffs(remote: &serde_json::Value, local: &serde_json::Value) -> Vec<String> {
    let remote = normalize_config(remote);
    let local = normalize_config(local);
    let (Some(remote), Some(local)) = (remote.as_object(), local.as_object()) else {
        return if remote == local {
            Vec::new()
        } else {
            vec!["config".to_string()]
        };
    };
    remote
        .keys()
        .chain(local.keys())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|key| remote.get(*key) != local.get(*key))
        .map(|key| {
            if key == "rootfs" {
                let r = remote.get("rootfs").and_then(|v| v.get("diff_ids"));
                let l = local.get("rootfs").and_then(|v| v.get("diff_ids"));
                if r != l {
                    "config.rootfs.diff_ids（ペイロード変更）".to_string()
                } else {
                    "config.rootfs".to_string()
                }
            } else {
                format!("config.{key}")
            }
        })
        .collect()
}

/// annotations の additive 適合: リモートの annotation が全て
/// ローカルに同値で存在する（ローカル側の追加は許容する
/// メタデータ追記。値の変更・キーの削除は拒否）。
/// `org.opencontainers.image.revision` のみ値差・欠落を両方向に許容
/// （git HEAD 連動のため同一成果物でも変わる）。
fn annotations_compatible(
    remote: &BTreeMap<String, String>,
    local: &BTreeMap<String, String>,
) -> bool {
    annotation_diffs(remote, local).is_empty()
}

/// annotation の不適合箇所を列挙する（値変更・キー削除。追加は適合）。
fn annotation_diffs(
    remote: &BTreeMap<String, String>,
    local: &BTreeMap<String, String>,
) -> Vec<String> {
    remote
        .iter()
        .filter(|(key, value)| key != &ANNOTATION_REVISION && local.get(*key) != Some(*value))
        .map(|(key, value)| match local.get(key) {
            Some(other) => format!("annotation {key}: '{value}'→'{other}'"),
            None => format!("annotation {key}: 削除"),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci::PlatformObject;
    use crate::registry::layout_io::PlatformManifest;
    use crate::target::{Amd64Variant, Target};

    fn digest(hex: &str) -> Digest {
        Digest::from_hex(hex.to_string())
    }

    fn descriptor(hex: &str) -> Descriptor {
        Descriptor {
            media_type: MEDIA_MANIFEST.to_string(),
            digest: digest(hex),
            size: 1,
            annotations: None,
            platform: None,
        }
    }

    fn manifest(config: &str, layer: &str, revision: Option<&str>) -> ImageManifest {
        let mut annotations = BTreeMap::new();
        if let Some(revision) = revision {
            annotations.insert(ANNOTATION_REVISION.to_string(), revision.to_string());
        }
        ImageManifest {
            schema_version: 2,
            media_type: MEDIA_MANIFEST.to_string(),
            artifact_type: None,
            config: descriptor(config),
            layers: vec![descriptor(layer)],
            annotations,
        }
    }

    #[test]
    fn revision_only_diff_is_compatible() {
        let remote = manifest(
            "a".repeat(64).as_str(),
            "b".repeat(64).as_str(),
            Some("abc"),
        );
        let local = manifest(
            "a".repeat(64).as_str(),
            "b".repeat(64).as_str(),
            Some("def"),
        );
        assert!(content_equal(&remote, &local));
    }

    #[test]
    fn layer_diff_is_incompatible() {
        let remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        let local = manifest("a".repeat(64).as_str(), "c".repeat(64).as_str(), None);
        assert!(!content_equal(&remote, &local));
    }

    #[test]
    fn other_annotation_diff_is_incompatible() {
        let mut remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        remote.annotations.insert(
            "org.opencontainers.image.title".to_string(),
            "x".to_string(),
        );
        let local = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        assert!(!content_equal(&remote, &local));
    }

    #[test]
    fn annotation_addition_is_additive() {
        // ローカルが annotation を追加するだけなら additive 適合
        // （リモートの annotation は全て同値で残っている）。
        let remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        let mut local = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        local.annotations.insert(
            "org.opencontainers.image.description".to_string(),
            "added".to_string(),
        );
        assert!(content_equal(&remote, &local));
    }

    #[test]
    fn annotation_value_change_is_incompatible() {
        let mut remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        remote.annotations.insert(
            "org.opencontainers.image.title".to_string(),
            "old".to_string(),
        );
        let mut local = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        local.annotations.insert(
            "org.opencontainers.image.title".to_string(),
            "new".to_string(),
        );
        assert!(!content_equal(&remote, &local));
    }

    #[test]
    fn recompressed_same_payload_is_compatible() {
        // 圧縮 blob/config digest が違っても、diff_ids 一致 +
        // created 差のみなら同一ペイロード（再エンコード）として適合。
        let remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        let local = manifest(
            "c".repeat(64).as_str(),
            "d".repeat(64).as_str(),
            Some("rev"),
        );
        let remote_config = serde_json::json!({
            "created": "2026-10-13T12:24:08Z",
            "architecture": "arm64",
            "os": "linux",
            "rootfs": {"type": "layers", "diff_ids": ["sha256:e"]}
        });
        let local_config = serde_json::json!({
            "created": "1970-01-01T00:00:00Z",
            "architecture": "arm64",
            "os": "linux",
            "rootfs": {"type": "layers", "diff_ids": ["sha256:e"]}
        });
        assert!(semantic_equal(
            &remote,
            &local,
            &remote_config,
            &local_config
        ));
    }

    #[test]
    fn different_diff_ids_is_incompatible() {
        // 展開後の内容が違う（diff_ids 不一致）のは実質的な内容変更。
        let remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        let local = manifest("c".repeat(64).as_str(), "d".repeat(64).as_str(), None);
        let remote_config = serde_json::json!({
            "created": "2026-10-13T12:24:08Z",
            "rootfs": {"type": "layers", "diff_ids": ["sha256:e"]}
        });
        let local_config = serde_json::json!({
            "created": "1970-01-01T00:00:00Z",
            "rootfs": {"type": "layers", "diff_ids": ["sha256:f"]}
        });
        assert!(!semantic_equal(
            &remote,
            &local,
            &remote_config,
            &local_config
        ));
    }

    #[test]
    fn different_config_field_is_incompatible() {
        // created 以外の config 差分（例: architecture）は拒否。
        let remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        let local = manifest("c".repeat(64).as_str(), "d".repeat(64).as_str(), None);
        let remote_config = serde_json::json!({
            "created": "x",
            "architecture": "arm64",
            "rootfs": {"type": "layers", "diff_ids": ["sha256:e"]}
        });
        let local_config = serde_json::json!({
            "created": "y",
            "architecture": "amd64",
            "rootfs": {"type": "layers", "diff_ids": ["sha256:e"]}
        });
        assert!(!semantic_equal(
            &remote,
            &local,
            &remote_config,
            &local_config
        ));
    }

    #[test]
    fn layer_media_type_change_is_incompatible() {
        // tar+gzip → tar+zstd のような形式変更は diff_ids が一致しても拒否。
        let mut remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        remote.layers[0].media_type = "application/vnd.oci.image.layer.v1.tar+gzip".to_string();
        let local = manifest("c".repeat(64).as_str(), "d".repeat(64).as_str(), None);
        let config = serde_json::json!({
            "rootfs": {"type": "layers", "diff_ids": ["sha256:e"]}
        });
        assert!(!semantic_equal(&remote, &local, &config, &config));
    }

    #[test]
    fn remote_platform_resolves_target() {
        let target = Target::Amd64(Amd64Variant::V4);
        let mut entry = descriptor(&"a".repeat(64));
        entry.platform = Some(PlatformObject::from(target));
        assert_eq!(remote_platform(&entry).unwrap(), target);
    }

    #[test]
    fn unknown_remote_platform_is_rejected() {
        let mut entry = descriptor(&"a".repeat(64));
        entry.platform = Some(PlatformObject {
            architecture: "mips64".to_string(),
            os: "linux".to_string(),
            variant: None,
        });
        assert!(remote_platform(&entry).is_err());
    }

    #[test]
    fn platform_manifest_struct_is_used() {
        // PlatformManifest のフィールドが本モジュールの前提と一致することの
        // コンパイル時確認。
        let _ = PlatformManifest {
            target: Target::Amd64(Amd64Variant::V4),
            digest: digest(&"a".repeat(64)),
            size: 0,
        };
    }

    /// 実レイアウトを1つ構築し、その `PublishedImage` を返す。
    /// `check` の flat/index 分岐を実 blob 相手に検査するための土台。
    fn local_layout(
        targets: &[Target],
    ) -> (
        tempfile::TempDir,
        std::path::PathBuf,
        layout_io::PublishedImage,
    ) {
        let tmp = tempfile::tempdir().unwrap();
        let layer = tmp.path().join("layer");
        std::fs::create_dir_all(&layer).unwrap();
        std::fs::write(layer.join("file.txt"), b"content").unwrap();
        let output = tmp.path().join("layout");
        let image = crate::oci::Image {
            name: "pkg".to_string(),
            version: "1".to_string(),
            description: None,
        };
        let layers: Vec<_> = targets
            .iter()
            .map(|target| (*target, vec![layer.clone()]))
            .collect();
        crate::oci::build(&output, &image, &layers, crate::oci::DEFAULT_ZSTD_LEVEL).unwrap();
        let published = layout_io::published(&output).unwrap();
        (tmp, output, published)
    }

    fn manifest_bytes(layout: &Path, digest: &Digest) -> Vec<u8> {
        std::fs::read(layout.join("blobs/sha256").join(digest.encoded())).unwrap()
    }

    #[test]
    fn flat_remote_manifest_matching_local_is_additive() {
        // リモートタグが manifest 直参照（flat）で、内容がローカルの
        // platform のどれかと一致するなら additive 適合。
        let (_tmp, layout, image) = local_layout(&[Target::Amd64(Amd64Variant::V4)]);
        let raw = manifest_bytes(&layout, &image.manifests[0].digest);
        check(&layout, &image, "ghcr.io/x/pkg", &raw, &[]).unwrap();
    }

    #[test]
    fn flat_remote_manifest_with_different_content_is_rejected() {
        let (_tmp, layout, image) = local_layout(&[Target::Amd64(Amd64Variant::V4)]);
        let raw = manifest_bytes(&layout, &image.manifests[0].digest);
        let mut remote: ImageManifest = serde_json::from_slice(&raw).unwrap();
        // layer の個数が違う = 明確に別物。semantic_prelim で落ちる
        // （config 往復を要さない経路を検査する）。
        remote.layers.push(descriptor(&"f".repeat(64)));
        let raw = serde_json::to_vec(&remote).unwrap();
        let error = check(&layout, &image, "ghcr.io/x/pkg", &raw, &[]).unwrap_err();
        assert!(error.to_string().contains("一致しません"), "{error}");
    }

    #[test]
    fn missing_local_manifest_blob_is_error_not_panic() {
        // published() は manifest blob の実在を保証しない。
        // 欠損 blob を参照する PublishedImage は panic ではなく Err
        // を返す（以前は expect でパニックしていた回帰テスト）。
        let (_tmp, layout, mut image) = local_layout(&[Target::Amd64(Amd64Variant::V4)]);
        image.manifests[0].digest = digest(&"e".repeat(64));
        let remote = manifest("a".repeat(64).as_str(), "b".repeat(64).as_str(), None);
        let raw = serde_json::to_vec(&remote).unwrap();
        assert!(check(&layout, &image, "ghcr.io/x/pkg", &raw, &[]).is_err());
    }

    #[test]
    fn index_remote_with_same_digests_is_additive() {
        // リモート index の platform が全てローカルと digest 一致
        // するなら、manifest blob の取得なしで additive 適合。
        let (_tmp, layout, image) = local_layout(&[
            Target::Amd64(Amd64Variant::V4),
            Target::Amd64(Amd64Variant::V1),
        ]);
        let remote = ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests: image
                .manifests
                .iter()
                .map(|entry| Descriptor {
                    media_type: MEDIA_MANIFEST.to_string(),
                    digest: entry.digest.clone(),
                    size: entry.size,
                    annotations: None,
                    platform: Some(PlatformObject::from(entry.target)),
                })
                .collect(),
            annotations: None,
        };
        let raw = serde_json::to_vec(&remote).unwrap();
        check(&layout, &image, "ghcr.io/x/pkg", &raw, &[]).unwrap();
    }

    #[test]
    fn allow_replace_skips_content_check_for_that_platform() {
        // allow_replace に含まれる platform は manifest blob を
        // 取得せず digest 不一致でも適合（置き換え許容）。
        // remote digest がローカル blob に存在しなくても通ることで
        // 「取得も比較もスキップされる」ことを検証する。
        let (_tmp, layout, image) = local_layout(&[Target::Amd64(Amd64Variant::V4)]);
        let remote = ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests: vec![Descriptor {
                media_type: MEDIA_MANIFEST.to_string(),
                digest: digest(&"9".repeat(64)),
                size: 1,
                annotations: None,
                platform: Some(PlatformObject::from(Target::Amd64(Amd64Variant::V4))),
            }],
            annotations: None,
        };
        let raw = serde_json::to_vec(&remote).unwrap();
        check(
            &layout,
            &image,
            "ghcr.io/x/pkg",
            &raw,
            &[Target::Amd64(Amd64Variant::V4)],
        )
        .unwrap();
    }

    #[test]
    fn allow_replace_still_rejects_platform_removal() {
        // allow_replace に含まれても、ローカルがその platform を
        // 持たなければ push は platform 削除になるため拒否する。
        let (_tmp, layout, image) = local_layout(&[Target::Amd64(Amd64Variant::V4)]);
        let remote = ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests: vec![
                Descriptor {
                    media_type: MEDIA_MANIFEST.to_string(),
                    digest: image.manifests[0].digest.clone(),
                    size: image.manifests[0].size,
                    annotations: None,
                    platform: Some(PlatformObject::from(Target::Amd64(Amd64Variant::V4))),
                },
                Descriptor {
                    media_type: MEDIA_MANIFEST.to_string(),
                    digest: digest(&"d".repeat(64)),
                    size: 1,
                    annotations: None,
                    platform: Some(PlatformObject::from(Target::Arm64(
                        crate::target::Arm64Variant::V8,
                    ))),
                },
            ],
            annotations: None,
        };
        let raw = serde_json::to_vec(&remote).unwrap();
        let error = check(
            &layout,
            &image,
            "ghcr.io/x/pkg",
            &raw,
            &[Target::Arm64(crate::target::Arm64Variant::V8)],
        )
        .unwrap_err();
        assert!(error.to_string().contains("削除します"), "{error}");
    }

    #[test]
    fn other_platform_still_checked_when_allow_replace() {
        // allow_replace 対象でない platform は通常どおり検証される。
        let (_tmp, layout, image) = local_layout(&[
            Target::Amd64(Amd64Variant::V4),
            Target::Amd64(Amd64Variant::V1),
        ]);
        let remote = ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests: vec![
                Descriptor {
                    media_type: MEDIA_MANIFEST.to_string(),
                    digest: digest(&"9".repeat(64)),
                    size: 1,
                    annotations: None,
                    platform: Some(PlatformObject::from(Target::Amd64(Amd64Variant::V4))),
                },
                Descriptor {
                    media_type: MEDIA_MANIFEST.to_string(),
                    digest: digest(&"8".repeat(64)),
                    size: 1,
                    annotations: None,
                    platform: Some(PlatformObject::from(Target::Amd64(Amd64Variant::V1))),
                },
            ],
            annotations: None,
        };
        let raw = serde_json::to_vec(&remote).unwrap();
        // V4 は許容、V1 は未許容 → V1 の digest 不一致で拒否される
        // （remote manifest blob は実在しないため取得エラーでも
        // いずれにせよ Err）。
        assert!(
            check(
                &layout,
                &image,
                "ghcr.io/x/pkg",
                &raw,
                &[Target::Amd64(Amd64Variant::V4)],
            )
            .is_err()
        );
    }

    #[test]
    fn index_remote_with_extra_platform_is_rejected() {
        // ローカルが持たない platform をリモートが持つ = platform の
        // 削除になるため拒否する。
        let (_tmp, layout, image) = local_layout(&[Target::Amd64(Amd64Variant::V4)]);
        let remote = ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests: vec![
                Descriptor {
                    media_type: MEDIA_MANIFEST.to_string(),
                    digest: image.manifests[0].digest.clone(),
                    size: image.manifests[0].size,
                    annotations: None,
                    platform: Some(PlatformObject::from(Target::Amd64(Amd64Variant::V4))),
                },
                Descriptor {
                    media_type: MEDIA_MANIFEST.to_string(),
                    digest: digest(&"d".repeat(64)),
                    size: 1,
                    annotations: None,
                    platform: Some(PlatformObject::from(Target::Arm64(
                        crate::target::Arm64Variant::V8,
                    ))),
                },
            ],
            annotations: None,
        };
        let raw = serde_json::to_vec(&remote).unwrap();
        let error = check(&layout, &image, "ghcr.io/x/pkg", &raw, &[]).unwrap_err();
        assert!(error.to_string().contains("削除します"), "{error}");
    }
}
