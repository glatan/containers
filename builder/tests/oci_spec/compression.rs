use containers_builder::oci::{self, ZstdLevel};

use super::support::{amd64, build_with_bytes, build_with_bytes_level, manifests};

#[test]
fn zstd_level_must_be_1_to_22() {
    assert!(oci::parse_zstd_level(0).is_err());
    assert!(oci::parse_zstd_level(23).is_err());
    assert_eq!(oci::parse_zstd_level(1).unwrap(), 1);
    assert_eq!(oci::parse_zstd_level(22).unwrap(), 22);
    assert_eq!(oci::DEFAULT_ZSTD_LEVEL, 3);
    assert_eq!(oci::PUBLISH_ZSTD_LEVEL, 22);
    assert!(!ZstdLevel::new(19).unwrap().needs_ultra());
    assert!(ZstdLevel::new(20).unwrap().needs_ultra());
}

#[test]
fn same_input_same_layer_digest() {
    // manifest digest は revision 注釈が HEAD コミット依存のため
    // コミット間では変動する（同一コミット内では決定的）。
    let first = build_with_bytes(&[amd64("v1")], b"hello");
    let second = build_with_bytes(&[amd64("v1")], b"hello");
    let first_layer = &manifests(&first)[0].1["layers"][0]["digest"];
    let second_layer = &manifests(&second)[0].1["layers"][0]["digest"];
    assert_eq!(first_layer, second_layer);
    assert_eq!(
        manifests(&first)[0].2["rootfs"]["diff_ids"][0],
        manifests(&second)[0].2["rootfs"]["diff_ids"][0]
    );
}

#[test]
fn publish_level_changes_blob_but_not_diff_id() {
    let default = build_with_bytes_level(&[amd64("v1")], b"hello", oci::DEFAULT_ZSTD_LEVEL);
    let publish = build_with_bytes_level(&[amd64("v1")], b"hello", oci::PUBLISH_ZSTD_LEVEL);
    let default_manifest = &manifests(&default)[0];
    let publish_manifest = &manifests(&publish)[0];
    assert_eq!(
        default_manifest.2["rootfs"]["diff_ids"][0],
        publish_manifest.2["rootfs"]["diff_ids"][0]
    );
    assert_ne!(
        default_manifest.1["layers"][0]["digest"],
        publish_manifest.1["layers"][0]["digest"]
    );
}
