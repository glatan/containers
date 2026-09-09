use std::path::PathBuf;

use clap::Subcommand;

use crate::error::{Error, Result};
use crate::oci::Platform;

mod build;
pub mod definition;
mod env;
mod fetch;

pub use build::build_package;
pub use definition::{PACKAGES, all, find, find_image, images};
pub use fetch::fetch_package;

pub trait Package: Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn license(&self) -> &'static str;
    fn build_only(&self) -> bool;
    fn network(&self) -> bool;
    fn source(&self) -> Source;
    fn dependencies(&self) -> &'static [Dependency];
    fn build_dependencies(&self) -> &'static [Dependency];
    fn build_script(&self) -> &'static str;

    fn source_dir(&self) -> PathBuf {
        PathBuf::from("packages/sources").join(self.name())
    }

    fn artifact_dir(&self, platform: Platform) -> PathBuf {
        PathBuf::from("packages/artifacts").join(format!(
            "{}-{}-{}-{}",
            self.name(),
            self.version(),
            platform.architecture,
            platform.variant
        ))
    }

    fn depends_on(&self, name: &str) -> bool {
        lists_name(self.dependencies(), name)
    }

    fn build_depends_on(&self, name: &str) -> bool {
        lists_name(self.build_dependencies(), name)
    }
}

fn lists_name(dependencies: &[Dependency], name: &str) -> bool {
    dependencies
        .iter()
        .any(|dependency| dependency.name == name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub url: &'static str,
    pub sha3_512: &'static str,
    pub blake3: &'static str,
    pub subdir: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: &'static str,
    pub version: &'static str,
}

impl Dependency {
    pub fn artifact_dir(self, platform: Platform) -> PathBuf {
        PathBuf::from("packages/artifacts").join(format!(
            "{}-{}-{}-{}",
            self.name, self.version, platform.architecture, platform.variant
        ))
    }
}

#[derive(Subcommand)]
pub enum PackageCommand {
    Ls,
    Build {
        package: Option<String>,
        #[arg(long)]
        all_target: bool,
        #[arg(long, requires = "variant")]
        architecture: Option<String>,
        #[arg(long, requires = "architecture")]
        variant: Option<String>,
    },
    Fetch {
        package: Option<String>,
        #[arg(long)]
        all: bool,
    },
    Info {
        package: Option<String>,
    },
    Env,
}

pub fn run(command: PackageCommand) -> Result<()> {
    match command {
        PackageCommand::Ls => ls(),
        PackageCommand::Build {
            package,
            all_target,
            architecture,
            variant,
        } => {
            let package =
                package.ok_or_else(|| Error::message("パッケージ名を指定してください"))?;
            build::run(&package, all_target, architecture, variant)
        }
        PackageCommand::Fetch { package, all } => fetch::run(package, all),
        PackageCommand::Info { package } => match package {
            Some(name) => info(&name),
            None => ls(),
        },
        PackageCommand::Env => {
            env::print();
            Ok(())
        }
    }
}

pub(crate) fn resolve(
    name: Option<String>,
    all_packages: bool,
) -> Result<Vec<&'static dyn Package>> {
    match name {
        Some(name) => Ok(vec![find(&name)?]),
        None if all_packages => Ok(PACKAGES.to_vec()),
        None => Err(Error::message("パッケージ名か --all を指定してください")),
    }
}

fn join_names(dependencies: &[Dependency]) -> String {
    dependencies
        .iter()
        .map(|dependency| dependency.name)
        .collect::<Vec<_>>()
        .join(", ")
}

fn list_deps(dependencies: &[Dependency]) -> String {
    if dependencies.is_empty() {
        return "なし".to_string();
    }
    dependencies
        .iter()
        .map(|dependency| format!("{} {}", dependency.name, dependency.version))
        .collect::<Vec<_>>()
        .join(", ")
}

fn ls() -> Result<()> {
    if PACKAGES.is_empty() {
        println!("まだ定義されたパッケージはありません。");
        return Ok(());
    }
    for package in PACKAGES {
        let runtime = join_names(package.dependencies());
        let build = join_names(package.build_dependencies());
        println!(
            "{}\t{}\t{runtime}\t{build}",
            package.name(),
            package.version()
        );
    }
    Ok(())
}

fn info(name: &str) -> Result<()> {
    let package = find(name)?;
    println!("名前: {}", package.name());
    println!("バージョン: {}", package.version());
    println!("ライセンス: {}", package.license());
    println!("ソース: {}", package.source().url);
    println!("sha3-512: {}", package.source().sha3_512);
    println!("blake3: {}", package.source().blake3);
    println!("依存: {}", list_deps(package.dependencies()));
    println!("ビルド依存: {}", list_deps(package.build_dependencies()));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

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
        }
        let error = crate::package::build_package(
            &Missing,
            false,
            Some("amd64".to_string()),
            Some("v1".to_string()),
        )
        .unwrap_err();
        assert!(error.to_string().contains("ソースが見つかりません"));
    }
}
