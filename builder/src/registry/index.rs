//! レジストリ index のスキーマと生成。
//!
//! 全パッケージの name→version→platform→manifest digest→依存関係を
//! 1つの JSON に集約し、OCI artifact として `oci-layouts/index/` に出力する。
//! 消費側は index を1回 pull すれば依存解決をローカルで完結できる。

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};
use crate::oci::{self, Digest, ImageIndex, ImageManifest};
use crate::package::{self, Package};
use crate::registry::layout_io;
use crate::target::Target;
use crate::util;
use crate::workspace;

pub const SCHEMA_VERSION: u32 = 1;
/// index artifact の artifactType（manifest と JSON blob の mediaType）。
pub const ARTIFACT_TYPE: &str = "application/vnd.containers.index.v1+json";
const MEDIA_EMPTY: &str = "application/vnd.oci.empty.v1+json";

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageIndex {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    /// `{registry}/packages`（消費側が org を推測しないよう自己記述）。
    pub registry: String,
    #[serde(default)]
    pub packages: Vec<Entry>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Entry {
    pub name: String,
    pub version: String,
    pub license: String,
    /// push 後に `{name}:{version}` タグが指す digest（ネスト index blob）。
    #[serde(rename = "indexDigest")]
    pub index_digest: Digest,
    pub platforms: Vec<PlatformEntry>,
    /// キーは `{arch}/{variant}`（その platform のイメージを引く消費側が
    /// 使う依存リスト）。値の platform は `artifact_target` 解決済みの
    /// 成果物 platform。
    #[serde(default)]
    pub dependencies: BTreeMap<String, Vec<Dependency>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlatformEntry {
    pub architecture: String,
    pub variant: String,
    #[serde(rename = "manifestDigest")]
    pub manifest_digest: Digest,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub name: String,
    pub version: String,
    pub platform: String,
}

impl PackageIndex {
    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.packages.iter().find(|entry| entry.name == name)
    }
}

impl Entry {
    pub fn manifest_for(&self, platform: &str) -> Option<&PlatformEntry> {
        self.platforms
            .iter()
            .find(|entry| format!("{}/{}", entry.architecture, entry.variant) == platform)
    }
}

/// パッケージの公開 layout の置き場。compose_root は `env compose` の
/// 合成出力（`current` symlink 経由）、それ以外は `pack` の出力。
pub fn layout_source(package: &dyn Package) -> Option<PathBuf> {
    if package.compose_root() {
        let link = workspace::compose_current_link(package.name(), workspace::BUILD_ENV_ARCH);
        return link.exists().then_some(link);
    }
    let dir = workspace::layout_dir(package.name(), package.version());
    dir.exists().then_some(dir)
}

/// `oci-layouts/` を走査して index JSON を構築する。
/// layout が存在する配布対象のみ収録し、依存は `artifact_target`
/// 解決済みの platform を持つ。
/// `all_issues` が真なら個別パッケージの失敗を集約して最後に報告する
/// （先頭の 1 件で止まらない）。
pub fn generate(registry: &str, all_issues: bool) -> Result<PackageIndex> {
    let mut packages = Vec::new();
    let mut skipped = Vec::new();
    let mut issues = Vec::new();
    for package in package::images() {
        let Some(dir) = layout_source(package) else {
            skipped.push(package.name());
            continue;
        };
        match collect_entry(package, &dir) {
            Ok(entry) => packages.push(entry),
            Err(error) => {
                if all_issues {
                    issues.push(error.to_string());
                } else {
                    return Err(error);
                }
            }
        }
    }
    let index = PackageIndex {
        schema_version: SCHEMA_VERSION,
        registry: registry.to_string(),
        packages,
    };
    issues.extend(validate(&index));
    if !issues.is_empty() {
        for issue in &issues {
            eprintln!("問題: {issue}");
        }
        return Err(Error::message(format!(
            "{} 件の問題があります",
            issues.len()
        )));
    }
    for name in &skipped {
        eprintln!("スキップ(layout なし): {name}");
    }
    Ok(index)
}

/// 1 パッケージ分の index エントリを組み立てる。
/// layout の読み取りは output lock の read 取得下で行い、並走する
/// pack/compose の rename 窓で偽の破損を検出しないようにする。
fn collect_entry(package: &dyn Package, dir: &Path) -> Result<Entry> {
    let lock = workspace::output_lock(dir)?;
    let published = {
        let _guard = lock.read().map_err(|source| Error::io(dir, source))?;
        layout_io::published(dir).map_err(|error| {
            Error::message(format!(
                "{} の layout が読めません: {error}",
                package.name()
            ))
        })?
    };
    let platforms = published
        .manifests
        .iter()
        .map(|manifest| PlatformEntry {
            architecture: manifest.target.architecture().to_string(),
            variant: manifest.target.variant().to_string(),
            manifest_digest: manifest.digest.clone(),
            size: manifest.size,
        })
        .collect::<Vec<_>>();
    let mut dependencies = BTreeMap::new();
    if !package.compose_root() {
        for manifest in &published.manifests {
            let key = format!(
                "{}/{}",
                manifest.target.architecture(),
                manifest.target.variant()
            );
            let deps = resolve_dependencies(package, manifest.target)?;
            dependencies.insert(key, deps);
        }
    }
    Ok(Entry {
        name: package.name().to_string(),
        version: package.version().to_string(),
        license: package.license().to_string(),
        index_digest: published.index_digest,
        platforms,
        dependencies,
    })
}

/// 依存を `{name, version, 解決済み platform}` に写像する。
fn resolve_dependencies(package: &dyn Package, target: Target) -> Result<Vec<Dependency>> {
    let mut dependencies = Vec::new();
    for dep in package.dependencies(target) {
        let found = package::find(dep.name)?;
        if found.version() != dep.version {
            return Err(Error::message(format!(
                "依存のバージョンが一致しません: {} は {} ですが定義は {}",
                dep.name,
                dep.version,
                found.version()
            )));
        }
        let artifact = package::build::artifact_target(found, target);
        dependencies.push(Dependency {
            name: dep.name.to_string(),
            version: dep.version.to_string(),
            platform: format!("{}/{}", artifact.architecture(), artifact.variant()),
        });
    }
    Ok(dependencies)
}

/// 依存辺が index 内で解決可能かを検査し、問題を全件返す。
/// - 依存名が index に存在し version が一致すること
/// - 依存先エントリが解決済み platform の manifest を持つこと
fn validate(index: &PackageIndex) -> Vec<String> {
    let mut issues = Vec::new();
    for entry in &index.packages {
        for (key, deps) in &entry.dependencies {
            for dep in deps {
                let Some(found) = index.find(&dep.name) else {
                    issues.push(format!(
                        "{} ({key}) の依存 {} が index にありません（pack してください）",
                        entry.name, dep.name
                    ));
                    continue;
                };
                if found.version != dep.version {
                    issues.push(format!(
                        "{} ({key}) の依存 {} のバージョンが一致しません: {} vs {}",
                        entry.name, dep.name, dep.version, found.version
                    ));
                }
                if found.manifest_for(&dep.platform).is_none() {
                    issues.push(format!(
                        "{} ({key}) の依存 {} に platform {} の manifest がありません（pack してください）",
                        entry.name, dep.name, dep.platform
                    ));
                }
            }
        }
    }
    issues
}

/// index JSON を OCI artifact として `output`（`oci-layouts/index/`）に書く。
/// 返り値は artifact manifest の digest（push 後にタグが指す値）。
/// output 単位の flock で pack/compose 等の layout 書き換えと直列化する。
pub fn write_artifact(index: &PackageIndex, output: &Path, tag: &str) -> Result<Digest> {
    let mut lock = workspace::output_lock(output)?;
    let _guard = lock.write().map_err(|source| Error::io(output, source))?;
    let parent = output.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let staging = tempfile::TempDir::new_in(parent).map_err(|source| Error::io(parent, source))?;
    let blobs = staging.path().join("blobs/sha256");
    std::fs::create_dir_all(&blobs).map_err(|source| Error::io(&blobs, source))?;

    let (json_digest, json_size) = oci::store_blob(&blobs, &serde_json::to_vec_pretty(index)?)?;
    let (config_digest, config_size) = oci::store_blob(&blobs, b"{}")?;
    let manifest = ImageManifest {
        schema_version: 2,
        media_type: oci::MEDIA_MANIFEST.to_string(),
        artifact_type: Some(ARTIFACT_TYPE.to_string()),
        config: oci::Descriptor {
            media_type: MEDIA_EMPTY.to_string(),
            digest: config_digest,
            size: config_size,
            platform: None,
            annotations: None,
        },
        layers: vec![oci::Descriptor {
            media_type: ARTIFACT_TYPE.to_string(),
            digest: json_digest,
            size: json_size,
            platform: None,
            annotations: None,
        }],
        annotations: {
            let mut annotations = BTreeMap::from([
                (
                    "org.opencontainers.image.title".to_string(),
                    "containers package index".to_string(),
                ),
                (
                    "org.opencontainers.image.version".to_string(),
                    tag.to_string(),
                ),
                (
                    "org.opencontainers.image.description".to_string(),
                    "containers package index".to_string(),
                ),
                (
                    "org.opencontainers.image.licenses".to_string(),
                    "AGPL-3.0-only".to_string(),
                ),
                (
                    "org.opencontainers.image.source".to_string(),
                    oci::SOURCE_URL.to_string(),
                ),
            ]);
            if let Some(revision) = util::revision() {
                annotations.insert("org.opencontainers.image.revision".to_string(), revision);
            }
            annotations
        },
    };
    let (manifest_digest, manifest_size) =
        oci::store_blob(&blobs, &serde_json::to_vec(&manifest)?)?;
    let top = ImageIndex {
        schema_version: 2,
        media_type: oci::MEDIA_INDEX.to_string(),
        manifests: vec![oci::Descriptor {
            media_type: oci::MEDIA_MANIFEST.to_string(),
            digest: manifest_digest.clone(),
            size: manifest_size,
            platform: Some(oci::PlatformObject {
                architecture: "unknown".to_string(),
                os: "unknown".to_string(),
                variant: None,
            }),
            annotations: Some(BTreeMap::from([(
                oci::ANNOTATION_REF_NAME.to_string(),
                tag.to_string(),
            )])),
        }],
        annotations: None,
    };
    util::write(
        &staging.path().join("index.json"),
        serde_json::to_vec(&top)?,
    )?;
    util::write(
        &staging.path().join("oci-layout"),
        serde_json::to_vec(&oci::OciLayout {
            image_layout_version: "1.0.0".to_string(),
        })?,
    )?;
    oci::replace_dir(output, staging)?;
    Ok(manifest_digest)
}

/// 生成済み artifact から index JSON を取り出す（検査・消費側共用）。
pub fn read_artifact(layout: &Path) -> Result<PackageIndex> {
    let manifest = layout_io::pulled_manifest(layout)?;
    let Some(layer) = manifest.layers.first() else {
        return Err(Error::message(format!(
            "{} の artifact に blob がありません",
            layout.display()
        )));
    };
    let index: PackageIndex = layout_io::read_blob(layout, &layer.digest)?;
    if index.schema_version != SCHEMA_VERSION {
        return Err(Error::message(format!(
            "index の schemaVersion {} は未対応です（{} が必要です）",
            index.schema_version, SCHEMA_VERSION
        )));
    }
    Ok(index)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_name_is_reserved() {
        // packages/index リポジトリとの衝突を避けるため、
        // `index` というパッケージ名は PACKAGES に存在してはならない。
        assert!(
            !package::PACKAGES
                .iter()
                .any(|package| package.name() == crate::registry::INDEX_REPOSITORY),
            "`index` は予約名です"
        );
    }

    #[test]
    fn schema_round_trip() {
        let index = PackageIndex {
            schema_version: SCHEMA_VERSION,
            registry: "ghcr.io/example".to_string(),
            packages: vec![Entry {
                name: "zlib".to_string(),
                version: "1.3.2".to_string(),
                license: "Zlib".to_string(),
                index_digest: Digest::from_hex("a".repeat(64)),
                platforms: vec![PlatformEntry {
                    architecture: "amd64".to_string(),
                    variant: "v4".to_string(),
                    manifest_digest: Digest::from_hex("b".repeat(64)),
                    size: 508,
                }],
                dependencies: BTreeMap::from([(
                    "amd64/v4".to_string(),
                    vec![Dependency {
                        name: "x86_64-linux-gnu-glibc".to_string(),
                        version: "2.44".to_string(),
                        platform: "amd64/v4".to_string(),
                    }],
                )]),
            }],
        };
        let json = serde_json::to_string(&index).unwrap();
        let parsed: PackageIndex = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.schema_version, SCHEMA_VERSION);
        assert_eq!(parsed.packages[0].name, "zlib");
        assert_eq!(
            parsed.packages[0].platforms[0].manifest_digest,
            Digest::from_hex("b".repeat(64))
        );
        assert_eq!(
            parsed.packages[0].dependencies["amd64/v4"][0].platform,
            "amd64/v4"
        );
    }

    #[test]
    fn write_artifact_produces_pullable_layout() {
        let tmp = tempfile::tempdir().unwrap();
        let index = PackageIndex {
            schema_version: SCHEMA_VERSION,
            registry: "ghcr.io/example".to_string(),
            packages: vec![],
        };
        let out = tmp.path().join("index");
        let digest = write_artifact(&index, &out, "20260923-0").unwrap();
        // index.json → artifact manifest → JSON blob の順に辿れる。
        let parsed = read_artifact(&out).unwrap();
        assert_eq!(parsed.registry, "ghcr.io/example");
        let manifest = layout_io::pulled_manifest(&out).unwrap();
        assert_eq!(manifest.artifact_type.as_deref(), Some(ARTIFACT_TYPE));
        assert_eq!(
            manifest.config.media_type,
            "application/vnd.oci.empty.v1+json"
        );
        assert!(digest.prefixed().starts_with("sha256:"));
        assert_eq!(
            manifest.annotations["org.opencontainers.image.source"],
            crate::oci::SOURCE_URL
        );
        assert_eq!(
            manifest.annotations["org.opencontainers.image.description"],
            "containers package index"
        );
        assert_eq!(
            manifest.annotations["org.opencontainers.image.licenses"],
            "AGPL-3.0-only"
        );
    }
}
