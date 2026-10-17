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

/// pack の platform 指定。`-t`（対象絞り込み）・`--remove-platform`
/// （除去）・merge/replace の選択を束ねる。
#[derive(Default)]
pub struct PlanFilter {
    /// pack 対象の platform。None は artifact が揃う全 platform。
    pub only: Option<BTreeSet<Target>>,
    /// 既存 layout から除去する platform。
    pub remove: Vec<Target>,
    /// 既存 layout の対象外 platform を保持する（既定の merge 動作）。
    /// false は全再生成（`--replace`）。
    pub merge: bool,
}

/// パッケージの成果物があるターゲットをレイヤーとして集める。
/// `only` があればその platform に絞る（merge 時の増分指定）。
fn plan(
    package: &dyn Package,
    only: Option<&BTreeSet<Target>>,
) -> (Vec<(Target, Vec<PathBuf>)>, Vec<String>) {
    let mut layers = Vec::new();
    let mut skipped = Vec::new();
    for target in Target::all() {
        if !package.supports(target) {
            continue;
        }
        if let Some(only) = only
            && !only.contains(&target)
        {
            continue;
        }
        let dir = package.artifact_dir(target);
        if workspace::artifact_ready(&dir) {
            layers.push((target, vec![dir]));
        } else {
            skipped.push(format!("スキップ(パッケージなし): {}", dir.display()));
        }
    }
    (layers, skipped)
}

fn package_image(package: &dyn Package) -> oci::Image {
    oci::Image {
        name: package.name().to_string(),
        version: package.version().to_string(),
        description: Some(format!(
            "containers package: {} {}",
            package.name(),
            package.version()
        )),
    }
}

/// 単一パッケージの layout を構築・確定する。
/// `filter.merge` 真のとき既存 layout の対象外 platform を保持し、
/// `filter.remove` の platform を既存 layout から除去する。
pub fn build(
    package: &'static dyn Package,
    filter: &PlanFilter,
    zstd_level: u8,
    workers: usize,
) -> Result<(PathBuf, Vec<String>)> {
    let (layers, skipped) = plan(package, filter.only.as_ref());
    if layers.is_empty() && filter.remove.is_empty() {
        return Err(Error::message(format!(
            "パッケージがありません: {} (build を実行してください)",
            package.name()
        )));
    }
    let output = workspace::layout_dir(package.name(), package.version());
    // sbom は extra_files で staging に同梱し、rename と一緒に確定させる
    // （確定後の追記だと rename と書き込みの間に不整合の窓が残る）。
    let sbom = sbom::json(package)?;
    let outcome = oci::build_all_annotated(
        vec![oci::LayoutJob {
            output: output.clone(),
            image: package_image(package),
            layers,
            config: ExecConfig::default(),
            annotate: Box::new(move |target| manifest_annotations(package, target)),
            merge: filter.merge,
            remove: filter.remove.clone(),
            extra_files: vec![("sbom.spdx.json".to_string(), sbom)],
        }],
        zstd_level,
        workers,
    )?;
    if let Some(failure) = outcome.failed.first() {
        return Err(Error::message(format!("レイアウト構築に失敗: {failure}")));
    }
    Ok((output, skipped))
}

/// 複数パッケージのレイアウトを共有ワーカープールで構築する。
/// 戻り値は (結果列, 失敗列)。結果列は (パッケージ名, 出力パス,
/// スキップ行) で、確定できなかったパッケージの出力パスは None
/// （レイヤーが無い・構築失敗の両方を含む）。
/// 途中で失敗したパッケージは `build_all_annotated` が集約して
/// 失敗列で返す（Err が返るのはスレッド panic 等の系統的失敗のみ）。
/// `build_all` の結果: (確定した (名前, 出力パス(失敗時 None), スキップ行) 一覧, 失敗一覧)。
pub type BuildAllResult = (Vec<(String, Option<PathBuf>, Vec<String>)>, Vec<String>);

