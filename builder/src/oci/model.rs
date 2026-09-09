use serde::Serialize;

use crate::error::{Error, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Platform {
    pub architecture: &'static str,
    pub variant: &'static str,
}

const fn platform(architecture: &'static str, variant: &'static str) -> Platform {
    Platform {
        architecture,
        variant,
    }
}

pub const TARGETS: &[Platform] = &[
    platform("arm64", "v8"),
    platform("arm64", "v8.1"),
    platform("arm64", "v8.2"),
    platform("arm64", "v8.3"),
    platform("arm64", "v8.4"),
    platform("arm64", "v8.5"),
    platform("arm64", "v8.6"),
    platform("arm64", "v8.7"),
    platform("arm64", "v8.8"),
    platform("arm64", "v8.9"),
    platform("arm64", "v9"),
    platform("arm64", "v9.1"),
    platform("arm64", "v9.2"),
    platform("arm64", "v9.3"),
    platform("arm64", "v9.4"),
    platform("arm64", "v9.5"),
    platform("ppc64le", "power8"),
    platform("ppc64le", "power9"),
    platform("ppc64le", "power10"),
    platform("ppc64le", "power11"),
    platform("riscv64", "rva20u64"),
    platform("riscv64", "rva22u64"),
    platform("riscv64", "rva23u64"),
    platform("amd64", "v1"),
    platform("amd64", "v2"),
    platform("amd64", "v3"),
    platform("amd64", "v4"),
    platform("s390x", "z900"),
    platform("s390x", "z990"),
    platform("s390x", "z9-ec"),
    platform("s390x", "z10"),
    platform("s390x", "z196"),
    platform("s390x", "zEC12"),
    platform("s390x", "z13"),
    platform("s390x", "z14"),
    platform("s390x", "z15"),
    platform("s390x", "z16"),
    platform("s390x", "z17"),
];

impl Platform {
    pub fn cflags(self) -> Result<String> {
        let march = match (self.architecture, self.variant) {
            ("amd64", "v1") => "x86-64",
            ("amd64", "v2") => "x86-64-v2",
            ("amd64", "v3") => "x86-64-v3",
            ("amd64", "v4") => "x86-64-v4",
            ("arm64", "v8") => "armv8-a",
            _ => {
                return Err(Error::message(format!(
                    "未対応のターゲットです: {}/{}",
                    self.architecture, self.variant
                )));
            }
        };
        Ok(format!("-O2 -march={march}"))
    }

    pub fn triple(self) -> Result<&'static str> {
        match self.architecture {
            "amd64" => Ok("x86_64-pc-linux-gnu"),
            "arm64" => Ok("aarch64-linux-gnu"),
            _ => Err(Error::message(format!(
                "未対応のターゲットです: {}",
                self.architecture
            ))),
        }
    }

    pub fn cross_prefix(self) -> Result<&'static str> {
        match self.architecture {
            "amd64" => Ok(""),
            "arm64" => Ok("aarch64-linux-gnu-"),
            _ => Err(Error::message(format!(
                "未対応のターゲットです: {}",
                self.architecture
            ))),
        }
    }
}

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

pub fn resolve_zstd_level(publish: bool, zstd_level: Option<u8>) -> Result<u8> {
    Ok(match (publish, zstd_level) {
        (true, None) => ZstdLevel::PUBLISH,
        (false, None) => ZstdLevel::DEFAULT,
        (true, Some(level)) if level != PUBLISH_ZSTD_LEVEL => {
            return Err(Error::message(
                "--publish の圧縮レベルは 22 です。--zstd-level と同時に指定しないでください",
            ));
        }
        (_, Some(level)) => ZstdLevel::new(level)?,
    }
    .get())
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

impl From<Platform> for PlatformObject {
    fn from(platform: Platform) -> Self {
        Self {
            architecture: platform.architecture,
            os: "linux",
            variant: platform.variant,
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
