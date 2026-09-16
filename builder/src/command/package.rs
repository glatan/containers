use clap::Subcommand;

use crate::error::{Error, Result};
use crate::package::build::build_env_ready;
use crate::package::{self, Dependency, env};
use crate::target::Target;
use crate::workspace;

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
    Clean {
        package: Option<String>,
        #[arg(long, conflicts_with = "package")]
        all: bool,
        #[arg(long, conflicts_with_all = ["architecture", "variant"])]
        all_target: bool,
        #[arg(long, requires = "variant")]
        architecture: Option<String>,
        #[arg(long, requires = "architecture")]
        variant: Option<String>,
        #[arg(long)]
        dry_run: bool,
    },
    Tree {
        package: String,
        #[arg(long, requires = "variant")]
        architecture: Option<String>,
        #[arg(long, requires = "architecture")]
        variant: Option<String>,
    },
    BootstrapPath {
        package: Option<String>,
    },
    Info {
        package: Option<String>,
        #[arg(long, requires = "variant")]
        architecture: Option<String>,
        #[arg(long, requires = "architecture")]
        variant: Option<String>,
    },
    Env,
    Bootstrap,
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
        PackageCommand::Clean {
            package,
            all,
            all_target,
            architecture,
            variant,
            dry_run,
        } => {
            let (targets, skip_unsupported) = select_targets(all_target, architecture, variant)?;
            let mut failure = None;
            for defined in super::named_packages(package, all)? {
                if let Err(error) = clean(defined, &targets, skip_unsupported, dry_run) {
                    eprintln!("{error}");
                    if failure.is_none() {
                        failure = Some(error);
                    }
                }
            }
            failure.map_or(Ok(()), Err)
        }
        PackageCommand::Tree {
            package,
            architecture,
            variant,
        } => {
            let target = match (architecture, variant) {
                (Some(architecture), Some(variant)) => Target::lookup(&architecture, &variant)
                    .ok_or_else(|| {
                        Error::message(format!("未対応のターゲットです: {architecture}/{variant}"))
                    })?,
                _ => Target::BUILD_ENV,
            };
            tree(&package, target)
        }
        PackageCommand::BootstrapPath { package } => bootstrap_path(package.as_deref()),
        PackageCommand::Info {
            package,
            architecture,
            variant,
        } => match package {
            Some(name) => {
                let target = match (architecture, variant) {
                    (Some(architecture), Some(variant)) => Target::lookup(&architecture, &variant)
                        .ok_or_else(|| {
                            Error::message(format!(
                                "未対応のターゲットです: {architecture}/{variant}"
                            ))
                        })?,
                    _ => Target::BUILD_ENV,
                };
                info(&name, target)
            }
            None => ls(),
        },
        PackageCommand::Env => env(),
        PackageCommand::Bootstrap => package::build_bootstrap(|line| println!("{line}")),
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

fn clean(
    defined: &'static dyn package::Package,
    targets: &[Target],
    skip_unsupported: bool,
    dry_run: bool,
) -> Result<()> {
    let mut failure = None;
    for target in targets {
        if skip_unsupported && !defined.supports(*target) {
            continue;
        }
        for path in [
            workspace::artifact_dir(defined.name(), defined.version(), *target),
            workspace::build_dir(defined.name(), defined.version(), *target),
            workspace::log_path(defined.name(), defined.version(), *target),
        ] {
            if let Err(error) = clean_path(&path, dry_run) {
                eprintln!("{error}");
                if failure.is_none() {
                    failure = Some(error);
                }
            }
        }
    }
    failure.map_or(Ok(()), Err)
}

fn clean_path(path: &std::path::Path, dry_run: bool) -> Result<()> {
    let Ok(metadata) = path.symlink_metadata() else {
        return Ok(());
    };
    // シンボリックリンク自身はリンクのみを外す。リンク以外は canonical 化した
    // 実体が packages/ ツリー内に留まることを確認してから削除する。親ディレクトリが
    // シンボリックリンクの場合にツリー外を消さないための防御である。
    if !metadata.file_type().is_symlink() {
        let root =
            std::fs::canonicalize("packages").map_err(|source| Error::io("packages", source))?;
        let real = std::fs::canonicalize(path).map_err(|source| Error::io(path, source))?;
        if !real.starts_with(&root) {
            return Err(Error::message(format!(
                "ツリー外を指すため削除しません: {}",
                path.display()
            )));
        }
    }
    if dry_run {
        println!("削除予定: {}", path.display());
        return Ok(());
    }
    let result = if metadata.is_dir() {
        std::fs::remove_dir_all(path)
    } else {
        std::fs::remove_file(path)
    };
    result.map_err(|source| Error::io(path, source))?;
    println!("削除: {}", path.display());
    Ok(())
}

