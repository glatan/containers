use crate::error::{Error, Result};
use crate::image;
use crate::oci::{self, ImageManifest};
use crate::package::{self, Package};
use crate::registry::{self, layout_io, pull, skopeo};
use crate::target::Target;
use crate::workspace;
use std::collections::{BTreeMap, BTreeSet};
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
    let active = workspace::active_lock()?;
    let _active = active
        .read()
        .map_err(|source| Error::io(workspace::active_lock_path(), source))?;
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
        all_targets || selection.all_packages,
        &options,
        |line| println!("{line}"),
    )
}

pub fn check(selection: &PackageSelection) -> Result<()> {
    let all = selection.all_packages;
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

/// `image::build` が layer を 1 つでも拾えるか（supports 真かつ
/// artifact_ready）。`only` 指定時はその platform に絞る。
fn has_supported_artifacts(defined: &dyn Package, only: Option<&BTreeSet<Target>>) -> bool {
    Target::all().any(|target| {
        only.is_none_or(|only| only.contains(&target))
            && defined.supports(target)
            && workspace::artifact_ready(&defined.artifact_dir(target))
    })
}

/// 既存 layout が `targets` のいずれかの platform を持つか。
/// `--all-packages --remove-platform` で対象外パッケージを除外する判定に使う。
fn layout_has_platforms(defined: &dyn Package, targets: &[Target]) -> bool {
    let layout = workspace::layout_dir(defined.name(), defined.version());
    let Ok(published) = layout_io::published(&layout) else {
        return false;
    };
    published
        .manifests
        .iter()
        .any(|manifest| targets.contains(&manifest.target))
}

#[allow(clippy::too_many_arguments)]
pub fn pack(
    selection: &PackageSelection,
    targets: &[Target],
    remove: &[Target],
    replace: bool,
    with_build: bool,
    compression: &Compression,
    jobs: Option<usize>,
) -> Result<()> {
    let active = workspace::active_lock()?;
    let _active = active
        .read()
        .map_err(|source| Error::io(workspace::active_lock_path(), source))?;
    if let Some(0) = jobs {
        return Err(Error::message("--jobs は 1 以上を指定してください"));
    }
    // pack の既定は release（配布用）。dev 用には --profile dev。
    let level = super::resolve_zstd_level_or(compression, oci::ZstdLevel::PUBLISH)?;
    let all = selection.all_packages;
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
    // -t 指定時はその platform のみ、省略時は成果物が揃う全 platform。
    // merge が既定なので絞り込んでも既存 layout の他 platform は保持される。
    // --remove-platform 単独（-t なし）は「何も pack しない除去のみ」
    // の意味にする（除去対象の成果物を誤って再封入しないため）。
    let only: Option<BTreeSet<Target>> = if !targets.is_empty() {
        Some(targets.iter().copied().collect())
    } else if !remove.is_empty() {
        Some(BTreeSet::new())
    } else {
        None
    };
    let filter = image::PlanFilter {
        only: only.clone(),
        remove: remove.to_vec(),
        merge: !replace,
    };
    // --with-build は -t に連動して対象 target のみビルドする。
    // 除去のみの操作（--remove-platform 単独）では pack 対象が無いため
    // ビルドも行わない。
    let build_targets: Vec<Target> = if !targets.is_empty() {
        targets.to_vec()
    } else if remove.is_empty() {
        Target::all().collect()
    } else {
        Vec::new()
    };
    if all {
        // (パッケージ, ターゲット) を1つのキューに載せて全コアを埋める
        // グローバル並列パス。ビルド失敗・レイアウト失敗ともに集約して
        // 他のパッケージは続行する。
        let workers = jobs.unwrap_or_else(oci::default_workers);
        let mut selected = Vec::new();
        let mut failures = Vec::new();
        for defined in packages.iter().copied() {
            if with_build
                && let Err(error) =
                    package::build_package(defined, &build_targets, true, |line| println!("{line}"))
            {
                eprintln!("{error}");
                failures.push(format!("{} (build)", defined.name()));
                continue;
            }
            // -t 指定時に「対応するが成果物が無い」platform は警告にする
            // （merge の no-op で指定が静かに無視されないように）。
            if let Some(only) = &only {
                let missing: Vec<String> = only
                    .iter()
                    .filter(|target| {
                        defined.supports(**target)
                            && !workspace::artifact_ready(&defined.artifact_dir(**target))
                    })
                    .map(|target| format!("{}/{}", target.architecture(), target.variant()))
                    .collect();
                if !missing.is_empty() {
                    eprintln!(
                        "警告: {} は {} の成果物がありません（pack から除外されます）",
                        defined.name(),
                        missing.join(", ")
                    );
                }
            }
            // --replace は全再生成のため、成果物が無い platform は
            // 保持されず index から落ちる（merge 既定と違い警告が要る）。
            if replace {
                let missing: Vec<String> = Target::all()
                    .filter(|target| {
                        defined.supports(*target)
                            && !remove.contains(target)
                            && !workspace::artifact_ready(&defined.artifact_dir(*target))
                    })
                    .map(|target| format!("{}/{}", target.architecture(), target.variant()))
                    .collect();
                if !missing.is_empty() {
                    eprintln!(
                        "警告: {} は {} の成果物がありません（--replace で layout から外れます）",
                        defined.name(),
                        missing.join(", ")
                    );
                }
            }
            // 除去のみの --all-packages では「layout がその platform を持つ
            // パッケージ」だけを対象にする（持たないものは除去対象外）。
            let removable = !remove.is_empty() && layout_has_platforms(defined, remove);
            if !has_supported_artifacts(defined, only.as_ref()) && !removable {
                println!("スキップ(パッケージなし): {}", defined.name());
                continue;
            }
            selected.push(defined);
        }
        match image::build_all(&selected, &filter, level, workers) {
            Ok((results, pkg_failures)) => {
                for (name, output, skipped) in results {
                    for line in skipped {
                        println!("{line}");
                    }
                    if let Some(output) = output {
                        println!(
                            "{name} をビルドしました (zstd {level}): {}",
                            output.display()
                        );
                    }
                }
                // 部分成功時も確定分は報告し、失敗は集約する。
                failures.extend(pkg_failures);
            }
            Err(error) => failures.push(error.to_string()),
        }
        if !failures.is_empty() {
            for failure in &failures {
                eprintln!("失敗: {failure}");
            }
            return Err(Error::message(format!(
                "{} 件が失敗しました",
                failures.len()
            )));
        }
        return Ok(());
    }
    let workers = jobs.unwrap_or_else(oci::default_workers);
    super::run_aggregated(packages, |defined| {
        if with_build {
            package::build_package(defined, &build_targets, true, |line| println!("{line}"))?;
        }
        // 単一パッケージで -t 指定時は「指定 platform の成果物不在」を
        // エラーにする（merge の no-op で要求が静かに無視されないように）。
        // --remove-platform 併用でも pack 対象の不在は同様にエラー。
        if let Some(only) = &only {
            let missing: Vec<String> = only
                .iter()
                .filter(|target| {
                    !defined.supports(**target)
                        || !workspace::artifact_ready(&defined.artifact_dir(**target))
                })
                .map(|target| format!("{}/{}", target.architecture(), target.variant()))
                .collect();
            if !missing.is_empty() {
                return Err(Error::message(format!(
                    "{} の成果物がありません: {} (build を実行してください)",
                    defined.name(),
                    missing.join(", ")
                )));
            }
        }
        // --replace（単一パッケージ）は全再生成のため、成果物が無い
        // supported platform は保持されず index から落ちる。落とす
        // 意図がある場合は --remove-platform を併用する（remove は
        // 既存 layout からの除去として扱われる）。
        if replace {
            let missing: Vec<String> = Target::all()
                .filter(|target| {
                    defined.supports(*target)
                        && !remove.contains(target)
                        && !workspace::artifact_ready(&defined.artifact_dir(*target))
                })
                .map(|target| format!("{}/{}", target.architecture(), target.variant()))
                .collect();
            if !missing.is_empty() {
                return Err(Error::message(format!(
                    "{} は {} の成果物がありません（--replace は全 platform の成果物が必要です。build するか --remove-platform で対象から外してください）",
                    defined.name(),
                    missing.join(", ")
                )));
            }
        }
        let (output, skipped) = image::build(defined, &filter, level, workers)?;
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

/// layout（または registry）の blob から成果物を packages/artifacts/ へ
/// 復元する。dep 解決や増分 pack の入力確保が目的。
/// `--from-registry` は layout 自体を `oci-layouts/` へ復元する
/// （成果物展開は別の経路として layout 確定後に続行する）。
pub fn restore(
    selection: &PackageSelection,
    targets: &[Target],
    from_registry: bool,
    registry: Option<&str>,
    force: bool,
) -> Result<()> {
    // 前置詞の欠落はロックを取る前に返す。
    let registry = if from_registry {
        Some(registry::registry_prefix(registry)?)
    } else {
        None
    };
    let active = workspace::active_lock()?;
    let _active = active
        .read()
        .map_err(|source| Error::io(workspace::active_lock_path(), source))?;
    let all = selection.all_packages;
    let packages = match &selection.package {
        Some(name) => vec![restorable(package::find(name)?)?],
        // images()（定義側の列挙）から compose_root を除く。
        // build-env は合成イメージで成果物を持たない。
        None => package::images()
            .filter(|package| !package.compose_root())
            .collect(),
    };
    if packages.is_empty() {
        println!("まだ定義されたパッケージイメージはありません。");
        return Ok(());
    }
    pull::require_tools()?;
    if from_registry {
        skopeo::require()?;
    }
    super::run_aggregated(packages, |defined| {
        restore_package(
            defined,
            targets,
            from_registry,
            registry.as_deref(),
            all,
            force,
        )
    })
}

fn restorable(defined: &'static dyn Package) -> Result<&'static dyn Package> {
    if defined.compose_root() {
        return Err(Error::message(
            "build-env は合成イメージのため成果物を復元できません",
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

/// 1 パッケージ分: layout 確保（必要なら registry から復元）→
/// manifest 経由で対象 platform の層を成果物として展開する。
fn restore_package(
    defined: &dyn Package,
    targets: &[Target],
    from_registry: bool,
    registry: Option<&str>,
    all: bool,
    force: bool,
) -> Result<()> {
    let layout = workspace::layout_dir(defined.name(), defined.version());
    let mut lock = workspace::output_lock(&layout)?;
    if !layout.join("index.json").is_file() {
        if !from_registry {
            if all {
                println!("スキップ(layout なし): {}", defined.name());
                return Ok(());
            }
            return Err(Error::message(format!(
                "{} の layout がありません: {}（pack するか --from-registry でレジストリから復元してください）",
                defined.name(),
                layout.display()
            )));
        }
        // layout 自体の復元: skopeo で staging に pull してから確定する。
        // タグは `{name}:{version}` 規約（compose_root は対象外）。
        let _guard = lock.write().map_err(|source| Error::io(&layout, source))?;
        if !layout.join("index.json").is_file() {
            std::fs::create_dir_all("oci-layouts")
                .map_err(|source| Error::io("oci-layouts", source))?;
            let staging = tempfile::TempDir::new_in("oci-layouts")
                .map_err(|source| Error::io("oci-layouts", source))?;
            let registry = registry.ok_or_else(|| {
                Error::message(
                    "レジストリ前置詞を指定してください（--registry または CONTAINERS_REGISTRY）",
                )
            })?;
            let reference = format!(
                "docker://{}:{}",
                registry::package_repo(registry, defined.name()),
                defined.version()
            );
            skopeo::pull_layout(&reference, staging.path())?;
            oci::replace_dir(&layout, staging)?;
            println!("復元(registry): {} <- {}", layout.display(), reference);
        }
    }
    // published() の index.json 読み取りから layer 展開まで、並走する
    // pack の rename 窓を除くため read ロックを保持する。
    let _guard = lock.read().map_err(|source| Error::io(&layout, source))?;
    let published = layout_io::published(&layout)?;
    let available: BTreeMap<Target, &layout_io::PlatformManifest> = published
        .manifests
        .iter()
        .map(|manifest| (manifest.target, manifest))
        .collect();
    // -t 省略時は layout が持つ全 platform を復元する。
    let selected: Vec<Target> = if targets.is_empty() {
        available.keys().copied().collect()
    } else {
        for target in targets {
            if !available.contains_key(target) {
                return Err(Error::message(format!(
                    "{} の layout に platform {}/{} がありません",
                    defined.name(),
                    target.architecture(),
                    target.variant()
                )));
            }
        }
        targets.to_vec()
    };
    for target in selected {
        restore_artifact(defined, target, &layout, available[&target], force)?;
    }
    Ok(())
}

/// manifest が指す層を順に展開して `{artifact}.tmp` → rename で
/// 公開する。build と同じ per-(name,target) ロックを取るため、
/// ビルド中の成果物を壊さない・半端な状態で artifact_ready を
/// 騙らない。
fn restore_artifact(
    defined: &dyn Package,
    target: Target,
    layout: &Path,
    manifest: &layout_io::PlatformManifest,
    force: bool,
) -> Result<()> {
    let artifact = defined.artifact_dir(target);
    let mut lock = workspace::artifact_lock(defined.name(), defined.version(), target)?;
    let _guard = lock
        .write()
        .map_err(|source| Error::io(&artifact, source))?;
    // ロック待ちの間に別プロセスが復元/ビルドを終えていれば skip。
    if !force && workspace::artifact_ready(&artifact) {
        println!("スキップ(既存): {}", artifact.display());
        return Ok(());
    }
    let tmp = PathBuf::from(format!("{}.tmp", artifact.display()));
    if tmp.exists() {
        std::fs::remove_dir_all(&tmp).map_err(|source| Error::io(&tmp, source))?;
    }
    std::fs::create_dir_all(&tmp).map_err(|source| Error::io(&tmp, source))?;
    let parsed: ImageManifest = layout_io::read_blob(layout, &manifest.digest)?;
    for layer in &parsed.layers {
        pull::extract_layer(layout, &layer.digest, &tmp)?;
    }
    // 公開は rename で原子的に。--force の上書きは既存を .restore-bak へ
    // 退避してから入れ替える（rename 失敗時に既存成果物を失わない）。
    if workspace::artifact_ready(&artifact) {
        let backup = PathBuf::from(format!("{}.restore-bak", artifact.display()));
        if backup.exists() {
            std::fs::remove_dir_all(&backup).map_err(|source| Error::io(&backup, source))?;
        }
        std::fs::rename(&artifact, &backup).map_err(|source| Error::io(&artifact, source))?;
        if let Err(source) = std::fs::rename(&tmp, &artifact) {
            let _ = std::fs::rename(&backup, &artifact);
            return Err(Error::io(&artifact, source));
        }
        let _ = std::fs::remove_dir_all(&backup);
    } else {
        std::fs::rename(&tmp, &artifact).map_err(|source| Error::io(&artifact, source))?;
    }
    println!(
        "復元: {} <- {}/{}",
        artifact.display(),
        target.architecture(),
        target.variant()
    );
    Ok(())
}

pub fn compose(
    name: &str,
    target: Target,
    with_build: bool,
    compression: &Compression,
) -> Result<()> {
    let active = workspace::active_lock()?;
    let _active = active
        .read()
        .map_err(|source| Error::io(workspace::active_lock_path(), source))?;
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
    target_args: &[String],
    all_targets: bool,
    paths: &super::CleanPaths,
    dry_run: bool,
) -> Result<()> {
    let cell_scope = paths.packages_build || paths.packages_artifacts || paths.packages_logs;
    let layout_scope = paths.oci_layouts || paths.oci_layouts_index;
    if cell_scope && target_args.is_empty() && !all_targets {
        return Err(Error::message(
            "--packages-build、--packages-artifacts、--packages-logs には -t または --all-targets が必要です",
        ));
    }
    if !cell_scope && layout_scope && (!target_args.is_empty() || all_targets) {
        return Err(Error::message(
            "--oci-layouts と --oci-layouts-index だけの削除に -t と --all-targets は指定できません",
        ));
    }
    let targets = if all_targets {
        Target::all().collect()
    } else {
        super::parse_targets(target_args)?
    };
    let packages = super::selected(selection, || package::PACKAGES.to_vec())?;
    let cells = clean_cells(&packages, &targets, all_targets);
    let needs_active = paths.packages_artifacts || layout_scope;
    if dry_run {
        if needs_active {
            ensure_active_available()?;
        }
        return clean_dry_run(paths, &cells, &packages);
    }
    if needs_active {
        let mut active = workspace::active_lock()?;
        let _active = match active.try_write() {
            Ok(guard) => guard,
            Err(source) if source.kind() == std::io::ErrorKind::WouldBlock => {
                return Err(Error::message(
                    "build、pack、restore、compose の実行中は packages/artifacts/ と oci-layouts/ を削除できません",
                ));
            }
            Err(source) => return Err(Error::io(workspace::active_lock_path(), source)),
        };
        clean_locked(paths, &cells, &packages)
    } else {
        clean_locked(paths, &cells, &packages)
    }
}

fn ensure_active_available() -> Result<()> {
    let mut active = workspace::active_lock()?;
    match active.try_write() {
        Ok(guard) => {
            drop(guard);
            Ok(())
        }
        Err(source) if source.kind() == std::io::ErrorKind::WouldBlock => Err(Error::message(
            "build、pack、restore、compose の実行中は packages/artifacts/ と oci-layouts/ を削除できません",
        )),
        Err(source) => Err(Error::io(workspace::active_lock_path(), source)),
    }
}

fn clean_cells<'a>(
    packages: &[&'a dyn Package],
    targets: &[Target],
    all_targets: bool,
) -> Vec<(&'a dyn Package, Target)> {
    let mut cells = Vec::new();
    for defined in packages {
        for target in targets {
            if all_targets && !defined.supports(*target) {
                continue;
            }
            cells.push((*defined, *target));
        }
    }
    cells
}

fn clean_dry_run(
    paths: &super::CleanPaths,
    cells: &[(&dyn Package, Target)],
    packages: &[&dyn Package],
) -> Result<()> {
    if paths.packages_artifacts {
        for (defined, target) in cells {
            if workspace::ExclusiveLock::try_acquire(&workspace::artifact_lock_path(
                defined.name(),
                defined.version(),
                *target,
            ))?
            .is_none()
            {
                return Err(Error::message(format!(
                    "{} {} ({}/{}) は実行中のため {} を削除できません",
                    defined.name(),
                    defined.version(),
                    target.architecture(),
                    target.variant(),
                    defined.artifact_dir(*target).display()
                )));
            }
        }
    }
    if paths.oci_layouts || paths.oci_layouts_index {
        let mut layout_paths = Vec::new();
        if paths.oci_layouts {
            for defined in packages {
                layout_paths.push(workspace::layout_dir(defined.name(), defined.version()));
            }
        }
        if paths.oci_layouts_index {
            layout_paths.push(workspace::index_layout_dir());
        }
        for path in &layout_paths {
            if workspace::ExclusiveLock::try_acquire(&workspace::output_lock_path(path))?.is_none()
            {
                return Err(Error::message(format!(
                    "{} は実行中のため削除できません",
                    path.display()
                )));
            }
        }
    }
    for (defined, target) in cells {
        let scratch = paths.packages_build || paths.packages_logs;
        if scratch
            && workspace::ExclusiveLock::try_acquire(&workspace::artifact_lock_path(
                defined.name(),
                defined.version(),
                *target,
            ))?
            .is_none()
        {
            println!(
                "実行中のためスキップ: {} {} ({}/{})",
                defined.name(),
                defined.version(),
                target.architecture(),
                target.variant()
            );
            continue;
        }
        remove_cell(paths, *defined, *target, true)?;
    }
    remove_layouts(paths, packages, true)
}

fn clean_locked(
    paths: &super::CleanPaths,
    cells: &[(&dyn Package, Target)],
    packages: &[&dyn Package],
) -> Result<()> {
    let mut cell_locks = Vec::new();
    let mut locked_cells = Vec::new();
    if paths.packages_artifacts || paths.packages_build || paths.packages_logs {
        for (defined, target) in cells {
            let path = workspace::artifact_lock_path(defined.name(), defined.version(), *target);
            match workspace::ExclusiveLock::try_acquire(&path)? {
                Some(lock) => {
                    cell_locks.push(lock);
                    locked_cells.push((*defined, *target));
                }
                None if paths.packages_artifacts => {
                    return Err(Error::message(format!(
                        "{} {} ({}/{}) は実行中のため {} を削除できません",
                        defined.name(),
                        defined.version(),
                        target.architecture(),
                        target.variant(),
                        defined.artifact_dir(*target).display()
                    )));
                }
                None => {
                    println!(
                        "実行中のためスキップ: {} {} ({}/{})",
                        defined.name(),
                        defined.version(),
                        target.architecture(),
                        target.variant()
                    );
                }
            }
        }
    }
    let mut layout_locks = Vec::new();
    let layout_paths = layout_deletion_paths(paths, packages);
    for path in &layout_paths {
        match workspace::ExclusiveLock::try_acquire(&workspace::output_lock_path(path))? {
            Some(lock) => layout_locks.push(lock),
            None => {
                return Err(Error::message(format!(
                    "{} は実行中のため削除できません",
                    path.display()
                )));
            }
        }
    }
    for (defined, target) in locked_cells {
        remove_cell(paths, defined, target, false)?;
    }
    remove_layouts(paths, packages, false)?;
    drop(layout_locks);
    drop(cell_locks);
    Ok(())
}

fn layout_deletion_paths(paths: &super::CleanPaths, packages: &[&dyn Package]) -> Vec<PathBuf> {
    let mut layout_paths = Vec::new();
    if paths.oci_layouts {
        for defined in packages {
            layout_paths.push(workspace::layout_dir(defined.name(), defined.version()));
        }
    }
    if paths.oci_layouts_index {
        layout_paths.push(workspace::index_layout_dir());
    }
    layout_paths
}

fn remove_layouts(
    paths: &super::CleanPaths,
    packages: &[&dyn Package],
    dry_run: bool,
) -> Result<()> {
    for path in layout_deletion_paths(paths, packages) {
        clean_path("oci-layouts", &path, dry_run)?;
    }
    Ok(())
}

fn remove_cell(
    paths: &super::CleanPaths,
    defined: &dyn Package,
    target: Target,
    dry_run: bool,
) -> Result<()> {
    let artifact = defined.artifact_dir(target);
    if paths.packages_build {
        for path in [
            workspace::build_dir(defined.name(), defined.version(), target),
            workspace::gen_dir(defined.name(), defined.version(), target),
            PathBuf::from(format!("{}.tmp", artifact.display())),
            PathBuf::from(format!("{}.restore-bak", artifact.display())),
        ] {
            clean_path("packages", &path, dry_run)?;
        }
    }
    if paths.packages_logs {
        clean_path(
            "packages",
            &workspace::log_path(defined.name(), defined.version(), target),
            dry_run,
        )?;
    }
    if paths.packages_artifacts {
        clean_path("packages", &artifact, dry_run)?;
    }
    Ok(())
}

/// `oci-layouts/` と `packages/artifacts/` の管理外残骸を列挙する
/// （読み取りのみ、削除しない）。既知の命名規約:
/// - oci-layouts/: `.locks`, `.tmp`, `index`, `{name}-{version}`,
///   compose の `{name}-{arch}` リンクと `{name}-{arch}-{date}-{rev}`
/// - artifacts/: `.locks`, `{name}-{version}-{arch}-{variant}`
///
/// `.bak`・`.paused`・`.newmeta`・staging/`.tmp*` の残留等が対象に
/// なる。
pub fn orphans() -> Result<()> {
    let mut expected_layouts: BTreeSet<String> = [".locks", ".tmp", "index"]
        .iter()
        .map(|name| name.to_string())
        .collect();
    let mut layout_prefixes: Vec<String> = Vec::new();
    let mut expected_artifacts: BTreeSet<String> = [".locks".to_string()].into_iter().collect();
    for defined in package::all() {
        for target in Target::all() {
            expected_artifacts.insert(format!(
                "{}-{}-{}-{}",
                defined.name(),
                defined.version(),
                target.architecture(),
                target.variant()
            ));
        }
        if defined.compose_root() {
            for arch in Target::all()
                .map(|target| target.architecture())
                .collect::<BTreeSet<_>>()
            {
                let base = format!("{}-{arch}", defined.name());
                expected_layouts.insert(base.clone());
                layout_prefixes.push(format!("{base}-"));
            }
        } else if !defined.build_only() {
            expected_layouts.insert(format!("{}-{}", defined.name(), defined.version()));
        }
    }
    let mut found = false;
    for (dir, expected, prefixes) in [
        (
            PathBuf::from("oci-layouts"),
            &expected_layouts,
            &layout_prefixes,
        ),
        (
            PathBuf::from("packages/artifacts"),
            &expected_artifacts,
            &Vec::new(),
        ),
    ] {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut names: Vec<String> = entries
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| {
                !expected.contains(name)
                    && !prefixes
                        .iter()
                        .any(|prefix| name.starts_with(prefix.as_str()))
            })
            .collect();
        names.sort();
        if !names.is_empty() {
            found = true;
            println!("{}:", dir.display());
            for name in names {
                println!("  {name}");
            }
        }
    }
    if !found {
        println!("管理外の残骸はありません");
    }
    Ok(())
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
