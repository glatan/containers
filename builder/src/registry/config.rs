//! 消費側の設定ファイル（`containers.json`）。
//!
//! ```json
//! {
//!   "schemaVersion": 1,
//!   "index": "ghcr.io/org/packages/index:20260923-0",
//!   "images": [
//!     {
//!       "name": "nginx",
//!       "dockerfile": "docker/nginx/Dockerfile",
//!       "platform": "amd64/v4",
//!       "mode": "block",
//!       "packages": ["openssl", "nginx"]
//!     }
//!   ]
//! }
//! ```
//!
//! `mode` は `block`（GENERATED ブロック＋COPY 連結）または
//! `compose`（合成ベースイメージ＋ARG pin）。`packages` は
//! トップレベル依存のみでよく、推移依存は index から解決する。

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::{Error, Result};
use crate::target::Target;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    /// index 参照。`--index` / `CONTAINERS_INDEX` より後、内蔵 pin より前。
    /// `"auto"` で `list-tags` による最新タグ解決。
    #[serde(default)]
    pub index: Option<String>,
    pub images: Vec<ImageSpec>,
}

#[derive(Debug, Deserialize)]
pub struct ImageSpec {
    /// イメージ識別子。mode B のリポジトリ名 `{registry}/images/{name}`
    /// と lockfile のキーに使う。
    pub name: String,
    /// 対象 Dockerfile。設定ファイルからの相対パス。
    pub dockerfile: PathBuf,
    /// `amd64/v4` 形式の解決 platform。
    pub platform: String,
    pub mode: Mode,
    /// トップレベル依存パッケージ名。
    #[serde(default)]
    pub packages: Vec<String>,
    /// mode B の push 先を明示的に上書きする場合のリポジトリ。
    #[serde(default)]
    pub repository: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Mode {
    /// GENERATED ブロックに pinned FROM/COPY を列挙する。
    Block,
    /// 合成ベースイメージを push し ARG で pin する。
    Compose,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let bytes = std::fs::read(path).map_err(|source| Error::io(path, source))?;
        let config: Self = serde_json::from_slice(&bytes).map_err(|source| {
            Error::message(format!("{} のパースに失敗: {source}", path.display()))
        })?;
        config.validate(path)?;
        Ok(config)
    }

    fn validate(&self, path: &Path) -> Result<()> {
        if self.schema_version != crate::registry::index::SCHEMA_VERSION {
            return Err(Error::message(format!(
                "schemaVersion {} は未対応です（{} を指定してください）",
                self.schema_version,
                crate::registry::index::SCHEMA_VERSION
            )));
        }
        if self.images.is_empty() {
            return Err(Error::message("images が空です"));
        }
        let base = path.parent().unwrap_or_else(|| Path::new("."));
        let mut names = BTreeSet::new();
        let mut dockerfiles = BTreeSet::new();
        for spec in &self.images {
            if !names.insert(spec.name.as_str()) {
                return Err(Error::message(format!(
                    "images の name が重複しています: {}",
                    spec.name
                )));
            }
            if Target::parse(&spec.platform).is_none() {
                return Err(Error::message(format!(
                    "{} の platform が不正です: {}",
                    spec.name, spec.platform
                )));
            }
            if spec.packages.is_empty() {
                return Err(Error::message(format!(
                    "{} の packages が空です",
                    spec.name
                )));
            }
            if spec.dockerfile.as_os_str().is_empty() {
                return Err(Error::message(format!(
                    "{} の dockerfile が空です",
                    spec.name
                )));
            }
            let dockerfile = spec.dockerfile_path(base);
            if !dockerfile.exists() {
                return Err(Error::message(format!(
                    "{} の dockerfile がありません: {}",
                    spec.name,
                    dockerfile.display()
                )));
            }
            // 同一 Dockerfile を複数エントリで共有すると、後のエントリが
            // 先のエントリの生成ブロックを上書きして消してしまう。
            if !dockerfiles.insert(dockerfile.clone()) {
                return Err(Error::message(format!(
                    "dockerfile が重複しています: {}",
                    dockerfile.display()
                )));
            }
        }
        Ok(())
    }
}

impl ImageSpec {
    pub fn target(&self) -> Target {
        Target::parse(&self.platform).expect("validate 済み")
    }

    /// 設定ファイルのあるディレクトリを基点にした Dockerfile パス。
    pub fn dockerfile_path(&self, base: &Path) -> PathBuf {
        if self.dockerfile.is_absolute() {
            self.dockerfile.clone()
        } else {
            base.join(&self.dockerfile)
        }
    }
}
