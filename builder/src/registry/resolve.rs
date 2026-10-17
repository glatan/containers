//! index に基づく依存閉包の解決。
//!
//! `image::visit_runtime` と同じ深さ優先（依存が先、利用側が後）で、
//! 依存エントリの `platform`（`artifact_target` 解決済み）に従って
//! 辿る。同一パッケージが異なる platform で複数回現れ得るため、
//! 訪問キーは `(name, platform)`。

use std::collections::BTreeSet;

use crate::error::{Error, Result};
use crate::oci::Digest;
use crate::registry::index::PackageIndex;
use crate::registry::package_repo;
use crate::target::Target;

/// 解決済みの1パッケージ（1 platform 分）。
#[derive(Debug, Clone)]
pub struct Resolved {
    pub name: String,
    pub version: String,
    /// 成果物 platform の `arch/variant`。
    pub platform: String,
    /// push された manifest の digest（platform 別 manifest を指す）。
    pub manifest_digest: Digest,
    /// `{registry}/packages/{name}@sha256:{manifest}` 形式の pin 参照。
    /// Dockerfile / lockfile にそのまま書ける裸の参照（スキームなし）。
    /// skopeo に渡すときは `docker://` を前置する。
    pub reference: String,
    /// 同じ name が複数 platform で現れるときに stage 名を一意にする suffix。
    pub unique: bool,
}

/// `roots`（トップレル依存名）から推移閉包を解決する。
/// 返り値は依存→利用の順（COPY / 層の順序として使う）。
pub fn closure(index: &PackageIndex, roots: &[String], platform: Target) -> Result<Vec<Resolved>> {
    let platform_key = format!("{}/{}", platform.architecture(), platform.variant());
    let mut order = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for name in roots {
        visit(
            index,
            name,
            &platform_key,
            &mut visiting,
            &mut visited,
            &mut order,
        )?;
    }
    // 同名の複数 platform があるかを判定して unique フラグを立てる。
    let mut counts: std::collections::BTreeMap<String, u32> = Default::default();
    for resolved in &order {
        *counts.entry(resolved.name.clone()).or_default() += 1;
    }
    for resolved in &mut order {
        resolved.unique = counts[&resolved.name] > 1;
    }
    Ok(order)
}

fn visit(
    index: &PackageIndex,
    name: &str,
    platform_key: &str,
    visiting: &mut BTreeSet<(String, String)>,
    visited: &mut BTreeSet<(String, String)>,
    order: &mut Vec<Resolved>,
) -> Result<()> {
    let key = (name.to_string(), platform_key.to_string());
    if visited.contains(&key) {
        return Ok(());
    }
    if !visiting.insert(key.clone()) {
        return Err(Error::message(format!("依存が循環しています: {name}")));
    }
    let entry = index
        .find(name)
        .ok_or_else(|| Error::message(format!("{name} が index にありません")))?;
    let platform_entry = entry.manifest_for(platform_key).ok_or_else(|| {
        Error::message(format!(
            "{name} に platform {platform_key} の manifest がありません"
        ))
    })?;
    for dep in entry
        .dependencies
        .get(platform_key)
        .map(Vec::as_slice)
        .unwrap_or_default()
    {
        visit(index, &dep.name, &dep.platform, visiting, visited, order)?;
    }
    visiting.remove(&key);
    visited.insert(key);
    order.push(Resolved {
        name: name.to_string(),
        version: entry.version.clone(),
        platform: platform_key.to_string(),
        manifest_digest: platform_entry.manifest_digest.clone(),
        reference: format!(
            "{}@{}",
            package_repo(&index.registry, name),
            platform_entry.manifest_digest.prefixed()
        ),
        unique: false,
    });
    Ok(())
}

/// Dockerfile stage 名。同一パッケージが複数 platform で現れるときだけ
/// platform を suffix にする。
pub fn stage_name(resolved: &Resolved) -> String {
    if resolved.unique {
        format!(
            "pkg-{}-{}",
            resolved.name,
            resolved.platform.replace('/', "-")
        )
    } else {
        format!("pkg-{}", resolved.name)
    }
}

