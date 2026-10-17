use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::error::{Error, Result};
use crate::oci::layer;
use crate::oci::model::{
    Descriptor, Digest, ExecConfig, Image, ImageConfig, ImageIndex, ImageManifest, OciLayout,
    PlatformObject, RootFs, ZstdLevel,
};
use crate::target::Target;
use crate::util;
use crate::workspace;

pub(crate) const MEDIA_INDEX: &str = "application/vnd.oci.image.index.v1+json";
pub(crate) const MEDIA_MANIFEST: &str = "application/vnd.oci.image.manifest.v1+json";
pub(crate) const MEDIA_CONFIG: &str = "application/vnd.oci.image.config.v1+json";
pub(crate) const MEDIA_LAYER: &str = "application/vnd.oci.image.layer.v1.tar+zstd";
/// containers/image の `oci:` トランスポートが受理するよう、トップレベル
/// index.json の descriptor に付ける参照名（buildx 出力と同じ規約）。
pub(crate) const ANNOTATION_REF_NAME: &str = "org.opencontainers.image.ref.name";
/// containers/image が zstd レイヤーを持つ manifest の descriptor に
/// push 時に付与するアノテーション。ネスト index blob がリモートの
/// manifest list と digest 一致するよう pack 時点で付けておく。
const ANNOTATION_ZSTD: &str = "io.github.containers.compression.zstd";
const ANNOTATION_TITLE: &str = "org.opencontainers.image.title";
const ANNOTATION_VERSION: &str = "org.opencontainers.image.version";
const ANNOTATION_DESCRIPTION: &str = "org.opencontainers.image.description";
/// manifest の digest が git リビジョンに連動する原因となる注釈。
/// additive push の内容比較ではこの差分のみを許容する。
pub(crate) const ANNOTATION_REVISION: &str = "org.opencontainers.image.revision";
/// `org.opencontainers.image.source`。GHCR はこの値が GitHub リポジトリ
/// URL のときパッケージをリポジトリへ自動リンクし、初回公開時にのみ
/// リポジトリのアクセス権を継承する（後からの手動リンクでは継承しない）。
const ANNOTATION_SOURCE: &str = "org.opencontainers.image.source";
pub(crate) const SOURCE_URL: &str = "https://github.com/glatan/containers";

/// manifest / index に共通で付ける自己記述アノテーション。
/// `source`/`description` は GHCR のパッケージページ表示とリポジトリ
/// 連携、`revision` はトレーサビリティ用。
fn image_annotations(image: &Image) -> BTreeMap<String, String> {
    let mut annotations = BTreeMap::from([
        (ANNOTATION_TITLE.to_string(), image.name.clone()),
        (ANNOTATION_VERSION.to_string(), image.version.clone()),
        (ANNOTATION_SOURCE.to_string(), SOURCE_URL.to_string()),
    ]);
    if let Some(description) = &image.description {
        annotations.insert(ANNOTATION_DESCRIPTION.to_string(), description.clone());
    }
    if let Some(revision) = util::revision() {
        annotations.insert(ANNOTATION_REVISION.to_string(), revision);
    }
    annotations
}

pub fn build(
    output: &Path,
    image: &Image,
    layers: &[(Target, Vec<PathBuf>)],
    zstd_level: u8,
) -> Result<()> {
    build_with_config(
        output,
        image,
        layers,
        zstd_level,
        ExecConfig::default(),
        Vec::new(),
    )
}

pub fn build_with_config(
    output: &Path,
    image: &Image,
    layers: &[(Target, Vec<PathBuf>)],
    zstd_level: u8,
    config: ExecConfig,
    extra_files: Vec<(String, Vec<u8>)>,
) -> Result<()> {
    build_annotated(
        output,
        image,
        layers,
        zstd_level,
        config,
        default_workers(),
        |_| BTreeMap::new(),
        extra_files,
    )
}

