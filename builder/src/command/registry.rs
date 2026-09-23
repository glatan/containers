//! `index` / `push` サブコマンド。
//!
//! - `index`: `oci-layouts/` を走査して index artifact を生成する
//! - `push`: パッケージイメージ（と `--all` では index artifact）を
//!   skopeo 経由でレジストリへ送る

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::oci::Digest;
use crate::package::{self, Package};
use crate::registry::{self, index as registry_index, layout_io, skopeo};
use crate::util;
use crate::workspace;

use super::PackageSelection;

/// index artifact を `oci-layouts/index/` に生成する。
/// `--check` は既存の生成物との差分のみ検査する。
pub fn index(registry: Option<&str>, check: bool) -> Result<()> {
    let registry = registry::registry_prefix(registry);
    let generated = registry_index::generate(&registry)?;
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

/// パッケージイメージを push する。`--all` は全イメージと index artifact。
///
/// イミュータブルタグ運用: 既存タグが同じ digest を指すなら冪等に
/// スキップし、別 digest を指すなら上書きせずエラーにする。
pub fn push(selection: &PackageSelection, registry: Option<&str>, dry_run: bool) -> Result<()> {
    let registry = registry::registry_prefix(registry);
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
        let image = layout_io::published(&layout)?;
        let repository = registry::package_repo(&registry, defined.name());
        let tag = push_tag(defined, &image);
        let reference = registry::tag_ref(&repository, &tag);
        match published_index.as_ref().and_then(|index| index.find(defined.name())) {
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
            reference,
            expected: image.index_digest,
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
    for target in &targets {
        push_immutable(&target.layout, &target.reference, &target.expected)?;
    }
    if all {
        push_index(&registry, published_index)?;
    } else {
        println!("index は push --all で公開されます");
    }
    Ok(())
}

struct PushTarget {
    name: String,
    layout: PathBuf,
    reference: String,
    expected: Digest,
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

/// 既存タグとの digest 照合を経て push する。
fn push_immutable(layout: &Path, reference: &str, expected: &Digest) -> Result<()> {
    skopeo::push_immutable(layout, reference, expected)
}

/// index artifact を `{registry}/packages/index:{yyyymmdd}-{rev}` で公開する。
/// rev は `list-tags` の同日タグの最大値 + 1。
fn push_index(registry: &str, index: Option<registry_index::PackageIndex>) -> Result<()> {
    let layout = workspace::index_layout_dir();
    if index.is_none() || !layout.exists() {
        return Err(Error::message(format!(
            "index artifact がありません: {} （index サブコマンドを実行してください）",
            layout.display()
        )));
    }
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
    let reference = registry::tag_ref(&repository, &format!("{date}-{revision}"));
    let expected = layout_io::top_digest(&layout)?;
    push_immutable(&layout, &reference, &expected)?;
    Ok(())
}
