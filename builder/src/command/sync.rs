//! `sync` サブコマンド。消費側リポジトリの設定ファイルを読み、
//! 各イメージ定義の Dockerfile にある GENERATED ブロックを同期する。
//!
//! - `mode: block` … pinned FROM + COPY 連結ブロックを生成・書換え
//! - `mode: compose` … 層を pull・合成して push し、ARG pin に書換え
//! - `--check` … 再生成結果と差分があれば失敗（CI 向け、書き込まない）

use std::path::{Path, PathBuf};

use crate::error::{Error, Result};
use crate::oci::Digest;
use crate::registry;
use crate::registry::config::{Config, ImageSpec, Mode};
use crate::registry::dockerfile;
use crate::registry::index::{self, PackageIndex};
use crate::registry::lockfile::{self, LockEntry, Locked, Lockfile};
use crate::registry::resolve::{self, Resolved};
use crate::registry::{cache, pull, skopeo};
use crate::util;

use super::Compression;

#[allow(clippy::too_many_arguments)]
pub fn sync(
    config_path: &Path,
    check: bool,
    index_flag: Option<&str>,
    registry_flag: Option<&str>,
    output: Option<&Path>,
    no_push: bool,
    compression: &Compression,
) -> Result<()> {
    let config = Config::load(config_path)?;
    let base = config_path.parent().unwrap_or_else(|| Path::new("."));
    let index_ref = registry::index_ref(index_flag, config.index.as_deref()).ok_or_else(|| {
        Error::message("index 参照を指定してください（--index / CONTAINERS_INDEX / 設定の index）")
    })?;
    // `"auto"` だけが `{prefix}/packages/index` を引く。完全な index 参照は
    // 前置詞を使わない。
    let registry = if index_ref == "auto" {
        Some(registry::registry_prefix(registry_flag)?)
    } else {
        None
    };
    skopeo::require()?;
    if config.images.iter().any(|spec| spec.mode == Mode::Compose) {
        pull::require_tools()?;
    }
    let reference = resolve_index_ref(&index_ref, registry.as_deref())?;
    let (index, index_digest) = fetch_index(&reference)?;

    let lock_path = lockfile::lockfile_path(config_path);
    let previous = lockfile::load(&lock_path)?.unwrap_or_default();
    let mut next = Lockfile {
        schema_version: lockfile::LOCKFILE_SCHEMA_VERSION,
        index: pinned_index_ref(&reference, &index_digest),
        images: Default::default(),
    };

    let mut stale: Vec<PathBuf> = Vec::new();
    for spec in &config.images {
        let resolved = resolve::closure(&index, &spec.packages, spec.target())?;
        let dockerfile_path = spec.dockerfile_path(base);
        let existing = std::fs::read_to_string(&dockerfile_path)
            .map_err(|source| Error::io(&dockerfile_path, source))?;
        let (block, composed) = match spec.mode {
            Mode::Block => (block_block(spec, &resolved, &index_digest), None),
            Mode::Compose => compose_block(
                spec,
                &resolved,
                &index,
                &index_digest,
                &previous,
                output,
                no_push,
                compression,
                check,
            )?,
        };
        let replaced = dockerfile::splice(&existing, &block, spec.mode == Mode::Compose)?;
        if check {
            if replaced != existing {
                stale.push(dockerfile_path);
            }
        } else if replaced != existing {
            util::write(&dockerfile_path, &replaced)?;
            println!("{}: GENERATED ブロックを更新しました", spec.name);
        }
        next.images.insert(
            spec.name.clone(),
            LockEntry {
                platform: spec.platform.clone(),
                mode: match spec.mode {
                    Mode::Block => "block".to_string(),
                    Mode::Compose => "compose".to_string(),
                },
                resolved: resolved.iter().map(locked_of).collect(),
                composed,
            },
        );
    }
    // lockfile 自身の差分も --check の対象。
    let lock_bytes = serde_json::to_vec_pretty(&next)?;
    let lock_stale = std::fs::read(&lock_path)
        .map(|existing| existing != lock_bytes)
        .unwrap_or(true);
    if check {
        if lock_stale {
            stale.push(lock_path.clone());
        }
        if !stale.is_empty() {
            for path in &stale {
                eprintln!("差分があります: {}", path.display());
            }
            return Err(Error::message(format!(
                "{} 件の生成物が古くなっています（sync を実行してください）",
                stale.len()
            )));
        }
        println!("GENERATED ブロックは最新です");
        return Ok(());
    }
    lockfile::write(&lock_path, &next)?;
    println!("lockfile を更新しました: {}", lock_path.display());
    Ok(())
}