/// `annotate` は manifest 単位の追加アノテーションを返す
/// （title/version は自動付与。依存の人間可読版など platform 別に変わる
/// 値を載せるため Target を受け取る）。
#[allow(clippy::too_many_arguments)]
pub fn build_annotated(
    output: &Path,
    image: &Image,
    layers: &[(Target, Vec<PathBuf>)],
    zstd_level: u8,
    config: ExecConfig,
    workers: usize,
    annotate: impl Fn(Target) -> BTreeMap<String, String> + Sync + Send + 'static,
    extra_files: Vec<(String, Vec<u8>)>,
) -> Result<()> {
    let outcome = build_all_annotated(
        vec![LayoutJob {
            output: output.to_path_buf(),
            image: image.clone(),
            layers: layers.to_vec(),
            config,
            annotate: Box::new(annotate),
            merge: false,
            remove: Vec::new(),
            extra_files,
        }],
        zstd_level,
        workers,
    )?;
    if !outcome.failed.is_empty() {
        return Err(Error::message(format!(
            "レイアウト構築に失敗: {}",
            outcome.failed.join(", ")
        )));
    }
    Ok(())
}

/// `build_all_annotated` の1レイアウト分の仕事。
pub struct LayoutJob {
    pub output: PathBuf,
    pub image: Image,
    pub layers: Vec<(Target, Vec<PathBuf>)>,
    pub config: ExecConfig,
    pub annotate: Box<dyn Fn(Target) -> BTreeMap<String, String> + Sync + Send>,
    /// 真なら既存 layout の platform のうち `layers`/`remove` に
    /// 含まれないものを descriptor ごと保持する（増分 merge）。
    /// 偽なら既存 layout を見ず全再生成する。
    pub merge: bool,
    /// 既存 layout から除去する platform（merge と独立に指定可能）。
    pub remove: Vec<Target>,
    /// 確定時に staging へ同梱する追加ファイル（相対名, 内容）。
    /// sbom.spdx.json など rename で原子的に載せたい非 OCI ファイル用。
    /// 確定後に output へ追記すると rename と書き込みの窓が残るため、
    /// staging 側に置いて一緒に確定する。
    pub extra_files: Vec<(String, Vec<u8>)>,
}

impl LayoutJob {
    /// 既存 layout を読む必要があるか（merge または platform 除去）。
    const fn merges_existing(&self) -> bool {
        self.merge || !self.remove.is_empty()
    }
}

/// zstd --ultra -22 は 1 プロセスあたり数百 MB を使うため、コア数を
/// そのまま使うとメモリを圧迫し得る。既定のワーカー上限。
pub fn default_workers() -> usize {
    std::thread::available_parallelism()
        .map_or(1, |n| n.get())
        .min(12)
}

/// `build_all_annotated` の結果。ジョブ単位の成否を保持する。
pub struct LayoutOutcome {
    /// 確定に成功したレイアウトのイメージ名（jobs の出現順）
    pub succeeded: Vec<String>,
    /// "名前 (原因)" 形式の失敗一覧
    pub failed: Vec<String>,
}

/// 複数レイアウトを共有ワーカープールで構築する。
/// (ジョブ, ターゲット) の全組を1つのキューに載せ、空いたワーカーが
/// 逐次取り出すため、ターゲット数の少ないジョブや重いレイヤーの
/// タイミングでコアが遊ばない。manifest が揃ったジョブから順に
/// index を組み立てて確定する。失敗したジョブは確定せず、
/// ジョブの入力レイヤー検証と staging / blob ディレクトリの確保。
/// 失敗は呼出側でジョブ単位の失敗として集約される。
fn prepare_job_staging(job: &LayoutJob) -> Result<(tempfile::TempDir, PathBuf)> {
    // merge/remove ジョブは新規 manifest がなくても既存 index の
    // 書き換えで成立するため layers 空を許容する。
    if job.layers.is_empty() && !job.merges_existing() {
        return Err(Error::message("レイヤーがありません"));
    }
    for (_, dirs) in &job.layers {
        for dir in dirs {
            if !dir.is_dir() {
                return Err(Error::message(format!(
                    "レイヤーのディレクトリが見つかりません: {}",
                    dir.display()
                )));
            }
        }
    }
    let parent = job.output.parent().unwrap_or_else(|| Path::new("."));
    if !parent.exists() {
        fs::create_dir_all(parent).map_err(|source| Error::io(parent, source))?;
    }
    let staging = tempfile::TempDir::new_in(parent).map_err(|source| Error::io(parent, source))?;
    let blobs = staging.path().join("blobs/sha256");
    fs::create_dir_all(&blobs).map_err(|source| Error::io(&blobs, source))?;
    Ok((staging, blobs))
}

