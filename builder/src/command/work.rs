use crate::error::{Error, Result};
use crate::image;
use crate::package::{self, Package};
use crate::target::Target;
use crate::workspace;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{Compression, PackageSelection, TargetSelection};

pub fn fetch(selection: &PackageSelection, deps: bool) -> Result<()> {
    let roots = super::selected(selection, || package::PACKAGES.to_vec())?;
    let packages = if deps {
        package::build::fetch_closure(&roots)?
    } else {
        roots
    };
    // 同一 URL は一度だけダウンロードし、後続パッケージには tarball を
    // コピーして `fetch_package` のキャッシュ経路（検証→展開）に乗せる。
    let mut tarballs: BTreeMap<&'static str, PathBuf> = BTreeMap::new();
    super::run_aggregated(packages, |defined| {
        share_source(defined, &tarballs)?;
        for line in package::fetch_package(defined)? {
            println!("{line}");
        }
        if let Some(tarball) = source_tarball(defined)
            && tarball.exists()
        {
            tarballs.insert(defined.source().url, tarball);
        }
        Ok(())
    })
}

fn source_tarball(defined: &dyn Package) -> Option<PathBuf> {
    let filename = Path::new(defined.source().url).file_name()?;
    Some(defined.source_dir().join(filename))
}

fn share_source(defined: &dyn Package, fetched: &BTreeMap<&'static str, PathBuf>) -> Result<()> {
    let Some(existing) = fetched.get(defined.source().url) else {
        return Ok(());
    };
    let Some(dest) = source_tarball(defined) else {
        return Ok(());
    };
    if dest.exists() {
        return Ok(());
    }
    let dir = defined.source_dir();
    std::fs::create_dir_all(&dir).map_err(|e| Error::io(&dir, e))?;
    std::fs::copy(existing, &dest).map_err(|e| Error::io(&dest, e))?;
    Ok(())
}

pub fn build(
    selection: &PackageSelection,
    targets: &TargetSelection,
    force: bool,
    keep_going: bool,
    no_deps: bool,
    dry_run: bool,
) -> Result<()> {
    let (targets, all_targets) = super::resolve_targets(targets)?;
    let roots = super::selected(selection, || {
        package::PACKAGES
            .iter()
            .copied()
            .filter(|package| !package.compose_root())
            .collect()
    })?;
    for root in &roots {
        if root.compose_root() {
            return Err(Error::message(
                "build-env は env compose で合成してください",
            ));
        }
    }
    let options = package::BuildOptions {
        force,
        keep_going,
        no_deps,
        dry_run,
    };
    package::build_packages(
        &roots,
        &targets,
        all_targets || selection.all,
        &options,
        |line| println!("{line}"),
    )
}

pub fn check(selection: &PackageSelection) -> Result<()> {
    let all = selection.all;
    let packages = super::selected(selection, || package::images().collect())?;
    if packages.is_empty() {
        println!("まだ定義されたパッケージイメージはありません。");
        return Ok(());
    }
    super::run_aggregated(packages, |defined| {
        if all && !has_artifacts(defined) {
            println!("スキップ(パッケージなし): {}", defined.name());
            return Ok(());
        }
        package::validate::check(defined)?;
        println!(
            "{} {} の検証に成功しました",
            defined.name(),
            defined.version()
        );
        Ok(())
    })
}

/// `validate::check` と同じ基準（`Target::all()` 上に artifact dir が 1 つでも存在）。
fn has_artifacts(defined: &dyn Package) -> bool {
    Target::all().any(|target| defined.artifact_dir(target).is_dir())
}

/// `image::build` が layer を 1 つでも拾えるか（supports 真かつ artifact_ready）。
fn has_supported_artifacts(defined: &dyn Package) -> bool {
    Target::all().any(|target| {
        defined.supports(target) && workspace::artifact_ready(&defined.artifact_dir(target))
    })
}

