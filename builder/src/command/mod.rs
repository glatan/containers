use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::error::{Error, Result};
use crate::oci::ZstdLevel;
use crate::package::{self, Package};
use crate::target::Target;

mod docs;
mod env;
mod query;
mod registry;
mod sync;
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
    /// 定義文書の生成ブロックを PACKAGES から再生成します
    Docs {
        /// 差分のみ検査して書き込みません
        #[arg(long)]
        check: bool,
    },
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
    /// oci-layouts/ からレジストリ index を生成します
    Index {
        /// レジストリ前置詞（既定: ghcr.io または CONTAINERS_REGISTRY）
        #[arg(long)]
        registry: Option<String>,
        /// 生成せず既存の生成物との差分のみ検査します
        #[arg(long)]
        check: bool,
    },
    /// パッケージイメージと index をレジストリへ push します
    Push {
        #[command(flatten)]
        selection: PackageSelection,
        /// レジストリ前置詞（既定: ghcr.io または CONTAINERS_REGISTRY）
        #[arg(long)]
        registry: Option<String>,
        /// push せず対象のみ表示します
        #[arg(long)]
        dry_run: bool,
    },
    /// 設定ファイルに基づき Dockerfile の GENERATED ブロックを同期します
    Sync {
        /// 設定ファイル（containers.json）
        config: std::path::PathBuf,
        /// 差分のみ検査して書き込みません
        #[arg(long)]
        check: bool,
        /// 使用する index 参照（既定: CONTAINERS_INDEX / 設定の index）
        #[arg(long)]
        index: Option<String>,
        /// レジストリ前置詞（既定: ghcr.io または CONTAINERS_REGISTRY）
        #[arg(long)]
        registry: Option<String>,
        /// mode=compose の合成イメージの出力先
        #[arg(long)]
        output: Option<std::path::PathBuf>,
        /// 合成イメージを push せずローカル layout のみ生成します
        #[arg(long)]
        no_push: bool,
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
    /// コンテナ内で BuildPlan を直接実行する（podman のエントリポイント）
    #[command(hide = true)]
    RunRecipe {
        /// plan JSON のパス
        plan: std::path::PathBuf,
        /// ターゲット <arch>/<variant>
        #[arg(short = 't', long, value_name = "ARCH/VARIANT")]
        target: String,
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
        Command::Docs { check } => docs::docs(check),
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
        Command::Index { registry, check } => registry::index(registry.as_deref(), check),
        Command::Push {
            selection,
            registry,
            dry_run,
        } => registry::push(&selection, registry.as_deref(), dry_run),
        Command::Sync {
            config,
            check,
            index,
            registry,
            output,
            no_push,
            compression,
        } => sync::sync(
            &config,
            check,
            index.as_deref(),
            registry.as_deref(),
            output.as_deref(),
            no_push,
            &compression,
        ),
        Command::RunRecipe { plan, target } => run_recipe(&plan, parse_target(&target)?),
        Command::Env(command) => env::run(command),
    }
}

/// コンテナ内エントリ: plan JSON を `ContainerSandbox` で直接実行する。
/// `BuildContext` は podman `-e` で注入済みの env から再構成する
/// （`TRIPLE`/`HOST_TRIPLE`/`PACKAGE_HOST`/`CROSS` は build_target が
/// 常に渡している）。`&'static str` フィールドを復元するため
/// plan ファイルは `Box::leak` で 'static 化する。
fn run_recipe(plan: &std::path::Path, target: Target) -> Result<()> {
    use crate::package::recipe::{BuildContext, BuildPlan};
    use crate::package::runner;
    use crate::package::sandbox::ContainerSandbox;

    let json = std::fs::read_to_string(plan)
        .map_err(|e| Error::message(format!("plan が読めません {}: {e}", plan.display())))?;
    let leaked: &'static str = Box::leak(json.into_boxed_str());
    let plan: BuildPlan = serde_json::from_str(leaked)
        .map_err(|e| Error::message(format!("plan のパースに失敗: {e}")))?;
    let getenv = |name: &str| {
        std::env::var(name).map_err(|_| Error::message(format!("環境変数 {name} がありません")))
    };
    let ctx = BuildContext {
        target,
        triple: getenv("TRIPLE")?,
        host_triple: getenv("HOST_TRIPLE")?,
        package_host: getenv("PACKAGE_HOST")?,
        cross_prefix: std::env::var("CROSS").unwrap_or_default(),
    };
    let mut sandbox = ContainerSandbox::new();
    runner::run_plan(&plan, &ctx, &mut sandbox)
        .map_err(|e| Error::message(format!("recipe の実行に失敗: {e}")))
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