/// 他のジョブは続行する（--all-packages 系の集約ポリシーと同じ）。
/// ジョブ単位の失敗は `LayoutOutcome.failed` に集約され、Err が返るのは
/// スレッド panic 等の系統的な失敗のみ。
pub fn build_all_annotated(
    jobs: Vec<LayoutJob>,
    zstd_level: u8,
    workers: usize,
) -> Result<LayoutOutcome> {
    let zstd_level = ZstdLevel::new(zstd_level)?;
    let mut stagings = Vec::with_capacity(jobs.len());
    let mut blob_dirs = Vec::with_capacity(jobs.len());
    // 入力検証・staging 確保もジョブ単位で集約する（1 パッケージの
    // 不備で --all-packages 全体を止めない。plan→構築間の成果物消失 TOCTOU も
    // ここでジョブ失敗として扱われる）。
    let mut failed = Vec::new();
    for job in &jobs {
        match prepare_job_staging(job) {
            Ok((staging, blobs)) => {
                stagings.push(Some(staging));
                blob_dirs.push(Some(blobs));
            }
            Err(error) => {
                stagings.push(None);
                blob_dirs.push(None);
                failed.push(format!("{}: {error}", job.image.name));
            }
        }
    }

    struct Item<'a> {
        job: usize,
        pos: usize,
        target: Target,
        dirs: &'a [PathBuf],
    }
    let items: Vec<Item> = jobs
        .iter()
        .enumerate()
        .filter(|(job, _)| blob_dirs[*job].is_some())
        .flat_map(|(job, j)| {
            j.layers
                .iter()
                .enumerate()
                .map(move |(pos, (target, dirs))| Item {
                    job,
                    pos,
                    target: *target,
                    dirs,
                })
        })
        .collect();
    let caches: Vec<layer::BlobCache> = (0..jobs.len()).map(|_| Default::default()).collect();
    let next = std::sync::atomic::AtomicUsize::new(0);
    let workers = workers.min(items.len()).max(1);
    let mut gathered: Vec<(usize, usize, Result<Descriptor>)> = Vec::new();
    std::thread::scope(|scope| -> Result<()> {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                let next = &next;
                let items = &items;
                let jobs = &jobs;
                let caches = &caches;
                let blob_dirs = &blob_dirs;
                scope.spawn(move || {
                    let mut out = Vec::new();
                    loop {
                        let i = next.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                        let Some(item) = items.get(i) else { break };
                        let job = &jobs[item.job];
                        out.push((
                            item.job,
                            item.pos,
                            build_manifest(
                                blob_dirs[item.job].as_ref().unwrap(),
                                &job.image,
                                item.target,
                                item.dirs,
                                zstd_level,
                                &job.config,
                                (job.annotate)(item.target),
                                &caches[item.job],
                            ),
                        ));
                    }
                    out
                })
            })
            .collect();
        for handle in handles {
            let chunk = handle.join().map_err(|payload| {
                Error::message(format!(
                    "レイアウト構築スレッドがパニックしました: {}",
                    util::panic_message(&*payload)
                ))
            })?;
            gathered.extend(chunk);
        }
        Ok(())
    })?;

    // manifest をジョブ・位置順に組み立て直す。失敗したジョブは
    // 確定せずスキップし、他のジョブは続行する。
    let mut slots: Vec<Vec<Option<Result<Descriptor>>>> = jobs
        .iter()
        .map(|j| {
            let mut v = Vec::new();
            v.resize_with(j.layers.len(), || None);
            v
        })
        .collect();
    for (job, pos, result) in gathered {
        slots[job][pos] = Some(result);
    }
    let mut succeeded = Vec::new();
    for (i, job) in jobs.iter().enumerate() {
        if blob_dirs[i].is_none() {
            // 検証失敗は事前に failed へ集約済み
            continue;
        }
        let slot = std::mem::take(&mut slots[i]);
        let mut manifests = Vec::with_capacity(slot.len());
        let mut errors = Vec::new();
        for (pos, result) in slot.into_iter().enumerate() {
            match result.unwrap_or_else(|| Err(Error::message("ターゲットの構築結果がありません")))
            {
                Ok(descriptor) => manifests.push((job.layers[pos].0, descriptor)),
                Err(error) => errors.push(error.to_string()),
            }
        }
        // レイヤー構築だけでなく確定フェーズの失敗も集約する。
        // 失敗ジョブの staging は tempdir の Drop で除去される。
        let result = if errors.is_empty() {
            let staging = stagings[i].take().expect("staging は一度だけ確定される");
            commit_job(job, staging, blob_dirs[i].as_ref().unwrap(), manifests)
                .map_err(|error| error.to_string())
        } else {
            Err(errors.join(", "))
        };
        match result {
            Ok(()) => succeeded.push(job.image.name.clone()),
            // 失敗は呼出側の集約エラーで一度だけ報告する。
            Err(error) => failed.push(format!("{} ({error})", job.image.name)),
        }
    }
    Ok(LayoutOutcome { succeeded, failed })
}

