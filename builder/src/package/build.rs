mod container;

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::package::{self, Dependency, Package, Role};
use crate::target::Target;
use crate::workspace;

/// `build` の実行制御フラグ。
#[derive(Default)]
pub struct BuildOptions {
    /// ルートパッケージの成果物が存在しても再ビルドする。
    pub force: bool,
    /// 失敗しても後続の独立ノードをビルドし、最後に失敗を集約する。
    pub keep_going: bool,
    /// 依存をビルドせずルートのみビルドする。
    pub no_deps: bool,
    /// ビルドせず計画のみ表示する。
    pub dry_run: bool,
}

pub fn build_package(
    package: &dyn Package,
    targets: &[Target],
    skip_unsupported: bool,
    report: impl FnMut(String),
) -> Result<()> {
    let root = package::find(package.name())?;
    build_packages(
        &[root],
        targets,
        skip_unsupported,
        &BuildOptions::default(),
        report,
    )
}

/// 複数ルートを `(name, target)` キーで重複排除しながら単一トポロジカル順に
/// ビルドする。`skip_unsupported` は supports 偽のルートをエラーにせずスキップする
/// （`--all` / `--all-targets` 時）。
pub fn build_packages(
    roots: &[&'static dyn Package],
    targets: &[Target],
    skip_unsupported: bool,
    options: &BuildOptions,
    mut report: impl FnMut(String),
) -> Result<()> {
    let lookup = |name: &str| package::find(name);
    let mut failed = Vec::new();
    for target in targets {
        let mut order = Vec::new();
        let mut visiting = BTreeSet::new();
        let mut visited = BTreeSet::new();
        let mut root_names = BTreeSet::new();
        for root in roots {
            if !root.supports(*target) {
                if skip_unsupported {
                    if options.dry_run {
                        report(format!(
                            "{} {} ({}/{}) [skip:unsupported]",
                            root.name(),
                            root.version(),
                            target.architecture(),
                            target.variant()
                        ));
                    } else {
                        report(format!(
                            "スキップ: 非対応のターゲットです: {} {} {}/{}",
                            root.name(),
                            root.version(),
                            target.architecture(),
                            target.variant()
                        ));
                    }
                    continue;
                }
                return Err(Error::message(format!(
                    "{} はこの platform を提供しません: {}/{}",
                    root.name(),
                    target.architecture(),
                    target.variant()
                )));
            }
            root_names.insert(root.name());
            visit(
                *root,
                &lookup,
                *target,
                &mut visiting,
                &mut visited,
                &mut order,
            )?;
        }
        if options.no_deps {
            order.retain(|(defined, _)| root_names.contains(defined.name()));
        }
        if options.dry_run {
            report_plan(
                &order,
                skip_unsupported,
                &root_names,
                options.force,
                &mut report,
            );
            continue;
        }
        let forced = if options.force {
            root_names.clone()
        } else {
            BTreeSet::new()
        };
        failed.extend(run_order(
            &order,
            skip_unsupported,
            options.keep_going,
            &forced,
            None,
            &mut report,
        )?);
    }
    if failed.is_empty() {
        Ok(())
    } else {
        Err(Error::message(format!(
            "ビルドが {} 件失敗しました: {}",
            failed.len(),
            failed.join(", ")
        )))
    }
}

fn report_plan(
    order: &[(&dyn Package, Target)],
    skip_unsupported: bool,
    roots: &BTreeSet<&'static str>,
    force: bool,
    report: &mut dyn FnMut(String),
) {
    let own = build_env_ready();
    for (defined, target) in order {
        let marker = if skip_unsupported
            && !own
            && matches!(target, Target::Ppc64le(_) | Target::S390x(_))
        {
            "[skip:no-cross]"
        } else if workspace::artifact_ready(&defined.artifact_dir(*target))
            && !(force && roots.contains(defined.name()))
        {
            "[cached]"
        } else {
            "[will-build]"
        };
        report(format!(
            "{} {} ({}/{}) {marker}",
            defined.name(),
            defined.version(),
            target.architecture(),
            target.variant()
        ));
    }
}

pub fn build_bootstrap(mut report: impl FnMut(String)) -> Result<()> {
    let log_path = workspace::bootstrap_log_path();
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let file = File::create(&log_path).map_err(|source| Error::io(&log_path, source))?;
    let combined = Arc::new(Mutex::new(file));
    let mut report = |line: String| {
        if let Ok(mut file) = combined.lock() {
            let _ = writeln!(file, "{line}");
        }
        report(line);
    };
    let lookup = |name: &str| package::find(name);
    let order = bootstrap_closure(&lookup, Target::BUILD_ENV)?;
    let failed = run_order(
        &order,
        false,
        true,
        &BTreeSet::new(),
        Some(combined.clone()),
        &mut report,
    )?;
    report(summarize_bootstrap(&failed, &log_path));
    if failed.is_empty() {
        Ok(())
    } else {
        Err(Error::message(format!(
            "bootstrap が {} 件失敗しました (ログ: {})",
            failed.len(),
            log_path.display()
        )))
    }
}

fn bootstrap_closure<'a>(
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
    target: Target,
) -> Result<Vec<(&'a dyn Package, Target)>> {
    let mut order = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for root in package::BOOTSTRAP {
        visit(
            lookup(root.name())?,
            lookup,
            target,
            &mut visiting,
            &mut visited,
            &mut order,
        )?;
    }
    Ok(order)
}

fn summarize_bootstrap(failed: &[String], log_path: &std::path::Path) -> String {
    if failed.is_empty() {
        format!("bootstrap は成功しました (ログ: {})", log_path.display())
    } else {
        format!(
            "bootstrap 失敗: {} (ログ: {})",
            failed.join(", "),
            log_path.display()
        )
    }
}

fn run_order(
    order: &[(&dyn Package, Target)],
    skip_unsupported: bool,
    continue_on_error: bool,
    force_roots: &BTreeSet<&'static str>,
    combined: Option<Arc<Mutex<File>>>,
    report: &mut dyn FnMut(String),
) -> Result<Vec<String>> {
    let own = build_env_ready();
    walk_nodes(
        order,
        skip_unsupported,
        continue_on_error,
        force_roots,
        true,
        report,
        &mut |defined, target| {
            container::build_target(
                defined,
                target,
                &container::linker_env(defined, target),
                own,
                combined.clone(),
            )
        },
    )
}

