use std::fs;

use containers_builder::oci;
use serde_json::Value;

use super::sample;
use super::support::{
    Layout, amd64, blobs_dir, build_with_bytes, index, manifest_index, nested_index, walk_blobs,
};

#[test]
fn layout_has_required_entries() {
    let layout = sample();
    assert!(layout.output.join("oci-layout").is_file());
    assert!(layout.output.join("index.json").is_file());
    assert!(blobs_dir(&layout).is_dir());
}

#[test]
fn oci_layout_version_is_1_0_0() {
    let layout = sample();
    let header: Value =
        serde_json::from_str(&fs::read_to_string(layout.output.join("oci-layout")).unwrap())
            .unwrap();
    assert!(header.is_object());
    assert_eq!(header["imageLayoutVersion"], "1.0.0");
}

#[test]
fn empty_layers_are_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let err = oci::build(
        &tmp.path().join("layout"),
        &oci::Image {
            name: "glibc".to_string(),
            version: "2.44".to_string(),
            description: None,
        },
        &[],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap_err();
    assert!(err.to_string().contains("レイヤー"));
}

#[test]
fn rebuild_replaces_stale_blobs() {
    // 単一 platform は flat index のため層・config・manifest の 3 blob。
    let first = build_with_bytes(&[amd64("v1")], b"hello");
    assert_eq!(walk_blobs(&blobs_dir(&first)).len(), 3);
    let second = build_with_bytes(&[amd64("v1")], b"changed");
    oci::build(
        &first.output,
        &oci::Image {
            name: "glibc".to_string(),
            version: "2.44".to_string(),
            description: None,
        },
        &[(amd64("v1"), vec![second.tmp().join("layer")])],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap();
    assert_eq!(
        walk_blobs(&blobs_dir(&first)).len(),
        3,
        "再ビルド後に古い blob が残ってはならない"
    );
}

#[test]
fn missing_layer_dir_is_rejected_before_replace() {
    let layout = sample();
    let marker = layout.output.join("keep.txt");
    fs::write(&marker, "keep").unwrap();
    let error = oci::build(
        &layout.output,
        &oci::Image {
            name: "glibc".to_string(),
            version: "2.44".to_string(),
            description: None,
        },
        &[(amd64("v1"), vec![layout.tmp().join("missing")])],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap_err();
    assert!(error.to_string().contains("レイヤーのディレクトリ"));
    assert_eq!(fs::read_to_string(&marker).unwrap(), "keep");
}

#[test]
fn failed_layer_after_staging_preserves_existing_layout() {
    use std::os::unix::fs::PermissionsExt;

    let layout = sample();
    let marker = layout.output.join("keep.txt");
    fs::write(&marker, "keep").unwrap();
    let original = fs::read_to_string(layout.output.join("index.json")).unwrap();

    let bad = tempfile::tempdir().unwrap();
    let layer = bad.path().join("layer");
    fs::create_dir_all(&layer).unwrap();
    let secret = layer.join("secret.bin");
    fs::write(&secret, b"cannot-read").unwrap();
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o000)).unwrap();

    let error = oci::build(
        &layout.output,
        &oci::Image {
            name: "glibc".to_string(),
            version: "2.44".to_string(),
            description: None,
        },
        &[(amd64("v1"), vec![layer])],
        oci::DEFAULT_ZSTD_LEVEL,
    );
    let _ = fs::set_permissions(&secret, fs::Permissions::from_mode(0o644));
    let error = error.expect_err("tar が読めないファイルで失敗するはずです");
    assert!(
        format!("{error:#}").contains("tar"),
        "staging 開始後の失敗であること: {error:#}"
    );
    assert_eq!(fs::read_to_string(&marker).unwrap(), "keep");
    assert_eq!(
        fs::read_to_string(layout.output.join("index.json")).unwrap(),
        original
    );
}

#[test]
fn nested_index_descriptors_carry_zstd_annotation() {
    // skopeo copy --all が push 時に付与するアノテーションを pack 側でも
    // 付け、ネスト index blob がリモート manifest list と digest 一致
    // することを保証する。
    let layout = sample();
    let nested = nested_index(&layout);
    for descriptor in nested["manifests"].as_array().unwrap() {
        assert_eq!(
            descriptor["annotations"]["io.github.containers.compression.zstd"],
            "true"
        );
    }
}

