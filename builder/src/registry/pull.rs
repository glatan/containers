//! mode B（合成ベースイメージ）の pull・展開・合成・push。
//!
//! 1. `skopeo copy docker://{name}@{manifestDigest} oci:{tmp}` で層を取得
//! 2. 各層 blob を `tar --zstd` で展開して XDG キャッシュのツリーに置く
//! 3. `image::assemble` で合成イメージを作り `skopeo copy --all` で push
//!
//! 展開済みツリーは manifest digest でキャッシュされるため、同じ
//! パッケージを複数イメージで使っても再 pull しない。

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{Error, Result};
use crate::oci::{self, Digest};
use crate::registry::{cache, layout_io, skopeo};
use crate::registry::resolve::Resolved;
use crate::target::Target;
use crate::util;

/// mode B の展開に必要な外部ツール（tar の zstd 展開）を確認する。
pub fn require_tools() -> Result<()> {
    for tool in ["tar", "zstd"] {
        let ok = Command::new(tool)
            .arg("--version")
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .is_ok_and(|status| status.success());
        if !ok {
            return Err(Error::message(format!(
                "{tool} が見つかりません。mode=compose の層展開には {tool} が必要です"
            )));
        }
    }
    Ok(())
}

/// `reference` の manifest を pull し、層を `trees/{digest}/rootfs` に展開する。
/// 完了マーカー `.extracted` はツリーの外（`trees/{digest}` 直下）に置き、
/// 返すパスは `rootfs` 側にする。マーカーを層ディレクトリ内に置くと
/// パス競合検査で別パッケージのマーカーと衝突するため。
pub fn ensure_tree(resolved: &Resolved) -> Result<PathBuf> {
    let dir = cache::tree_dir(&resolved.manifest_digest);
    let rootfs = dir.join("rootfs");
    if dir.join(".extracted").exists() {
        return Ok(rootfs);
    }
    let parent = dir.parent().expect("trees の親");
    std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    // 層を展開する先の staging（rename で dir になる）。
    let staging = tempfile::tempdir_in(parent).map_err(|source| Error::io(parent, source))?;
    let stage_rootfs = staging.path().join("rootfs");
    std::fs::create_dir_all(&stage_rootfs).map_err(|source| Error::io(&stage_rootfs, source))?;
    // pull した layout は別の staging（tree に混入しないよう分ける）。
    let pulled = tempfile::tempdir_in(parent).map_err(|source| Error::io(parent, source))?;
    let layout_dir = pulled.path().join("layout");
    skopeo::pull_layout(&format!("docker://{}", resolved.reference), &layout_dir)?;
    let manifest = layout_io::pulled_manifest(&layout_dir)?;
    if manifest.layers.is_empty() {
        return Err(Error::message(format!(
            "{} の manifest に層がありません",
            resolved.reference
        )));
    }
    for layer in &manifest.layers {
        extract_layer(&layout_dir, &layer.digest, &stage_rootfs)?;
    }
    util::write(
        &staging.path().join(".extracted"),
        resolved.manifest_digest.prefixed(),
    )?;
    oci::replace_dir(&dir, staging)?;
    Ok(rootfs)
}

/// 層 blob（`tar+zstd`）を `tree` に展開する。
fn extract_layer(layout: &Path, digest: &Digest, tree: &Path) -> Result<()> {
    let blob = layout.join("blobs/sha256").join(digest.encoded());
    let status = Command::new("tar")
        .args(["--zstd", "-xf"])
        .arg(&blob)
        .arg("-C")
        .arg(tree)
        .status()
        .map_err(|source| Error::message(format!("tar の起動に失敗: {source}")))?;
    if !status.success() {
        return Err(Error::message(format!(
            "層の展開に失敗しました: {}",
            blob.display()
        )));
    }
    Ok(())
}

/// 解決済み閉包から合成ベースイメージを作り、`{repository}:{tag}` へ push する。
/// `push=false` のときはローカル layout のみ生成する。
/// 返り値は `(layout のパス, push した参照 or ローカル参照)`.
#[allow(clippy::too_many_arguments)]
pub fn compose_base(
    name: &str,
    resolved: &[Resolved],
    target: Target,
    zstd_level: u8,
    output: Option<&Path>,
    repository: &str,
    tag: &str,
    push: bool,
) -> Result<(PathBuf, String)> {
    let mut layers = Vec::new();
    for item in resolved {
        let tree = ensure_tree(item)?;
        layers.push((item.name.as_str(), tree));
    }
    let out = output
        .map(|path| path.join(format!("{name}-{tag}")))
        .unwrap_or_else(|| cache::composed_dir(name, tag));
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let image = oci::Image {
        name: format!("{name}-base"),
        version: tag.to_string(),
    };
    crate::image::assemble(&out, &image, target, layers, zstd_level)?;
    if !push {
        return Ok((out.clone(), format!("oci:{}", out.display())));
    }
    // イミュータブルタグ運用: 既存タグが別 digest を指す場合は上書きしない。
    // タグが指すのはネスト index blob だが、Dockerfile の pin は
    // 子 manifest の digest にする。index digest で pin すると
    // containerd/platforms の platform マッチングを通り、amd64/v4 等の
    // variant 付き単一 manifest が既定 platform で解決不能になるため。
    let reference = crate::registry::tag_ref(repository, tag);
    let expected = layout_io::top_digest(&out)?;
    skopeo::push_immutable(&out, &reference, &expected)?;
    let published = layout_io::published(&out)?;
    let manifest_digest = published
        .manifests
        .first()
        .ok_or_else(|| Error::message(format!("{} に manifest がありません", out.display())))?
        .digest
        .clone();
    // Dockerfile / lockfile にはスキームなしの manifest pin 参照を返す。
    Ok((out, format!("{repository}@{}", manifest_digest.prefixed())))
}
