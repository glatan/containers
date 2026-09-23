use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::oci::layer;
use crate::oci::model::{
    Descriptor, Digest, ExecConfig, Image, ImageConfig, ImageIndex, ImageManifest, OciLayout,
    PlatformObject, RootFs, ZstdLevel,
};
use crate::target::Target;
use crate::util::{self, timestamp};

pub(crate) const MEDIA_INDEX: &str = "application/vnd.oci.image.index.v1+json";
pub(crate) const MEDIA_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub(crate) const MEDIA_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
pub(crate) const MEDIA_LAYER: &str = "application/vnd.oci.image.layer.v1.tar+zstd";
/// containers/image の `oci:` トランスポートが受理するよう、トップレベル
/// index.json の descriptor に付ける参照名（buildx 出力と同じ規約）。
pub(crate) const ANNOTATION_REF_NAME: &str = "org.opencontainers.image.ref.name";

pub fn build(
    output: &Path,
    image: &Image,
    layers: &[(Target, Vec<PathBuf>)],
    zstd_level: u8,
) -> Result<()> {
    build_with_config(output, image, layers, zstd_level, ExecConfig::default())
}

pub fn build_with_config(
    output: &Path,
    image: &Image,
    layers: &[(Target, Vec<PathBuf>)],
    zstd_level: u8,
    config: ExecConfig,
) -> Result<()> {
    build_annotated(output, image, layers, zstd_level, config, &|_| BTreeMap::new())
}

/// `annotate` は manifest 単位の追加アノテーションを返す
/// （title/version は自動付与。依存の人間可読版など platform 別に変わる
/// 値を載せるため Target を受け取る）。
pub fn build_annotated(
    output: &Path,
    image: &Image,
    layers: &[(Target, Vec<PathBuf>)],
    zstd_level: u8,
    config: ExecConfig,
    annotate: &dyn Fn(Target) -> BTreeMap<String, String>,
) -> Result<()> {
    let zstd_level = ZstdLevel::new(zstd_level)?;
    if layers.is_empty() {
        return Err(Error::message("レイヤーがありません"));
    }
    for (_, dirs) in layers {
        for dir in dirs {
            if !dir.is_dir() {
                return Err(Error::message(format!(
                    "レイヤーのディレクトリが見つかりません: {}",
                    dir.display()
                )));
            }
        }
    }
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let staging = tempfile::TempDir::new_in(parent).map_err(|source| Error::io(parent, source))?;
    let blobs = staging.path().join("blobs/sha256");
    fs::create_dir_all(&blobs).map_err(|source| Error::io(&blobs, source))?;
    let mut manifests = Vec::new();
    for (target, layer_dirs) in layers {
        manifests.push(build_manifest(
            &blobs,
            image,
            *target,
            layer_dirs,
            zstd_level,
            &config,
            annotate(*target),
        )?);
    }
    // containers/image の oci: トランスポートは index.json の複数エントリを
    // 受理しない（ErrMoreThanOneImage）ため、buildx 出力と同じく
    // 「index.json → 単一 descriptor → ネストした Image Index blob」の形にする。
    // push 後にタグが指す digest はネスト index blob の digest になる。
    let nested = ImageIndex {
        schema_version: 2,
        media_type: MEDIA_INDEX.to_string(),
        manifests,
    };
    let (nested_digest, nested_size) = store_blob(&blobs, &serde_json::to_vec(&nested)?)?;
    let index = ImageIndex {
        schema_version: 2,
        media_type: MEDIA_INDEX.to_string(),
        manifests: vec![Descriptor {
            media_type: MEDIA_INDEX.to_string(),
            digest: nested_digest,
            size: nested_size,
            platform: Some(PlatformObject {
                architecture: "unknown".to_string(),
                os: "unknown".to_string(),
                variant: None,
            }),
            annotations: Some(BTreeMap::from([(
                ANNOTATION_REF_NAME.to_string(),
                image.version.clone(),
            )])),
        }],
    };
    util::write(
        &staging.path().join("index.json"),
        serde_json::to_vec(&index)?,
    )?;
    util::write(
        &staging.path().join("oci-layout"),
        serde_json::to_vec(&OciLayout {
            image_layout_version: "1.0.0".to_string(),
        })?,
    )?;
    replace_dir(output, staging)
}

