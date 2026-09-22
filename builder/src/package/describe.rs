//! `Package` の構造的情報の収集。
//!
//! `docs`（markdown 生成）と `query`（端末表示）の共通ソース。
//! 文書は評価済み Recipe と trait メタデータの射影であり、
//! 解釈ロジックはここに集約する。

use std::collections::BTreeMap;

use crate::flags::{self, Cflag, Ldflag, Pgo};
use crate::package::Package;
use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step};
use crate::target::Target;

/// `supports()` を通る target の一覧（`Target::all()` 順）。
pub(crate) fn supported_targets(defined: &dyn Package) -> Vec<Target> {
    Target::all().filter(|t| defined.supports(*t)).collect()
}

/// 対応ターゲットの一覧ラベル（`amd64: v1 v4; arm64: v8 …` 形式）。
pub(crate) fn supported_label(defined: &dyn Package) -> String {
    let mut grouped: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
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

/// target 集合のコンパクト表示。`universe`（評価対象集合）全体なら
/// 「常時」、arch の全 variant が揃うときは `arch`、部分的なら
/// `arch/variant` を列挙する。
pub(crate) fn target_set_label(targets: &[Target], universe: &[Target]) -> String {
    if !universe.is_empty() && universe.iter().all(|t| targets.contains(t)) {
        return "常時".to_string();
    }
    let mut all: BTreeMap<&str, usize> = BTreeMap::new();
    let mut present: BTreeMap<&str, Vec<Target>> = BTreeMap::new();
    for target in universe {
        *all.entry(target.architecture()).or_default() += 1;
        if targets.contains(target) {
            present
                .entry(target.architecture())
                .or_default()
                .push(*target);
        }
    }
    present
        .iter()
        .map(|(arch, targets)| {
            if targets.len() == all[arch] {
                (*arch).to_string()
            } else {
                targets
                    .iter()
                    .map(|t| format!("{arch}/{}", t.variant()))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// `flags(target)` と `tool_flags(target)` のトークン差分（展開済み文字列）。
pub(crate) struct FlagDiff {
    /// 既定に無い追加トークン（CFLAGS）。
    pub added_cflags: Vec<String>,
    /// 既定から除外したトークン（CFLAGS）。
    pub removed_cflags: Vec<String>,
    /// 既定に無い追加トークン（LDFLAGS）。
    pub added_ldflags: Vec<String>,
    /// 既定から除外したトークン（LDFLAGS）。
    pub removed_ldflags: Vec<String>,
    /// `flags()` が空（CFLAGS/LDFLAGS とも）。
    pub empty: bool,
    /// 既定セットと同一。
    pub same: bool,
}

pub(crate) fn flag_diff(defined: &dyn Package, target: Target) -> FlagDiff {
    let flags = defined.flags(target);
    let default = crate::flags::tool_flags(target);
    let expand_c = |f: &crate::flags::Cflag| -> String { Vec::<String>::from(*f).join(" ") };
    let expand_l = |f: &crate::flags::Ldflag| -> String { Vec::<String>::from(*f).join(" ") };
    let added_cflags: Vec<String> = flags
        .cflags()
        .iter()
        .filter(|f| !default.cflags().contains(f))
        .map(expand_c)
        .collect();
    let removed_cflags: Vec<String> = default
        .cflags()
        .iter()
        .filter(|f| !flags.cflags().contains(f))
        .map(expand_c)
        .collect();
    let added_ldflags: Vec<String> = flags
        .ldflags()
        .iter()
        .filter(|f| !default.ldflags().contains(f))
        .map(expand_l)
        .collect();
    let removed_ldflags: Vec<String> = default
        .ldflags()
        .iter()
        .filter(|f| !flags.ldflags().contains(f))
        .map(expand_l)
        .collect();
    FlagDiff {
        empty: flags.cflags().is_empty() && flags.ldflags().is_empty(),
        same: added_cflags.is_empty()
            && removed_cflags.is_empty()
            && added_ldflags.is_empty()
            && removed_ldflags.is_empty(),
        added_cflags,
        removed_cflags,
        added_ldflags,
        removed_ldflags,
    }
}

/// configure 系呼出（`Step::Configure`/`Cmake`/`Meson`/`ConfigureScript`）
/// の 1 回分の呼出を全 supported target の評価を通じて集約したもの。
/// `Step::Configure` 等の引数を全 supported target で評価し、
/// 「引数 → 出現 target 集合」へ集約する（先出順を保持）。
#[derive(Debug)]
pub(crate) struct ConfigureInvocation {
    /// 呼出ラベル（`autotools_configure`、`"${SOURCE}/configure"`、
    /// `cmake -S "${SOURCE}" -B "${BUILD}/gen"`、`meson setup` 等）。
    pub label: String,
    /// 囲む `If` の実行時条件（then は `cond`、else は `! cond`）。
    pub conds: Vec<String>,
    /// env 代入のレンダリング（`NAME=value`）。
    pub env: Vec<String>,
    /// この呼出が出現する target（評価順を保持）。
    pub targets: Vec<Target>,
    /// 引数 → 出現 target 集合（先出順を保持）。
    pub args: Vec<(String, Vec<Target>)>,
}

/// 全 supported target で `recipe()` を評価し、configure 系呼出を
/// 呼出単位（ラベル・囲み条件・env が同一のものを同一呼出とみなす）で
/// 集約する。PGO の gen/use 系統や `If` の then/else 分岐は別呼出になる。
pub(crate) fn configure_invocations(defined: &dyn Package) -> Vec<ConfigureInvocation> {
    let mut merged: Vec<ConfigureInvocation> = Vec::new();
    for target in supported_targets(defined) {
        let ctx = BuildContext::of(defined.gnu_triple(), target);
        let Recipe::Steps(steps) = defined.recipe(&ctx) else {
            continue;
        };
        let mut found: Vec<(String, Vec<String>, Vec<String>, Vec<String>)> = Vec::new();
        collect_invocations(&steps.steps, &[], &mut found);
        for (label, conds, env, args) in found {
            let inv = match merged
                .iter_mut()
                .find(|inv| inv.label == label && inv.conds == conds && inv.env == env)
            {
                Some(inv) => inv,
                None => {
                    merged.push(ConfigureInvocation {
                        label,
                        conds,
                        env,
                        targets: Vec::new(),
                        args: Vec::new(),
                    });
                    merged.last_mut().unwrap()
                }
            };
            inv.targets.push(target);
            for arg in args {
                match inv.args.iter_mut().find(|(a, _)| *a == arg) {
                    Some((_, targets)) => {
                        if !targets.contains(&target) {
                            targets.push(target);
                        }
                    }
                    None => inv.args.push((arg, vec![target])),
                }
            }
        }
    }
    merged
}

/// 評価済み recipe から configure 系呼出を拾い、
/// `(ラベル, 囲み条件, env, 引数)` の列に展開する。
fn collect_invocations(
    steps: &[Step],
    conds: &[String],
    out: &mut Vec<(String, Vec<String>, Vec<String>, Vec<String>)>,
) {
    for step in steps {
        match step {
            Step::Configure(args) => {
                out.push((
                    "autotools_configure".to_string(),
                    conds.to_vec(),
                    Vec::new(),
                    args.clone(),
                ));
            }
            Step::Cmake { .. } | Step::Meson { .. } | Step::ConfigureScript { .. } => {
                let (env, pipe) = step.configure_pipe().unwrap();
                let argv = &pipe.stages[0];
                let (label, args) = match step {
                    // argv = cmake -S <src> -B <dir> <args…>
                    Step::Cmake { .. } => (
                        argv[..5]
                            .iter()
                            .map(Arg::render)
                            .collect::<Vec<_>>()
                            .join(" "),
                        &argv[5..],
                    ),
                    // argv = meson setup "${SOURCE}" <args…>
                    Step::Meson { .. } => ("meson setup".to_string(), &argv[3..]),
                    Step::ConfigureScript { program, .. } => (
                        program
                            .iter()
                            .map(Arg::render)
                            .collect::<Vec<_>>()
                            .join(" "),
                        &argv[program.len()..],
                    ),
                    _ => unreachable!(),
                };
                out.push((
                    label,
                    conds.to_vec(),
                    env.iter()
                        .map(|(name, value)| format!("{name}={}", value.render()))
                        .collect(),
                    args.iter().map(Arg::render).collect(),
                ));
            }
            Step::If { cond, then, else_ } => {
                let mut then_conds = conds.to_vec();
                then_conds.push(cond.render());
                collect_invocations(then, &then_conds, out);
                if !else_.is_empty() {
                    let mut else_conds = conds.to_vec();
                    else_conds.push(format!("! {}", cond.render()));
                    collect_invocations(else_, &else_conds, out);
                }
            }
            Step::ForEach { body, .. } | Step::Subshell(body) => {
                collect_invocations(body, conds, out)
            }
            Step::Or { lhs, rhs } => {
                collect_invocations(lhs, conds, out);
                collect_invocations(rhs, conds, out);
            }
            _ => {}
        }
    }
}

/// `Step::Exec` 内に configure 相当の呼出（`*configure*`・`meson setup`・
/// `cmake` の `-D`/`-S` 付き）があるか。`Step::Configure` を持たない
/// パッケージのフラグ表を「未転記」と注記するために使う。
/// 全 supported target で評価し、If/ForEach/Subshell/Or の内部も再帰的に見る。
pub(crate) fn has_untyped_configure(defined: &dyn Package) -> bool {
    supported_targets(defined).iter().any(|&target| {
        let ctx = BuildContext::of(defined.gnu_triple(), target);
        match defined.recipe(&ctx) {
            Recipe::Steps(steps) => steps_contain_configure(&steps.steps),
            Recipe::Noop => false,
        }
    })
}

fn steps_contain_configure(steps: &[Step]) -> bool {
    steps.iter().any(|step| match step {
        Step::Configure(_) => true,
        Step::Exec { pipe, .. } => is_configure_pipe(pipe),
        Step::If { then, else_, .. } => {
            steps_contain_configure(then) || steps_contain_configure(else_)
        }
        Step::ForEach { body, .. } | Step::Subshell(body) => steps_contain_configure(body),
        Step::Or { lhs, rhs } => steps_contain_configure(lhs) || steps_contain_configure(rhs),
        _ => false,
    })
}

fn is_configure_pipe(pipe: &Pipe) -> bool {
    let argv = &pipe.stages[0];
    let Some(cmd) = argv.first().map(Arg::render) else {
        return false;
    };
    let base = cmd
        .trim_matches('"')
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if base.contains("configure") {
        return true;
    }
    match base.as_str() {
        "meson" => argv.get(1).map(Arg::render).as_deref() == Some("setup"),
        "cmake" => argv.iter().skip(1).any(|arg| {
            let text = arg.render();
            text.starts_with("-D") || text.starts_with("-S")
        }),
        _ => false,
    }
}

/// 全 supported target で `rendered_script` を評価し、同一スクリプトの
/// target をグルーピングする（`Target::all()` 順、先出順を保持）。
/// BUILD_ENV を含む系統があれば先頭に並べる。
/// 戻り値は `(target 集合, スクリプト)` のリスト。
pub(crate) fn script_variants(defined: &dyn Package) -> Vec<(Vec<Target>, String)> {
    let mut variants: Vec<(Vec<Target>, String)> = Vec::new();
    for target in supported_targets(defined) {
        let script = defined.rendered_script(target);
        match variants.iter_mut().find(|(_, s)| *s == script) {
            Some((targets, _)) => targets.push(target),
            None => variants.push((vec![target], script)),
        }
    }
    if let Some(index) = variants
        .iter()
        .position(|(targets, _)| targets.contains(&Target::BUILD_ENV))
    {
        let first = variants.remove(index);
        variants.insert(0, first);
    }
    variants
}

/// recipe の概要（`info` 表示用）。
/// PGO（`-fprofile-generate` を含む手順）が実際に発火する target 集合。
/// レンダリング済みスクリプトへの出現で判定するため、テンプレートの
/// `!cross()` ゲートだけでなく `Pgo::Custom` のレシピ内条件
/// （`cross()`/`canadian()`/target 指定）も実値で反映される。
pub(crate) fn pgo_targets(defined: &dyn Package) -> Vec<Target> {
    if matches!(defined.pgo(), crate::flags::Pgo::None) {
        return Vec::new();
    }
    supported_targets(defined)
        .into_iter()
        .filter(|t| defined.rendered_script(*t).contains("profile-generate"))
        .collect()
}

pub(crate) fn recipe_summary(recipe: &Recipe) -> String {
    use crate::package::recipe::{Env, PgoTemplate, Step, Workdir};
    match recipe {
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
                    Step::Cmake { build_dir, .. } => {
                        format!("cmake({})", build_dir.render())
                    }
                    Step::Meson { .. } => "meson-setup".to_owned(),
                    Step::ConfigureScript { program, .. } => format!(
                        "configure-script({})",
                        program
                            .iter()
                            .map(Arg::render)
                            .collect::<Vec<_>>()
                            .join(" ")
                    ),
                    Step::Make { .. } => "make".to_owned(),
                    Step::MakeInstall { .. } => "make-install".to_owned(),
                    Step::Install { .. } => "install".to_owned(),
                    Step::Symlink { .. } => "symlink".to_owned(),
                    Step::Exec { pipe, .. } => format!("exec({})", pipe_label(pipe)),
                    Step::Cd(_) => "cd".to_owned(),
                    Step::Export { name, .. } => format!("export({name})"),
                    Step::ExportBare(name) => format!("export({name})"),
                    Step::Set { name, .. } => format!("set({name})"),
                    Step::Unset(name) => format!("unset({name})"),
                    Step::If { .. } => "if".to_owned(),
                    Step::ForEach { var, .. } => format!("for({var})"),
                    Step::Or { .. } => "or".to_owned(),
                    Step::WriteFile { .. } => "write-file".to_owned(),
                    Step::Subshell(_) => "subshell".to_owned(),
                    Step::Comment(_) => "comment".to_owned(),
                })
                .collect();
            let pgo = match steps.pgo {
                Some(PgoTemplate::Autotools(_)) => "autotools",
                Some(PgoTemplate::Make(_)) => "make",
                None => "none",
            };
            format!(
                "steps(env={env} std={} sysroot={} workdir={workdir} pgo={pgo}): {}",
                steps.std.as_str(),
                steps.sysroot,
                kinds.join(" ")
            )
        }
    }
}

fn pipe_label(pipe: &crate::package::recipe::Pipe) -> String {
    pipe.stages
        .iter()
        .map(|argv| {
            argv.first()
                .map(|arg| format!("{arg:?}"))
                .unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join("|")
}

/// `BUILD_ENV`（amd64/v4）で評価した recipe の env プリリュード。
/// `Recipe::Noop`（compose_root）では `None`。
pub(crate) fn recipe_env(defined: &dyn Package) -> Option<Env> {
    let ctx = BuildContext::of(defined.gnu_triple(), Target::BUILD_ENV);
    match defined.recipe(&ctx) {
        Recipe::Noop => None,
        Recipe::Steps(steps) => Some(steps.env),
    }
}

/// PGO ポリシーの表示ラベル。`Profiled` は `Steps::pgo` の
/// テンプレート種別まで分解する（`BUILD_ENV` で評価）。
pub(crate) fn pgo_label(defined: &dyn Package) -> String {
    use crate::package::recipe::PgoTemplate;
    match defined.pgo() {
        Pgo::None => "None".to_string(),
        Pgo::Custom => "Custom".to_string(),
        Pgo::Profiled => {
            let ctx = BuildContext::of(defined.gnu_triple(), Target::BUILD_ENV);
            let template = match defined.recipe(&ctx) {
                Recipe::Steps(steps) => match steps.pgo {
                    Some(PgoTemplate::Autotools(_)) => "A（autotools）",
                    Some(PgoTemplate::Make(_)) => "B（in-source make）",
                    None => "テンプレートなし",
                },
                Recipe::Noop => "テンプレートなし",
            };
            format!("Profiled（{template}）")
        }
    }
}

/// Host tools 集合: 全対象 Platform を support し、toolchain 構成物でも
/// カタログでも合成ルートでもビルド専用でもないパッケージ。
/// （qemu は amd64/v4 のみ対応のためここに含まれない。）
pub(crate) fn host_tools() -> Vec<&'static dyn Package> {
    crate::package::PACKAGES
        .iter()
        .copied()
        .filter(|p| {
            p.gnu_triple().is_none()
                && !p.compose_root()
                && !p.catalog()
                && !p.build_only()
                && Target::all().all(|t| p.supports(t))
        })
        .collect()
}

/// パッケージ名の一覧表示。`{triple}-` プレフィックス付き toolchain
/// 構成物は `{triple}-<suffix>` に畳む。
pub(crate) fn package_list_label(packages: &[&dyn Package]) -> String {
    let mut plain: Vec<&str> = Vec::new();
    let mut tripled: Vec<String> = Vec::new();
    for p in packages {
        match p.gnu_triple() {
            Some(triple) => {
                let suffix = p
                    .name()
                    .strip_prefix(&format!("{triple}-"))
                    .unwrap_or(p.name());
                let folded = format!("{{triple}}-{suffix}");
                if !tripled.contains(&folded) {
                    tripled.push(folded);
                }
            }
            None => plain.push(p.name()),
        }
    }
    plain.extend(tripled.iter().map(String::as_str));
    plain.join(" / ")
}

/// `tool_cflags` の全トークン行（初出順）。各行は
/// `(表記名, 展開, 種別, 対象ラベル)`。展開は `March` のみ variant 依存の
/// ためプレースホルダ表記、それ以外は実フラグ文字列。
pub(crate) fn tool_cflag_rows() -> Vec<(&'static str, String, &'static str, String)> {
    let universe: Vec<Target> = Target::all().collect();
    // (正規化トークン, 出現 target 集合) の初出順リスト
    let mut seen: Vec<(Cflag, Vec<Target>)> = Vec::new();
    for target in &universe {
        for flag in flags::tool_cflags(*target) {
            // March は payload（評価 target）で正規化して集合化する
            let key = match flag {
                Cflag::March(_) => Cflag::March(Target::BUILD_ENV),
                _ => flag,
            };
            match seen.iter_mut().find(|(f, _)| *f == key) {
                Some((_, targets)) => targets.push(*target),
                None => seen.push((key, vec![*target])),
            }
        }
    }
    seen.iter()
        .map(|(flag, targets)| {
            let expansion = match flag {
                Cflag::March(_) => "`-march=<variant>`（ppc64le は `-mcpu=`）".to_string(),
                _ => Vec::<String>::from(*flag)
                    .iter()
                    .map(|f| format!("`{f}`"))
                    .collect::<Vec<_>>()
                    .join(" "),
            };
            (
                flag.name(),
                expansion,
                flag.kind(),
                target_set_label(targets, &universe),
            )
        })
        .collect()
}

/// `tool_ldflags` の全トークン行（初出順）。各行は
/// `(表記名, 展開, 対象ラベル)`。
pub(crate) fn tool_ldflag_rows() -> Vec<(&'static str, String, String)> {
    let universe: Vec<Target> = Target::all().collect();
    let mut seen: Vec<(Ldflag, Vec<Target>)> = Vec::new();
    for target in &universe {
        for flag in flags::tool_ldflags(*target) {
            match seen.iter_mut().find(|(f, _)| *f == flag) {
                Some((_, targets)) => targets.push(*target),
                None => seen.push((flag, vec![*target])),
            }
        }
    }
    seen.iter()
        .map(|(flag, targets)| {
            (
                flag.name(),
                Vec::<String>::from(*flag)
                    .iter()
                    .map(|f| format!("`{f}`"))
                    .collect::<Vec<_>>()
                    .join(" "),
                target_set_label(targets, &universe),
            )
        })
        .collect()
}

/// 既定セットに入らない opt-in トークン（`ALL` − 全 target の既定集合）。
/// 戻り値は `(cflags, ldflags)`。
pub(crate) fn opt_in_flags() -> (Vec<Cflag>, Vec<Ldflag>) {
    let default_c: Vec<Cflag> = Target::all()
        .flat_map(flags::tool_cflags)
        .map(|f| match f {
            Cflag::March(_) => Cflag::March(Target::BUILD_ENV),
            _ => f,
        })
        .collect();
    let default_l: Vec<Ldflag> = Target::all().flat_map(flags::tool_ldflags).collect();
    (
        Cflag::ALL
            .iter()
            .copied()
            .filter(|f| !default_c.contains(f))
            .collect(),
        Ldflag::ALL
            .iter()
            .copied()
            .filter(|f| !default_l.contains(f))
            .collect(),
    )
}

/// ツールチェーンのステージ種別（表示順）。
pub(crate) const TOOLCHAIN_STAGES: &[&str] = &[
    "binutils",
    "gcc-stage1",
    "glibc-headers",
    "gcc-stage2",
    "glibc",
    "gcc",
];

/// `PACKAGES` に現れる GNU triple の一覧（初出順）。
pub(crate) fn gnu_triples() -> Vec<&'static str> {
    let mut triples = Vec::new();
    for p in crate::package::PACKAGES {
        if let Some(triple) = p.gnu_triple()
            && !triples.contains(&triple)
        {
            triples.push(triple);
        }
    }
    triples
}

/// `{triple}-{suffix}` のパッケージを探す。
fn toolchain_package(triple: &str, suffix: &str) -> Option<&'static dyn Package> {
    let name = format!("{triple}-{suffix}");
    crate::package::PACKAGES
        .iter()
        .copied()
        .find(|p| p.name() == name)
}

/// recipe 内の最初の configure 系呼出の引数（program を除く）。
/// `If`/`Subshell` 等のネストも先出順で探索する。
fn first_configure_args(steps: &[Step]) -> Option<Vec<String>> {
    for step in steps {
        let found = match step {
            Step::Configure(args) => Some(args.clone()),
            Step::Cmake { args, .. } | Step::Meson { args, .. } => {
                Some(args.iter().map(Arg::render).collect())
            }
            Step::ConfigureScript { args, .. } => Some(args.iter().map(Arg::render).collect()),
            Step::If { then, else_, .. } => {
                first_configure_args(then).or_else(|| first_configure_args(else_))
            }
            Step::ForEach { body, .. } | Step::Subshell(body) => first_configure_args(body),
            Step::Or { lhs, rhs } => {
                first_configure_args(lhs).or_else(|| first_configure_args(rhs))
            }
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

/// GNU triple の architecture 部分を `Target::architecture()` の
/// 表記へ対応させる。
fn triple_arch(triple: &str) -> &str {
    match triple.split('-').next().unwrap_or("") {
        "x86_64" => "amd64",
        "aarch64" => "arm64",
        "powerpc64le" => "ppc64le",
        other => other,
    }
}

/// `{triple}-{suffix}` 各パッケージの主 configure 呼出引数を評価し、
/// 引数ベクタが完全一致する triple でグルーピングする。
/// 評価 target は triple の arch と一致する最初の supported target
/// （製品キューの同 arch ケース）を優先し、無ければ先頭 target。
/// 戻り値は `(引数列, triple ラベル)` の初出順。
pub(crate) fn toolchain_configure_groups(suffix: &str) -> Vec<(Vec<String>, String)> {
    let triples = gnu_triples();
    let mut groups: Vec<(Vec<String>, Vec<&str>)> = Vec::new();
    for triple in &triples {
        let Some(defined) = toolchain_package(triple, suffix) else {
            continue;
        };
        let supported = supported_targets(defined);
        let Some(&target) = supported
            .iter()
            .find(|t| t.architecture() == triple_arch(triple))
            .or_else(|| supported.first())
        else {
            continue;
        };
        let ctx = BuildContext::of(defined.gnu_triple(), target);
        let Recipe::Steps(steps) = defined.recipe(&ctx) else {
            continue;
        };
        let Some(args) = first_configure_args(&steps.steps) else {
            continue;
        };
        match groups.iter_mut().find(|(a, _)| *a == args) {
            Some((_, ts)) => ts.push(triple),
            None => groups.push((args, vec![triple])),
        }
    }
    groups
        .into_iter()
        .map(|(args, ts)| {
            let label = if ts.len() == triples.len() {
                "全 triple".to_string()
            } else {
                ts.iter()
                    .map(|t| format!("`{t}`"))
                    .collect::<Vec<_>>()
                    .join(" ")
            };
            (args, label)
        })
        .collect()
}

/// ツールチェーン 1 ステージ種別の `supports` ラベル。
/// 全 triple で共通の形（全 target / HOST のみ / 自 arch 全 variant）を
/// 期待し、それに合わない場合は代表パッケージの列挙にフォールバック。
pub(crate) fn toolchain_support_label(suffix: &str) -> String {
    let Some(first) = gnu_triples()
        .iter()
        .find_map(|t| toolchain_package(t, suffix))
    else {
        return "—".to_string();
    };
    let supported = supported_targets(first);
    let universe: Vec<Target> = Target::all().collect();
    if supported.len() == universe.len() {
        return "`Target::all()`".to_string();
    }
    if supported == [Target::BUILD_ENV] {
        return "HOST（amd64 / v4）のみ".to_string();
    }
    // パッケージ triple の architecture の全 variant か
    if let Some(triple) = first.gnu_triple() {
        let arch = triple_arch(triple);
        let own: Vec<Target> = universe
            .iter()
            .copied()
            .filter(|t| t.architecture() == arch)
            .collect();
        if supported == own {
            return "その architecture の全 variant".to_string();
        }
    }
    supported_label(first)
}

/// ツールチェーン 1 ステージ種別の行
/// `(folded 名, バージョン, イメージ化ラベル)`。
pub(crate) fn toolchain_stage_row(suffix: &str) -> Option<(String, &'static str, &'static str)> {
    let first = gnu_triples()
        .iter()
        .find_map(|t| toolchain_package(t, suffix))?;
    let image = if first.build_only() {
        "対象外（ビルド専用中間ステージ）"
    } else {
        "対象"
    };
    Some((format!("`{{triple}}-{suffix}`"), first.version(), image))
}

/// 上流ソースの行 `(上流名, version, url, subdir, sha3, blake3)`。
/// 代表パッケージ（x86_64 系）から取得する。
pub(crate) fn upstream_rows() -> Vec<(&'static str, Source6)> {
    let reps = [
        ("binutils", "x86_64-linux-gnu-binutils"),
        ("gcc", "x86_64-linux-gnu-gcc"),
        ("glibc", "x86_64-linux-gnu-glibc"),
    ];
    reps.iter()
        .filter_map(|(label, name)| {
            let p = crate::package::PACKAGES
                .iter()
                .copied()
                .find(|p| p.name() == *name)?;
            let s = p.source();
            Some((
                *label,
                Source6 {
                    version: p.version(),
                    url: s.url,
                    subdir: s.subdir,
                    sha3: s.sha3_512,
                    blake3: s.blake3,
                },
            ))
        })
        .collect()
}

/// `upstream_rows` の行データ。
pub(crate) struct Source6 {
    pub version: &'static str,
    pub url: &'static str,
    pub subdir: &'static str,
    pub sha3: &'static str,
    pub blake3: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::{Amd64Variant, Arm64Variant, Ppc64leVariant};

    #[test]
    fn target_set_label_all_is_always() {
        let universe = Target::all().collect::<Vec<_>>();
        assert_eq!(target_set_label(&universe, &universe), "常時");
    }

    #[test]
    fn target_set_label_full_arch_collapses() {
        let universe = Target::all().collect::<Vec<_>>();
        let arm64: Vec<Target> = universe
            .iter()
            .copied()
            .filter(|t| t.architecture() == "arm64")
            .collect();
        assert_eq!(target_set_label(&arm64, &universe), "arm64");
        let two = vec![
            Target::Amd64(Amd64Variant::V4),
            Target::Arm64(Arm64Variant::V8),
        ];
        assert_eq!(target_set_label(&two, &universe), "amd64/v4, arm64/v8");
        let partial = vec![Target::Ppc64le(Ppc64leVariant::Power9)];
        assert_eq!(target_set_label(&partial, &universe), "ppc64le/power9");
    }

    #[test]
    fn configure_invocations_group_by_target_set() {
        let gmp = crate::package::find("gmp").unwrap();
        let invocations = configure_invocations(gmp);
        assert_eq!(invocations.len(), 1, "{invocations:?}");
        let args = &invocations[0].args;
        assert!(args.iter().any(|(a, _)| a == "--prefix=/usr"));
        let abi: Vec<&(String, Vec<Target>)> =
            args.iter().filter(|(a, _)| a.starts_with("ABI=")).collect();
        // ABI=64 / mode64 / standard が target 別に分かれる
        assert!(abi.len() >= 2, "{abi:?}");
        assert!(abi.iter().any(|(a, _)| a == "ABI=mode64"));
    }

    #[test]
    fn configure_invocations_split_by_call_site() {
        // zlib-ng はネイティブで gen/use の 2 系統、クロスで 1 系統の
        // cmake 呼出になる。-DCMAKE_SYSTEM_NAME=Linux はクロス専用。
        let zlib_ng = crate::package::find("zlib-ng").unwrap();
        let invocations = configure_invocations(zlib_ng);
        assert!(invocations.len() >= 2, "{invocations:?}");
        assert!(
            invocations
                .iter()
                .all(|inv| inv.label.starts_with("cmake -S")),
            "{invocations:?}"
        );
        let system_name = invocations
            .iter()
            .flat_map(|inv| inv.args.iter())
            .find(|(a, _)| a == "-DCMAKE_SYSTEM_NAME=Linux");
        let Some((_, targets)) = system_name else {
            panic!("-DCMAKE_SYSTEM_NAME=Linux が無い: {invocations:?}")
        };
        assert!(!targets.contains(&Target::BUILD_ENV), "{targets:?}");
    }

    #[test]
    fn flag_diff_same_for_default() {
        let bash = crate::package::find("bash").unwrap();
        let diff = flag_diff(bash, Target::BUILD_ENV);
        assert!(diff.same);
        assert!(!diff.empty);
    }

    #[test]
    fn flag_diff_empty_for_flagless() {
        let qemu = crate::package::find("qemu").unwrap();
        let diff = flag_diff(qemu, Target::BUILD_ENV);
        assert!(diff.empty);
    }

    #[test]
    fn script_variants_groups_identical_scripts() {
        let bash = crate::package::find("bash").unwrap();
        let variants = script_variants(bash);
        // ネイティブ系とクロス系で少なくとも 2 系統に分かれる
        assert!(variants.len() >= 2, "{} 系統", variants.len());
        let total: usize = variants.iter().map(|(t, _)| t.len()).sum();
        assert_eq!(total, supported_targets(bash).len());
    }

    #[test]
    fn untyped_configure_detected() {
        // Exec の argv0 が configure 系なら検出される（typed step への
        // 移行漏れの安全網。validate の lint も同じ形状を拒否する）。
        let steps = [Step::Exec {
            env: vec![],
            pipe: Pipe::cmd(vec![Arg::lit("./configure"), Arg::lit("--prefix=/usr")]),
        }];
        assert!(steps_contain_configure(&steps));
        // 移行済みパッケージは untyped を持たず typed 呼出が拾える。
        let openssl = crate::package::find("openssl").unwrap();
        assert!(!has_untyped_configure(openssl));
        assert!(!configure_invocations(openssl).is_empty());
        let bash = crate::package::find("bash").unwrap();
        assert!(!configure_invocations(bash).is_empty());
    }
}