fn tree(name: &str, target: Target) -> Result<()> {
    let defined = package::find(name)?;
    println!(
        "{} {} ({}/{})",
        defined.name(),
        defined.version(),
        target.architecture(),
        target.variant()
    );
    let mut shown = std::collections::BTreeSet::from([defined.name()]);
    print_tree(defined, target, "", &mut shown)
}

fn print_tree(
    defined: &'static dyn package::Package,
    target: Target,
    prefix: &str,
    shown: &mut std::collections::BTreeSet<&'static str>,
) -> Result<()> {
    let (runtime, build) = package::build::graph_deps(defined, target);
    let edges: Vec<(&Dependency, bool)> = runtime
        .iter()
        .map(|dependency| (dependency, false))
        .chain(build.iter().map(|dependency| (dependency, true)))
        .collect();
    let total = edges.len();
    for (index, (dependency, is_build)) in edges.into_iter().enumerate() {
        let last = index + 1 == total;
        let connector = if last { "└── " } else { "├── " };
        let found = package::find(dependency.name)?;
        let dep_target = package::build::artifact_target(found, target);
        let mut tags = Vec::new();
        if is_build {
            tags.push("build".to_string());
        }
        if dep_target != target {
            tags.push(format!(
                "{}/{}",
                dep_target.architecture(),
                dep_target.variant()
            ));
        }
        if package::find_bootstrap(dependency.name).is_some() {
            tags.push("bootstrap".to_string());
        }
        let tags = if tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", tags.join(", "))
        };
        if shown.contains(dependency.name) {
            println!(
                "{prefix}{connector}{} {}{} (再掲)",
                dependency.name, dependency.version, tags
            );
            continue;
        }
        println!(
            "{prefix}{connector}{} {}{}",
            dependency.name, dependency.version, tags
        );
        shown.insert(dependency.name);
        let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
        print_tree(found, dep_target, &child_prefix, shown)?;
    }
    Ok(())
}

fn bootstrap_path(name: Option<&str>) -> Result<()> {
    let lookup = |name: &str| package::find(name);
    let requested = name.map(package::find).transpose()?;
    let bootstrap_root = match requested {
        Some(defined) if package::find_bootstrap(defined.name()).is_some() => defined,
        _ => package::find(workspace::BUILD_ENV_NAME)?,
    };
    let order =
        package::build::packages_in_graph_order(bootstrap_root, &lookup, Target::BUILD_ENV)?;
    let seed: Vec<_> = order
        .iter()
        .filter(|(defined, _)| defined.name() != workspace::BUILD_ENV_NAME)
        .collect();
    println!(
        "Phase 1: シード（Arch base-devel）上でビルド ({} 件)",
        seed.len()
    );
    for (index, (defined, _)) in seed.iter().enumerate() {
        println!("{:>3}. {} {}", index + 1, defined.name(), defined.version());
    }
    let Some(root) = requested else {
        println!("Phase 2: image compose build-env");
        println!("→ この集合があれば他のパッケージは合成済みビルド環境でビルドできます");
        return Ok(());
    };
    if root.name() == bootstrap_root.name() {
        return Ok(());
    }
    let seed_names: std::collections::BTreeSet<&'static str> =
        seed.iter().map(|(defined, _)| defined.name()).collect();
    let rest: Vec<_> = package::build::packages_in_graph_order(root, &lookup, Target::BUILD_ENV)?
        .into_iter()
        .filter(|(defined, _)| !seed_names.contains(defined.name()))
        .collect();
    println!("Phase 2: image compose build-env");
    println!("Phase 3: 合成済みビルド環境でビルド ({} 件)", rest.len());
    for (defined, target) in rest {
        println!(
            "    {} {} ({}/{})",
            defined.name(),
            defined.version(),
            target.architecture(),
            target.variant()
        );
    }
    Ok(())
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
        let runtime = join_names(&defined.dependencies(Target::BUILD_ENV));
        let build = join_names(&defined.build_dependencies(Target::BUILD_ENV));
        println!(
            "{}\t{}\t{runtime}\t{build}",
            defined.name(),
            defined.version()
        );
    }
    Ok(())
}

