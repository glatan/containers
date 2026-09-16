use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::error::{Error, Result};
use crate::oci::ZstdLevel;
use crate::package::{self, Package};
use crate::target::Target;

mod env;
mod query;
mod work;

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
    /// 定義済みパッケージの一覧を表示します
    Ls {
        /// 配布対象（ビルド専用を除く）に絞ります
        #[arg(long)]
        images: bool,
    },
    /// パッケージの情報を表示します
    Info {
        /// パッケージ名
        package: String,
        /// ターゲット <arch>/<variant>
        #[arg(short = 't', long, value_name = "ARCH/VARIANT")]
        target: String,
    },
    /// 依存ツリーを表示します
    Tree {
        /// パッケージ名
        package: String,
        /// ターゲット <arch>/<variant>
        #[arg(short = 't', long, value_name = "ARCH/VARIANT")]
        target: String,
        /// このパッケージに依存する側を表示します（逆依存）
        #[arg(long)]
        reverse: bool,
    },
    /// 有効なターゲットの一覧を表示します
    Targets,
    /// ソースアーカイブを取得します
    Fetch {
        #[command(flatten)]
        selection: PackageSelection,
        /// 依存閉包も取得します
        #[arg(long)]
        deps: bool,
    },
    /// パッケージをビルドします
    Build {
        #[command(flatten)]
        selection: PackageSelection,
        #[command(flatten)]
        targets: TargetSelection,
        /// 成果物が存在しても対象パッケージを再ビルドします
        #[arg(long)]
        force: bool,
        /// 失敗しても後続のビルドを継続します
        #[arg(long)]
        keep_going: bool,
        /// 依存をビルドせず対象パッケージのみビルドします
        #[arg(long, conflicts_with_all = ["all", "keep_going"])]
        no_deps: bool,
        /// ビルドせず計画のみ表示します
        #[arg(long)]
        dry_run: bool,
    },
    /// パッケージ成果物の動的依存を検証します
    Check {
        #[command(flatten)]
        selection: PackageSelection,
    },
    /// パッケージ成果物を OCI Image Layout に封入します
    Pack {
        #[command(flatten)]
        selection: PackageSelection,
        /// 封入前にパッケージをビルドします
        #[arg(long)]
        with_build: bool,
        #[command(flatten)]
        compression: Compression,
    },
    /// パッケージを合成して rootfs イメージを作成します
    Compose {
        /// パッケージ名
        package: String,
        /// ターゲット <arch>/<variant>（C++ ランタイム依存を持つパッケージは現状 amd64/v4 のみ合成可能）
        #[arg(short = 't', long, value_name = "ARCH/VARIANT")]
        target: String,
        /// 合成前に必要なパッケージをビルドします
        #[arg(long)]
        with_build: bool,
        #[command(flatten)]
        compression: Compression,
    },
    /// 成果物・作業ツリー・ログを削除します
    Clean {
        #[command(flatten)]
        selection: PackageSelection,
        #[command(flatten)]
        targets: TargetSelection,
        /// OCI Image Layout（oci-layouts/）も削除対象にします
        #[arg(long)]
        images: bool,
        /// 削除せず対象のみ表示します
        #[arg(long)]
        dry_run: bool,
    },
    /// ビルド環境の管理
    #[command(subcommand)]
    Env(env::EnvCommand),
}

/// `<name | --all>` のパッケージ選択。
#[derive(Args)]
#[group(required = true, multiple = false)]
struct PackageSelection {
    /// パッケージ名
    package: Option<String>,
    /// 全パッケージを対象にします
    #[arg(long)]
    all: bool,
}

/// `-t <arch>/<variant>`（複数回可）または `--all-targets` のターゲット選択。
#[derive(Args)]
#[group(required = true, multiple = false)]
struct TargetSelection {
    /// ターゲット <arch>/<variant>（複数回指定可）
    #[arg(short = 't', long, value_name = "ARCH/VARIANT")]
    targets: Vec<String>,
    /// サポートする全ターゲットを対象にします
    #[arg(long)]
    all_targets: bool,
}

