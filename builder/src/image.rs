use std::path::Path;

use clap::Subcommand;

use crate::error::{Error, Result};
use crate::oci::{self, TARGETS};
use crate::package;
use crate::sbom;
use crate::validate;

#[derive(Subcommand)]
pub enum ImageCommand {
    Ls,
    Build {
        package: Option<String>,
        #[arg(long)]
        all: bool,
        #[arg(long)]
        with_packages: bool,
        #[arg(long, conflicts_with = "zstd_level")]
        publish: bool,
        #[arg(
            long,
            value_parser = clap::value_parser!(u8).range(1..=22),
            conflicts_with = "publish"
        )]
        zstd_level: Option<u8>,
    },
    Validate {
        package: Option<String>,
        #[arg(long)]
        all: bool,
    },
}

pub fn run(command: ImageCommand) -> Result<()> {
    match command {
        ImageCommand::Ls => ls(),
        ImageCommand::Build {
            package,
            all,
            with_packages,
            publish,
            zstd_level,
        } => build(BuildArgs {
            package,
            all,
            with_packages,
            publish,
            zstd_level,
        }),
        ImageCommand::Validate { package, all } => validate::run(package, all),
    }
}

struct BuildArgs {
    package: Option<String>,
    all: bool,
    with_packages: bool,
    publish: bool,
    zstd_level: Option<u8>,
}

fn ls() -> Result<()> {
    let packages: Vec<_> = package::images().collect();
    if packages.is_empty() {
        println!("まだ定義されたパッケージイメージはありません。");
        return Ok(());
    }
    for package in packages {
        println!("{}\t{}", package.name(), package.version());
    }
    Ok(())
}

fn build(args: BuildArgs) -> Result<()> {
    let zstd_level = oci::resolve_zstd_level(args.publish, args.zstd_level)?;
    let targets = match args.package {
        Some(name) => vec![package::find_image(&name)?],
        None if args.all => package::images().collect(),
        None => return Err(Error::message("パッケージ名か --all を指定してください")),
    };
    if targets.is_empty() {
        println!("まだ定義されたパッケージイメージはありません。");
        return Ok(());
    }
    for package in targets {
        if args.with_packages {
            package::build_package(package, true, None, None)?;
        }
        let mut layers = Vec::new();
        for platform in TARGETS {
            let dir = package.artifact_dir(*platform);
            if dir.is_dir() {
                layers.push((*platform, vec![dir]));
            } else {
                eprintln!("スキップ(パッケージなし): {}", dir.display());
            }
        }
        if layers.is_empty() {
            return Err(Error::message(format!(
                "パッケージがありません: {} (package build を実行してください)",
                package.name()
            )));
        }
        let output =
            Path::new("oci-layouts").join(format!("{}-{}", package.name(), package.version()));
        let image = oci::Image {
            name: package.name().to_string(),
            version: package.version().to_string(),
        };
        oci::build(&output, &image, &layers, zstd_level)?;
        crate::util::write(&output.join("sbom.spdx.json"), &sbom::json(package)?)?;
        println!(
            "{} をビルドしました (zstd {zstd_level}): {}",
            image.name,
            output.display()
        );
    }
    Ok(())
}