pub fn build_all(
    packages: &[&'static dyn Package],
    filter: &PlanFilter,
    zstd_level: u8,
    workers: usize,
) -> Result<BuildAllResult> {
    let mut jobs = Vec::new();
    let mut results = Vec::new();
    let mut failed = Vec::new();
    for package in packages.iter().copied() {
        let (layers, skipped) = plan(package, filter.only.as_ref());
        let output = workspace::layout_dir(package.name(), package.version());
        if layers.is_empty() && filter.remove.is_empty() {
            results.push((package.name().to_string(), None, skipped));
            continue;
        }
        // sbom は extra_files で staging に同梱し rename と一緒に確定する。
        // 生成失敗はジョブ失敗として集約する（出力パスは None）。
        let sbom = match sbom::json(package) {
            Ok(bytes) => bytes,
            Err(error) => {
                failed.push(format!("{} の sbom ({error})", package.name()));
                results.push((package.name().to_string(), None, skipped));
                continue;
            }
        };
        jobs.push(oci::LayoutJob {
            output: output.clone(),
            image: package_image(package),
            layers,
            config: ExecConfig::default(),
            annotate: Box::new(move |target| manifest_annotations(package, target)),
            merge: filter.merge,
            remove: filter.remove.clone(),
            extra_files: vec![("sbom.spdx.json".to_string(), sbom)],
        });
        results.push((package.name().to_string(), Some(output), skipped));
    }
    let outcome = oci::build_all_annotated(jobs, zstd_level, workers)?;
    failed.extend(outcome.failed);
    // 確定しなかったジョブの出力パスを外す（部分成功の可視化）。
    for (name, output, _) in &mut results {
        if output.is_some() && !outcome.succeeded.iter().any(|ok| ok == name) {
            *output = None;
        }
    }
    Ok((results, failed))
}

/// manifest 単位の自己記述アノテーション。依存は platform 評価済みの
/// 人間可読版（`name version` のカンマ区切り）で、index artifact 不在時の
/// 保険として残す。機械的な解決は index が正本。
fn manifest_annotations(package: &dyn Package, target: Target) -> BTreeMap<String, String> {
    let mut annotations = BTreeMap::from([(
        "org.opencontainers.image.licenses".to_string(),
        package.license().to_string(),
    )]);
    let deps = package
        .dependencies(target)
        .iter()
        .map(|dependency| format!("{} {}", dependency.name, dependency.version))
        .collect::<Vec<_>>()
        .join(",");
    if !deps.is_empty() {
        annotations.insert("dev.containers.dependencies".to_string(), deps);
    }
    annotations
}

pub fn compose(root: &dyn Package, target: Target, zstd_level: u8) -> Result<(PathBuf, String)> {
    let packages = compose_packages(root, target)?;
    let layers = compose_layer_dirs(&packages, target)?;
    let date = crate::util::utc_date();
    let arch = target.architecture();
    // リビジョン採番〜生成〜current リンク更新を、リンク名のロックで
    // 直列化する。採番をロック外に出すと 2 つの compose が同一
    // リビジョンを選んで互いの出力を上書きし、リンク更新の窓で
    // push/status の読み取りが瞬断する。
    let current = workspace::compose_current_link(root.name(), arch);
    let mut link_lock = workspace::output_lock(&current)?;
    let _guard = link_lock
        .write()
        .map_err(|source| Error::io(&current, source))?;
    let revision = workspace::next_compose_revision(root.name(), arch, &date);
    let tag = format!("{}-{}:{}-{}", root.name(), arch, date, revision);
    let output = workspace::compose_layout_dir(root.name(), arch, &date, revision);
    let name = format!("{}-{arch}", root.name());
    let version = format!("{date}-{revision}");
    let image = oci::Image {
        description: Some(format!("containers composed image: {name} {version}")),
        name,
        version,
    };
    // sbom は extra_files で staging に同梱し rename と一緒に確定する。
    let extra_files = vec![("sbom.spdx.json".to_string(), sbom::json_many(&packages)?)];
    assemble(&output, &image, target, layers, zstd_level, extra_files)?;
    // exists() はリンク先をたどるため、dangling symlink は false に
    // なり symlink() が EEXIST で失敗する。リンク自身の実在で判定する。
    if current.symlink_metadata().is_ok() {
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
    Ok((output, tag))
}

/// 層ディレクトリ列から runnable な OCI イメージを合成する。
/// {triple}-glibc は bin/sbin の usrmerge シンボリックリンクを同梱する。
/// 閉包に libc が無い場合（ca-certificates 単独など）だけ合成側で足す。
/// `compose` と消費側（registry::pull の mode B）の共通経路。
pub fn assemble(
    output: &Path,
    image: &oci::Image,
    target: Target,
    mut layers: Vec<(&str, PathBuf)>,
    zstd_level: u8,
    extra_files: Vec<(String, Vec<u8>)>,
) -> Result<()> {
    let usrmerge = if has_usrmerge(&layers) {
        None
    } else {
        let dir = tempfile::tempdir().map_err(|source| Error::io(Path::new("usrmerge"), source))?;
        write_usrmerge(dir.path())?;
        layers.push(("usrmerge", dir.path().to_path_buf()));
        Some(dir)
    };
    check_path_conflicts(&layers)?;
    let dirs: Vec<PathBuf> = layers.iter().map(|(_, dir)| dir.clone()).collect();
    let result = oci::build_with_config(
        output,
        image,
        &[(target, dirs)],
        zstd_level,
        ExecConfig::runnable(),
        extra_files,
    );
    drop(usrmerge);
    result
}

/// いずれかの層が `bin`/`sbin` の usrmerge シンボリックリンクを持つか。
/// 実体が無くても symlink 自体の存在で判定する（ dangling を許容）。
/// 実ディレクトリ・通常ファイルは usrmerge 済みとはみなさない。
fn has_usrmerge(layers: &[(&str, PathBuf)]) -> bool {
    layers.iter().any(|(_, dir)| {
        ["bin", "sbin"].iter().all(|name| {
            dir.join(name)
                .symlink_metadata()
                .is_ok_and(|metadata| metadata.file_type().is_symlink())
        })
    })
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
    if !package.build_only() && !package.compose_root() {
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
                "パッケージがありません: {} (build を実行してください)",
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
        // info の dir ファイルは install-info が生成する索引。
        // --infodir=/usr/share/info/{triple} を使うパッケージ間で
        // usr/share/info/{triple}/dir が同名衝突するため配下全体を除外する
        if name == "dir" && child_rel.starts_with("usr/share/info") {
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
        for sub in ["usr/share/info", "usr/share/info/x86_64-linux-gnu"] {
            fs::create_dir_all(a.join(sub)).unwrap();
            fs::create_dir_all(b.join(sub)).unwrap();
            fs::write(a.join(sub).join("dir"), "a").unwrap();
            fs::write(b.join(sub).join("dir"), "b").unwrap();
        }
        check_path_conflicts(&[("a", a), ("b", b)]).unwrap();
    }

    #[test]
    fn openssl_runtime_closure_is_glibc_then_openssl() {
        let openssl = crate::package::find("openssl").unwrap();
        let names: Vec<_> = compose_packages(openssl, Target::BUILD_ENV)
            .unwrap()
            .iter()
            .map(|package| package.name())
            .collect();
        assert_eq!(names, ["x86_64-linux-gnu-glibc", "openssl"]);
    }

    #[test]
    fn build_env_compose_skips_empty_root_layer() {
        let env = crate::package::find("build-env").unwrap();
        let names: Vec<_> = compose_packages(env, Target::BUILD_ENV)
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
