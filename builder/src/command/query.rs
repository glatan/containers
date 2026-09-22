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
        let host = join_names(&defined.host_dependencies(Target::BUILD_ENV));
        println!(
            "{}\t{}\t{}\t{}\t{}\t{}",
            defined.name(),
            defined.version(),
            describe_role(defined),
            if runtime.is_empty() {
                "なし"
            } else {
                &runtime
            },
            if build.is_empty() { "なし" } else { &build },
            if host.is_empty() { "なし" } else { &host }
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
    println!("ロール: {}", describe_role(defined));
    println!("ソース: {}", defined.source().url);
    println!("sha3-512: {}", defined.source().sha3_512);
    println!("blake3: {}", defined.source().blake3);
    println!("依存: {}", list_deps(&defined.dependencies(target)));
    println!(
        "ビルド依存: {}",
        list_deps(&defined.build_dependencies(target))
    );
    let host_deps = defined.host_dependencies(target);
    if !host_deps.is_empty() {
        println!("ホスト依存: {}", list_deps(&host_deps));
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
        crate::flags::Pgo::Profiled => println!("PGO: native"),
        crate::flags::Pgo::Custom => println!("PGO: custom"),
        crate::flags::Pgo::None => {}
    }
    let ctx = crate::package::recipe::BuildContext::of(defined.gnu_triple(), target);
    println!(
        "recipe: {}",
        crate::package::describe::recipe_summary(&defined.recipe(&ctx))
    );
    println!(
        "対応ターゲット: {}",
        crate::package::describe::supported_label(defined)
    );
    Ok(())
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
    let graph = package::build::graph_deps(defined, target);
    let edges: Vec<(&Dependency, &'static str)> = graph
        .runtime
        .iter()
        .map(|dependency| (dependency, "runtime"))
        .chain(graph.build.iter().map(|dependency| (dependency, "build")))
        .chain(graph.host.iter().map(|dependency| (dependency, "host")))
        .collect();
    let total = edges.len();
    for (index, (dependency, category)) in edges.into_iter().enumerate() {
        let last = index + 1 == total;
        let connector = if last { "└── " } else { "├── " };
        let found = package::find(dependency.name)?;
        let dep_target = package::build::artifact_target(
            found,
            if category == "host" {
                Target::BUILD_ENV
            } else {
                target
            },
        );
        let mut tags = Vec::new();
        match category {
            "build" | "host" => tags.push(category.to_string()),
            _ => {}
        }
        if let Some(tag) = via_tag(dependency.via) {
            tags.push(tag);
        }
        if dep_target != target {
            tags.push(format!(
                "{}/{}",
                dep_target.architecture(),
                dep_target.variant()
            ));
        }
        if found.as_bootstrap().is_some() {
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
) -> Result<Vec<(&'static dyn package::Package, &'static str, package::Via)>> {
    let mut dependers = Vec::new();
    for candidate in package::PACKAGES {
        let candidate_target = package::build::artifact_target(*candidate, node_target);
        let graph = package::build::graph_deps(*candidate, candidate_target);
        let mut category: Option<&'static str> = None;
        let mut via = package::Via::Ordered;
        for (deps, edge_category, edge_target) in [
            (&graph.runtime, "runtime", candidate_target),
            (&graph.build, "build", candidate_target),
            (&graph.host, "host", Target::BUILD_ENV),
        ] {
            for dependency in deps.iter() {
                if dependency.name != name {
                    continue;
                }
                let found = package::find(dependency.name)?;
                if package::build::artifact_target(found, edge_target) == node_target
                    && category.is_none()
                {
                    category = Some(edge_category);
                    via = dependency.via;
                }
            }
        }
        if let Some(category) = category {
            dependers.push((*candidate, category, via));
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
    for (index, &(depender, category, via)) in dependers.iter().enumerate() {
        let last = index + 1 == total;
        let connector = if last { "└── " } else { "├── " };
        let depender_target = package::build::artifact_target(depender, node_target);
        let mut tags = Vec::new();
        if category != "runtime" {
            tags.push(category.to_string());
        }
        if let Some(tag) = via_tag(via) {
            tags.push(tag);
        }
        if depender_target != node_target {
            tags.push(format!(
                "{}/{}",
                depender_target.architecture(),
                depender_target.variant()
            ));
        }
        if depender.as_bootstrap().is_some() {
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

/// 切断辺の表示タグ。Ordered 辺（通常の順序依存）はタグなし。
fn via_tag(via: package::Via) -> Option<String> {
    match via {
        package::Via::Ordered => None,
        package::Via::Seed(distro) => Some(if distro.is_empty() {
            "seed".to_string()
        } else {
            format!("seed:{distro}")
        }),
        package::Via::Artifact => Some("artifact".to_string()),
        package::Via::Optional => Some("optional".to_string()),
    }
}

fn list_deps(dependencies: &[Dependency]) -> String {
    if dependencies.is_empty() {
        return "なし".to_string();
    }
    dependencies
        .iter()
        .map(|dependency| {
            let tag = via_tag(dependency.via)
                .map(|tag| format!(" ({tag})"))
                .unwrap_or_default();
            format!("{} {}{}", dependency.name, dependency.version, tag)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn describe_role(defined: &dyn package::Package) -> String {
    if defined.compose_root() {
        "build-env".to_owned()
    } else if defined.catalog() {
        "catalog".to_owned()
    } else {
        "package".to_owned()
    }
}