/// `--profile dev|release` または `--zstd-level` の圧縮指定。
#[derive(Args)]
struct Compression {
    /// 圧縮プロファイル (dev: zstd 3, release: zstd 22 + ultra)
    #[arg(long, value_enum, conflicts_with = "zstd_level")]
    profile: Option<Profile>,
    /// zstd 圧縮レベル (1-22)
    #[arg(long, value_parser = clap::value_parser!(u8).range(1..=22))]
    zstd_level: Option<u8>,
}

#[derive(Clone, Copy, ValueEnum)]
enum Profile {
    /// 開発用 (zstd 3)
    Dev,
    /// 配布用 (zstd 22 + ultra)
    Release,
}

fn resolve_zstd_level(compression: &Compression) -> Result<u8> {
    let level = match (compression.profile, compression.zstd_level) {
        (Some(Profile::Release), _) => ZstdLevel::PUBLISH,
        (_, Some(level)) => ZstdLevel::new(level)?,
        _ => ZstdLevel::DEFAULT,
    };
    Ok(level.get())
}

pub fn run() -> Result<()> {
    match Cli::parse().command {
        Command::Ls { images } => query::ls(images),
        Command::Info { package, target } => query::info(&package, parse_target(&target)?),
        Command::Tree {
            package,
            target,
            reverse,
        } => query::tree(&package, parse_target(&target)?, reverse),
        Command::Targets => query::targets(),
        Command::Fetch { selection, deps } => work::fetch(&selection, deps),
        Command::Build {
            selection,
            targets,
            force,
            keep_going,
            no_deps,
            dry_run,
        } => work::build(&selection, &targets, force, keep_going, no_deps, dry_run),
        Command::Check { selection } => work::check(&selection),
        Command::Pack {
            selection,
            with_build,
            compression,
        } => work::pack(&selection, with_build, &compression),
        Command::Compose {
            package,
            target,
            with_build,
            compression,
        } => work::compose(&package, parse_target(&target)?, with_build, &compression),
        Command::Clean {
            selection,
            targets,
            images,
            dry_run,
        } => work::clean(&selection, &targets, images, dry_run),
        Command::Env(command) => env::run(command),
    }
}

/// `-t` の値を Target に解決する。失敗時は `targets` コマンドへの誘導を添える。
fn parse_target(text: &str) -> Result<Target> {
    Target::parse(text).ok_or_else(|| {
        Error::message(format!(
            "未対応のターゲットです: {text} (targets コマンドで有効なターゲットを確認してください)"
        ))
    })
}

/// `(Vec<Target>, skip_unsupported)` に解決する。重複した `-t` は除去する。
fn resolve_targets(selection: &TargetSelection) -> Result<(Vec<Target>, bool)> {
    if selection.all_targets {
        return Ok((Target::all().collect(), true));
    }
    let mut targets = Vec::new();
    for text in &selection.targets {
        let target = parse_target(text)?;
        if !targets.contains(&target) {
            targets.push(target);
        }
    }
    Ok((targets, false))
}

/// 選択されたパッケージを返す。`--all` 時の集合はコマンドごとに `all` で定義する。
fn selected(
    selection: &PackageSelection,
    all: impl FnOnce() -> Vec<&'static dyn Package>,
) -> Result<Vec<&'static dyn Package>> {
    match &selection.package {
        Some(name) => Ok(vec![package::find(name)?]),
        None => Ok(all()),
    }
}

/// 全件を処理し、失敗を集約して最後に非 0 で終了する `--all` 系ポリシー。
fn run_aggregated(
    packages: Vec<&'static dyn Package>,
    mut run: impl FnMut(&'static dyn Package) -> Result<()>,
) -> Result<()> {
    let mut failed = Vec::new();
    for package in packages {
        if let Err(error) = run(package) {
            eprintln!("{error}");
            failed.push(package.name());
        }
    }
    if failed.is_empty() {
        return Ok(());
    }
    for name in &failed {
        eprintln!("失敗: {name}");
    }
    Err(Error::message(format!("{} 件が失敗しました", failed.len())))
}
