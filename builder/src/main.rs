use std::process::ExitCode;

use clap::{Parser, Subcommand};

use containers_builder::{image, package};

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

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Package(command) => package::run(command),
        Command::Image(command) => image::run(command),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("エラー: {error}");
            ExitCode::FAILURE
        }
    }
}