#[test]
fn flat_index_descriptor_merges_ref_name_and_zstd() {
    // 単一 platform の flat index では descriptor に ref.name を付けるが、
    // 既存の zstd アノテーションを上書きで消してはならない。
    let layout = super::build(&[amd64("v1")]);
    let flat = index(&layout);
    let entries = flat["manifests"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    let descriptor = &entries[0];
    assert_eq!(
        descriptor["annotations"]["org.opencontainers.image.ref.name"],
        "2.44"
    );
    assert_eq!(
        descriptor["annotations"]["io.github.containers.compression.zstd"],
        "true"
    );
}

#[test]
fn failed_job_does_not_block_other_layouts() {
    use std::collections::BTreeMap;
    use std::os::unix::fs::PermissionsExt;

    let tmp = tempfile::tempdir().unwrap();
    let good_layer = tmp.path().join("good-layer");
    fs::create_dir_all(&good_layer).unwrap();
    fs::write(good_layer.join("ok.txt"), "ok").unwrap();
    let bad_layer = tmp.path().join("bad-layer");
    fs::create_dir_all(&bad_layer).unwrap();
    let secret = bad_layer.join("secret.bin");
    fs::write(&secret, b"cannot-read").unwrap();
    fs::set_permissions(&secret, fs::Permissions::from_mode(0o000)).unwrap();

    let good_out = tmp.path().join("good-layout");
    let bad_out = tmp.path().join("bad-layout");
    let job =
        |output: std::path::PathBuf, name: &str, dirs: Vec<std::path::PathBuf>| oci::LayoutJob {
            output,
            image: oci::Image {
                name: name.to_string(),
                version: "1".to_string(),
                description: None,
            },
            layers: vec![(amd64("v1"), dirs)],
            config: oci::ExecConfig::default(),
            annotate: Box::new(|_| BTreeMap::new()),
            merge: false,
            remove: Vec::new(),
            extra_files: Vec::new(),
        };
    let outcome = oci::build_all_annotated(
        vec![
            job(good_out.clone(), "good", vec![good_layer]),
            job(bad_out.clone(), "bad", vec![bad_layer]),
        ],
        oci::DEFAULT_ZSTD_LEVEL,
        4,
    )
    .unwrap();
    let _ = fs::set_permissions(&secret, fs::Permissions::from_mode(0o644));

    assert_eq!(outcome.succeeded, ["good"]);
    assert_eq!(outcome.failed.len(), 1);
    assert!(outcome.failed[0].starts_with("bad"), "{:?}", outcome.failed);
    assert!(good_out.join("index.json").is_file());
    assert!(!bad_out.exists(), "失敗ジョブの出力は確定されないはずです");
}

/// merge/remove ジョブを組み立てる。`layers` 空の除去専用ジョブも作れる。
fn merge_job(
    output: std::path::PathBuf,
    layers: Vec<(containers_builder::target::Target, Vec<std::path::PathBuf>)>,
    remove: Vec<containers_builder::target::Target>,
) -> oci::LayoutJob {
    use std::collections::BTreeMap;
    oci::LayoutJob {
        output,
        image: oci::Image {
            name: "glibc".to_string(),
            version: "2.44".to_string(),
            description: None,
        },
        layers,
        config: oci::ExecConfig::default(),
        annotate: Box::new(|_| BTreeMap::new()),
        merge: true,
        remove,
        extra_files: Vec::new(),
    }
}

fn layer_dir(tmp: &tempfile::TempDir, bytes: &[u8]) -> std::path::PathBuf {
    let dir = tmp.path().join("layer");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("file.txt"), bytes).unwrap();
    dir
}