/// `"auto"` は `list-tags` の最新 `{date}-{rev}` に解決する。
fn resolve_index_ref(reference: &str, registry: Option<&str>) -> Result<String> {
    if reference != "auto" {
        return Ok(with_docker_scheme(reference));
    }
    let registry = registry.ok_or_else(|| {
        Error::message(
            "レジストリ前置詞を指定してください（--registry または CONTAINERS_REGISTRY）",
        )
    })?;
    let repository = registry::index_repo(registry);
    let tags = skopeo::list_tags(&repository)?;
    let tag = tags
        .iter()
        // `{yyyymmdd}-{rev}` 形式のみ（漂流タグに乗っ取られないよう絞る）。
        .filter(|tag| is_date_rev(tag))
        .filter_map(|tag| tag.rsplit_once('-'))
        .filter_map(|(date, rev)| rev.parse::<u32>().ok().map(|rev| (date, rev)))
        .max()
        .map(|(date, rev)| format!("{date}-{rev}"))
        .ok_or_else(|| {
            Error::message(format!(
                "{repository} に index タグ（{{yyyymmdd}}-{{rev}} 形式）がありません"
            ))
        })?;
    Ok(registry::tag_ref(&repository, &tag))
}

/// `{8桁の日付}-{数値rev}` 形式か。
fn is_date_rev(tag: &str) -> bool {
    let Some((date, rev)) = tag.rsplit_once('-') else {
        return false;
    };
    date.len() == 8
        && date.bytes().all(|byte| byte.is_ascii_digit())
        && !rev.is_empty()
        && rev.bytes().all(|byte| byte.is_ascii_digit())
}

fn with_docker_scheme(reference: &str) -> String {
    // `oci:{dir}` はローカル検証用にそのまま通す。
    if reference.starts_with("docker://") || reference.starts_with("oci:") {
        reference.to_string()
    } else {
        format!("docker://{reference}")
    }
}

/// index artifact をキャッシュへ pull し、パースする。
/// 返り値は `(index, index artifact の manifest digest)`。
/// pull は digest pin で行い、digest 計算と pull の間にタグが
/// 動く TOCTOU を避ける（oci: のローカル参照はそのまま使う）。
fn fetch_index(reference: &str) -> Result<(PackageIndex, Digest)> {
    let digest = skopeo::remote_digest(reference)?;
    let dir = cache::index_layout_dir(&digest);
    if !dir.exists() {
        let parent = dir.parent().expect("index キャッシュの親");
        std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
        // staging と dir は同一親配下なので rename は同一 FS 内で完結する。
        let staging = tempfile::tempdir_in(parent).map_err(|source| Error::io(parent, source))?;
        let pulled = staging.path().join("layout");
        let source = if reference.starts_with("oci:") {
            reference.to_string()
        } else {
            format!("docker://{}@{}", repo_part(reference), digest.prefixed())
        };
        skopeo::pull_layout(&source, &pulled)?;
        std::fs::rename(&pulled, &dir).map_err(|source| Error::io(&dir, source))?;
    }
    Ok((index::read_artifact(&dir)?, digest))
}

/// 参照からリポジトリ部分（`{host}/{path}`）を取り出す。
/// `docker://` 前置詞・`@digest`・`:tag` を落とす。
/// タグ区切りの `:` は最後の `/` より後にあるものだけを削る
/// （`localhost:5000/repo` のポートを削らない）。
fn repo_part(reference: &str) -> String {
    let bare = reference.strip_prefix("docker://").unwrap_or(reference);
    if let Some((repo, _)) = bare.split_once('@') {
        return repo.to_string();
    }
    let (head, tail) = match bare.rfind('/') {
        Some(slash) => bare.split_at(slash + 1),
        None => ("", bare),
    };
    match tail.rsplit_once(':') {
        Some((name, _)) => format!("{head}{name}"),
        None => bare.to_string(),
    }
}