fn build_manifest(
    blobs: &Path,
    image: &Image,
    target: Target,
    layer_dirs: &[PathBuf],
    zstd_level: ZstdLevel,
    exec: &ExecConfig,
    extra_annotations: BTreeMap<String, String>,
) -> Result<Descriptor> {
    let mut layers = Vec::new();
    let mut diff_ids = Vec::new();
    for dir in layer_dirs {
        let layer = layer::build(blobs, dir, zstd_level)?;
        diff_ids.push(layer.diff_id);
        layers.push(Descriptor {
            media_type: MEDIA_LAYER.to_string(),
            digest: layer.blob,
            size: layer.size,
            platform: None,
            annotations: None,
        });
    }
    let config = ImageConfig {
        created: timestamp(),
        architecture: target.architecture().to_string(),
        os: "linux".to_string(),
        variant: Some(target.variant().to_string()),
        config: exec.clone(),
        rootfs: RootFs {
            type_: "layers".to_string(),
            diff_ids,
        },
    };
    let (config_digest, config_size) = store_blob(blobs, &serde_json::to_vec(&config)?)?;
    let mut annotations = BTreeMap::from([
        (
            "org.opencontainers.image.title".to_string(),
            image.name.clone(),
        ),
        (
            "org.opencontainers.image.version".to_string(),
            image.version.clone(),
        ),
    ]);
    annotations.extend(extra_annotations);
    let manifest = ImageManifest {
        schema_version: 2,
        media_type: MEDIA_MANIFEST.to_string(),
        artifact_type: None,
        config: Descriptor {
            media_type: MEDIA_CONFIG.to_string(),
            digest: config_digest,
            size: config_size,
            platform: None,
            annotations: None,
        },
        layers,
        annotations,
    };
    let (digest, size) = store_blob(blobs, &serde_json::to_vec(&manifest)?)?;
    Ok(Descriptor {
        media_type: MEDIA_MANIFEST.to_string(),
        digest,
        size,
        platform: Some(PlatformObject::from(target)),
        annotations: None,
    })
}

pub(crate) fn store_blob(blobs: &Path, data: &[u8]) -> Result<(Digest, u64)> {
    let digest = Digest::from_hex(crate::util::sha256_hex(data));
    let path = blobs.join(digest.encoded());
    util::write(&path, data)?;
    Ok((digest, data.len() as u64))
}

fn backup_path(output: &Path) -> PathBuf {
    let mut name = output
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("layout"))
        .to_os_string();
    name.push(".bak");
    match output.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(name),
        _ => PathBuf::from(name),
    }
}

pub(crate) fn replace_dir(output: &Path, staging: tempfile::TempDir) -> Result<()> {
    let backup = backup_path(output);
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(|source| Error::io(&backup, source))?;
    }
    if output.exists() {
        fs::rename(output, &backup).map_err(|source| Error::io(output, source))?;
    }
    let staging_path = staging.keep();
    if let Err(source) = fs::rename(&staging_path, output) {
        if backup.exists() {
            let _ = fs::rename(&backup, output);
        }
        let _ = fs::remove_dir_all(&staging_path);
        return Err(Error::io(output, source));
    }
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(|source| Error::io(&backup, source))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::backup_path;
    use std::path::Path;

    #[test]
    fn backup_path_keeps_version_suffix() {
        assert_eq!(
            backup_path(Path::new("oci-layouts/glibc-2.44")),
            Path::new("oci-layouts/glibc-2.44.bak")
        );
        assert_eq!(
            backup_path(Path::new("oci-layouts/glibc-2.45")),
            Path::new("oci-layouts/glibc-2.45.bak")
        );
        assert_ne!(
            backup_path(Path::new("oci-layouts/glibc-2.44")),
            backup_path(Path::new("oci-layouts/glibc-2.45"))
        );
    }
}
