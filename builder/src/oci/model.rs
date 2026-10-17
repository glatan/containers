use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

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

    pub const fn get(self) -> u8 {
        self.0
    }

    pub const fn needs_ultra(self) -> bool {
        self.0 >= 20
    }
}

pub fn parse_zstd_level(level: u8) -> Result<u8> {
    Ok(ZstdLevel::new(level)?.get())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Digest {
    hex: String,
}

impl Digest {
    pub const fn from_hex(hex: String) -> Self {
        Self { hex }
    }

    /// `sha256:{64桁の小文字hex}` をパースする。
    pub fn parse(text: &str) -> Result<Self> {
        let valid = text.strip_prefix("sha256:").is_some_and(Self::valid_hex);
        if valid {
            Ok(Self {
                hex: text[7..].to_string(),
            })
        } else {
            Err(Error::message(format!("digest の形式が不正です: {text}")))
        }
    }

    fn valid_hex(hex: &str) -> bool {
        hex.len() == 64
            && hex
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
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

impl<'de> Deserialize<'de> for Digest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone)]
pub struct Image {
    pub name: String,
    pub version: String,
    /// `org.opencontainers.image.description`。GHCR のパッケージページ
    /// に表示される説明文。省略可能。
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct OciLayout {
    #[serde(rename = "imageLayoutVersion")]
    pub image_layout_version: String,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Descriptor {
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub digest: Digest,
    pub size: u64,
    // containers/image が push 時に付与する descriptor アノテーションの
    // シリアライズ順に合わせるため platform より先に置く
    // （index blob とリモート manifest list の digest 一致が前提）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformObject>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct PlatformObject {
    pub architecture: String,
    pub os: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
}

impl From<Target> for PlatformObject {
    fn from(target: Target) -> Self {
        Self {
            architecture: target.architecture().to_string(),
            os: "linux".to_string(),
            variant: Some(target.variant().to_string()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ImageIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    pub manifests: Vec<Descriptor>,
    // GHCR は multi-arch イメージの description 等を index の
    // annotations から読む。Go の Index marshal 順（artifactType/subject
    // は omitempty で欠落）に合わせ manifests の後に置く。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub annotations: Option<BTreeMap<String, String>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ImageManifest {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    #[serde(rename = "mediaType")]
    pub media_type: String,
    #[serde(
        rename = "artifactType",
        default,
        skip_serializing_if = "Option::is_none"
    )]
    pub artifact_type: Option<String>,
    pub config: Descriptor,
    pub layers: Vec<Descriptor>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub annotations: BTreeMap<String, String>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct ImageConfig {
    pub created: String,
    pub architecture: String,
    pub os: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,
    // skopeo pull した外部 config は config フィールドを持たない
    // 場合があるため欠落を許容する（platform 復元には不要）。
    #[serde(default)]
    pub config: ExecConfig,
    pub rootfs: RootFs,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecConfig {
    // 書き出しでは空を省略し、読み取りでは欠落を空として許容する
    // （pack が書いた config を restore/merge で読み戻すため）。
    #[serde(rename = "Env", default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,
    #[serde(rename = "Cmd", default, skip_serializing_if = "Vec::is_empty")]
    pub cmd: Vec<String>,
    #[serde(rename = "WorkingDir", skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
}

impl ExecConfig {
    pub fn runnable() -> Self {
        Self {
            env: vec![
                "PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin".to_string(),
                "LD_LIBRARY_PATH=/usr/lib64:/usr/lib:/lib64:/lib".to_string(),
            ],
            cmd: vec!["/usr/bin/bash".to_string()],
            working_dir: Some("/".to_string()),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub(crate) struct RootFs {
    #[serde(rename = "type")]
    pub type_: String,
    pub diff_ids: Vec<Digest>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn digest_parse_round_trip() {
        let hex = "a".repeat(64);
        let digest = Digest::parse(&format!("sha256:{hex}")).unwrap();
        assert_eq!(digest.encoded(), hex);
        assert_eq!(digest.prefixed(), format!("sha256:{}", "a".repeat(64)));
        for bad in [
            "sha256:abc",
            "md5:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
            "sha256:",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            assert!(Digest::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn digest_deserialize_validates_prefix() {
        let good = format!("\"sha256:{}\"", "b".repeat(64));
        let digest: Digest = serde_json::from_str(&good).unwrap();
        assert_eq!(digest.encoded(), "b".repeat(64));
        assert!(serde_json::from_str::<Digest>("\"sha256:zz\"").is_err());
    }
}
