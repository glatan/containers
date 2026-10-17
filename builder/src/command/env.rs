use clap::Subcommand;

use crate::error::Result;
use crate::package::build::build_env_ready;
use crate::package::{self, env as seed};
use crate::target::Target;
use crate::workspace;

use super::Compression;

#[derive(Subcommand)]
pub enum EnvCommand {
    /// 合成済みビルド環境の参照を表示します
    Status,
    /// ブートストラップ集合をシード上でビルドします
    Bootstrap,
    /// ブートストラップの計画を表示します
    Plan {
        /// パッケージ名
        package: Option<String>,
    },
    /// ビルド環境イメージを合成します
    Compose {
        #[command(flatten)]
        compression: Compression,
    },
}

pub fn run(command: EnvCommand) -> Result<()> {
    match command {
        EnvCommand::Status => status(),
        EnvCommand::Bootstrap => package::build_bootstrap(|line| println!("{line}")),
        EnvCommand::Plan { package } => plan(package.as_deref()),
        EnvCommand::Compose { compression } => compose(&compression),
    }
}

fn status() -> Result<()> {
    if build_env_ready() {
        let current =
            workspace::compose_current_link(workspace::BUILD_ENV_NAME, workspace::BUILD_ENV_ARCH);
        let path = std::fs::canonicalize(&current)
            .map_err(|source| crate::error::Error::io(&current, source))?;
        println!("正本: oci:{}", path.display());
    } else {
        println!("正本: 未合成 (env compose で合成してください)");
    }
    println!("シード名: {}", seed::BUILD_IMAGE.name);
    println!("シードタグ: {}", seed::BUILD_IMAGE.tag);
    println!("digest: {}", seed::BUILD_IMAGE.digest);
    println!("シード参照: {}", seed::BUILD_IMAGE.reference());
    Ok(())
}

fn plan(name: Option<&str>) -> Result<()> {
    let lookup = |name: &str| package::find(name);
    let requested = name.map(package::find).transpose()?;
    let bootstrap_root = match requested {
        Some(defined) if defined.as_bootstrap().is_some() => defined,
        _ => package::find(workspace::BUILD_ENV_NAME)?,
    };
    let order =
        package::build::packages_in_graph_order(bootstrap_root, &lookup, Target::BUILD_ENV)?;
    let seed: Vec<_> = order
        .iter()
        .filter(|(defined, _)| !defined.compose_root())
        .collect();
    println!(
        "Phase 1: シード（Arch base-devel）上でビルド ({} 件)",
        seed.len()
    );
    for (index, (defined, _)) in seed.iter().enumerate() {
        println!("{:>3}. {} {}", index + 1, defined.name(), defined.version());
    }
    let Some(root) = requested else {
        println!("Phase 2: env compose");
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
    println!("Phase 2: env compose");
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

fn compose(compression: &Compression) -> Result<()> {
    let active = workspace::active_lock()?;
    let _active = active
        .read()
        .map_err(|source| crate::error::Error::io(workspace::active_lock_path(), source))?;
    let level = super::resolve_zstd_level(compression)?;
    let defined = package::find(workspace::BUILD_ENV_NAME)?;
    let (output, tag) = crate::image::compose(defined, Target::BUILD_ENV, level)?;
    println!("{tag} を合成しました (zstd {level}): {}", output.display());
    Ok(())
}
