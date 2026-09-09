use std::fs;

use containers_builder::oci;
use serde_json::Value;

use super::sample;
use super::support::{amd64, blobs_dir, build_with_bytes, walk_blobs};

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
        },
        &[],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap_err();
    assert!(err.to_string().contains("レイヤー"));
}

#[test]
fn rebuild_replaces_stale_blobs() {
    let first = build_with_bytes(&[amd64("v1")], b"hello");
    assert_eq!(walk_blobs(&blobs_dir(&first)).len(), 3);
    let second = build_with_bytes(&[amd64("v1")], b"changed");
    oci::build(
        &first.output,
        &oci::Image {
            name: "glibc".to_string(),
            version: "2.44".to_string(),
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
        },
        &[(amd64("v1"), vec![layer])],
        oci::DEFAULT_ZSTD_LEVEL,
    );
    let _ = fs::set_permissions(&secret, fs::Permissions::from_mode(0o644));
    let error = error.expect_err("tar が読めないファイルで失敗するはずです");
    assert!(
        error.to_string().contains("tar"),
        "staging 開始後の失敗であること: {error}"
    );
    assert_eq!(fs::read_to_string(&marker).unwrap(), "keep");
    assert_eq!(
        fs::read_to_string(layout.output.join("index.json")).unwrap(),
        original
    );
}
