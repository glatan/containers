use std::fs;
use std::process::Command;

use crate::error::{Error, Result};
use crate::flags::Flags;
use crate::package::Package;
use crate::process;
use crate::target::Target;
use crate::workspace;

pub fn build_package(
    package: &dyn Package,
    targets: &[Target],
    skip_unsupported: bool,
) -> Result<()> {
    for target in targets {
        match Flags::resolve(*target) {
            Ok(flags) => build_target(package, *target, &flags)?,
            Err(error) if skip_unsupported => eprintln!("スキップ: {error}"),
            Err(error) => return Err(error),
        }
    }
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
    process::status(
        command
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{Dependency, Source};
    use crate::target::{Amd64Variant, Target};

    const ZERO_512: &str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn build_package_requires_fetched_source() {
        struct Missing;
        impl Package for Missing {
            fn name(&self) -> &'static str {
                "__missing_package__"
            }
            fn version(&self) -> &'static str {
                "0"
            }
            fn license(&self) -> &'static str {
                "MIT"
            }
            fn build_only(&self) -> bool {
                false
            }
            fn network(&self) -> bool {
                false
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
                &[]
            }
            fn build_dependencies(&self) -> &'static [Dependency] {
                &[]
            }
            fn build_script(&self) -> &'static str {
                "false"
            }
            fn flags(&self) -> &'static [crate::flags::Var] {
                &[]
            }
        }
        let error = build_package(&Missing, &[Target::Amd64(Amd64Variant::V1)], false).unwrap_err();
        assert!(error.to_string().contains("ソースが見つかりません"));
    }
}