/// manifest_index の platform 別 digest を (arch, variant) → digest に畳む。
fn platform_digests(layout: &Layout) -> std::collections::BTreeMap<String, String> {
    manifest_index(layout)["manifests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let platform = &entry["platform"];
            (
                format!(
                    "{}/{}",
                    platform["architecture"].as_str().unwrap(),
                    platform["variant"].as_str().unwrap_or("")
                ),
                entry["digest"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn merge_adds_platform_preserving_existing_manifest() {
    // flat（単一 platform）から merge で nested へ遷移し、
    // 保持した platform の manifest digest が不変であること。
    let first = build_with_bytes(&[amd64("v1")], b"one");
    let v1 = platform_digests(&first)["amd64/v1"].clone();

    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"two");
    let outcome = oci::build_all_annotated(
        vec![merge_job(
            first.output.clone(),
            vec![(amd64("v4"), vec![layer])],
            vec![],
        )],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.succeeded, ["glibc"], "{:?}", outcome.failed);

    let digests = platform_digests(&first);
    assert_eq!(digests.len(), 2);
    assert_eq!(digests["amd64/v1"], v1, "保持 manifest の digest は不変");
    assert!(digests.contains_key("amd64/v4"));
    // 保持 3 blob + 新規 3 blob + nested index = 7。
    assert_eq!(walk_blobs(&blobs_dir(&first)).len(), 7);
}

#[test]
fn merge_rebuild_replaces_only_that_platform() {
    // 同 platform の再 pack はその manifest のみ差し替え、
    // 他 platform の digest と blob はそのまま残る。
    let layout = build_with_bytes(&[amd64("v1"), amd64("v4")], b"before");
    let before = platform_digests(&layout);

    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"after");
    let outcome = oci::build_all_annotated(
        vec![merge_job(
            layout.output.clone(),
            vec![(amd64("v1"), vec![layer])],
            vec![],
        )],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.succeeded, ["glibc"], "{:?}", outcome.failed);

    let after = platform_digests(&layout);
    assert_eq!(after.len(), 2);
    assert_ne!(
        after["amd64/v1"], before["amd64/v1"],
        "再 pack した platform は差し替わる"
    );
    assert_eq!(after["amd64/v4"], before["amd64/v4"], "他 platform は不変");
    // 旧 v1 の manifest/layer blob が GC されている（保持3+新規3+index=7）。
    let blobs = walk_blobs(&blobs_dir(&layout));
    assert_eq!(blobs.len(), 7);
    let names: Vec<_> = blobs.iter().map(|(name, _)| name.clone()).collect();
    let old_manifest = before["amd64/v1"].strip_prefix("sha256:").unwrap();
    assert!(
        !names.contains(&old_manifest.to_string()),
        "旧 manifest blob は GC される"
    );
}

#[test]
fn remove_platform_returns_single_to_flat() {
    // 2 platform から1つ除去すると flat index に戻り、
    // 残った manifest の descriptor が引き継がれる。
    let layout = build_with_bytes(&[amd64("v1"), amd64("v4")], b"keep");
    let v1 = platform_digests(&layout)["amd64/v1"].clone();

    let outcome = oci::build_all_annotated(
        vec![merge_job(layout.output.clone(), vec![], vec![amd64("v4")])],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.succeeded, ["glibc"], "{:?}", outcome.failed);

    let flat = index(&layout);
    let entries = flat["manifests"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_eq!(
        entry["mediaType"], "application/vnd.oci.image.manifest.v1+json",
        "1件に戻るときは flat index であること"
    );
    assert_eq!(entry["digest"].as_str().unwrap(), v1);
    assert_eq!(entry["platform"]["variant"].as_str().unwrap(), "v1");
    // v4 の manifest blob は GC されている。
    assert_eq!(walk_blobs(&blobs_dir(&layout)).len(), 3);
}

#[test]
fn remove_last_platform_is_rejected() {
    let layout = build_with_bytes(&[amd64("v1")], b"only");
    let before = fs::read_to_string(layout.output.join("index.json")).unwrap();
    let outcome = oci::build_all_annotated(
        vec![merge_job(layout.output.clone(), vec![], vec![amd64("v1")])],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.failed.len(), 1);
    assert!(
        outcome.failed[0].contains("platform"),
        "{:?}",
        outcome.failed
    );
    assert_eq!(
        fs::read_to_string(layout.output.join("index.json")).unwrap(),
        before,
        "失敗時は layout が変わらないこと"
    );
}

#[test]
fn remove_missing_platform_is_rejected() {
    let layout = build_with_bytes(&[amd64("v1")], b"only");
    let outcome = oci::build_all_annotated(
        vec![merge_job(layout.output.clone(), vec![], vec![amd64("v3")])],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.failed.len(), 1);
    assert!(outcome.failed[0].contains("v3"), "{:?}", outcome.failed);
}

#[test]
fn merge_on_missing_layout_builds_fresh() {
    // 既存 layout が無い merge は新規構築と同等。
    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"fresh");
    let output = tmp.path().join("layout");
    let outcome = oci::build_all_annotated(
        vec![merge_job(
            output.clone(),
            vec![(amd64("v1"), vec![layer])],
            vec![],
        )],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.succeeded, ["glibc"], "{:?}", outcome.failed);
    let layout = Layout::at(output);
    assert_eq!(platform_digests(&layout).len(), 1);
}

#[test]
fn merge_result_matches_fresh_build_digest() {
    // 「v1 を pack してから v4 を merge 追加」した layout と、
    // 「v1+v4 を一度に pack」した layout のネスト index digest が
    // 一致すること（増分 merge は全再生成と同じ最終形を作る）。
    let fresh = build_with_bytes(&[amd64("v1"), amd64("v4")], b"same");
    let merged = build_with_bytes(&[amd64("v1")], b"same");
    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"same");
    let outcome = oci::build_all_annotated(
        vec![merge_job(
            merged.output.clone(),
            vec![(amd64("v4"), vec![layer])],
            vec![],
        )],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.succeeded, ["glibc"], "{:?}", outcome.failed);

    let fresh_index = index(&fresh);
    let merged_index = index(&merged);
    assert_eq!(
        merged_index["manifests"][0]["digest"].as_str().unwrap(),
        fresh_index["manifests"][0]["digest"].as_str().unwrap(),
        "merge と全再生成でネスト index digest が一致するはずです"
    );
}

#[test]
fn merge_on_corrupt_layout_is_rejected() {
    // 破損した index.json への merge は黙って全再生成せず拒否する
    // （既存 platform を失わないため --replace を促す）。
    let layout = build_with_bytes(&[amd64("v1")], b"keep");
    fs::write(layout.output.join("index.json"), b"not json").unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"new");
    let outcome = oci::build_all_annotated(
        vec![merge_job(
            layout.output.clone(),
            vec![(amd64("v4"), vec![layer])],
            vec![],
        )],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.failed.len(), 1);
    assert!(
        outcome.failed[0].contains("--replace"),
        "破損 layout への merge は --replace を促す: {:?}",
        outcome.failed
    );
}