fn env() -> Result<()> {
    if build_env_ready() {
        let current =
            workspace::compose_current_link(workspace::BUILD_ENV_NAME, workspace::BUILD_ENV_ARCH);
        let path = std::fs::canonicalize(&current).map_err(|source| Error::io(&current, source))?;
        println!("正本: oci:{}", path.display());
    } else {
        println!("正本: 未合成 (image compose build-env)");
    }
    println!("シード名: {}", env::BUILD_IMAGE.name);
    println!("シードタグ: {}", env::BUILD_IMAGE.tag);
    println!("digest: {}", env::BUILD_IMAGE.digest);
    println!("シード参照: {}", env::BUILD_IMAGE.reference());
    Ok(())
}

fn info(name: &str, target: Target) -> Result<()> {
    let defined = package::find(name)?;
    println!("名前: {}", defined.name());
    println!("バージョン: {}", defined.version());
    println!("ライセンス: {}", defined.license());
    println!("ターゲット: {}/{}", target.architecture(), target.variant());
    println!("ロール: {}", describe_role(defined.role()));
    println!("ソース: {}", defined.source().url);
    println!("sha3-512: {}", defined.source().sha3_512);
    println!("blake3: {}", defined.source().blake3);
    println!("依存: {}", list_deps(&defined.dependencies(target)));
    println!(
        "ビルド依存: {}",
        list_deps(&defined.build_dependencies(target))
    );
    let host_tools = defined.host_tools(target);
    if !host_tools.is_empty() {
        println!("ホストツール: {}", host_tools.join(" "));
    }
    // ビルド設定（型付きフラグ/recipe の構造出力）
    let flags = defined.flags(target);
    let cflags = flags
        .cflags()
        .iter()
        .copied()
        .flat_map(Vec::<String>::from)
        .collect::<Vec<_>>()
        .join(" ");
    let ldflags = flags
        .ldflags()
        .iter()
        .copied()
        .flat_map(Vec::<String>::from)
        .collect::<Vec<_>>()
        .join(" ");
    if !cflags.is_empty() {
        println!("CFLAGS: {cflags}");
    }
    if !ldflags.is_empty() {
        println!("LDFLAGS: {ldflags}");
    }
    match defined.pgo() {
        crate::flags::Pgo::Native => println!("PGO: native"),
        crate::flags::Pgo::None => {}
    }
    let ctx = crate::package::recipe::BuildContext::of(defined.gnu_triple(), target);
    println!("recipe: {}", describe_recipe(&defined.recipe(&ctx)));
    Ok(())
}

fn describe_role(role: package::Role) -> String {
    match role {
        package::Role::Catalog => "catalog".to_owned(),
        package::Role::Package => "package".to_owned(),
        package::Role::LinuxHeaders => "linux-headers".to_owned(),
        package::Role::BuildEnv => "build-env".to_owned(),
        package::Role::Toolchain { triple, stage } => format!("toolchain({triple}, {stage:?})"),
    }
}

fn describe_recipe(recipe: &crate::package::recipe::Recipe) -> String {
    use crate::package::recipe::{Env, Recipe, Step, Workdir};
    match recipe {
        Recipe::Shell(_) => "shell".to_owned(),
        Recipe::Noop => "noop".to_owned(),
        Recipe::Steps(steps) => {
            let env = match steps.env {
                Env::Autotools => "autotools",
                Env::Lto => "lto",
                Env::None => "none",
            };
            let workdir = match steps.workdir {
                Workdir::Build => "build",
                Workdir::InSource => "in-source",
                Workdir::None => "none",
            };
            let kinds: Vec<String> = steps
                .steps
                .iter()
                .map(|step| match step {
                    Step::Configure(args) => format!("configure({})", args.join(" ")),
                    Step::Make { .. } => "make".to_owned(),
                    Step::MakeInstall { .. } => "make-install".to_owned(),
                    Step::Install { .. } => "install".to_owned(),
                    Step::Symlink { .. } => "symlink".to_owned(),
                    Step::Shell(_) => "shell".to_owned(),
                })
                .collect();
            format!(
                "steps(env={env} std={} sysroot={} workdir={workdir}): {}",
                steps.std.as_str(),
                steps.sysroot,
                kinds.join(" ")
            )
        }
    }
}
