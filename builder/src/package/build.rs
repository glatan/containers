use std::collections::BTreeSet;
use std::fs;
use std::process::Command;

use crate::error::{Error, Result};
use crate::flags::Flags;
use crate::package::{self, Package};
use crate::process;
use crate::target::Target;
use crate::workspace;

pub fn build_package(
    package: &dyn Package,
    targets: &[Target],
    skip_unsupported: bool,
    mut report: impl FnMut(String),
) -> Result<()> {
    build_package_with(
        package,
        targets,
        skip_unsupported,
        &|name| package::find(name),
        &mut report,
    )
}

fn build_package_with<'a>(
    package: &'a dyn Package,
    targets: &[Target],
    skip_unsupported: bool,
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
    report: &mut dyn FnMut(String),
) -> Result<()> {
    let order = packages_in_build_order(package, lookup)?;
    for defined in order {
        let mut fetched = false;
        for target in targets {
            match Flags::resolve(*target) {
                Ok(flags) => {
                    if workspace::artifact_ready(&defined.artifact_dir(*target)) {
                        report(format!(
                            "{} {} ({}/{}) はビルド済みです",
                            defined.name(),
                            defined.version(),
                            target.architecture(),
                            target.variant()
                        ));
                        continue;
                    }
                    if !fetched {
                        for line in super::fetch_package(defined)? {
                            report(line);
                        }
                        fetched = true;
                    }
                    build_target(defined, *target, &flags)?;
                    report(format!(
                        "{} {} ({}/{}) をビルドしました",
                        defined.name(),
                        defined.version(),
                        target.architecture(),
                        target.variant()
                    ));
                }
                Err(error) if skip_unsupported => report(format!("スキップ: {error}")),
                Err(error) => return Err(error),
            }
        }
    }
    Ok(())
}

pub(crate) fn packages_in_build_order<'a>(
    package: &'a dyn Package,
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
) -> Result<Vec<&'a dyn Package>> {
    let mut order = Vec::new();
    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    visit(package, lookup, &mut visiting, &mut visited, &mut order)?;
    Ok(order)
}

fn visit<'a>(
    package: &'a dyn Package,
    lookup: &dyn Fn(&str) -> Result<&'a dyn Package>,
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
    for dependency in package
        .dependencies()
        .iter()
        .chain(package.build_dependencies())
    {
        let found = lookup(dependency.name)?;
        if found.version() != dependency.version {
            return Err(Error::message(format!(
                "依存のバージョンが一致しません: {} は {} ですが定義は {}",
                dependency.name,
                dependency.version,
                found.version()
            )));
        }
        visit(found, lookup, visiting, visited, order)?;
    }
    visiting.remove(name);
    visited.insert(name);
    order.push(package);
    Ok(())
}

