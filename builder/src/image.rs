use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::oci::{self, ExecConfig};
use crate::package::{self, Package};
use crate::sbom;
use crate::target::Target;
use crate::workspace;

pub const COMPOSE_PLATFORM: Target = Target::BUILD_ENV;

pub fn build(package: &dyn Package, zstd_level: u8) -> Result<(PathBuf, Vec<String>)> {
    let mut layers = Vec::new();
    let mut skipped = Vec::new();
    for target in Target::all() {
        if !package.supports(target) {
            continue;
        }
        let dir = package.artifact_dir(target);
        if workspace::artifact_ready(&dir) {
            layers.push((target, vec![dir]));
        } else {
            skipped.push(format!("スキップ(パッケージなし): {}", dir.display()));
        }
    }
    if layers.is_empty() {
        return Err(Error::message(format!(
            "パッケージがありません: {} (package build を実行してください)",
            package.name()
        )));
    }
    let output = workspace::layout_dir(package.name(), package.version());
    let image = oci::Image {
        name: package.name().to_string(),
        version: package.version().to_string(),
    };
    oci::build(&output, &image, &layers, zstd_level)?;
    crate::util::write(&output.join("sbom.spdx.json"), &sbom::json(package)?)?;
    Ok((output, skipped))
}

pub fn compose(
    root: &dyn Package,
    target: Target,
    zstd_level: u8,
) -> Result<(PathBuf, String, Vec<String>)> {
    let packages = compose_packages(root, target)?;
    let mut layers = compose_layer_dirs(&packages, target)?;
    let usrmerge =
        tempfile::tempdir().map_err(|source| Error::io(Path::new("usrmerge"), source))?;
    write_usrmerge(usrmerge.path())?;
    layers.push(("usrmerge", usrmerge.path().to_path_buf()));
    check_path_conflicts(&layers)?;
    let date = crate::util::utc_date();
    let arch = target.architecture();
    let revision = workspace::next_compose_revision(root.name(), arch, &date);
    let tag = format!("{}-{}:{}-{}", root.name(), arch, date, revision);
    let output = workspace::compose_layout_dir(root.name(), arch, &date, revision);
    let image = oci::Image {
        name: format!("{}-{arch}", root.name()),
        version: format!("{date}-{revision}"),
    };
    let dirs: Vec<PathBuf> = layers.iter().map(|(_, dir)| dir.clone()).collect();
    oci::build_with_config(
        &output,
        &image,
        &[(target, dirs)],
        zstd_level,
        ExecConfig::runnable(),
    )?;
    crate::util::write(&output.join("sbom.spdx.json"), &sbom::json_many(&packages)?)?;
    let current = workspace::compose_current_link(root.name(), arch);
    if current.exists() {
        fs::remove_file(&current)
            .or_else(|_| fs::remove_dir_all(&current))
            .map_err(|source| Error::io(&current, source))?;
    }
    if let Some(parent) = current.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let file_name = output
        .file_name()
        .ok_or_else(|| Error::message("合成 Layout のパスが不正です"))?;
    symlink(file_name, &current).map_err(|source| Error::io(&current, source))?;
    drop(usrmerge);
    let skipped = Vec::new();
    Ok((output, tag, skipped))
}

fn write_usrmerge(dir: &Path) -> Result<()> {
    symlink("usr/bin", dir.join("bin")).map_err(|source| Error::io(dir, source))?;
    symlink("usr/sbin", dir.join("sbin")).map_err(|source| Error::io(dir, source))?;
    Ok(())
}

pub fn compose_packages(root: &dyn Package, target: Target) -> Result<Vec<&'static dyn Package>> {
    let root = package::find(root.name())?;
    let mut order = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit_runtime(root, target, &mut visiting, &mut visited, &mut order)?;
    Ok(order)
}

fn visit_runtime<'a>(
    package: &'a dyn Package,
    target: Target,
    visiting: &mut BTreeSet<&'static str>,
    visited: &mut BTreeSet<&'static str>,
    order: &mut Vec<&'a dyn Package>,
) -> Result<()> {
    let name = package.name();
    if visited.contains(name) {
        return Ok(());
    }
    if !visiting.insert(name) {
        return Err(Error::message(format!("依存が循環しています: {name}")));
    }
    for dependency in package.dependencies(target) {
        let found = package::find(dependency.name)?;
        if found.version() != dependency.version {
            return Err(Error::message(format!(
                "依存のバージョンが一致しません: {} は {} ですが定義は {}",
                dependency.name,
                dependency.version,
                found.version()
            )));
        }
        visit_runtime(found, target, visiting, visited, order)?;
    }
    visiting.remove(name);
    visited.insert(name);
    if !package.build_only() && !matches!(package.role(), crate::package::Role::BuildEnv) {
        order.push(package);
    }
    Ok(())
}

