//! `index` / `push` サブコマンド。
//!
//! - `index`: `oci-layouts/` を走査して index artifact を生成する
//! - `push`: パッケージイメージ（と `--all-packages` では index artifact）を
//!   skopeo 経由でレジストリへ送る

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::oci::Digest;
use crate::package::{self, Package};
use crate::registry::{self, index as registry_index, layout_io, skopeo};
use crate::target::Target;
use crate::util;
use crate::workspace;

use super::PackageSelection;

/// index artifact を `oci-layouts/index/` に生成する。
/// `--check` は既存の生成物との差分のみ検査する。
/// `--all-issues` は個別パッケージの失敗を集約して全件報告する。
pub fn index(registry: Option<&str>, check: bool, all_issues: bool) -> Result<()> {
    let registry = registry::registry_prefix(registry)?;
    let generated = registry_index::generate(&registry, all_issues)?;
    println!(
        "{} 件のパッケージを収録しました (registry: {registry})",
        generated.packages.len()
    );
    let output = workspace::index_layout_dir();
    if check {
        if !output.exists() {
            return Err(Error::message(format!(
                "index が生成されていません: {} （index サブコマンドを実行してください）",
                output.display()
            )));
        }
        let existing = registry_index::read_artifact(&output)?;
        if serde_json::to_vec(&existing)? != serde_json::to_vec(&generated)? {
            return Err(Error::message(
                "index が古くなっています（index サブコマンドで再生成してください）",
            ));
        }
        println!("index は最新です");
        return Ok(());
    }
    let tag = format!("{}-0", util::utc_date());
    let digest = registry_index::write_artifact(&generated, &output, &tag)?;
    println!(
        "index を生成しました: {} (manifest {})",
        output.display(),
        digest.prefixed()
    );
    Ok(())
}