pub fn pack(
    selection: &PackageSelection,
    with_build: bool,
    compression: &Compression,
) -> Result<()> {
    let level = super::resolve_zstd_level(compression)?;
    let all = selection.all;
    let packages = match &selection.package {
        Some(name) => vec![packable(package::find(name)?)?],
        None => package::PACKAGES
            .iter()
            .copied()
            .filter(|package| !package.build_only() && !package.compose_root())
            .collect(),
    };
    if packages.is_empty() {
        println!("まだ定義されたパッケージイメージはありません。");
        return Ok(());
    }
    super::run_aggregated(packages, |defined| {
        if all && !has_supported_artifacts(defined) {
            println!("スキップ(パッケージなし): {}", defined.name());
            return Ok(());
        }
        if with_build {
            let targets: Vec<_> = Target::all().collect();
            package::build_package(defined, &targets, true, |line| println!("{line}"))?;
        }
        let (output, skipped) = image::build(defined, level)?;
        for line in skipped {
            println!("{line}");
        }
        println!(
            "{} をビルドしました (zstd {level}): {}",
            defined.name(),
            output.display()
        );
        Ok(())
    })
}

fn packable(defined: &'static dyn Package) -> Result<&'static dyn Package> {
    if defined.compose_root() {
        return Err(Error::message(
            "build-env は env compose で合成してください",
        ));
    }
    if defined.build_only() {
        return Err(Error::message(format!(
            "{} はビルド専用のパッケージです",
            defined.name()
        )));
    }
    Ok(defined)
}

pub fn compose(
    name: &str,
    target: Target,
    with_build: bool,
    compression: &Compression,
) -> Result<()> {
    let level = super::resolve_zstd_level(compression)?;
    let defined = packable(package::find(name)?)?;
    if with_build {
        package::build_packages(
            &[defined],
            &[target],
            false,
            &package::BuildOptions::default(),
            |line| println!("{line}"),
        )?;
    }
    let (output, tag) = image::compose(defined, target, level)?;
    println!("{tag} を合成しました (zstd {level}): {}", output.display());
    Ok(())
}

pub fn clean(
    selection: &PackageSelection,
    targets: &TargetSelection,
    images: bool,
    dry_run: bool,
) -> Result<()> {
    let (targets, skip_unsupported) = super::resolve_targets(targets)?;
    let packages = super::selected(selection, || package::PACKAGES.to_vec())?;
    super::run_aggregated(packages, |defined| {
        clean_package(defined, &targets, skip_unsupported, images, dry_run)
    })
}

fn clean_package(
    defined: &'static dyn Package,
    targets: &[Target],
    skip_unsupported: bool,
    images: bool,
    dry_run: bool,
) -> Result<()> {
    let mut failure = None;
    for target in targets {
        if skip_unsupported && !defined.supports(*target) {
            continue;
        }
        let artifact = workspace::artifact_dir(defined.name(), defined.version(), *target);
        for path in [
            artifact.clone(),
            // 中断したビルドの atomic publish 残骸
            PathBuf::from(format!("{}.tmp", artifact.display())),
            workspace::build_dir(defined.name(), defined.version(), *target),
            workspace::gen_dir(defined.name(), defined.version(), *target),
            workspace::log_path(defined.name(), defined.version(), *target),
        ] {
            if let Err(error) = clean_path("packages", &path, dry_run) {
                eprintln!("{error}");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
        }
    }
    if images {
        let path = workspace::layout_dir(defined.name(), defined.version());
        if let Err(error) = clean_path("oci-layouts", &path, dry_run) {
            eprintln!("{error}");
            if failure.is_none() {
                failure = Some(error);
            }
        }
    }
    failure.map_or(Ok(()), Err)
}

fn clean_path(root: &str, path: &std::path::Path, dry_run: bool) -> Result<()> {
    let Ok(metadata) = path.symlink_metadata() else {
        return Ok(());
    };
    // シンボリックリンク自身はリンクのみを外す。リンク以外は canonical 化した
    // 実体が root（packages/ または oci-layouts/）内に留まることを確認してから
    // 削除する。親ディレクトリがシンボリックリンクの場合にツリー外を消さない
    // ための防御である。
    if !metadata.file_type().is_symlink() {
        let root = std::fs::canonicalize(root).map_err(|source| Error::io(root, source))?;
        let real = std::fs::canonicalize(path).map_err(|source| Error::io(path, source))?;
        if !real.starts_with(&root) {
            return Err(Error::message(format!(
                "ツリー外を指すため削除しません: {}",
                path.display()
            )));
        }
    }
    if dry_run {
        println!("削除予定: {}", path.display());
        return Ok(());
    }
    let result = if metadata.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    result.map_err(|source| Error::io(path, source))?;
    println!("削除: {}", path.display());
    Ok(())
}