/// ジョブの確定。output 単位の flock を取ってから index を組み立て、
/// `replace_dir` で原子的に入れ替える。merge/remove ジョブでは既存
/// layout の保持対象 manifest を descriptor ごと引き継ぎ、その blob
/// （manifest + config + layers）を staging へハードリンクする。
/// 参照されなくなった blob は staging に乗らないため、この過程が
/// 孤立 blob の GC も兼ねる。
fn commit_job(
    job: &LayoutJob,
    staging: tempfile::TempDir,
    blobs: &Path,
    new_manifests: Vec<(Target, Descriptor)>,
) -> Result<()> {
    // 既存 layout の読み取り〜index 書き換え〜確定を output 単位で
    // 直列化する（並走する pack/push との競合防止）。
    let mut lock = workspace::output_lock(&job.output)?;
    let _guard = lock
        .write()
        .map_err(|source| Error::io(&job.output, source))?;

    let manifests: Vec<Descriptor> = if job.merges_existing() {
        let mut all = existing_manifests(&job.output)?;
        for target in &job.remove {
            if all.remove(target).is_none() {
                return Err(Error::message(format!(
                    "{} の layout に platform {}/{} がありません",
                    job.output.display(),
                    target.architecture(),
                    target.variant()
                )));
            }
        }
        let rebuilt: BTreeSet<Target> = new_manifests.iter().map(|(target, _)| *target).collect();
        if let Some(conflict) = job.remove.iter().find(|t| rebuilt.contains(t)) {
            return Err(Error::message(format!(
                "{}/{} は pack と remove に同時指定できません",
                conflict.architecture(),
                conflict.variant()
            )));
        }
        // 保持分の blob を staging へ引き継ぐ。再構築対象の platform は
        // 旧 descriptor のまま all に残っているため除外する（引き継ぐと
        // 旧 blob が GC されず残る）。
        for (target, descriptor) in &all {
            if rebuilt.contains(target) {
                continue;
            }
            carry_manifest_blobs(&job.output, blobs, descriptor)?;
        }
        // 再構築対象でペイロード（展開後 tar の diff_id）が変わる
        // platform を事前に可視化する。再エンコード（digest 差のみで
        // diff_ids 一致）は additive push が通るが、diff_ids 差は
        // push で正当な拒否になるためここで予告する。
        for (target, new_descriptor) in &new_manifests {
            let Some(old) = all.get(target) else {
                continue;
            };
            if old.digest == new_descriptor.digest {
                continue;
            }
            if payload_changed(
                &job.output,
                staging.path(),
                &old.digest,
                &new_descriptor.digest,
            )
            .unwrap_or(false)
            {
                eprintln!(
                    "注意: {}/{} のペイロードが変わります（正当なリビルドなら push --replace-platform {}/{} で置換してください）",
                    target.architecture(),
                    target.variant(),
                    target.architecture(),
                    target.variant()
                );
            }
        }
        all.extend(new_manifests);
        // Target の Ord は Target::all() の宣言順なので、merge と
        // 全再生成で同一集合なら同一の nested index digest になる。
        all.into_values().collect()
    } else {
        new_manifests.into_iter().map(|(_, d)| d).collect()
    };
    if manifests.is_empty() {
        return Err(Error::message(format!(
            "{} の platform が0件になります（layout 全体の削除は clean --oci-layouts を使用してください）",
            job.output.display()
        )));
    }
    assemble_index(blobs, staging.path(), &job.image, manifests)?;
    // 追加ファイル（sbom 等）は staging 側に書いて rename と一緒に
    // 確定する。output 確定後の追記だと「layout あり・sbom 無し」の
    // 窓が残る。名前は単一コンポーネントの非予約名に限る
    // （"../x" で staging を脱出したり index.json/blobs を
    // 上書きしたりする入力を拒否する）。
    for (name, bytes) in &job.extra_files {
        let mut components = Path::new(name).components();
        let valid = matches!(components.next(), Some(std::path::Component::Normal(_)))
            && components.next().is_none()
            && !matches!(
                name.as_str(),
                "index.json" | "oci-layout" | "blobs" | ".locks"
            );
        if !valid {
            return Err(Error::message(format!(
                "extra_files の名前が不正です: {name}（単一のファイル名のみ指定できます）"
            )));
        }
        util::write(&staging.path().join(name), bytes)?;
    }
    replace_dir(&job.output, staging)
}