fn compose_layer_dirs(
    packages: &[&dyn Package],
    target: Target,
) -> Result<Vec<(&'static str, PathBuf)>> {
    let mut layers = Vec::new();
    for package in packages {
        if !package.supports(target) {
            return Err(Error::message(format!(
                "{} はこの platform を提供しません: {}/{}",
                package.name(),
                target.architecture(),
                target.variant()
            )));
        }
        let dir = package.artifact_dir(target);
        if !workspace::artifact_ready(&dir) {
            return Err(Error::message(format!(
                "パッケージがありません: {} (package build を実行してください)",
                dir.display()
            )));
        }
        layers.push((package.name(), dir));
    }
    Ok(layers)
}

pub fn check_path_conflicts(layers: &[(&str, PathBuf)]) -> Result<()> {
    let mut owners: BTreeMap<PathBuf, String> = BTreeMap::new();
    for (name, dir) in layers {
        collect_rel_paths(dir, Path::new(""), name, &mut owners)?;
    }
    Ok(())
}

fn collect_rel_paths(
    root: &Path,
    rel: &Path,
    owner: &str,
    owners: &mut BTreeMap<PathBuf, String>,
) -> Result<()> {
    let dir = root.join(rel);
    for entry in fs::read_dir(&dir).map_err(|source| Error::io(&dir, source))? {
        let entry = entry.map_err(|source| Error::io(&dir, source))?;
        let name = entry.file_name();
        let child_rel = rel.join(&name);
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| Error::io(&path, source))?;
        if metadata.is_dir() && !metadata.file_type().is_symlink() {
            collect_rel_paths(root, &child_rel, owner, owners)?;
            continue;
        }
        if child_rel == Path::new("usr/share/info/dir") {
            continue;
        }
        if let Some(existing) = owners.insert(child_rel.clone(), owner.to_string())
            && existing != owner
        {
            return Err(Error::message(format!(
                "パスが衝突しています: {} ({existing} と {owner})",
                child_rel.display()
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn path_conflict_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        fs::create_dir_all(a.join("usr/bin")).unwrap();
        fs::create_dir_all(b.join("usr/bin")).unwrap();
        fs::write(a.join("usr/bin/sh"), "a").unwrap();
        fs::write(b.join("usr/bin/sh"), "b").unwrap();
        let error = check_path_conflicts(&[("a", a), ("b", b)]).unwrap_err();
        assert!(error.to_string().contains("衝突"));
    }

    #[test]
    fn gnu_info_dir_is_not_a_conflict() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        fs::create_dir_all(a.join("usr/share/info")).unwrap();
        fs::create_dir_all(b.join("usr/share/info")).unwrap();
        fs::write(a.join("usr/share/info/dir"), "a").unwrap();
        fs::write(b.join("usr/share/info/dir"), "b").unwrap();
        check_path_conflicts(&[("a", a), ("b", b)]).unwrap();
    }

    #[test]
    fn openssl_runtime_closure_is_glibc_then_openssl() {
        let openssl = crate::package::find("openssl").unwrap();
        let names: Vec<_> = compose_packages(openssl, COMPOSE_PLATFORM)
            .unwrap()
            .iter()
            .map(|package| package.name())
            .collect();
        assert_eq!(names, ["x86_64-linux-gnu-glibc", "openssl"]);
    }

    #[test]
    fn build_env_compose_skips_empty_root_layer() {
        let env = crate::package::find("build-env").unwrap();
        let names: Vec<_> = compose_packages(env, COMPOSE_PLATFORM)
            .unwrap()
            .iter()
            .map(|package| package.name())
            .collect();
        assert!(!names.contains(&"build-env"), "{names:?}");
        assert!(names.contains(&"x86_64-linux-gnu-gcc"), "{names:?}");
        assert!(names.contains(&"mold"), "{names:?}");
    }

    #[test]
    fn distinct_paths_are_allowed() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        fs::create_dir_all(a.join("usr/bin")).unwrap();
        fs::create_dir_all(b.join("usr/lib")).unwrap();
        fs::write(a.join("usr/bin/sh"), "a").unwrap();
        fs::write(b.join("usr/lib/libx.so"), "b").unwrap();
        check_path_conflicts(&[("a", a), ("b", b)]).unwrap();
    }

    #[test]
    fn usrmerge_bin_does_not_conflict_with_usr_bin() {
        let tmp = tempfile::tempdir().unwrap();
        let pkg = tmp.path().join("pkg");
        let merge = tmp.path().join("merge");
        fs::create_dir_all(pkg.join("usr/bin")).unwrap();
        fs::create_dir_all(&merge).unwrap();
        fs::write(pkg.join("usr/bin/bash"), "bash").unwrap();
        write_usrmerge(&merge).unwrap();
        check_path_conflicts(&[("pkg", pkg), ("usrmerge", merge)]).unwrap();
    }
}
