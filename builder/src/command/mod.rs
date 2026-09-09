use clap::{Parser, Subcommand};

use crate::error::{Error, Result};

mod image;
mod package;

#[derive(Parser)]
#[command(
    name = "containers-builder",
    version,
    about = "パッケージとパッケージイメージのビルドツール"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    #[command(subcommand)]
    Package(package::PackageCommand),
    #[command(subcommand)]
    Image(image::ImageCommand),
}

pub fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Package(command) => package::run(command),
        Command::Image(command) => image::run(command),
    }
}

fn named_packages(
    name: Option<String>,
    all: bool,
) -> Result<Vec<&'static dyn crate::package::Package>> {
    match name {
        Some(name) => Ok(vec![crate::package::find(&name)?]),
        None if all => Ok(crate::package::PACKAGES.to_vec()),
        None => Err(Error::message("パッケージ名か --all を指定してください")),
    }
}

fn named_images(
    name: Option<String>,
    all: bool,
) -> Result<Vec<&'static dyn crate::package::Package>> {
    match name {
        Some(name) => Ok(vec![crate::package::find_image(&name)?]),
        None if all => Ok(crate::package::images().collect()),
        None => Err(Error::message("パッケージ名か --all を指定してください")),
    }
}
