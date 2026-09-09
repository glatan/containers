use clap::Subcommand;

use crate::error::{Error, Result};
use crate::image;
use crate::oci::{PUBLISH_ZSTD_LEVEL, ZstdLevel};
use crate::package;

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
        } => {
            let level = resolve_zstd_level(publish, zstd_level)?;
            let packages = super::named_images(package, all)?;
            if packages.is_empty() {
                println!("まだ定義されたパッケージイメージはありません。");
                return Ok(());
            }
            for defined in packages {
                let output = image::build(defined, level, with_packages)?;
                println!(
                    "{} をビルドしました (zstd {level}): {}",
                    defined.name(),
                    output.display()
                );
            }
            Ok(())
        }
        ImageCommand::Validate { package, all } => {
            let packages = super::named_images(package, all)?;
            if packages.is_empty() {
                println!("まだ定義されたパッケージイメージはありません。");
                return Ok(());
            }
            for defined in packages {
                package::validate::check(defined)?;
                println!(
                    "{} {} の検証に成功しました",
                    defined.name(),
                    defined.version()
                );
            }
            Ok(())
        }
    }
}

fn ls() -> Result<()> {
    let packages: Vec<_> = package::images().collect();
    if packages.is_empty() {
        println!("まだ定義されたパッケージイメージはありません。");
        return Ok(());
    }
    for defined in packages {
        println!("{}\t{}", defined.name(), defined.version());
    }
    Ok(())
}

fn resolve_zstd_level(publish: bool, zstd_level: Option<u8>) -> Result<u8> {
    Ok(match (publish, zstd_level) {
        (true, None) => ZstdLevel::PUBLISH,
        (false, None) => ZstdLevel::DEFAULT,
        (true, Some(level)) if level != PUBLISH_ZSTD_LEVEL => {
            return Err(Error::message(
                "--publish の圧縮レベルは 22 です。--zstd-level と同時に指定しないでください",
            ));
        }
        (_, Some(level)) => ZstdLevel::new(level)?,
    }
    .get())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::oci;

    #[test]
    fn resolve_zstd_level_follows_publish_and_override() {
        assert_eq!(
            resolve_zstd_level(false, None).unwrap(),
            oci::DEFAULT_ZSTD_LEVEL
        );
        assert_eq!(
            resolve_zstd_level(true, None).unwrap(),
            oci::PUBLISH_ZSTD_LEVEL
        );
        assert_eq!(resolve_zstd_level(false, Some(7)).unwrap(), 7);
        assert_eq!(resolve_zstd_level(true, Some(22)).unwrap(), 22);
        assert!(resolve_zstd_level(true, Some(3)).is_err());
    }
}