/// パッケージイメージを push する。`--all-packages` は全イメージと index artifact。
///
/// イミュータブルタグ運用: 既存タグが同じ digest を指すなら冪等に
/// スキップし、別 digest を指すなら additive 判定（remote ⊆ local）で
/// platform 追加のみ許容する。`replace_platforms` は指定 platform の
/// 内容置換を許容（正当なリビルドの公開用）。
/// additive 不適合はパッケージ単位で集約し、適合したものだけ push
/// したうえで最後に全不適合を報告する。
pub fn push(
    selection: &PackageSelection,
    registry: Option<&str>,
    dry_run: bool,
    force_replace: bool,
    replace_platforms: &[Target],
    with_index: bool,
) -> Result<()> {
    let registry = registry::registry_prefix(registry)?;
    let all = selection.package.is_none();
    let packages: Vec<&'static dyn Package> = match &selection.package {
        Some(name) => {
            let defined = package::find(name)?;
            if defined.build_only() {
                return Err(Error::message(format!(
                    "{} はビルド専用のパッケージです",
                    defined.name()
                )));
            }
            vec![defined]
        }
        None => package::images().collect(),
    };
    // index artifact（あれば）とローカル layout の digest 照合。
    // 世代ズレ（pack 後に index を作り直していない等）をここで止める。
    let index_path = workspace::index_layout_dir();
    let published_index = if index_path.exists() {
        // index の再生成（write_artifact の rename）と競合しないよう
        // read ロックの下で読む。
        let lock = workspace::output_lock(&index_path)?;
        let _guard = lock
            .read()
            .map_err(|source| Error::io(&index_path, source))?;
        Some(registry_index::read_artifact(&index_path)?)
    } else {
        None
    };
    // index の自己記述 registry と実 push 先の不一致を拒否する。
    // 消費側は index.registry 経由でリポジトリを解決するため、
    // 別前置詞へ公開すると参照先が食い違う。
    if let Some(index) = &published_index
        && index.registry != registry
    {
        return Err(Error::message(format!(
            "index は {} 向けに生成されていますが push 先は {registry} です（--registry を合わせるか index を再生成してください）",
            index.registry
        )));
    }
    let mut targets: Vec<PushTarget> = Vec::new();
    for defined in packages {
        let Some(layout) = registry_index::layout_source(defined) else {
            if all {
                continue;
            }
            return Err(Error::message(format!(
                "{} の layout がありません（pack または env compose を実行してください）",
                defined.name()
            )));
        };
        // pack/restore の rename 確定と競合しないよう read ロックの下で読む。
        let lock = workspace::output_lock(&layout)?;
        let image = {
            let _guard = lock.read().map_err(|source| Error::io(&layout, source))?;
            layout_io::published(&layout)?
        };
        let repository = registry::package_repo(&registry, defined.name());
        let tag = push_tag(defined, &image);
        let reference = registry::tag_ref(&repository, &tag);
        match published_index
            .as_ref()
            .and_then(|index| index.find(defined.name()))
        {
            Some(entry) if entry.index_digest != image.index_digest => {
                return Err(Error::message(format!(
                    "{} の layout が index と世代ズレしています（index を再生成してください）",
                    defined.name()
                )));
            }
            Some(_) => {}
            None => {
                if published_index.is_some() {
                    eprintln!(
                        "警告: {} は index に未収録です（index を再生成すると含まれます）",
                        defined.name()
                    );
                }
            }
        }
        targets.push(PushTarget {
            name: defined.name().to_string(),
            layout,
            lock,
            repository,
            reference,
            image,
        });
    }
    if targets.is_empty() {
        println!("push 対象がありません（pack 済みの layout がありません）");
        return Ok(());
    }
    for target in &targets {
        println!("{} -> {}", target.name, target.reference);
    }
    if dry_run {
        if all {
            println!(
                "index -> {}:{{yyyymmdd}}-{{rev}}",
                registry::index_repo(&registry)
            );
        }
        return Ok(());
    }
    skopeo::require()?;
    let mut failures: Vec<String> = Vec::new();
    for target in &mut targets {
        // skopeo copy が layout を読む間も pack の rename と競合しない
        // よう read ロックを保持する。
        let _guard = target
            .lock
            .read()
            .map_err(|source| Error::io(&target.layout, source))?;
        // 収集時点から layout が変わっていたら push せず中断する
        // （merge が間に入った場合 expected digest が陳腐化する）。
        let current = layout_io::published(&target.layout)?;
        if current.index_digest != target.image.index_digest {
            return Err(Error::message(format!(
                "{} の layout が対象確定後に変更されました（再度 index と push を実行してください）",
                target.name
            )));
        }
        match push_package(target, force_replace, replace_platforms)? {
            Verdict::Done | Verdict::Skipped => {}
            Verdict::Rejected(reason) => {
                eprintln!("拒否: {}: {}", target.reference, reason);
                failures.push(format!("{}: {}", target.name, reason));
            }
        }
    }
    if !failures.is_empty() {
        return Err(Error::message(format!(
            "{} 件のパッケージが additive 判定を通りませんでした:\n{}\n正当なリビルドの置き換えには --replace-platform <arch>/<variant> を、全体置換には --force-replace を使用してください",
            failures.len(),
            failures
                .iter()
                .map(|failure| format!("  - {failure}"))
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }
    if all || with_index {
        push_index(&registry, published_index)?;
    } else {
        println!("index は push --all-packages または --with-index で公開されます");
    }
    Ok(())
}

/// `oci-layouts/` の layout が参照する blob の実在と digest を検査する。
/// push 前の健全性検査。`--all-packages` は layout が無いパッケージをスキップする。
pub fn verify(selection: &PackageSelection) -> Result<()> {
    let all = selection.package.is_none();
    let packages = super::selected(selection, || package::images().collect())?;
    super::run_aggregated(packages, |defined| {
        let Some(layout) = registry_index::layout_source(defined) else {
            if all {
                println!("スキップ(layout なし): {}", defined.name());
                return Ok(());
            }
            return Err(Error::message(format!(
                "{} の layout がありません（pack または env compose を実行してください）",
                defined.name()
            )));
        };
        let lock = workspace::output_lock(&layout)?;
        let issues = {
            let _guard = lock.read().map_err(|source| Error::io(&layout, source))?;
            layout_io::verify(&layout)?
        };
        if issues.is_empty() {
            println!("{} {}: OK", defined.name(), defined.version());
            return Ok(());
        }
        for issue in &issues {
            println!("{}: {issue}", defined.name());
        }
        Err(Error::message(format!(
            "{} の layout に {} 件の問題があります",
            defined.name(),
            issues.len()
        )))
    })
}

/// パッケージごとの ローカル（artifacts/layout）・index・レジストリの
/// 状態を一覧する。`local_only` はリモート問い合わせを省略する
/// （`--all-packages` での skopeo 呼び出しは高コストなため）。
pub fn status(
    selection: &PackageSelection,
    registry: Option<&str>,
    local_only: bool,
) -> Result<()> {
    // --local はリモートを見ない。前置詞が無くても一覧できる。
    let registry = if local_only {
        None
    } else {
        Some(registry::registry_prefix(registry)?)
    };
    if !local_only {
        skopeo::require()?;
    }
    let index_path = workspace::index_layout_dir();
    let published_index = if index_path.exists() {
        // index の再生成（write_artifact の rename）と競合しないよう
        // read ロックの下で読む。
        let lock = workspace::output_lock(&index_path)?;
        let _guard = lock
            .read()
            .map_err(|source| Error::io(&index_path, source))?;
        Some(registry_index::read_artifact(&index_path)?)
    } else {
        None
    };
    let packages = super::selected(selection, || package::images().collect())?;
    for defined in packages {
        let artifacts: Vec<String> = Target::all()
            .filter(|target| workspace::artifact_ready(&defined.artifact_dir(*target)))
            .map(|target| format!("{}/{}", target.architecture(), target.variant()))
            .collect();
        let (image, broken) = match registry_index::layout_source(defined) {
            Some(dir) => {
                // 並走する pack の rename 窓で「破損」を誤表示しないよう
                // read ロックの下で読む。
                let lock = workspace::output_lock(&dir)?;
                let _guard = lock.read().map_err(|source| Error::io(&dir, source))?;
                match layout_io::published(&dir) {
                    Ok(image) => (Some(image), false),
                    Err(_) => (None, true),
                }
            }
            None => (None, false),
        };
        let layout_state = match (&image, broken) {
            (None, false) => "なし".to_string(),
            (None, true) => "破損".to_string(),
            (Some(image), _) => format!("{} platform", image.manifests.len()),
        };
        let index_state = match published_index
            .as_ref()
            .and_then(|index| index.find(defined.name()))
        {
            Some(entry) => match &image {
                Some(image) if entry.index_digest == image.index_digest => "収録済み".to_string(),
                Some(_) => "世代ズレ".to_string(),
                None => "収録済み".to_string(),
            },
            None => "未収録".to_string(),
        };
        let remote_state = if let Some(registry) = registry.as_deref() {
            match &image {
                Some(image) => {
                    let repository = registry::package_repo(registry, defined.name());
                    let reference = registry::tag_ref(&repository, &push_tag(defined, image));
                    match skopeo::remote_digest(&reference) {
                        Ok(remote) if remote == image.index_digest => "一致".to_string(),
                        Ok(remote) => format!("別 digest ({:.12})", remote.prefixed()),
                        Err(error) if skopeo::is_not_found(&error) => "未公開".to_string(),
                        Err(_) => "取得失敗".to_string(),
                    }
                }
                None => "（layout なし）".to_string(),
            }
        } else {
            "-".to_string()
        };
        println!(
            "{} {}  artifacts: {}  layout: {}  index: {}  remote: {}",
            defined.name(),
            defined.version(),
            if artifacts.is_empty() {
                "なし".to_string()
            } else {
                artifacts.join(" ")
            },
            layout_state,
            index_state,
            remote_state,
        );
    }
    Ok(())
}

struct PushTarget {
    name: String,
    layout: PathBuf,
    lock: fd_lock::RwLock<std::fs::File>,
    repository: String,
    reference: String,
    image: layout_io::PublishedImage,
}

/// compose_root（build-env）は合成 layout の `ref.name` 注釈
/// （`{date}-{rev}` の合成バージョン）をタグにする。
/// 注釈が無い旧形式 layout は当日の `-0` にフォールバックする。
fn push_tag(defined: &dyn Package, image: &layout_io::PublishedImage) -> String {
    if defined.compose_root() {
        return image
            .ref_name
            .clone()
            .unwrap_or_else(|| format!("{}-0", util::utc_date()));
    }
    defined.version().to_string()
}

/// パッケージイメージを push する（additive イミュータブルタグ運用）。
///
/// - 既存タグが同一 digest → 冪等にスキップ
/// - リモートの platform 集合 ⊆ ローカル（共通 platform の内容一致）→
///   platform 追加として上書きを許容
/// - platform の削除・内容変更 → 上書きせずエラー
/// - `inspect --raw` が「参照なし」（404 系）で失敗 → push 実行
/// - `force_replace` → additive 判定をスキップして強制置換
/// - `replace_platforms` → 指定 platform のみ内容置換を許容
///   （正当なリビルドで変わったペイロードの公開用）
///
/// additive 不適合は `Verdict::Rejected` として返し、呼び出し側が
/// 全パッケージ分を集約して報告する（先頭の不適合で残りを止めない）。
enum Verdict {
    /// push を実行した
    Done,
    /// 同一 digest でスキップした
    Skipped,
    /// additive 判定を通らなかった（理由つき）
    Rejected(String),
}

fn push_package(
    target: &PushTarget,
    force_replace: bool,
    replace_platforms: &[Target],
) -> Result<Verdict> {
    if force_replace {
        eprintln!("強制置換: {}", target.reference);
    } else {
        match skopeo::remote_raw(&target.reference) {
            Ok(raw) => {
                let remote = Digest::from_hex(util::sha256_hex(&raw));
                if remote == target.image.index_digest {
                    println!("push 済み（同一 digest）: {}", target.reference);
                    return Ok(Verdict::Skipped);
                }
                if let Err(error) = registry::additive::check(
                    &target.layout,
                    &target.image,
                    &target.repository,
                    &raw,
                    replace_platforms,
                ) {
                    return Ok(Verdict::Rejected(error.to_string()));
                }
                println!(
                    "{} は additive 更新です（リモート ⊆ ローカル）: {}",
                    target.name, target.reference
                );
            }
            Err(error) if skopeo::is_not_found(&error) => {}
            Err(error) => return Err(error),
        }
    }
    let digest = skopeo::push_layout(&target.layout, &target.reference)?;
    if digest != target.image.index_digest {
        return Err(Error::message(format!(
            "push 後の digest が一致しません: {} vs {}",
            digest.prefixed(),
            target.image.index_digest.prefixed()
        )));
    }
    println!("push: {} @ {}", target.reference, digest.prefixed());
    Ok(Verdict::Done)
}

/// 既存タグとの digest 照合を経て push する（index artifact など
/// 一意タグ向けの厳密比較。additive 判定は行わない）。
fn push_immutable(layout: &Path, reference: &str, expected: &Digest) -> Result<()> {
    skopeo::push_immutable(layout, reference, expected)
}

/// index artifact を `{registry}/packages/index:{yyyymmdd}-{rev}` で公開する。
/// rev は `list-tags` の同日タグの最大値 + 1。
fn push_index(registry: &str, index: Option<registry_index::PackageIndex>) -> Result<()> {
    let layout = workspace::index_layout_dir();
    let Some(index) = index else {
        return Err(Error::message(format!(
            "index artifact がありません: {} （index サブコマンドを実行してください）",
            layout.display()
        )));
    };
    let date = util::utc_date();
    let repository = registry::index_repo(registry);
    let tags = skopeo::list_tags(&repository).unwrap_or_default();
    let revision = tags
        .iter()
        .filter_map(|tag| tag.strip_prefix(&format!("{date}-")))
        .filter_map(|rest| rest.parse::<u32>().ok())
        .max()
        .map(|rev| rev + 1)
        .unwrap_or(0);
    // artifact の自己記述（image.version / ref.name）を実 push タグに
    // 合わせて再生成する。index サブコマンドが書いた `{date}-0` は
    // 同日の再 push でずれ得るため、ここで確定値に上書きする。
    let tag = format!("{date}-{revision}");
    let expected = registry_index::write_artifact(&index, &layout, &tag)?;
    let reference = registry::tag_ref(&repository, &tag);
    push_immutable(&layout, &reference, &expected)?;
    Ok(())
}
