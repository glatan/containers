use clap::Subcommand;

use crate::error::{Error, Result};
use crate::package::{self, Dependency, env};
use crate::target::Target;

#[derive(Subcommand)]
pub enum PackageCommand {
    Ls,
    Build {
        package: Option<String>,
        #[arg(long, conflicts_with_all = ["architecture", "variant"])]
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
            let name = package.ok_or_else(|| Error::message("パッケージ名を指定してください"))?;
            let defined = package::find(&name)?;
            let (targets, skip_unsupported) = select_targets(all_target, architecture, variant)?;
            package::build_package(defined, &targets, skip_unsupported, |line| {
                println!("{line}")
            })
        }
        PackageCommand::Fetch { package, all } => {
            for defined in super::named_packages(package, all)? {
                emit(package::fetch_package(defined)?)?;
            }
            Ok(())
        }
        PackageCommand::Info { package } => match package {
            Some(name) => info(&name),
            None => ls(),
        },
        PackageCommand::Env => {
            println!("名前: {}", env::BUILD_IMAGE.name);
            println!("タグ: {}", env::BUILD_IMAGE.tag);
            println!("digest: {}", env::BUILD_IMAGE.digest);
            println!("参照: {}", env::BUILD_IMAGE.reference());
            Ok(())
        }
    }
}

fn emit(lines: Vec<String>) -> Result<()> {
    for line in lines {
        println!("{line}");
    }
    Ok(())
}

fn select_targets(
    all_target: bool,
    architecture: Option<String>,
    variant: Option<String>,
) -> Result<(Vec<Target>, bool)> {
    if all_target {
        return Ok((Target::all().collect(), true));
    }
    match (architecture, variant) {
        (Some(architecture), Some(variant)) => Target::lookup(&architecture, &variant)
            .map(|target| (vec![target], false))
            .ok_or_else(|| {
                Error::message(format!("未対応のターゲットです: {architecture}/{variant}"))
            }),
        _ => Err(Error::message(
            "--architecture と --variant を指定するか、--all-target を指定してください",
        )),
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
    if package::PACKAGES.is_empty() {
        println!("まだ定義されたパッケージはありません。");
        return Ok(());
    }
    for defined in package::PACKAGES {
        let runtime = join_names(defined.dependencies());
        let build = join_names(defined.build_dependencies());
        println!(
            "{}\t{}\t{runtime}\t{build}",
            defined.name(),
            defined.version()
        );
    }
    Ok(())
}

fn info(name: &str) -> Result<()> {
    let defined = package::find(name)?;
    println!("名前: {}", defined.name());
    println!("バージョン: {}", defined.version());
    println!("ライセンス: {}", defined.license());
    println!("ソース: {}", defined.source().url);
    println!("sha3-512: {}", defined.source().sha3_512);
    println!("blake3: {}", defined.source().blake3);
    println!("依存: {}", list_deps(defined.dependencies()));
    println!("ビルド依存: {}", list_deps(defined.build_dependencies()));
    Ok(())
}
