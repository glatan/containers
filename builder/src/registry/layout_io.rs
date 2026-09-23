//! OCI Image Layout の読み取り側。`oci::model` の Deserialize 型を使い、
//! ネスト index（トップレベル index.json → 単一 descriptor →
//! ネストした Image Index blob）を解決する。

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::oci::{Descriptor, Digest, ImageIndex, ImageManifest, MEDIA_INDEX, MEDIA_MANIFEST};
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
    read_json(
        &layout
            .join("blobs/sha256")
            .join(digest.encoded()),
    )
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
        let platform = descriptor.platform.as_ref().ok_or_else(|| {
            Error::message(format!(
                "{} の manifest に platform がありません",
                layout.display()
            ))
        })?;
        let variant = platform.variant.as_deref().unwrap_or_default();
        let target = Target::lookup(&platform.architecture, variant).ok_or_else(|| {
            Error::message(format!(
                "{} の manifest に未知の platform があります: {}/{}",
                layout.display(),
                platform.architecture,
                variant
            ))
        })?;
        manifests.push(PlatformManifest {
            target,
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

/// index.json が manifest を直接指す旧形式の単一 platform layout。
fn single_manifest(layout: &Path, entry: &Descriptor) -> Result<PublishedImage> {
    let platform = entry.platform.as_ref().ok_or_else(|| {
        Error::message(format!(
            "{} の manifest descriptor に platform がありません",
            layout.display()
        ))
    })?;
    let variant = platform.variant.as_deref().unwrap_or_default();
    let target = Target::lookup(&platform.architecture, variant).ok_or_else(|| {
        Error::message(format!(
            "{} の manifest に未知の platform があります: {}/{}",
            layout.display(),
            platform.architecture,
            variant
        ))
    })?;
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