/// 既存 layout の platform 別 manifest descriptor を読む。
/// flat（index.json → manifest 直参照）と nested（index.json →
/// index blob → manifest 群）の両形式を受理する。未知 platform や
/// 破損は Err（呼び出し側が --replace を促すメッセージを含む）。
fn existing_manifests(output: &Path) -> Result<BTreeMap<Target, Descriptor>> {
    let mut manifests = BTreeMap::new();
    if !output.join("index.json").is_file() {
        return Ok(manifests);
    }
    let top: ImageIndex = read_layout_json(&output.join("index.json")).map_err(|error| {
        Error::message(format!(
            "{} の index.json が読めません: {error}（--replace で全再生成してください）",
            output.display()
        ))
    })?;
    let mut entries = top.manifests.into_iter();
    let Some(entry) = entries.next() else {
        return Err(Error::message(format!(
            "{} の index.json に manifest がありません（--replace で全再生成してください）",
            output.display()
        )));
    };
    if entries.next().is_some() {
        return Err(Error::message(format!(
            "{} の index.json が複数エントリを持ちます（--replace で全再生成してください）",
            output.display()
        )));
    }
    let descriptors = match entry.media_type.as_str() {
        MEDIA_MANIFEST => vec![entry],
        MEDIA_INDEX => {
            let nested: ImageIndex = read_blob_json(output, &entry.digest).map_err(|error| {
                Error::message(format!(
                    "{} のネスト index が読めません: {error}（--replace で全再生成してください）",
                    output.display()
                ))
            })?;
            nested.manifests
        }
        other => {
            return Err(Error::message(format!(
                "{} の index.json が想定外の型を指しています: {other}（--replace で全再生成してください）",
                output.display()
            )));
        }
    };
    for descriptor in descriptors {
        if descriptor.media_type != MEDIA_MANIFEST {
            return Err(Error::message(format!(
                "{} が manifest 以外を指しています: {}（--replace で全再生成してください）",
                output.display(),
                descriptor.media_type
            )));
        }
        let mut descriptor = descriptor;
        // descriptor の platform をまず使う。skopeo pull した flat layout
        // は platform が欠落し得るため、manifest → config blob に
        // フォールバックする（layout_io::resolve_target と同じ規則）。
        let target = match descriptor.platform.as_ref().and_then(|platform| {
            Target::lookup(
                &platform.architecture,
                platform.variant.as_deref().unwrap_or_default(),
            )
        }) {
            Some(target) => target,
            None => config_target(output, &descriptor.digest).map_err(|error| {
                Error::message(format!("{error}（--replace で全再生成してください）"))
            })?,
        };
        // platform が欠落・不完全な descriptor を正規化する。
        // このまま nested index に載せると platform 情報を失った
        // manifest エントリが残り、消費側が platform 選択できない
        // （descriptor の platform は index メタデータで、manifest
        //  blob の digest には影響しない）。
        if descriptor
            .platform
            .as_ref()
            .is_none_or(|platform| platform.variant.is_none())
        {
            descriptor.platform = Some(PlatformObject::from(target));
        }
        manifests.insert(target, descriptor);
    }
    Ok(manifests)
}

