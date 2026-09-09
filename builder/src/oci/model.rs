use serde::Serialize;

use crate::error::{Error, Result};
use crate::target::Target;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZstdLevel(u8);

pub const DEFAULT_ZSTD_LEVEL: u8 = 3;
pub const PUBLISH_ZSTD_LEVEL: u8 = 22;

impl ZstdLevel {
    pub const DEFAULT: Self = Self(DEFAULT_ZSTD_LEVEL);
    pub const PUBLISH: Self = Self(PUBLISH_ZSTD_LEVEL);

    pub fn new(level: u8) -> Result<Self> {
        if (1..=22).contains(&level) {
            Ok(Self(level))
        } else {
            Err(Error::message(format!(
                "zstd の圧縮レベルは 1〜22 です: {level}"
            )))
        }
    }

    pub fn get(self) -> u8 {
        self.0
    }

    pub fn needs_ultra(self) -> bool {
        self.0 >= 20
    }
}

pub fn parse_zstd_level(level: u8) -> Result<u8> {
    Ok(ZstdLevel::new(level)?.get())
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Digest {
    hex: String,
}

impl Digest {
    pub fn from_hex(hex: String) -> Self {
        Self { hex }
    }

    pub fn encoded(&self) -> &str {
        &self.hex
    }

    pub fn prefixed(&self) -> String {
        format!("sha256:{}", self.hex)
    }
}

impl Serialize for Digest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.prefixed())
    }
}

pub struct Image {
    pub name: String,
    pub version: String,
}

#[derive(Serialize)]
pub(super) struct OciLayout {
    #[serde(rename = "imageLayoutVersion")]
    pub image_layout_version: &'static str,
}

#[derive(Serialize)]
pub(super) struct Descriptor {
    #[serde(rename = "mediaType")]
    pub media_type: &'static str,
    pub digest: Digest,
    pub size: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformObject>,
}

#[derive(Serialize)]
pub(super) struct PlatformObject {
    pub architecture: &'static str,
    pub os: &'static str,
    pub variant: &'static str,
}

impl From<Target> for PlatformObject {
    fn from(target: Target) -> Self {
        Self {
            architecture: target.architecture(),
            os: "linux",
            variant: target.variant(),
        }
    }
}

#[derive(Serialize)]
pub(super) struct ImageIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: &'static str,
    pub manifests: Vec<Descriptor>,
}

#[derive(Serialize)]
pub(super) struct ImageManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: &'static str,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
    pub annotations: Annotations,
}

#[derive(Serialize)]
pub(super) struct Annotations {
    #[serde(rename = "org.opencontainers.image.title")]
    pub title: String,
    #[serde(rename = "org.opencontainers.image.version")]
    pub version: String,
}

#[derive(Serialize)]
pub(super) struct ImageConfig {
    pub created: String,
    pub architecture: &'static str,
    pub os: &'static str,
    pub variant: &'static str,
    pub config: EmptyObject,
    pub rootfs: RootFs,
}

#[derive(Serialize)]
pub(super) struct EmptyObject {}

#[derive(Serialize)]
pub(super) struct RootFs {
    #[serde(rename = "type")]
    pub type_: &'static str,
    pub diff_ids: Vec<Digest>,
}
