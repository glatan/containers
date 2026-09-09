use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::error::{Error, Result};
use crate::oci::{Platform, TARGETS};
use crate::package::{Package, find};
use crate::process;

pub fn run(
    name: &str,
    all_target: bool,
    architecture: Option<String>,
    variant: Option<String>,
) -> Result<()> {
    build_package(find(name)?, all_target, architecture, variant)
}

pub fn build_package(
    package: &dyn Package,
    all_target: bool,
    architecture: Option<String>,
    variant: Option<String>,
) -> Result<()> {
    let selected = select_platforms(all_target, architecture.as_deref(), variant.as_deref())?;
    for platform in selected {
        match platform.cflags() {
            Ok(flags) => build_target(package, platform, &flags)?,
            Err(error) if all_target => eprintln!("スキップ: {error}"),
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn select_platforms(
    all_target: bool,
    architecture: Option<&str>,
    variant: Option<&str>,
) -> Result<Vec<Platform>> {
    if all_target {
        return Ok(TARGETS.to_vec());
    }
    match (architecture, variant) {
        (Some(architecture), Some(variant)) => TARGETS
            .iter()
            .copied()
            .find(|platform| platform.architecture == architecture && platform.variant == variant)
            .map(|platform| vec![platform])
            .ok_or_else(|| {
                Error::message(format!("未対応のターゲットです: {architecture}/{variant}"))
            }),
        _ => Err(Error::message(
            "--architecture と --variant を指定するか、--all-target を指定してください",
        )),
    }
}

fn build_target(package: &dyn Package, platform: Platform, flags: &str) -> Result<()> {
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
    let architecture = platform.architecture;
    let variant = platform.variant;
    let build_dir = PathBuf::from("packages/build")
        .join(format!("{architecture}-{variant}"))
        .join(format!("{}-{}", package.name(), package.version()));
    let out_dir = package.artifact_dir(platform);
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
    if !package.network() {
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
        let dep_dir = dependency.artifact_dir(platform);
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
    process::status(
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
            .arg(format!("TRIPLE={}", platform.triple()?))
            .args(["-e"])
            .arg("HOST_TRIPLE=x86_64-pc-linux-gnu")
            .args(["-e"])
            .arg(format!("CROSS={}", platform.cross_prefix()?))
            .args(["-e"])
            .arg(format!("CFLAGS={flags}"))
            .arg(super::env::BUILD_IMAGE.reference())
            .args(["/bin/bash", "-c"])
            .arg(package.build_script()),
        "podman",
    )?;
    println!(
        "{} {} ({architecture}/{variant}) をビルドしました",
        package.name(),
        package.version()
    );
    Ok(())
}