fn build_target(package: &dyn Package, target: Target, flags: &Flags) -> Result<()> {
    let sources = package.source_dir();
    if !sources.is_dir() {
        return Err(Error::message(format!(
            "ソースが見つかりません: {} (package fetch を実行してください)",
            sources.display()
        )));
    }
    let sources = fs::canonicalize(&sources).map_err(|source| Error::io(&sources, source))?;
    let source = sources.join(package.source().subdir);
    if !source.is_dir() {
        return Err(Error::message(format!(
            "ソースが見つかりません: {} (package fetch を実行してください)",
            source.display()
        )));
    }
    let architecture = target.architecture();
    let variant = target.variant();
    let build_dir = workspace::build_dir(package.name(), package.version(), target);
    let out_dir = package.artifact_dir(target);
    if build_dir.exists() {
        fs::remove_dir_all(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    }
    if out_dir.exists() {
        fs::remove_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    }
    fs::create_dir_all(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    fs::create_dir_all(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    let build_dir = fs::canonicalize(&build_dir).map_err(|source| Error::io(&build_dir, source))?;
    let out_dir = fs::canonicalize(&out_dir).map_err(|source| Error::io(&out_dir, source))?;
    let mut command = Command::new("podman");
    command.args(["run", "--rm"]);
    if !allow_network(package, target) {
        command.args(["--network=none"]);
    }
    command
        .args(["-v"])
        .arg(format!("{}:/src:ro", sources.display()));
    for dependency in package
        .dependencies()
        .iter()
        .chain(package.build_dependencies())
    {
        let dep_dir = dependency.artifact_dir(target);
        if !dep_dir.is_dir() {
            return Err(Error::message(format!(
                "依存パッケージがありません: {} (package build を実行してください)",
                dep_dir.display()
            )));
        }
        let dep_dir = fs::canonicalize(&dep_dir).map_err(|source| Error::io(&dep_dir, source))?;
        command
            .args(["-v"])
            .arg(format!("{}:/dep/{}:ro", dep_dir.display(), dependency.name));
    }
    command
        .args(["-v"])
        .arg(format!("{}:/build", build_dir.display()))
        .args(["-v"])
        .arg(format!("{}:/out", out_dir.display()))
        .args(["-e"])
        .arg(format!("SOURCE=/src/{}", package.source().subdir))
        .args(["-e", "BUILD=/build"])
        .args(["-e", "OUT=/out"])
        .args(["-e"])
        .arg(format!("ARCH={architecture}"))
        .args(["-e"])
        .arg(format!("VARIANT={variant}"))
        .args(["-e"])
        .arg(format!("TRIPLE={}", target.format_tuple()))
        .args(["-e"])
        .arg(format!("HOST_TRIPLE={}", Target::HOST.format_tuple()))
        .args(["-e"])
        .arg(format!("CROSS={}", target.cross_prefix()));
    for (name, value) in flags.env(package.flags()) {
        command.args(["-e"]).arg(format!("{name}={value}"));
    }
    let result = process::status(
        command
            .arg(super::env::BUILD_IMAGE.reference())
            .args(["/bin/bash", "-c"])
            .arg(package.build_script()),
        "podman",
    );
    if result.is_err() {
        let _ = fs::remove_dir_all(&out_dir);
    }
    result
}

fn allow_network(package: &dyn Package, target: Target) -> bool {
    package.network() || target.format_tuple() != Target::HOST.format_tuple()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{Dependency, Source};
    use crate::target::{Amd64Variant, Target};
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
        network: bool,
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
        fn build_only(&self) -> bool {
            false
        }
        fn network(&self) -> bool {
            self.network
        }
        fn source(&self) -> Source {
            Source {
                url: "https://example.invalid/x.tar.gz",
                sha3_512: ZERO_512,
                blake3: ZERO_512,
                subdir: "x",
            }
        }
        fn dependencies(&self) -> &'static [Dependency] {
            self.dependencies
        }
        fn build_dependencies(&self) -> &'static [Dependency] {
            self.build_dependencies
        }
        fn build_script(&self) -> &'static str {
            self.build_script
        }
        fn flags(&self) -> &'static [crate::flags::Var] {
            &[]
        }
        fn source_dir(&self) -> PathBuf {
            self.source_dir.clone()
        }
        fn artifact_dir(&self, target: Target) -> PathBuf {
            self.artifacts
                .join(format!("{}-{}", target.architecture(), target.variant()))
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
        let headers = Fake {
            name: "linux-headers",
            version: "1",
            dependencies: &[],
            build_dependencies: &[],
            source_dir: source.clone(),
            artifacts: artifacts.clone(),
            build_script: "false",
            network: false,
        };
        let glibc = Fake {
            name: "glibc",
            version: "1",
            dependencies: &[],
            build_dependencies: &[Dependency {
                name: "linux-headers",
                version: "1",
            }],
            source_dir: source.clone(),
            artifacts,
            build_script: "false",
            network: false,
        };
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
        let a = Fake {
            name: "a",
            version: "1",
            dependencies: &[Dependency {
                name: "b",
                version: "1",
            }],
            build_dependencies: &[],
            source_dir: source.clone(),
            artifacts: artifacts.clone(),
            build_script: "false",
            network: false,
        };
        let b = Fake {
            name: "b",
            version: "1",
            dependencies: &[Dependency {
                name: "a",
                version: "1",
            }],
            build_dependencies: &[],
            source_dir: source,
            artifacts,
            build_script: "false",
            network: false,
        };
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
        let package = Fake {
            name: "cached",
            version: "1",
            dependencies: &[],
            build_dependencies: &[],
            source_dir: source,
            artifacts,
            build_script: "false",
            network: false,
        };
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
        let package = Fake {
            name: "empty",
            version: "1",
            dependencies: &[],
            build_dependencies: &[],
            source_dir: source,
            artifacts,
            build_script: "false",
            network: false,
        };
        let dir = package.artifact_dir(target);
        fs::create_dir_all(&dir).unwrap();
        assert!(!workspace::artifact_ready(&dir));
    }

    #[test]
    fn build_order_rejects_version_mismatch() {
        let (_keep, source, artifacts) = empty_dirs();
        let headers = Fake {
            name: "linux-headers",
            version: "1",
            dependencies: &[],
            build_dependencies: &[],
            source_dir: source.clone(),
            artifacts: artifacts.clone(),
            build_script: "false",
            network: false,
        };
        let glibc = Fake {
            name: "glibc",
            version: "1",
            dependencies: &[],
            build_dependencies: &[Dependency {
                name: "linux-headers",
                version: "2",
            }],
            source_dir: source,
            artifacts,
            build_script: "false",
            network: false,
        };
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
        let package = Fake {
            name: "cached",
            version: "1",
            dependencies: &[],
            build_dependencies: &[],
            source_dir: source,
            artifacts,
            build_script: "false",
            network: false,
        };
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
        let isolated = Fake {
            name: "zlib",
            version: "1",
            dependencies: &[],
            build_dependencies: &[],
            source_dir: source.clone(),
            artifacts: artifacts.clone(),
            build_script: "false",
            network: false,
        };
        let always = Fake {
            name: "headers",
            version: "1",
            dependencies: &[],
            build_dependencies: &[],
            source_dir: source,
            artifacts,
            build_script: "false",
            network: true,
        };
        assert!(!allow_network(&isolated, Target::HOST));
        assert!(allow_network(
            &isolated,
            Target::Arm64(crate::target::Arm64Variant::V8)
        ));
        assert!(allow_network(&always, Target::HOST));
    }
}
