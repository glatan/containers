use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::oci::layer;
use crate::oci::model::{
    Annotations, Descriptor, Digest, EmptyObject, Image, ImageConfig, ImageIndex, ImageManifest,
    OciLayout, Platform, PlatformObject, RootFs, ZstdLevel,
};
use crate::util::{self, timestamp};

const MEDIA_INDEX: &str = "application/vnd.oci.image.index.v1+json";
const MEDIA_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
const MEDIA_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
const MEDIA_LAYER: &str = "application/vnd.oci.image.layer.v1.tar+zstd";

pub fn build(
    output: &Path,
    image: &Image,
    layers: &[(Platform, Vec<PathBuf>)],
    zstd_level: u8,
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
    for (platform, layer_dirs) in layers {
        manifests.push(build_manifest(
            &blobs, image, *platform, layer_dirs, zstd_level,
        )?);
    }
    let index = ImageIndex {
        schema_version: 2,
        media_type: MEDIA_INDEX,
        manifests,
    };
    util::write(
        &staging.path().join("index.json"),
        serde_json::to_vec(&index)?,
    )?;
    util::write(
        &staging.path().join("oci-layout"),
        serde_json::to_vec(&OciLayout {
            image_layout_version: "1.0.0",
        })?,
    )?;
    replace_dir(output, staging)
}

fn build_manifest(
    blobs: &Path,
    image: &Image,
    platform: Platform,
    layer_dirs: &[PathBuf],
    zstd_level: ZstdLevel,
) -> Result<Descriptor> {
    let mut layers = Vec::new();
    let mut diff_ids = Vec::new();
    for dir in layer_dirs {
        let layer = layer::build(blobs, dir, zstd_level)?;
        diff_ids.push(layer.diff_id);
        layers.push(Descriptor {
            media_type: MEDIA_LAYER,
            digest: layer.blob,
            size: layer.size,
            platform: None,
        });
    }
    let config = ImageConfig {
        created: timestamp(),
        architecture: platform.architecture,
        os: "linux",
        variant: platform.variant,
        config: EmptyObject {},
        rootfs: RootFs {
            type_: "layers",
            diff_ids,
        },
    };
    let (config_digest, config_size) = store_blob(blobs, &serde_json::to_vec(&config)?)?;
    let manifest = ImageManifest {
        schema_version: 2,
        media_type: MEDIA_MANIFEST,
        config: Descriptor {
            media_type: MEDIA_CONFIG,
            digest: config_digest,
            size: config_size,
            platform: None,
        },
        layers,
        annotations: Annotations {
            title: image.name.clone(),
            version: image.version.clone(),
        },
    };
    let (digest, size) = store_blob(blobs, &serde_json::to_vec(&manifest)?)?;
    Ok(Descriptor {
        media_type: MEDIA_MANIFEST,
        digest,
        size,
        platform: Some(PlatformObject::from(platform)),
    })
}

fn store_blob(blobs: &Path, data: &[u8]) -> Result<(Digest, u64)> {
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

fn replace_dir(output: &Path, staging: tempfile::TempDir) -> Result<()> {
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