#[cfg(test)]
fn build_package_with<'a>(
    package: &'a dyn Package,
    targets: &[Target],
    skip_unsupported: bool,
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
    report: &mut dyn FnMut(String),
) -> Result<()> {
    for target in targets {
        if !package.supports(*target) {
            if skip_unsupported {
                report(format!(
                    "スキップ: 非対応のターゲットです: {} {} {}/{}",
                    package.name(),
                    package.version(),
                    target.architecture(),
                    target.variant()
                ));
                continue;
            }
            return Err(Error::message(format!(
                "{} はこの platform を提供しません: {}/{}",
                package.name(),
                target.architecture(),
                target.variant()
            )));
        }
        let order = packages_in_graph_order(package, lookup, *target)?;
        run_order(
            &order,
            skip_unsupported,
            false,
            &BTreeSet::new(),
            None,
            report,
        )?;
    }
    Ok(())
}

pub fn build_env_ready() -> bool {
    workspace::compose_current_link(workspace::BUILD_ENV_NAME, workspace::BUILD_ENV_ARCH).exists()
}

#[cfg(test)]
pub(crate) fn packages_in_build_order<'a>(
    package: &'a dyn Package,
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
) -> Result<Vec<&'a dyn Package>> {
    Ok(packages_in_graph_order(package, lookup, Target::BUILD_ENV)?
        .into_iter()
        .map(|(package, _)| package)
        .collect())
}

pub(crate) fn packages_in_graph_order<'a>(
    package: &'a dyn Package,
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
    target: Target,
) -> Result<Vec<(&'a dyn Package, Target)>> {
    let mut order = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit(
        package,
        lookup,
        target,
        &mut visiting,
        &mut visited,
        &mut order,
    )?;
    Ok(order)
}

