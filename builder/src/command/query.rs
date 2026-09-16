use crate::error::Result;
use crate::package::{self, Dependency};
use crate::target::Target;

pub fn ls(images: bool) -> Result<()> {
    let packages: Vec<&'static dyn package::Package> = if images {
        package::images().collect()
    } else {
        package::PACKAGES.to_vec()
    };
    if packages.is_empty() {
        println!("まだ定義されたパッケージはありません。");
        return Ok(());
    }
    for defined in packages {
        let runtime = join_names(&defined.dependencies(Target::BUILD_ENV));
        let build = join_names(&defined.build_dependencies(Target::BUILD_ENV));
        println!(
            "{}\t{}\t{}\t{}\t{}",
            defined.name(),
            defined.version(),
            describe_role(defined.role()),
            if runtime.is_empty() {
                "なし"
            } else {
                &runtime
            },
            if build.is_empty() { "なし" } else { &build }
        );
    }
    Ok(())
}

pub fn info(name: &str, target: Target) -> Result<()> {
    let defined = package::find(name)?;
    println!("名前: {}", defined.name());
    println!("バージョン: {}", defined.version());
    println!("ライセンス: {}", defined.license());
    println!("ターゲット: {}/{}", target.architecture(), target.variant());
    let artifact = package::build::artifact_target(defined, target);
    if artifact != target {
        println!(
            "成果物ターゲット: {}/{}",
            artifact.architecture(),
            artifact.variant()
        );
    }
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
    println!("対応ターゲット: {}", describe_supported(defined));
    Ok(())
}

fn describe_supported(defined: &dyn package::Package) -> String {
    let mut grouped: std::collections::BTreeMap<&str, Vec<&str>> =
        std::collections::BTreeMap::new();
    for target in Target::all() {
        if defined.supports(target) {
            grouped
                .entry(target.architecture())
                .or_default()
                .push(target.variant());
        }
    }
    if grouped.is_empty() {
        return "なし".to_string();
    }
    grouped
        .iter()
        .map(|(arch, variants)| format!("{arch}: {}", variants.join(" ")))
        .collect::<Vec<_>>()
        .join("; ")
}

pub fn tree(name: &str, target: Target, reverse: bool) -> Result<()> {
    let defined = package::find(name)?;
    println!(
        "{} {} ({}/{})",
        defined.name(),
        defined.version(),
        target.architecture(),
        target.variant()
    );
    let mut shown = std::collections::BTreeSet::from([defined.name()]);
    if reverse {
        print_reverse(defined, target, "", &mut shown)
    } else {
        print_tree(defined, target, "", &mut shown)
    }
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

/// `node_target` において `name` に依存する全パッケージを `(depender, is_build)` で
/// 返す。depender のノードターゲットは順方向と同じ `artifact_target(p, node_target)`
/// で正規化し、辺の解決先が `node_target` に一致するものだけを採用する。
fn dependers_of(
    name: &'static str,
    node_target: Target,
) -> Result<Vec<(&'static dyn package::Package, bool)>> {
    let mut dependers = Vec::new();
    for candidate in package::PACKAGES {
        let candidate_target = package::build::artifact_target(*candidate, node_target);
        let (runtime, build) = package::build::graph_deps(*candidate, candidate_target);
        let mut matched = false;
        let mut is_build = false;
        for (deps, build_edge) in [(&runtime, false), (&build, true)] {
            for dependency in deps.iter() {
                if dependency.name != name {
                    continue;
                }
                let found = package::find(dependency.name)?;
                if package::build::artifact_target(found, candidate_target) == node_target {
                    is_build |= build_edge;
                    matched = true;
                }
            }
        }
        if matched {
            dependers.push((*candidate, is_build));
        }
    }
    Ok(dependers)
}

fn print_reverse(
    defined: &'static dyn package::Package,
    node_target: Target,
    prefix: &str,
    shown: &mut std::collections::BTreeSet<&'static str>,
) -> Result<()> {
    let dependers = dependers_of(defined.name(), node_target)?;
    let total = dependers.len();
    for (index, &(depender, is_build)) in dependers.iter().enumerate() {
        let last = index + 1 == total;
        let connector = if last { "└── " } else { "├── " };
        let depender_target = package::build::artifact_target(depender, node_target);
        let mut tags = Vec::new();
        if is_build {
            tags.push("build".to_string());
        }
        if depender_target != node_target {
            tags.push(format!(
                "{}/{}",
                depender_target.architecture(),
                depender_target.variant()
            ));
        }
        if package::find_bootstrap(depender.name()).is_some() {
            tags.push("bootstrap".to_string());
        }
        let tags = if tags.is_empty() {
            String::new()
        } else {
            format!(" [{}]", tags.join(", "))
        };
        if shown.contains(depender.name()) {
            println!(
                "{prefix}{connector}{} {}{} (再掲)",
                depender.name(),
                depender.version(),
                tags
            );
            continue;
        }
        println!(
            "{prefix}{connector}{} {}{}",
            depender.name(),
            depender.version(),
            tags
        );
        shown.insert(depender.name());
        let child_prefix = format!("{prefix}{}", if last { "    " } else { "│   " });
        print_reverse(depender, depender_target, &child_prefix, shown)?;
    }
    Ok(())
}

pub fn targets() -> Result<()> {
    let mut all: Vec<Target> = Target::all().collect();
    all.sort_by_key(|target| target.architecture());
    for target in all {
        println!(
            "{}/{}\t{}\t{}",
            target.architecture(),
            target.variant(),
            target.gnu_triple(),
            target.march()
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
