mod container;

use std::collections::BTreeSet;
use std::fs::{self, File};
use std::io::Write;
use std::sync::{Arc, Mutex};

use crate::error::{Error, Result};
use crate::package::{self, Dependency, Package, Role, Stage};
use crate::target::Target;
use crate::workspace;

pub fn build_package(
    package: &dyn Package,
    targets: &[Target],
    skip_unsupported: bool,
    mut report: impl FnMut(String),
) -> Result<()> {
    let lookup = |name: &str| package::find(name);
    let root = package::find(package.name())?;
    for target in targets {
        if !root.supports(*target) {
            if skip_unsupported {
                report(format!(
                    "スキップ: 非対応のターゲットです: {} {} {}/{}",
                    root.name(),
                    root.version(),
                    target.architecture(),
                    target.variant()
                ));
                continue;
            }
            return Err(Error::message(format!(
                "{} はこの platform を提供しません: {}/{}",
                root.name(),
                target.architecture(),
                target.variant()
            )));
        }
        let order = packages_in_graph_order(root, &lookup, *target)?;
        run_order(&order, skip_unsupported, false, None, &mut report)?;
    }
    Ok(())
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
    let failed = run_order(&order, false, true, Some(combined.clone()), &mut report)?;
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
    combined: Option<Arc<Mutex<File>>>,
    report: &mut dyn FnMut(String),
) -> Result<Vec<String>> {
    let own = build_env_ready();
    walk_nodes(
        order,
        skip_unsupported,
        continue_on_error,
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
        run_order(&order, skip_unsupported, false, None, report)?;
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

/// 実行時依存とビルド依存のペアを返す。
/// toolchain パッケージは `Bootstrap::seed_dependencies` でシード相の
/// グラフ差分（glibc 循環回避）を上書きできる。
pub(crate) fn graph_deps(
    package: &dyn Package,
    target: Target,
) -> (Vec<Dependency>, Vec<Dependency>) {
    if matches!(package.role(), Role::Toolchain { .. })
        && let Some(bootstrap) = package::find_bootstrap(package.name())
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
    if matches!(
        package.role(),
        Role::Toolchain {
            stage: Stage::Binutils | Stage::GccStage1 | Stage::GccStage2 | Stage::Gcc,
            ..
        }
    ) {
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
        if workspace::artifact_ready(&defined.artifact_dir(target)) {
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
    use super::container::{allow_network, compiler_cc, extra_host_tool_names, wrap_script};
    use super::*;
    use crate::package::{Dependency, Role, Source};
    use crate::target::{Amd64Variant, Arm64Variant, Target};
    use std::path::PathBuf;

    const ZERO_512: &str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    struct Fake {
        name: &'static str,
        version: &'static str,
        dependencies: &'static [Dependency],
        build_dependencies: &'static [Dependency],
        source_dir: PathBuf,
        artifacts: PathBuf,
        build_script: &'static str,
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
        fn build_script(&self) -> &'static str {
            self.build_script
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

    fn fake<'a>(
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
            build_script: "false",
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

    #[test]
    fn seed_cmake_installs_arch_mold_and_exposes_prefixed_gcc() {
        let cmake = crate::package::find("cmake").unwrap();
        let script = wrap_script(cmake, true, Target::BUILD_ENV);
        assert!(script.contains("pacman -Sy --noconfirm mold"), "{script}");
        assert!(script.contains("/tmp/gcc-b"), "{script}");
        assert!(script.contains("/tmp/meson-bin"), "{script}");
        assert!(script.contains("/tmp/host-strip"), "{script}");
        assert!(script.contains("/tmp/unprefixed-binutils"), "{script}");
        assert!(script.contains("/tmp/host-cc"), "{script}");
        assert!(script.contains("/tmp/la-overlay"), "{script}");
        assert!(script.contains("/tmp/pc-overlay"), "{script}");
        assert!(script.contains("-Wl,-rpath-link="), "{script}");
        assert!(script.contains("/tmp/pkg-config-bin"), "{script}");
        assert!(script.contains("-L"), "{script}");
        assert!(script.contains("liblto_plugin.so"), "{script}");
        let gcc = crate::package::find("x86_64-linux-gnu-gcc").unwrap();
        let gcc_script = wrap_script(gcc, false, Target::Arm64(Arm64Variant::V8));
        assert!(
            gcc_script.contains("INJECT_TRIPLE_GCC_SPECS=no"),
            "{gcc_script}"
        );
        let glibc = crate::package::find("x86_64-linux-gnu-glibc").unwrap();
        let glibc_script = wrap_script(glibc, false, Target::Amd64(Amd64Variant::V1));
        assert!(
            glibc_script.contains("INJECT_TRIPLE_GCC_SPECS=no"),
            "{glibc_script}"
        );
        assert!(
            glibc_script.contains("LD=\"${TRIPLE}-ld\""),
            "{glibc_script}"
        );
        let binutils = crate::package::find("x86_64-linux-gnu-binutils").unwrap();
        let bu_script = wrap_script(binutils, false, Target::Arm64(Arm64Variant::V8));
        assert!(
            bu_script.contains("_cc_triple=\"${PACKAGE_HOST:-$TRIPLE}\""),
            "{bu_script}"
        );
        assert!(
            bu_script.contains("/dep/${_cc_triple}-gcc/usr/lib/gcc/${_cc_triple}"),
            "{bu_script}"
        );
        assert!(
            !bu_script.contains("export LD=\"${_cc_triple}-gcc\""),
            "{bu_script}"
        );
        assert!(
            bu_script.contains("*/\"${HOST_TRIPLE}-gcc\"/*"),
            "{bu_script}"
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
        let perl_script = perl.build_script();
        assert!(
            perl_script.contains("-Accflags=\"$CFLAGS\""),
            "{perl_script}"
        );
        assert!(
            perl_script.contains("-Aldflags=\"$LDFLAGS\""),
            "{perl_script}"
        );
        assert!(
            perl_script.contains("-Dhostperl=/tmp/perl-host/miniperl"),
            "{perl_script}"
        );
        assert!(
            perl_script.contains("PERL_USE_UNSAFE_INC=1"),
            "{perl_script}"
        );
        assert!(
            perl_script.contains("-Dtargetarch=\"$TRIPLE\""),
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
        let mpfr_wrap = wrap_script(mpfr, false, Target::Arm64(Arm64Variant::V8));
        assert!(mpfr_wrap.contains("/tmp/mold-ld-cc"), "{mpfr_wrap}");
        let glibc = crate::package::find("aarch64-linux-gnu-glibc").unwrap();
        let glibc_wrap = wrap_script(glibc, false, Target::Arm64(Arm64Variant::V8));
        assert!(!glibc_wrap.contains("/tmp/mold-ld-cc"), "{glibc_wrap}");
    }
}