/// lockfile に記録する index 参照（`{repo}@sha256:{digest}`）。
fn pinned_index_ref(reference: &str, digest: &Digest) -> String {
    format!("{}@{}", repo_part(reference), digest.prefixed())
}

/// mode A: pinned FROM + COPY 連結ブロック。
fn block_block(spec: &ImageSpec, resolved: &[Resolved], index_digest: &Digest) -> String {
    let stages: Vec<(String, String, String)> = resolved
        .iter()
        .map(|item| {
            (
                resolve::stage_name(item),
                item.reference.clone(),
                resolve::docker_platform(&item.platform),
            )
        })
        .collect();
    dockerfile::block_lines(&spec.platform, index_digest, &stages)
}

/// mode B: 合成イメージの pin を返す。
/// 解決結果が lockfile と一致しリモートに残っていれば再利用し、
/// なければ pull・合成・push する。`check` 時は lockfile の pin のみ使い
/// 合成を実行しない。
#[allow(clippy::too_many_arguments)]
fn compose_block(
    spec: &ImageSpec,
    resolved: &[Resolved],
    index: &PackageIndex,
    index_digest: &Digest,
    previous: &Lockfile,
    output: Option<&Path>,
    no_push: bool,
    compression: &Compression,
    check: bool,
) -> Result<(String, Option<String>)> {
    let locked = previous.images.get(&spec.name);
    let same_resolution = locked.is_some_and(|entry| {
        entry.mode == "compose"
            && entry.platform == spec.platform
            && entry.resolved == resolved.iter().map(locked_of).collect::<Vec<_>>()
    });
    let locked_composed = same_resolution
        .then(|| locked.and_then(|entry| entry.composed.clone()))
        .flatten();
    // --check ではレジストリ到達性に依らず差分だけを検査したいため、
    // リモート存在確認は実際に再利用する書き込み時のみ行う。
    let reusable = if check {
        locked_composed.clone()
    } else {
        locked_composed.filter(|reference| remote_has(reference))
    };
    if check {
        let Some(reference) = reusable else {
            // 再解決結果が lockfile と一致しない = 差分あり。
            // ブロックは暫定値で splice し差分検出に委ねる。
            return Ok((dockerfile::arg_block(index_digest, "out-of-date"), None));
        };
        return Ok((
            dockerfile::arg_block(index_digest, &reference),
            Some(reference),
        ));
    }
    if let Some(reference) = reusable {
        println!("{}: 合成イメージを再利用します: {reference}", spec.name);
        return Ok((
            dockerfile::arg_block(index_digest, &reference),
            Some(reference),
        ));
    }
    let repository = spec
        .repository
        .clone()
        .unwrap_or_else(|| registry::image_repo(&index.registry, &spec.name));
    let date = util::utc_date();
    let revision = skopeo::list_tags(&repository)
        .unwrap_or_default()
        .iter()
        .filter_map(|tag| tag.rsplit_once('-'))
        .filter_map(|(tag_date, rev)| {
            (tag_date == date)
                .then(|| rev.parse::<u32>().ok())
                .flatten()
        })
        .max()
        .map(|rev| rev + 1)
        .unwrap_or(0);
    let tag = format!("{date}-{revision}");
    let level = super::resolve_zstd_level(compression)?;
    let (_, reference) = pull::compose_base(
        &spec.name,
        resolved,
        spec.target(),
        level,
        output,
        &repository,
        &tag,
        !no_push,
    )?;
    println!("{}: 合成イメージを公開しました: {reference}", spec.name);
    Ok((
        dockerfile::arg_block(index_digest, &reference),
        Some(reference),
    ))
}

/// `{repo}@sha256:{digest}` 参照がリモートに残っているか。
fn remote_has(reference: &str) -> bool {
    if reference.starts_with("oci:") {
        return PathBuf::from(reference.trim_start_matches("oci:")).exists();
    }
    skopeo::remote_digest(&with_docker_scheme(reference)).is_ok()
}

fn locked_of(resolved: &Resolved) -> Locked {
    Locked {
        name: resolved.name.clone(),
        version: resolved.version.clone(),
        platform: resolved.platform.clone(),
        manifest_digest: resolved.manifest_digest.clone(),
        reference: resolved.reference.clone(),
    }
}
