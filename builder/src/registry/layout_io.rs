//! OCI Image Layout の読み取り側。`oci::model` の Deserialize 型を使い、
//! トップレベル index.json を解決する（単一 platform は manifest 直参照の
//! flat 形式、複数 platform はネストした Image Index blob を辿る）。

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::oci::{
    Descriptor, Digest, ImageConfig, ImageIndex, ImageManifest, MEDIA_INDEX, MEDIA_MANIFEST,
};
use crate::target::Target;

/// ネスト index の1要素: platform 別 manifest の参照。
#[derive(Debug)]
pub struct PlatformManifest {
    pub target: Target,
    pub digest: Digest,
    pub size: u64,
}

/// layout が公開するイメージの索引情報。
#[derive(Debug)]
pub struct PublishedImage {
    /// ネスト index blob の digest。push 後にタグが指す digest。
    pub index_digest: Digest,
    /// トップレベル descriptor の `org.opencontainers.image.ref.name`
    /// 注釈（compose 生成物のタグ復元に使う）。
    pub ref_name: Option<String>,
    pub manifests: Vec<PlatformManifest>,
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|source| Error::io(path, source))?;
    serde_json::from_slice(&bytes)
        .map_err(|source| Error::message(format!("{} のパースに失敗: {source}", path.display())))
}

/// `blobs/sha256/{hex}` を JSON として読む。
pub fn read_blob<T: DeserializeOwned>(layout: &Path, digest: &Digest) -> Result<T> {
    read_json(&layout.join("blobs/sha256").join(digest.encoded()))
}

/// トップレベルの index.json を読む。
pub fn top_index(layout: &Path) -> Result<ImageIndex> {
    read_json(&layout.join("index.json"))
}

/// index.json の単一 descriptor が指す digest。
/// パッケージイメージではネスト index blob、index artifact では
/// artifact manifest を指す。
pub fn top_digest(layout: &Path) -> Result<Digest> {
    Ok(single_entry(layout)?.digest)
}

/// index.json の単一 descriptor を返す（複数エントリは oci:
/// トランスポート互換のため生成しない）。
fn single_entry(layout: &Path) -> Result<Descriptor> {
    let top = top_index(layout)?;
    let mut manifests = top.manifests.into_iter();
    let Some(entry) = manifests.next() else {
        return Err(Error::message(format!(
            "{} の index.json に manifest がありません",
            layout.display()
        )));
    };
    if manifests.next().is_some() {
        return Err(Error::message(format!(
            "{} の index.json が複数エントリを持ちます（ネスト index 形式ではありません）",
            layout.display()
        )));
    }
    Ok(entry)
}

/// パッケージイメージの layout からネスト index を解決する。
/// 単一 platform の旧形式（index.json → manifest 直参照）も受理する。
pub fn published(layout: &Path) -> Result<PublishedImage> {
    let entry = single_entry(layout)?;
    if entry.media_type == MEDIA_MANIFEST {
        return single_manifest(layout, &entry);
    }
    if entry.media_type != MEDIA_INDEX {
        return Err(Error::message(format!(
            "{} の index.json がネスト index を指していません: {}",
            layout.display(),
            entry.media_type
        )));
    }
    let ref_name = entry
        .annotations
        .as_ref()
        .and_then(|annotations| annotations.get(crate::oci::ANNOTATION_REF_NAME))
        .cloned();
    let nested: ImageIndex = read_blob(layout, &entry.digest)?;
    if nested.media_type != MEDIA_INDEX {
        return Err(Error::message(format!(
            "{} のネスト blob が Image Index ではありません",
            layout.display()
        )));
    }
    let mut manifests = Vec::new();
    for descriptor in &nested.manifests {
        if descriptor.media_type != MEDIA_MANIFEST {
            return Err(Error::message(format!(
                "{} のネスト index が manifest 以外を指しています: {}",
                layout.display(),
                descriptor.media_type
            )));
        }
        manifests.push(PlatformManifest {
            target: resolve_target(layout, descriptor)?,
            digest: descriptor.digest.clone(),
            size: descriptor.size,
        });
    }
    Ok(PublishedImage {
        index_digest: entry.digest,
        ref_name,
        manifests,
    })
}

