//! 消費側のロックファイル（`containers.lock.json`）。
//!
//! 設定ファイルと同じディレクトリに置き、解決結果（パッケージ・
//! platform・manifest digest・pin 参照）と mode B の合成イメージ pin を
//! 記録する。`sync --check` は再解決結果とこのファイルを比較する。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::oci::Digest;

pub const LOCKFILE_NAME: &str = "containers.lock.json";
/// lockfile 独自のスキーマ版（index の schemaVersion とは独立）。
pub const LOCKFILE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Lockfile {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    /// 解決に使った index 参照（`{repo}@sha256:{digest}`）。
    pub index: String,
    #[serde(default)]
    pub images: BTreeMap<String, LockEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockEntry {
    pub platform: String,
    /// "block" | "compose"
    pub mode: String,
    pub resolved: Vec<Locked>,
    /// mode B の合成イメージ pin 参照（`{repo}@sha256:{digest}`）。
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composed: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Locked {
    pub name: String,
    pub version: String,
    /// 成果物 platform の `arch/variant`。
    pub platform: String,
    #[serde(rename = "manifestDigest")]
    pub manifest_digest: Digest,
    /// `{registry}/packages/{name}@sha256:{manifest}`。
    pub reference: String,
}

pub fn lockfile_path(config: &Path) -> PathBuf {
    config
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join(LOCKFILE_NAME)
}

pub fn load(path: &Path) -> Result<Option<Lockfile>> {
    if !path.exists() {
        return Ok(None);
    }
    let bytes = std::fs::read(path).map_err(|source| Error::io(path, source))?;
    let lockfile: Lockfile = serde_json::from_slice(&bytes).map_err(|source| {
        Error::message(format!("{} のパースに失敗: {source}", path.display()))
    })?;
    Ok(Some(lockfile))
}

pub fn write(path: &Path, lockfile: &Lockfile) -> Result<()> {
    crate::util::write(path, serde_json::to_vec_pretty(lockfile)?)?;
    Ok(())
}