/// 新旧 manifest の config が指す `rootfs.diff_ids`（展開後 tar の
/// digest 列）が変わるかを比較する。読み取り不能なら None
/// （commit の成否には関与しない情報提供のため）。
fn payload_changed(
    old_layout: &Path,
    new_layout: &Path,
    old_digest: &Digest,
    new_digest: &Digest,
) -> Option<bool> {
    let old_manifest: ImageManifest = read_blob_json(old_layout, old_digest).ok()?;
    let old_config: ImageConfig = read_blob_json(old_layout, &old_manifest.config.digest).ok()?;
    let new_manifest: ImageManifest = read_blob_json(new_layout, new_digest).ok()?;
    let new_config: ImageConfig = read_blob_json(new_layout, &new_manifest.config.digest).ok()?;
    Some(old_config.rootfs.diff_ids != new_config.rootfs.diff_ids)
}

/// descriptor の platform が欠落しているとき、manifest blob の
/// config（architecture/variant を保持）から `Target` を復元する。
fn config_target(output: &Path, digest: &Digest) -> Result<Target> {
    let manifest: ImageManifest = read_blob_json(output, digest).map_err(|error| {
        Error::message(format!(
            "{} の manifest blob が読めません: {error}",
            output.display()
        ))
    })?;
    let config: crate::oci::ImageConfig =
        read_blob_json(output, &manifest.config.digest).map_err(|error| {
            Error::message(format!(
                "{} の config blob が読めません: {error}",
                output.display()
            ))
        })?;
    Target::lookup(
        &config.architecture,
        config.variant.as_deref().unwrap_or_default(),
    )
    .ok_or_else(|| {
        Error::message(format!(
            "{} の manifest に platform を特定できません: {}",
            output.display(),
            digest.prefixed()
        ))
    })
}

/// 保持する manifest が参照する blob 群（manifest 自体・config・
/// layers）を既存 layout から staging の blobs へ引き継ぐ。
/// 同一 FS なのでハードリンクを優先し、失敗時はコピーに落ちる。
fn carry_manifest_blobs(output: &Path, staging_blobs: &Path, manifest: &Descriptor) -> Result<()> {
    let source_blobs = output.join("blobs/sha256");
    let parsed: ImageManifest = read_blob_json(output, &manifest.digest).map_err(|error| {
        Error::message(format!(
            "{} の manifest blob が読めません: {error}（--replace で全再生成してください）",
            output.display()
        ))
    })?;
    let mut digests = vec![manifest.digest.clone(), parsed.config.digest.clone()];
    digests.extend(parsed.layers.iter().map(|layer| layer.digest.clone()));
    for digest in digests {
        let src = source_blobs.join(digest.encoded());
        let dst = staging_blobs.join(digest.encoded());
        if dst.exists() {
            continue;
        }
        if !src.is_file() {
            return Err(Error::message(format!(
                "{} の blob が欠損しています: {}（--replace で全再生成してください）",
                output.display(),
                digest.prefixed()
            )));
        }
        if fs::hard_link(&src, &dst).is_err() {
            fs::copy(&src, &dst).map_err(|source| Error::io(&dst, source))?;
        }
    }
    Ok(())
}

fn read_layout_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).map_err(|source| Error::io(path, source))?;
    serde_json::from_slice(&bytes)
        .map_err(|source| Error::message(format!("{} のパースに失敗: {source}", path.display())))
}

fn read_blob_json<T: DeserializeOwned>(layout: &Path, digest: &Digest) -> Result<T> {
    read_layout_json(&layout.join("blobs/sha256").join(digest.encoded()))
}