/// `fetch --deps` 用: ルートが supports する全ターゲットで `graph_deps` 閉包を
/// 取り、パッケージ名で重複排除した最初出現順の集合を返す（ルート自身を含む）。
pub(crate) fn fetch_closure(roots: &[&'static dyn Package]) -> Result<Vec<&'static dyn Package>> {
    let lookup = |name: &str| package::find(name);
    let mut order = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    for root in roots {
        for target in Target::all() {
            if !root.supports(target) {
                continue;
            }
            visit(
                *root,
                &lookup,
                target,
                &mut visiting,
                &mut visited,
                &mut order,
            )?;
        }
    }
    let mut seen = BTreeSet::new();
    Ok(order
        .into_iter()
        .filter(|(defined, _)| seen.insert(defined.name()))
        .map(|(defined, _)| defined)
        .collect())
}

/// 実行時依存とビルド依存のペアを返す。
/// `find_bootstrap` 登録パッケージは
/// `Bootstrap::seed_dependencies` でグラフを無条件に上書きできる
/// （binutils が glibc との循環を避けて実行時依存を空にする用途）。
pub(crate) fn graph_deps(
    package: &dyn Package,
    target: Target,
) -> (Vec<Dependency>, Vec<Dependency>) {
    if let Some(bootstrap) = package::find_bootstrap(package.name())
        && let Some(deps) = bootstrap.seed_dependencies(target)
    {
        return deps;
    }
    (
        package.dependencies(target),
        package.build_dependencies(target),
    )
}

pub(crate) fn artifact_target(package: &dyn Package, depender: Target) -> Target {
    if matches!(package.role(), Role::LinuxHeaders) {
        return Target::BUILD_ENV;
    }
    if package.runs_on_host() && package.supports(Target::BUILD_ENV) {
        return Target::BUILD_ENV;
    }
    if package.gnu_triple().is_some() && !package.target_artifact() {
        return Target::BUILD_ENV;
    }
    if package.supports(depender) {
        return depender;
    }
    if package.supports(depender.baseline()) {
        return depender.baseline();
    }
    if let Some(target) =
        Target::all().find(|target| *target == target.baseline() && package.supports(*target))
    {
        return target;
    }
    Target::all()
        .find(|target| package.supports(*target))
        .unwrap_or(Target::BUILD_ENV)
}

fn visit<'a>(
    package: &'a dyn Package,
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
    target: Target,
    visiting: &mut BTreeSet<(&'static str, Target)>,
    visited: &mut BTreeSet<(&'static str, Target)>,
    order: &mut Vec<(&'a dyn Package, Target)>,
) -> Result<()> {
    let name = package.name();
    let target = if matches!(package.role(), Role::LinuxHeaders) {
        Target::BUILD_ENV
    } else {
        target
    };
    let key = (name, target);
    if visited.contains(&key) {
        return Ok(());
    }
    if !visiting.insert(key) {
        return Err(Error::message(format!(
            "依存が循環しています: {name} ({}/{})",
            target.architecture(),
            target.variant()
        )));
    }
    if package.runs_on_host() && target != Target::BUILD_ENV && package.supports(Target::BUILD_ENV)
    {
        visit(package, lookup, Target::BUILD_ENV, visiting, visited, order)?;
    }
    let (runtime, build) = graph_deps(package, target);
    for dependency in runtime.iter().chain(build.iter()) {
        let found = lookup(dependency.name)?;
        if found.version() != dependency.version {
            return Err(Error::message(format!(
                "依存のバージョンが一致しません: {} は {} ですが定義は {}",
                dependency.name,
                dependency.version,
                found.version()
            )));
        }
        visit(
            found,
            lookup,
            artifact_target(found, target),
            visiting,
            visited,
            order,
        )?;
    }
    visiting.remove(&key);
    visited.insert(key);
    order.push((package, target));
    Ok(())
}

fn walk_nodes(
    order: &[(&dyn Package, Target)],
    skip_unsupported: bool,
    continue_on_error: bool,
    force_roots: &BTreeSet<&'static str>,
    fetch: bool,
    report: &mut dyn FnMut(String),
    compile: &mut dyn FnMut(&dyn Package, Target) -> Result<()>,
) -> Result<Vec<String>> {
    let own = build_env_ready();
    let mut blocked = BTreeSet::new();
    let mut failed = Vec::new();
    for (defined, target) in order {
        let defined = *defined;
        let target = *target;
        if skip_unsupported && !own && matches!(target, Target::Ppc64le(_) | Target::S390x(_)) {
            report(format!(
                "スキップ: シードにクロスコンパイラがありません: {}/{}",
                target.architecture(),
                target.variant()
            ));
            continue;
        }
        if !force_roots.contains(defined.name())
            && workspace::artifact_ready(&defined.artifact_dir(target))
        {
            report(format!(
                "{} {} ({}/{}) はビルド済みです",
                defined.name(),
                defined.version(),
                target.architecture(),
                target.variant()
            ));
            continue;
        }
        if continue_on_error {
            report(format!(
                "===== {} {} =====",
                defined.name(),
                defined.version()
            ));
            if let Some(reason) = blocked_reason(defined, target, &blocked) {
                report(format!(
                    "スキップ: {} {} ({}/{}): {reason}",
                    defined.name(),
                    defined.version(),
                    target.architecture(),
                    target.variant()
                ));
                blocked.insert(defined.name());
                continue;
            }
        }
        if fetch {
            match super::fetch_package(defined) {
                Ok(lines) => {
                    for line in lines {
                        report(line);
                    }
                }
                Err(error) if continue_on_error => {
                    report(format!("{} の取得に失敗しました: {error}", defined.name()));
                    blocked.insert(defined.name());
                    failed.push(defined.name().to_string());
                    continue;
                }
                Err(error) => return Err(error),
            }
        }
        report(format!(
            "{} {} ({}/{}) をビルドします",
            defined.name(),
            defined.version(),
            target.architecture(),
            target.variant()
        ));
        match compile(defined, target) {
            Ok(()) => report(format!(
                "{} {} ({}/{}) をビルドしました",
                defined.name(),
                defined.version(),
                target.architecture(),
                target.variant()
            )),
            Err(error) if continue_on_error => {
                report(format!(
                    "{} {} ({}/{}) は失敗しました: {error}",
                    defined.name(),
                    defined.version(),
                    target.architecture(),
                    target.variant()
                ));
                blocked.insert(defined.name());
                failed.push(defined.name().to_string());
            }
            Err(error) => return Err(error),
        }
    }
    Ok(failed)
}

fn blocked_reason(
    package: &dyn Package,
    target: Target,
    blocked: &BTreeSet<&str>,
) -> Option<String> {
    let (runtime, build) = graph_deps(package, target);
    for dependency in runtime.iter().chain(build.iter()) {
        if blocked.contains(dependency.name) {
            return Some(format!("依存が失敗しています: {}", dependency.name));
        }
        let Some(found) = package::find(dependency.name).ok() else {
            return Some(format!("依存パッケージがありません: {}", dependency.name));
        };
        let mounted = artifact_target(found, target);
        if !workspace::artifact_ready(&found.artifact_dir(mounted)) {
            return Some(format!("依存パッケージがありません: {}", dependency.name));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::container::{
        Mount, Prep, allow_network, build_plan, compiler_cc, dep_paths, extra_host_tool_names,
        stage_generated, wrap_script,
    };
    use super::*;
    use crate::package::{Dependency, Role, Source};
    use crate::target::{Amd64Variant, Arm64Variant, Target};
    use std::path::{Path, PathBuf};

    const ZERO_512: &str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    struct Fake {
        name: &'static str,
        version: &'static str,
        dependencies: &'static [Dependency],
        build_dependencies: &'static [Dependency],
        source_dir: PathBuf,
        artifacts: PathBuf,
        role: Role,
    }

    impl Package for Fake {
        fn name(&self) -> &'static str {
            self.name
        }
        fn version(&self) -> &'static str {
            self.version
        }
        fn license(&self) -> &'static str {
            "MIT"
        }
        fn role(&self) -> Role {
            self.role
        }
        fn source(&self) -> Source {
            Source {
                url: "https://example.invalid/x.tar.gz",
                sha3_512: ZERO_512,
                blake3: ZERO_512,
                subdir: "x",
            }
        }
        fn dependencies(&self, _target: Target) -> Vec<Dependency> {
            self.dependencies.to_vec()
        }
        fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
            self.build_dependencies.to_vec()
        }
        /// 失敗するビルドを再現するため常に `false` を実行する。
        fn recipe(
            &self,
            _ctx: &crate::package::recipe::BuildContext,
        ) -> crate::package::recipe::Recipe {
            use crate::package::recipe::{Arg, Env, Pipe, Recipe, Step, Steps};
            Recipe::Steps(Steps::new(Env::None).step(Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(vec![Arg::lit("false")]),
            }))
        }
        fn flags(&self, _target: Target) -> crate::flags::Flag {
            crate::flags::Flag::new(vec![], vec![])
        }
        fn source_dir(&self) -> PathBuf {
            self.source_dir.clone()
        }
        fn artifact_dir(&self, target: Target) -> PathBuf {
            self.artifacts
                .join(format!("{}-{}", target.architecture(), target.variant()))
        }
    }

    fn fake(
        name: &'static str,
        dependencies: &'static [Dependency],
        build_dependencies: &'static [Dependency],
        source_dir: PathBuf,
        artifacts: PathBuf,
    ) -> Fake {
        Fake {
            name,
            version: "1",
            dependencies,
            build_dependencies,
            source_dir,
            artifacts,
            role: Role::Package,
        }
    }

    fn lookup<'a>(packages: &'a [&'a Fake]) -> impl Fn(&str) -> Result<&'a dyn Package> + 'a {
        |name| {
            packages
                .iter()
                .copied()
                .find(|package| package.name == name)
                .map(|package| package as &dyn Package)
                .ok_or_else(|| Error::message(format!("パッケージが見つかりません: {name}")))
        }
    }

    fn empty_dirs() -> (tempfile::TempDir, PathBuf, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let source = tmp.path().join("src");
        let artifacts = tmp.path().join("out");
        fs::create_dir_all(&source).unwrap();
        fs::create_dir_all(&artifacts).unwrap();
        (tmp, source, artifacts)
    }

    #[test]
    fn build_order_puts_dependencies_first() {
        let (_keep, source, artifacts) = empty_dirs();
        let mut headers = fake("linux-headers", &[], &[], source.clone(), artifacts.clone());
        headers.role = Role::LinuxHeaders;
        let glibc = fake(
            "glibc",
            &[],
            &[Dependency {
                name: "linux-headers",
                version: "1",
            }],
            source.clone(),
            artifacts,
        );
        let packages = [&headers, &glibc];
        let find = lookup(&packages);
        let order = packages_in_build_order(&glibc, &find).unwrap();
        assert_eq!(
            order
                .iter()
                .map(|package| package.name())
                .collect::<Vec<_>>(),
            ["linux-headers", "glibc"]
        );
    }

    #[test]
    fn build_order_rejects_cycles() {
        let (_keep, source, artifacts) = empty_dirs();
        let a = fake(
            "a",
            &[Dependency {
                name: "b",
                version: "1",
            }],
            &[],
            source.clone(),
            artifacts.clone(),
        );
        let b = fake(
            "b",
            &[Dependency {
                name: "a",
                version: "1",
            }],
            &[],
            source,
            artifacts,
        );
        let packages = [&a, &b];
        let find = lookup(&packages);
        let error = match packages_in_build_order(&a, &find) {
            Err(error) => error,
            Ok(_) => panic!("循環を検出できませんでした"),
        };
        assert!(error.to_string().contains("循環"));
    }

    #[test]
    fn skips_targets_with_existing_artifacts() {
        let (_keep, source, artifacts) = empty_dirs();
        let target = Target::Amd64(Amd64Variant::V1);
        let package = fake("cached", &[], &[], source, artifacts);
        let dir = package.artifact_dir(target);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("kept"), "ok").unwrap();
        let packages = [&package];
        let find = lookup(&packages);
        let mut log = Vec::new();
        build_package_with(&package, &[target], false, &find, &mut |line| {
            log.push(line)
        })
        .unwrap();
        assert!(log.iter().any(|line| line.contains("ビルド済み")));
        assert_eq!(fs::read_to_string(dir.join("kept")).unwrap(), "ok");
    }

    #[test]
    fn empty_artifact_dir_is_not_skipped() {
        let (_keep, source, artifacts) = empty_dirs();
        let target = Target::Amd64(Amd64Variant::V1);
        let package = fake("empty", &[], &[], source, artifacts);
        let dir = package.artifact_dir(target);
        fs::create_dir_all(&dir).unwrap();
        assert!(!workspace::artifact_ready(&dir));
    }

    #[test]
    fn build_order_rejects_version_mismatch() {
        let (_keep, source, artifacts) = empty_dirs();
        let mut headers = fake("linux-headers", &[], &[], source.clone(), artifacts.clone());
        headers.role = Role::LinuxHeaders;
        let glibc = fake(
            "glibc",
            &[],
            &[Dependency {
                name: "linux-headers",
                version: "2",
            }],
            source,
            artifacts,
        );
        let packages = [&headers, &glibc];
        let find = lookup(&packages);
        let error = match packages_in_build_order(&glibc, &find) {
            Err(error) => error,
            Ok(_) => panic!("バージョン不一致を検出できませんでした"),
        };
        assert!(error.to_string().contains("バージョンが一致しません"));
    }

    #[test]
    fn reports_completed_nodes_before_later_error() {
        let (_keep, source, artifacts) = empty_dirs();
        let target = Target::Amd64(Amd64Variant::V1);
        let package = fake("cached", &[], &[], source, artifacts);
        let dir = package.artifact_dir(target);
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("kept"), "ok").unwrap();
        let packages = [&package];
        let find = lookup(&packages);
        let mut log = Vec::new();
        let error = build_package_with(
            &package,
            &[target, Target::S390x(crate::target::S390xVariant::Z17)],
            false,
            &find,
            &mut |line| log.push(line),
        )
        .unwrap_err();
        assert!(
            log.iter().any(|line| line.contains("ビルド済み")),
            "{log:?}"
        );
        assert!(!error.to_string().is_empty());
    }

    #[test]
    fn native_without_network_flag_is_isolated() {
        let (_keep, source, artifacts) = empty_dirs();
        let isolated = fake("zlib", &[], &[], source, artifacts);
        assert!(!allow_network(
            &isolated,
            Target::Amd64(Amd64Variant::V1),
            false
        ));
        assert!(allow_network(
            &isolated,
            Target::Arm64(Arm64Variant::V8),
            false
        ));
        assert!(!allow_network(
            &isolated,
            Target::Amd64(Amd64Variant::V1),
            true
        ));
    }

    #[test]
    fn catalog_cross_uses_target_prefixed_cc() {
        let openssl = crate::package::find("openssl").unwrap();
        let env = compiler_cc(openssl, Target::Arm64(Arm64Variant::V8), false);
        let cc = env
            .iter()
            .find(|(name, _)| *name == "CC")
            .map(|(_, value)| value.as_str());
        assert_eq!(cc, Some("aarch64-linux-gnu-gcc"));
        let cxx = env
            .iter()
            .find(|(name, _)| *name == "CXX")
            .map(|(_, value)| value.as_str());
        assert_eq!(cxx, Some("aarch64-linux-gnu-g++"));
    }

    #[test]
    fn linux_headers_uses_seed_cc() {
        let headers = crate::package::find("linux-headers").unwrap();
        for target in [Target::BUILD_ENV, Target::Amd64(Amd64Variant::V4)] {
            let env = compiler_cc(headers, target, false);
            let cc = env
                .iter()
                .find(|(name, _)| *name == "CC")
                .map(|(_, value)| value.as_str());
            assert_eq!(cc, Some("gcc"), "{target:?}");
        }
    }

    #[test]
    fn prefixed_artifacts_follow_package_platform() {
        let arm = Target::Arm64(Arm64Variant::V8);
        let glibc = crate::package::find("aarch64-linux-gnu-glibc").unwrap();
        assert_eq!(artifact_target(glibc, arm), arm);
        let gcc = crate::package::find("aarch64-linux-gnu-gcc").unwrap();
        assert_eq!(artifact_target(gcc, arm), Target::BUILD_ENV);
        let names = extra_host_tool_names(glibc, arm);
        for tool in ["bison", "python", "m4"] {
            assert!(names.contains(&tool), "{names:?}");
        }
        assert!(names.contains(&"x86_64-linux-gnu-gcc"), "{names:?}");
        let x86_glibc = crate::package::find("x86_64-linux-gnu-glibc").unwrap();
        let names = extra_host_tool_names(x86_glibc, Target::Amd64(Amd64Variant::V4));
        assert!(!names.contains(&"x86_64-linux-gnu-gcc"), "{names:?}");
    }

    /// プリリュード（`build_plan` の `prelude`）を正準 JSON で
    /// `tests/golden-plans/prelude/` と byte 比較する。`mounts`/`prep` は
    /// 空固定（実依存は fixture 不要の構造カバレッジのみ）。
    /// UPDATE_GOLDEN=1 で再生成。孤立ファイルも検出する。
    #[test]
    fn prelude_plans_match_golden() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden-plans/prelude");
        std::fs::create_dir_all(&dir).unwrap();
        let bless = std::env::var_os("UPDATE_GOLDEN").is_some();
        let mut expected = std::collections::BTreeSet::new();
        let mut mismatches = Vec::new();
        for package in crate::package::definition::all() {
            let targets: &[Target] = if package.gnu_triple().is_some() {
                &[Target::Amd64(Amd64Variant::V4)]
            } else {
                &[
                    Target::Amd64(Amd64Variant::V4),
                    Target::Arm64(Arm64Variant::V8),
                ]
            };
            for target in targets {
                if !package.supports(*target) {
                    continue;
                }
                let slug = match target {
                    Target::Amd64(Amd64Variant::V4) => "amd64-v4",
                    Target::Arm64(Arm64Variant::V8) => "arm64-v8",
                    _ => continue,
                };
                let file = format!("{}-{slug}.json", package.name());
                expected.insert(file.clone());
                let plan = build_plan(*package, false, *target, &[], &Prep::default());
                let value = serde_json::to_value(&plan)
                    .unwrap_or_else(|_| panic!("{} の plan シリアライズに失敗", package.name()));
                let json = format!("{}\n", serde_json::to_string_pretty(&value).unwrap());
                let path = dir.join(&file);
                if bless {
                    std::fs::write(&path, &json).unwrap();
                    continue;
                }
                let actual = std::fs::read(&path).unwrap_or_else(|_| {
                    panic!(
                        "golden が無い: {}（UPDATE_GOLDEN=1 で生成）",
                        path.display()
                    )
                });
                if actual != json.as_bytes() {
                    mismatches.push(file);
                }
            }
        }
        let mut orphans = Vec::new();
        for entry in std::fs::read_dir(&dir).unwrap() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            if name.ends_with(".json") && !expected.contains(&name) {
                orphans.push(name);
            }
        }
        assert!(orphans.is_empty(), "孤立 prelude golden: {orphans:?}");
        assert!(
            mismatches.is_empty(),
            "prelude golden と不一致: {mismatches:?}（意図的な変更なら UPDATE_GOLDEN=1 で再生成）"
        );
    }

    #[test]
    fn wrap_script_passes_bash_n() {
        use std::io::Write;
        let cmake = crate::package::find("cmake").unwrap();
        for seed in [false, true] {
            let script = wrap_script(cmake, seed, Target::BUILD_ENV, &[], &Prep::default());
            let mut child = std::process::Command::new("bash")
                .arg("-n")
                .stdin(std::process::Stdio::piped())
                .spawn()
                .expect("bash -n の起動に失敗");
            child
                .stdin
                .as_mut()
                .unwrap()
                .write_all(script.as_bytes())
                .unwrap();
            let output = child.wait_with_output().unwrap();
            if !output.status.success() {
                let lines: Vec<&str> = script.lines().collect();
                panic!(
                    "{}
{}",
                    lines[..30].join(
                        "
"
                    ),
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
    }

    #[test]
    fn seed_cmake_installs_arch_mold_and_exposes_prefixed_gcc() {
        use std::os::unix::fs::PermissionsExt;
        let cmake = crate::package::find("cmake").unwrap();
        let script = wrap_script(cmake, true, Target::BUILD_ENV, &[], &Prep::default());
        assert!(script.contains("pacman -Sy --noconfirm mold"), "{script}");
        assert!(script.contains("/tmp/la-overlay"), "{script}");
        assert!(script.contains("/tmp/pc-overlay"), "{script}");
        // /dep/pkgconf が無い場合は command -v フォールバックが残る
        assert!(script.contains("/tmp/pkg-config-bin"), "{script}");

        // ツールチェーン一式をマウントした場合の Prep と生成スクリプト
        let tmp = tempfile::tempdir().unwrap();
        let mk = |name: &str| {
            let dir = tmp.path().join(name);
            fs::create_dir_all(&dir).unwrap();
            dir
        };
        let touch_x = |path: &Path| {
            fs::write(path, "#!/bin/sh\n").unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        };
        let cc_gcc = mk("aarch64-linux-gnu-gcc");
        fs::create_dir_all(cc_gcc.join("usr/bin")).unwrap();
        fs::create_dir_all(cc_gcc.join("usr/lib/gcc/aarch64-linux-gnu/16.0.0")).unwrap();
        fs::create_dir_all(cc_gcc.join("usr/libexec/gcc/aarch64-linux-gnu/16.0.0")).unwrap();
        touch_x(&cc_gcc.join("usr/bin/aarch64-linux-gnu-gcc"));
        touch_x(&cc_gcc.join("usr/bin/aarch64-linux-gnu-g++"));
        fs::write(
            cc_gcc.join("usr/libexec/gcc/aarch64-linux-gnu/16.0.0/liblto_plugin.so"),
            "",
        )
        .unwrap();
        let cc_bu = mk("aarch64-linux-gnu-binutils");
        fs::create_dir_all(cc_bu.join("usr/bin")).unwrap();
        for tool in ["ar", "ranlib", "nm", "strip", "objcopy", "objdump"] {
            touch_x(&cc_bu.join(format!("usr/bin/aarch64-linux-gnu-{tool}")));
        }
        let t_bu = mk("x86_64-linux-gnu-binutils");
        fs::create_dir_all(t_bu.join("usr/bin")).unwrap();
        let h_gcc = mk("x86_64-linux-gnu-gcc");
        fs::create_dir_all(h_gcc.join("usr/bin")).unwrap();
        touch_x(&h_gcc.join("usr/bin/x86_64-linux-gnu-gcc"));
        touch_x(&h_gcc.join("usr/bin/x86_64-linux-gnu-g++"));
        let pkgconf = mk("pkgconf");
        fs::create_dir_all(pkgconf.join("usr/bin")).unwrap();
        touch_x(&pkgconf.join("usr/bin/pkgconf"));
        let meson = mk("meson");
        fs::create_dir_all(meson.join("usr/bin")).unwrap();
        fs::write(
            meson.join("usr/bin/meson"),
            "#!/usr/bin/python3\nimport sys\n",
        )
        .unwrap();
        let python = mk("python");
        fs::create_dir_all(python.join("usr/bin")).unwrap();
        touch_x(&python.join("usr/bin/python3"));
        let mold = mk("mold");
        fs::create_dir_all(mold.join("usr/bin")).unwrap();
        touch_x(&mold.join("usr/bin/mold"));
        let zlib = mk("zlib");
        fs::create_dir_all(zlib.join("usr/bin")).unwrap();
        fs::create_dir_all(zlib.join("usr/lib/pkgconfig")).unwrap();
        fs::create_dir_all(zlib.join("usr/include")).unwrap();
        fs::write(
            zlib.join("usr/lib/pkgconfig/zlib.pc"),
            "prefix=/usr
libdir=/usr/lib
",
        )
        .unwrap();
        let cc_glibc = mk("aarch64-linux-gnu-glibc");
        fs::create_dir_all(cc_glibc.join("usr/aarch64-linux-gnu/lib")).unwrap();
        let mounts: Vec<Mount> = [
            ("aarch64-linux-gnu-gcc", &cc_gcc),
            ("aarch64-linux-gnu-binutils", &cc_bu),
            ("x86_64-linux-gnu-binutils", &t_bu),
            ("x86_64-linux-gnu-gcc", &h_gcc),
            ("pkgconf", &pkgconf),
            ("meson", &meson),
            ("python", &python),
            ("mold", &mold),
            ("zlib", &zlib),
            ("aarch64-linux-gnu-glibc", &cc_glibc),
        ]
        .into_iter()
        .map(|(name, dir)| Mount {
            name: name.to_owned(),
            dir: dir.clone(),
        })
        .collect();

        let binutils = crate::package::find("x86_64-linux-gnu-binutils").unwrap();
        let gen_dir = tmp.path().join("gen");
        let prep =
            stage_generated(binutils, &mounts, &gen_dir, Target::Arm64(Arm64Variant::V8)).unwrap();
        assert!(prep.pkg_config);
        assert_eq!(prep.meson.as_deref(), Some("/dep/python/usr/bin/python3"));
        assert!(prep.cc_binutils && prep.host_strip && prep.triple_binutils);
        assert_eq!(
            prep.gcc_lib.as_deref(),
            Some("/dep/aarch64-linux-gnu-gcc/usr/lib/gcc/aarch64-linux-gnu/16.0.0")
        );
        assert!(prep.lto_binutils && prep.triple_gcc && prep.unprefixed && prep.host_cc);
        assert!(prep.pc);
        assert!(prep.mold);
        // meson の shebang が dep python に書き換わる
        let meson_out = fs::read_to_string(gen_dir.join("meson-bin/meson")).unwrap();
        assert!(
            meson_out.starts_with("#!/dep/python/usr/bin/python3\n"),
            "{meson_out}"
        );

        let bu_script = wrap_script(
            binutils,
            false,
            Target::Arm64(Arm64Variant::V8),
            &mounts,
            &prep,
        );
        assert!(
            bu_script.contains("_cc_triple=\"${PACKAGE_HOST:-$TRIPLE}\""),
            "{bu_script}"
        );
        for needle in [
            "/tmp/gcc-b",
            "/tmp/lto-binutils",
            "/tmp/host-strip",
            "/tmp/unprefixed-binutils",
            "/tmp/host-cc",
            "/tmp/meson-bin",
            "/dep/mold/usr/bin",
            "-B/dep/aarch64-linux-gnu-gcc/usr/lib/gcc/aarch64-linux-gnu/16.0.0",
            "-L/dep/zlib/usr/lib -Wl,-rpath-link=/dep/zlib/usr/lib",
            "-I/dep/zlib/usr/include",
            "-L/dep/aarch64-linux-gnu-glibc/usr/aarch64-linux-gnu/lib -Wl,-rpath-link=",
            "/dep/zlib/usr/lib/pkgconfig",
        ] {
            assert!(bu_script.contains(needle), "{needle} not in {bu_script}");
        }
        // 実際のラッパファイルが生成される
        let strip = fs::read_link(gen_dir.join("host-strip/strip")).unwrap();
        assert_eq!(
            strip,
            Path::new("/dep/aarch64-linux-gnu-binutils/usr/bin/aarch64-linux-gnu-strip")
        );
        let ar = fs::read_to_string(gen_dir.join("lto-binutils/aarch64-linux-gnu-ar")).unwrap();
        assert!(ar.contains("--plugin"), "{ar}");

        // gcc/glibc は triple gcc specs 注入対象外（gcc-b/lto-binutils を生成しない）
        let gcc = crate::package::find("x86_64-linux-gnu-gcc").unwrap();
        let gcc_prep = stage_generated(
            gcc,
            &mounts,
            &tmp.path().join("gen2"),
            Target::Arm64(Arm64Variant::V8),
        )
        .unwrap();
        assert!(gcc_prep.gcc_lib.is_none() && !gcc_prep.lto_binutils);
        let gcc_script = wrap_script(
            gcc,
            false,
            Target::Arm64(Arm64Variant::V8),
            &mounts,
            &gcc_prep,
        );
        assert!(!gcc_script.contains("/tmp/gcc-b"), "{gcc_script}");

        let glibc = crate::package::find("x86_64-linux-gnu-glibc").unwrap();
        let glibc_script = wrap_script(
            glibc,
            false,
            Target::Amd64(Amd64Variant::V1),
            &[],
            &Prep::default(),
        );
        assert!(
            glibc_script.contains("LD=\"${TRIPLE}-ld\""),
            "{glibc_script}"
        );
        assert!(allow_network(cmake, Target::BUILD_ENV, false));
        assert!(!allow_network(cmake, Target::BUILD_ENV, true));
        assert!(
            cmake
                .rendered_script(Target::BUILD_ENV)
                .contains("CMAKE_DISABLE_FIND_PACKAGE_Libidn2"),
            "{}",
            cmake.rendered_script(Target::BUILD_ENV)
        );
    }

    #[test]
    fn continue_on_error_skips_dependents_and_builds_unrelated() {
        let (_keep, source, artifacts) = empty_dirs();
        let target = Target::BUILD_ENV;
        let a = fake("a", &[], &[], source.clone(), artifacts.clone());
        let b = fake(
            "b",
            &[Dependency {
                name: "a",
                version: "1",
            }],
            &[],
            source.clone(),
            artifacts.clone(),
        );
        let c = fake("c", &[], &[], source, artifacts);
        let order: Vec<(&dyn Package, Target)> = vec![(&a, target), (&b, target), (&c, target)];
        let mut compiled = Vec::new();
        let mut log = Vec::new();
        let failed = walk_nodes(
            &order,
            false,
            true,
            &BTreeSet::new(),
            false,
            &mut |line| log.push(line),
            &mut |package, _target| {
                compiled.push(package.name());
                if package.name() == "a" {
                    Err(Error::message("a が失敗"))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap();
        assert_eq!(failed, ["a"]);
        assert_eq!(compiled, ["a", "c"]);
        assert!(
            log.iter()
                .any(|line| line.contains("スキップ") && line.contains("b")),
            "{log:?}"
        );
        assert!(
            log.iter()
                .any(|line| line.contains("c") && line.contains("ビルドしました")),
            "{log:?}"
        );
    }

    #[test]
    fn prefixed_gcc_uses_bootstrap_graph_without_cycle() {
        let lookup = |name: &str| crate::package::find(name);
        let gcc = crate::package::find("x86_64-linux-gnu-gcc").unwrap();
        let order = packages_in_graph_order(gcc, &lookup, Target::BUILD_ENV).unwrap();
        let names: Vec<_> = order.iter().map(|(package, _)| package.name()).collect();
        assert!(names.contains(&"linux-headers"));
        assert!(names.contains(&"x86_64-linux-gnu-binutils"));
        let openssl = crate::package::find("openssl").unwrap();
        let arm_order =
            packages_in_graph_order(openssl, &lookup, Target::Arm64(Arm64Variant::V8)).unwrap();
        assert!(arm_order.iter().all(|(package, target)| {
            package.name() != "libxcrypt" || *target == Target::BUILD_ENV
        }));
        let perl = crate::package::find("perl").unwrap();
        let perl_arm =
            packages_in_graph_order(perl, &lookup, Target::Arm64(Arm64Variant::V8)).unwrap();
        let perl_targets: Vec<_> = perl_arm
            .iter()
            .filter(|(package, _)| package.name() == "perl")
            .map(|(_, target)| *target)
            .collect();
        assert_eq!(
            perl_targets,
            [Target::BUILD_ENV, Target::Arm64(Arm64Variant::V8)]
        );
        let perl_script = perl.rendered_script(Target::BUILD_ENV);
        assert!(
            perl_script.contains("-Accflags=\"${CFLAGS}\""),
            "{perl_script}"
        );
        assert!(
            perl_script.contains("-Aldflags=\"${LDFLAGS}\""),
            "{perl_script}"
        );
        assert!(
            perl_script.contains("PERL_USE_UNSAFE_INC=1"),
            "{perl_script}"
        );
        // cross 専用ステップは静的特化によりクロス向けレンダーにのみ出る。
        let perl_script = perl.rendered_script(Target::Arm64(Arm64Variant::V8));
        assert!(
            perl_script.contains("-Dhostperl=/tmp/perl-host/miniperl"),
            "{perl_script}"
        );
        assert!(
            perl_script.contains("-Dtargetarch=\"${TRIPLE}\""),
            "{perl_script}"
        );
        assert!(perl_script.contains("run=:"), "{perl_script}");
        assert!(perl_script.contains("xxx=Ok"), "{perl_script}");
        assert!(perl_script.contains("-Dcharbits=8"), "{perl_script}");
        assert!(perl_script.contains("-Dlseeksize=8"), "{perl_script}");
        assert!(perl_script.contains("-Dssizetype=ssize_t"), "{perl_script}");
        assert!(perl_script.contains("-Uebcdic"), "{perl_script}");
        assert!(
            perl_script.contains("-Divdformat='\"ld\"'"),
            "{perl_script}"
        );
        assert!(!perl_script.contains("/tmp/perl-qemu-run"), "{perl_script}");
        assert!(!perl_script.contains("-Dtargethost="), "{perl_script}");
        let mpfr = crate::package::find("mpfr").unwrap();
        let mpfr_wrap = wrap_script(
            mpfr,
            false,
            Target::Arm64(Arm64Variant::V8),
            &[],
            &Prep::default(),
        );
        assert!(mpfr_wrap.contains("/tmp/mold-ld-cc"), "{mpfr_wrap}");
        let glibc = crate::package::find("aarch64-linux-gnu-glibc").unwrap();
        let glibc_wrap = wrap_script(
            glibc,
            false,
            Target::Arm64(Arm64Variant::V8),
            &[],
            &Prep::default(),
        );
        assert!(!glibc_wrap.contains("/tmp/mold-ld-cc"), "{glibc_wrap}");
    }
    #[test]
    fn dep_paths_expands_mounts_in_byte_order() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().to_path_buf();
        fs::create_dir_all(dir.join("usr/lib/pkgconfig")).unwrap();
        fs::create_dir_all(dir.join("usr/lib64/pkgconfig")).unwrap();
        fs::write(dir.join("usr/lib/libb.la"), "").unwrap();
        fs::write(dir.join("usr/lib/liba.la"), "").unwrap();
        fs::write(dir.join("usr/lib/.hidden.la"), "").unwrap();
        fs::write(dir.join("usr/lib/x.txt"), "").unwrap();
        let mounts = [Mount {
            name: "zlib".to_owned(),
            dir,
        }];
        // bash の /dep/*/usr/lib/*.la と同じくバイト順・dotfile 除外
        assert_eq!(
            dep_paths(&mounts, "usr/lib/*.la"),
            vec![
                "/dep/zlib/usr/lib/liba.la".to_owned(),
                "/dep/zlib/usr/lib/libb.la".to_owned()
            ]
        );
        // 未マッチの glob は空（ループ非出力）でスキップ相当
        assert!(dep_paths(&mounts, "usr/include").is_empty());
        assert_eq!(
            dep_paths(&mounts, "usr/lib64/pkgconfig"),
            vec!["/dep/zlib/usr/lib64/pkgconfig".to_owned()]
        );
    }
    #[test]
    fn stage_overlays_rewrites_la_pc_and_prefixes() {
        let tmp = tempfile::tempdir().unwrap();
        let gmp = tmp.path().join("gmp");
        fs::create_dir_all(gmp.join("usr/lib")).unwrap();
        fs::create_dir_all(gmp.join("usr/include")).unwrap();
        fs::write(gmp.join("usr/include/gmp.h"), "").unwrap();
        fs::write(gmp.join("usr/lib/libgmp.so"), "").unwrap();
        fs::write(
            gmp.join("usr/lib/libgmp.la"),
            "libdir='/usr/lib'
installed=yes
dependency_libs=' -L/usr/lib /dep/mpfr/usr/lib/libmpfr.la'
",
        )
        .unwrap();
        let zlib = tmp.path().join("zlib");
        fs::create_dir_all(zlib.join("usr/lib/pkgconfig")).unwrap();
        fs::write(
            zlib.join("usr/lib/pkgconfig/zlib.pc"),
            "prefix=/usr
libdir=/usr/lib
includedir=/usr/include
",
        )
        .unwrap();
        let mounts = [
            Mount {
                name: "gmp".to_owned(),
                dir: gmp,
            },
            Mount {
                name: "zlib".to_owned(),
                dir: zlib,
            },
        ];
        let gen_dir = tmp.path().join("gen");
        let gmp_pkg = crate::package::find("gmp").unwrap();
        let overlays = stage_generated(gmp_pkg, &mounts, &gen_dir, Target::BUILD_ENV).unwrap();
        assert!(overlays.la && overlays.pc);
        let la = fs::read_to_string(gen_dir.join("la-overlay/libgmp.la")).unwrap();
        assert!(la.contains("libdir='/tmp/la-overlay'"), "{la}");
        assert!(la.contains("installed=no"), "{la}");
        assert!(la.contains("-L/tmp/la-overlay"), "{la}");
        assert!(la.contains("/tmp/la-overlay/libmpfr.la"), "{la}");
        // シンボリックリンクはコンテナ内パスを指す
        let link = fs::read_link(gen_dir.join("la-overlay/libgmp.so")).unwrap();
        assert_eq!(link, Path::new("/dep/gmp/usr/lib/libgmp.so"));
        let inc = fs::read_link(gen_dir.join("gmp-prefix/include/gmp.h")).unwrap();
        assert_eq!(inc, Path::new("/dep/gmp/usr/include/gmp.h"));
        let lib = fs::read_link(gen_dir.join("gmp-prefix/lib/libgmp.so")).unwrap();
        assert_eq!(lib, Path::new("/tmp/la-overlay/libgmp.so"));
        let pc = fs::read_to_string(gen_dir.join("pc-overlay/zlib.pc")).unwrap();
        assert!(
            pc.contains(
                "prefix=/dep/zlib/usr
"
            ),
            "{pc}"
        );
        assert!(
            pc.contains(
                "libdir=/dep/zlib/usr/lib
"
            ),
            "{pc}"
        );
        assert!(pc.contains("includedir=/dep/zlib/usr/include"), "{pc}");
    }
    #[test]
    fn fetch_closure_deduplicates_names_across_targets() {
        let zlib = crate::package::find("zlib").unwrap();
        let closure = super::fetch_closure(&[zlib]).unwrap();
        let names: Vec<_> = closure.iter().map(|defined| defined.name()).collect();
        // 全ターゲットの閉包を合流しても名前は一意（linux-headers は各ターゲット
        // の閉包に出るが 1 回だけ）
        let unique: std::collections::BTreeSet<_> = names.iter().collect();
        assert_eq!(names.len(), unique.len());
        assert!(names.contains(&"zlib"));
        assert!(names.contains(&"x86_64-linux-gnu-glibc"));
        assert!(names.contains(&"aarch64-linux-gnu-glibc"));
        assert!(names.contains(&"linux-headers"));
    }

    #[test]
    fn graph_deps_binutils_override_applies_unconditionally() {
        // `seed_dependencies` は名前に反して seed 相限定ではなく、
        // `find_bootstrap` 登録パッケージの全ターゲットでグラフを
        // 上書きする（旧 Prefixed と同じ契約）。aarch64 のような
        // non-seed タプルでも実行時依存は空になる。
        let binutils = crate::package::find("aarch64-linux-gnu-binutils").unwrap();
        let (runtime, _build) = super::graph_deps(binutils, Target::Arm64(Arm64Variant::V8));
        assert!(runtime.is_empty(), "{runtime:?}");
        // 非登録パッケージは `Package` の宣言がそのまま使われる。
        let zlib = crate::package::find("zlib").unwrap();
        let (runtime, _) = super::graph_deps(zlib, Target::Arm64(Arm64Variant::V8));
        assert!(runtime.iter().any(|d| d.name == "aarch64-linux-gnu-glibc"));
    }
}