/// descriptor の platform から `Target` を解決する。
/// skopeo pull した flat layout は descriptor に platform が欠落
/// （または variant 欠落）し得るため、その場合は manifest blob の
/// config（architecture/variant を保持）から復元する。
fn resolve_target(layout: &Path, descriptor: &Descriptor) -> Result<Target> {
    if let Some(platform) = &descriptor.platform
        && let Some(target) = Target::lookup(
            &platform.architecture,
            platform.variant.as_deref().unwrap_or_default(),
        )
    {
        return Ok(target);
    }
    let manifest: ImageManifest = read_blob(layout, &descriptor.digest)?;
    let config: ImageConfig = read_blob(layout, &manifest.config.digest)?;
    Target::lookup(
        &config.architecture,
        config.variant.as_deref().unwrap_or_default(),
    )
    .ok_or_else(|| {
        Error::message(format!(
            "{} の manifest に platform を特定できません: {}",
            layout.display(),
            descriptor.digest.prefixed()
        ))
    })
}

/// index.json が manifest を直接指す単一 platform の flat layout。
fn single_manifest(layout: &Path, entry: &Descriptor) -> Result<PublishedImage> {
    let target = resolve_target(layout, entry)?;
    let ref_name = entry
        .annotations
        .as_ref()
        .and_then(|annotations| annotations.get(crate::oci::ANNOTATION_REF_NAME))
        .cloned();
    Ok(PublishedImage {
        index_digest: entry.digest.clone(),
        ref_name,
        manifests: vec![PlatformManifest {
            target,
            digest: entry.digest.clone(),
            size: entry.size,
        }],
    })
}

/// layout が参照する全 blob の実在と digest を検査する。
/// index.json 自体の構造不良は `Err`、参照先 blob の問題は検出した
/// 全件を返す（空なら健全）。push 前の健全性検査に使う。
pub fn verify(layout: &Path) -> Result<Vec<String>> {
    let entry = single_entry(layout)?;
    let mut issues = Vec::new();
    match entry.media_type.as_str() {
        MEDIA_INDEX => {
            if check_blob(layout, &entry.digest, "index", &mut issues) {
                match read_blob::<ImageIndex>(layout, &entry.digest) {
                    Ok(nested) => {
                        for descriptor in &nested.manifests {
                            let label = descriptor
                                .platform
                                .as_ref()
                                .map(|platform| {
                                    format!(
                                        "{}/{}",
                                        platform.architecture,
                                        platform.variant.as_deref().unwrap_or_default()
                                    )
                                })
                                .unwrap_or_else(|| descriptor.digest.encoded()[..12].to_string());
                            verify_manifest(layout, &descriptor.digest, &label, &mut issues);
                        }
                    }
                    Err(error) => issues.push(format!("ネスト index のパースに失敗: {error}")),
                }
            }
        }
        MEDIA_MANIFEST => verify_manifest(layout, &entry.digest, "manifest", &mut issues),
        other => issues.push(format!(
            "index.json が未知の mediaType を指しています: {other}"
        )),
    }
    Ok(issues)
}

/// blob が存在し digest と一致するか。問題があれば `issues` に追記する。
fn check_blob(layout: &Path, digest: &Digest, label: &str, issues: &mut Vec<String>) -> bool {
    let path = layout.join("blobs/sha256").join(digest.encoded());
    match fs::read(&path) {
        Err(_) => {
            issues.push(format!("{label}: blob がありません: {}", digest.prefixed()));
            false
        }
        Ok(bytes) => {
            if crate::util::sha256_hex(&bytes) == digest.encoded() {
                true
            } else {
                issues.push(format!(
                    "{label}: digest が一致しません: {}",
                    digest.prefixed()
                ));
                false
            }
        }
    }
}

/// manifest blob とその config・layer 参照を検査する。
fn verify_manifest(layout: &Path, digest: &Digest, label: &str, issues: &mut Vec<String>) {
    if !check_blob(layout, digest, label, issues) {
        return;
    }
    let manifest: ImageManifest = match read_blob(layout, digest) {
        Ok(manifest) => manifest,
        Err(error) => {
            issues.push(format!("{label}: manifest のパースに失敗: {error}"));
            return;
        }
    };
    check_blob(layout, &manifest.config.digest, label, issues);
    for layer in &manifest.layers {
        check_blob(layout, &layer.digest, label, issues);
    }
}

/// pull した単一 manifest の layout（index.json → manifest）を読む。
/// `skopeo copy docker://{name}@{manifestDigest} oci:{dir}` の出力形。
pub fn pulled_manifest(layout: &Path) -> Result<ImageManifest> {
    let entry = single_entry(layout)?;
    if entry.media_type != MEDIA_MANIFEST {
        return Err(Error::message(format!(
            "{} の index.json が manifest を指していません: {}",
            layout.display(),
            entry.media_type
        )));
    }
    read_blob(layout, &entry.digest)
}