/// `--platform=linux/{arch}/{variant}`。containerd/platforms が解釈できる
/// arch/variant のみ付ける（ppc64le/s390x 等は pin された digest が真値）。
pub fn docker_platform(platform: &str) -> String {
    let Some(target) = Target::parse(platform) else {
        return String::new();
    };
    let variant = target.variant();
    let supported = match target.architecture() {
        "amd64" => matches!(variant, "v1" | "v2" | "v3" | "v4"),
        // containerd/platforms は arm64 の v8/v9 のみ正規化できる
        "arm64" => matches!(variant, "v8" | "v9"),
        _ => false,
    };
    if supported {
        format!("linux/{}/{variant}", target.architecture())
    } else {
        String::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::index::{Dependency, Entry, PlatformEntry};
    use std::collections::BTreeMap;

    fn digest(byte: char) -> Digest {
        Digest::from_hex(byte.to_string().repeat(64))
    }

    fn platform(arch: &str, variant: &str, byte: char) -> PlatformEntry {
        PlatformEntry {
            architecture: arch.to_string(),
            variant: variant.to_string(),
            manifest_digest: digest(byte),
            size: 100,
        }
    }

    fn entry(
        name: &str,
        platforms: Vec<PlatformEntry>,
        dependencies: BTreeMap<String, Vec<Dependency>>,
    ) -> Entry {
        Entry {
            name: name.to_string(),
            version: "1.0".to_string(),
            license: "MIT".to_string(),
            index_digest: digest('x'),
            platforms,
            dependencies,
        }
    }

    fn dep(name: &str, platform: &str) -> Dependency {
        Dependency {
            name: name.to_string(),
            version: "1.0".to_string(),
            platform: platform.to_string(),
        }
    }

    fn index(packages: Vec<Entry>) -> PackageIndex {
        PackageIndex {
            schema_version: 1,
            registry: "ghcr.io/example".to_string(),
            packages,
        }
    }

    #[test]
    fn closure_is_dependencies_first() {
        let idx = index(vec![
            entry("glibc", vec![platform("amd64", "v4", 'a')], BTreeMap::new()),
            entry(
                "openssl",
                vec![platform("amd64", "v4", 'b')],
                BTreeMap::from([("amd64/v4".to_string(), vec![dep("glibc", "amd64/v4")])]),
            ),
            entry(
                "nginx",
                vec![platform("amd64", "v4", 'c')],
                BTreeMap::from([(
                    "amd64/v4".to_string(),
                    vec![dep("openssl", "amd64/v4"), dep("glibc", "amd64/v4")],
                )]),
            ),
        ]);
        let roots = vec!["nginx".to_string()];
        let resolved = closure(&idx, &roots, Target::BUILD_ENV).unwrap();
        let names: Vec<_> = resolved.iter().map(|r| r.name.as_str()).collect();
        assert_eq!(names, ["glibc", "openssl", "nginx"]);
        // digest pin 参照が自己記述の registry を使うこと（裸参照）。
        assert_eq!(
            resolved[0].reference,
            format!("ghcr.io/example/packages/glibc@{}", digest('a').prefixed())
        );
    }

    #[test]
    fn closure_uses_resolved_dep_platform() {
        // arm64 depender が amd64 成果物の依存を持つ場合、
        // dep.platform（artifact_target 解決済み）で辿る。
        let idx = index(vec![
            entry(
                "aarch64-linux-gnu-gcc",
                vec![platform("amd64", "v4", 'a')],
                BTreeMap::new(),
            ),
            entry(
                "openssl",
                vec![platform("arm64", "v8", 'b')],
                BTreeMap::from([(
                    "arm64/v8".to_string(),
                    vec![dep("aarch64-linux-gnu-gcc", "amd64/v4")],
                )]),
            ),
        ]);
        let roots = vec!["openssl".to_string()];
        let arm64v8 = Target::lookup("arm64", "v8").unwrap();
        let resolved = closure(&idx, &roots, arm64v8).unwrap();
        let names: Vec<_> = resolved
            .iter()
            .map(|r| (r.name.as_str(), r.platform.as_str()))
            .collect();
        assert_eq!(
            names,
            [
                ("aarch64-linux-gnu-gcc", "amd64/v4"),
                ("openssl", "arm64/v8")
            ]
        );
    }

    #[test]
    fn closure_rejects_missing_platform() {
        let idx = index(vec![entry(
            "openssl",
            vec![platform("arm64", "v8", 'b')],
            BTreeMap::new(),
        )]);
        let roots = vec!["openssl".to_string()];
        let error = closure(&idx, &roots, Target::BUILD_ENV).unwrap_err();
        assert!(error.to_string().contains("amd64/v4"));
    }

    #[test]
    fn closure_rejects_cycle() {
        let idx = index(vec![
            entry(
                "a",
                vec![platform("amd64", "v4", 'a')],
                BTreeMap::from([("amd64/v4".to_string(), vec![dep("b", "amd64/v4")])]),
            ),
            entry(
                "b",
                vec![platform("amd64", "v4", 'b')],
                BTreeMap::from([("amd64/v4".to_string(), vec![dep("a", "amd64/v4")])]),
            ),
        ]);
        let roots = vec!["a".to_string()];
        let error = closure(&idx, &roots, Target::BUILD_ENV).unwrap_err();
        assert!(error.to_string().contains("循環"));
    }

    #[test]
    fn docker_platform_only_for_containerd_variants() {
        assert_eq!(docker_platform("amd64/v4"), "linux/amd64/v4");
        assert_eq!(docker_platform("arm64/v8"), "linux/arm64/v8");
        assert_eq!(docker_platform("arm64/v8.5"), "");
        assert_eq!(docker_platform("ppc64le/power9"), "");
        assert_eq!(docker_platform("s390x/zEC12"), "");
        assert_eq!(docker_platform("riscv64/rva23u64"), "");
    }
}