/// manifest 集合から index.json / oci-layout を組み立てる。
/// containers/image の oci: トランスポートは index.json の複数エントリを
/// 受理しない（ErrMoreThanOneImage）。複数 manifest のときは buildx 出力と
/// 同じく「index.json → 単一 descriptor → ネストした Image Index blob」の
/// 形にする（push 後にタグが指す digest はネスト index blob の digest）。
/// 単一 manifest のときは manifest を直接指す平坦 index にする:
/// podman は platform 選択で nested index へ降りないため、nested 化すると
/// `oci:` の直接実行（build-env 等）が "no image found" で失敗する。
fn assemble_index(
    blobs: &Path,
    staging: &Path,
    image: &Image,
    manifests: Vec<Descriptor>,
) -> Result<()> {
    let mut manifests = manifests;
    let index = if manifests.len() == 1 {
        let mut manifest = manifests.remove(0);
        manifest
            .annotations
            .get_or_insert_with(BTreeMap::new)
            .insert(ANNOTATION_REF_NAME.to_string(), image.version.clone());
        ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests: vec![manifest],
            annotations: Some(image_annotations(image)),
        }
    } else {
        // flat 形式の top descriptor には ref.name が付くため、merge で
        // flat → nested に遷移する保持 manifest からは外して正規化する
        // （nested 内の manifest descriptor は zstd 注釈のみが正準形）。
        for manifest in &mut manifests {
            if let Some(annotations) = manifest.annotations.as_mut() {
                annotations.remove(ANNOTATION_REF_NAME);
                if annotations.is_empty() {
                    manifest.annotations = None;
                }
            }
        }
        // GHCR は multi-arch タグの description 等を index の
        // annotations から読むため、ネスト index にも付与する。
        let nested = ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests,
            annotations: Some(image_annotations(image)),
        };
        let (nested_digest, nested_size) = store_blob(blobs, &serde_json::to_vec(&nested)?)?;
        ImageIndex {
            schema_version: 2,
            media_type: MEDIA_INDEX.to_string(),
            manifests: vec![Descriptor {
                media_type: MEDIA_INDEX.to_string(),
                digest: nested_digest,
                size: nested_size,
                platform: Some(PlatformObject {
                    architecture: "unknown".to_string(),
                    os: "unknown".to_string(),
                    variant: None,
                }),
                annotations: Some(BTreeMap::from([(
                    ANNOTATION_REF_NAME.to_string(),
                    image.version.clone(),
                )])),
            }],
            annotations: Some(image_annotations(image)),
        }
    };
    util::write(&staging.join("index.json"), serde_json::to_vec(&index)?)?;
    util::write(
        &staging.join("oci-layout"),
        serde_json::to_vec(&OciLayout {
            image_layout_version: "1.0.0".to_string(),
        })?,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_manifest(
    blobs: &Path,
    image: &Image,
    target: Target,
    layer_dirs: &[PathBuf],
    zstd_level: ZstdLevel,
    exec: &ExecConfig,
    extra_annotations: BTreeMap<String, String>,
    cache: &layer::BlobCache,
) -> Result<Descriptor> {
    // 層の tar 読み取りは成果物ロックの read 取得下で行う。
    // restore --force 等が成果物を rename する窓で読み取りが瞬断
    // しないための保護（lock の basename 規約は build/restore と同一）。
    let mut artifact_locks: Vec<_> = layer_dirs
        .iter()
        .map(|dir| workspace::artifact_dir_lock(dir))
        .collect::<Result<_>>()?;
    let _artifact_guards: Vec<_> = artifact_locks
        .iter_mut()
        .map(|lock| {
            lock.read()
                .map_err(|source| Error::message(format!("成果物ロックの取得に失敗: {source}")))
        })
        .collect::<Result<_>>()?;
    let mut layers = Vec::new();
    let mut diff_ids = Vec::new();
    for dir in layer_dirs {
        let layer = layer::build(blobs, dir, zstd_level, cache)?;
        diff_ids.push(layer.diff_id);
        layers.push(Descriptor {
            media_type: MEDIA_LAYER.to_string(),
            digest: layer.blob,
            size: layer.size,
            platform: None,
            annotations: None,
        });
    }
    let config = ImageConfig {
        created: util::source_date_epoch(),
        architecture: target.architecture().to_string(),
        os: "linux".to_string(),
        variant: Some(target.variant().to_string()),
        config: exec.clone(),
        rootfs: RootFs {
            type_: "layers".to_string(),
            diff_ids,
        },
    };
    let (config_digest, config_size) = store_blob(blobs, &serde_json::to_vec(&config)?)?;
    let mut annotations = image_annotations(image);
    annotations.extend(extra_annotations);
    let manifest = ImageManifest {
        schema_version: 2,
        media_type: MEDIA_MANIFEST.to_string(),
        artifact_type: None,
        config: Descriptor {
            media_type: MEDIA_CONFIG.to_string(),
            digest: config_digest,
            size: config_size,
            platform: None,
            annotations: None,
        },
        layers,
        annotations,
    };
    let (digest, size) = store_blob(blobs, &serde_json::to_vec(&manifest)?)?;
    Ok(Descriptor {
        media_type: MEDIA_MANIFEST.to_string(),
        digest,
        size,
        annotations: Some(BTreeMap::from([(
            ANNOTATION_ZSTD.to_string(),
            "true".to_string(),
        )])),
        platform: Some(PlatformObject::from(target)),
    })
}

pub(crate) fn store_blob(blobs: &Path, data: &[u8]) -> Result<(Digest, u64)> {
    let digest = Digest::from_hex(crate::util::sha256_hex(data));
    let path = blobs.join(digest.encoded());
    // 並列ビルドで同一 blob を書く競合に備え、tempfile + persist
    // （rename）でアトミックに確定する。
    let tmp = tempfile::NamedTempFile::new_in(blobs).map_err(|source| Error::io(blobs, source))?;
    util::write(tmp.path(), data)?;
    tmp.persist(&path).map_err(|source| {
        Error::message(format!(
            "{} を {} へ配置できません: {}",
            source.file.path().display(),
            path.display(),
            source.error
        ))
    })?;
    Ok((digest, data.len() as u64))
}

fn backup_path(output: &Path) -> PathBuf {
    let mut name = output
        .file_name()
        .unwrap_or_else(|| std::ffi::OsStr::new("layout"))
        .to_os_string();
    name.push(".bak");
    match output.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.join(name),
        _ => PathBuf::from(name),
    }
}