#[test]
fn merge_recovers_platform_from_config_blob() {
    // skopeo pull した flat layout は index.json の descriptor に
    // platform が欠落し得る。その場合でも manifest → config blob の
    // architecture/variant から target を復元して merge できること。
    let layout = build_with_bytes(&[amd64("v1")], b"one");
    let v1 = platform_digests(&layout)["amd64/v1"].clone();
    let index_path = layout.output.join("index.json");
    let mut flat: Value = serde_json::from_str(&fs::read_to_string(&index_path).unwrap()).unwrap();
    flat["manifests"][0]
        .as_object_mut()
        .unwrap()
        .remove("platform");
    fs::write(&index_path, serde_json::to_vec(&flat).unwrap()).unwrap();

    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"two");
    let outcome = oci::build_all_annotated(
        vec![merge_job(
            layout.output.clone(),
            vec![(amd64("v4"), vec![layer])],
            vec![],
        )],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.succeeded, ["glibc"], "{:?}", outcome.failed);
    let digests = platform_digests(&layout);
    assert_eq!(digests.len(), 2);
    assert_eq!(digests["amd64/v1"], v1, "platform 欠落の flat も保持される");
}

#[test]
fn extra_files_rejects_path_escape_and_reserved_names() {
    // extra_files の相対名は単一コンポーネントの非予約名に限る。
    // "../" 脱出や index.json/blobs の上書きで確定物を破壊しない。
    for bad in [
        "../escape",
        "a/b",
        "index.json",
        "oci-layout",
        "blobs",
        ".locks",
    ] {
        let tmp = tempfile::tempdir().unwrap();
        let layer = layer_dir(&tmp, b"x");
        let output = tmp.path().join("layout");
        let mut job = merge_job(output.clone(), vec![(amd64("v1"), vec![layer])], vec![]);
        job.extra_files = vec![(bad.to_string(), b"data".to_vec())];
        let outcome = oci::build_all_annotated(vec![job], oci::DEFAULT_ZSTD_LEVEL, 2).unwrap();
        assert_eq!(outcome.failed.len(), 1, "{bad} は拒否されるはずです");
        assert!(
            outcome.failed[0].contains("extra_files"),
            "{bad}: {:?}",
            outcome.failed
        );
    }
}

#[test]
fn extra_files_are_committed_atomically() {
    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"x");
    let output = tmp.path().join("layout");
    let mut job = merge_job(output.clone(), vec![(amd64("v1"), vec![layer])], vec![]);
    job.extra_files = vec![("sbom.spdx.json".to_string(), b"{\"spdx\":true}".to_vec())];
    let outcome = oci::build_all_annotated(vec![job], oci::DEFAULT_ZSTD_LEVEL, 2).unwrap();
    assert_eq!(outcome.succeeded, ["glibc"], "{:?}", outcome.failed);
    assert_eq!(
        fs::read_to_string(output.join("sbom.spdx.json")).unwrap(),
        "{\"spdx\":true}"
    );
}

#[test]
fn pack_and_remove_same_target_is_rejected() {
    // 同一 platform を pack 対象と remove に同時指定した場合、
    // merge の解釈が一意でないため拒否する。
    let layout = build_with_bytes(&[amd64("v1"), amd64("v4")], b"keep");
    let tmp = tempfile::tempdir().unwrap();
    let layer = layer_dir(&tmp, b"new");
    let outcome = oci::build_all_annotated(
        vec![merge_job(
            layout.output.clone(),
            vec![(amd64("v1"), vec![layer])],
            vec![amd64("v1")],
        )],
        oci::DEFAULT_ZSTD_LEVEL,
        2,
    )
    .unwrap();
    assert_eq!(outcome.failed.len(), 1);
    assert!(
        outcome.failed[0].contains("同時指定"),
        "pack/remove の衝突は拒否する: {:?}",
        outcome.failed
    );
}