pub(crate) fn replace_dir(output: &Path, staging: tempfile::TempDir) -> Result<()> {
    let backup = backup_path(output);
    if backup.exists() {
        fs::remove_dir_all(&backup).map_err(|source| Error::io(&backup, source))?;
    }
    if output.exists() {
        fs::rename(output, &backup).map_err(|source| Error::io(output, source))?;
    }
    let staging_path = staging.keep();
    if let Err(source) = fs::rename(&staging_path, output) {
        if backup.exists() {
            let _ = fs::rename(&backup, output);
        }
        let _ = fs::remove_dir_all(&staging_path);
        return Err(Error::io(output, source));
    }
    // .bak の削除失敗で出力自体を失敗扱いにはしない。
    // 残骸は次回実行時に再削除される。
    if backup.exists()
        && let Err(error) = fs::remove_dir_all(&backup)
    {
        eprintln!(
            "警告: 旧レイアウトのバックアップ {} の削除に失敗しました: {error}",
            backup.display()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::backup_path;
    use std::path::Path;

    #[test]
    fn backup_path_keeps_version_suffix() {
        assert_eq!(
            backup_path(Path::new("oci-layouts/glibc-2.44")),
            Path::new("oci-layouts/glibc-2.44.bak")
        );
        assert_eq!(
            backup_path(Path::new("oci-layouts/glibc-2.45")),
            Path::new("oci-layouts/glibc-2.45.bak")
        );
        assert_ne!(
            backup_path(Path::new("oci-layouts/glibc-2.44")),
            backup_path(Path::new("oci-layouts/glibc-2.45"))
        );
    }
}
