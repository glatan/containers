use std::fs;

use containers_builder::oci;
use serde_json::Value;

use super::sample;
use super::support::{amd64, blob, manifest_index, manifests};

#[test]
fn manifest_required_fields() {
    let layout = sample();
    for (_, manifest, _) in manifests(&layout) {
        assert_eq!(manifest["schemaVersion"], 2);
        assert_eq!(
            manifest["mediaType"],
            "application/vnd.oci.image.manifest.v1+json"
        );
        assert_eq!(
            manifest["config"]["mediaType"],
            "application/vnd.oci.image.config.v1+json"
        );
        let layers = manifest["layers"].as_array().unwrap();
        assert!(!layers.is_empty());
        assert_eq!(
            layers[0]["mediaType"],
            "application/vnd.oci.image.layer.v1.tar+zstd"
        );
        let magic = blob(&layout, layers[0]["digest"].as_str().unwrap());
        assert_eq!(
            &magic[..4],
            &[0x28, 0xB5, 0x2F, 0xFD],
            "zstd フレームではありません"
        );
    }
}

#[test]
fn manifest_annotations() {
    let layout = sample();
    for (_, manifest, _) in manifests(&layout) {
        let annotations = manifest["annotations"].as_object().unwrap();
        assert_eq!(
            annotations["org.opencontainers.image.title"].as_str(),
            Some("glibc")
        );
        assert_eq!(
            annotations["org.opencontainers.image.version"].as_str(),
            Some("2.44")
        );
    }
}

#[test]
fn json_special_characters_are_escaped() {
    let tmp = tempfile::tempdir().unwrap();
    let layer = tmp.path().join("layer");
    fs::create_dir_all(&layer).unwrap();
    fs::write(layer.join("file.txt"), "hello").unwrap();
    let output = tmp.path().join("layout");
    oci::build(
        &output,
        &oci::Image {
            name: r#"glibc"quote"#.to_string(),
            version: "2.44".to_string(),
            description: None,
        },
        &[(amd64("v1"), vec![layer])],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap();
    let layout = super::support::Layout::at(output.clone());
    let mindex = manifest_index(&layout);
    let digest = mindex["manifests"][0]["digest"].as_str().unwrap();
    let hex = digest.trim_start_matches("sha256:");
    let manifest: Value =
        serde_json::from_slice(&fs::read(output.join("blobs/sha256").join(hex)).unwrap()).unwrap();
    assert_eq!(
        manifest["annotations"]["org.opencontainers.image.title"],
        r#"glibc"quote"#
    );
}

#[test]
fn manifest_carries_publish_annotations() {
    // source は全イメージ共通。description は Image.description が
    // あるときのみ付与される（support の Image は None）。
    let layout = sample();
    for (_, manifest, _) in manifests(&layout) {
        let annotations = manifest["annotations"].as_object().unwrap();
        assert_eq!(
            annotations["org.opencontainers.image.source"].as_str(),
            Some("https://github.com/glatan/containers")
        );
        assert!(!annotations.contains_key("org.opencontainers.image.description"));
    }
}

#[test]
fn image_description_is_propagated_to_manifest() {
    let tmp = tempfile::tempdir().unwrap();
    let layer = tmp.path().join("layer");
    fs::create_dir_all(&layer).unwrap();
    fs::write(layer.join("file.txt"), "hello").unwrap();
    let output = tmp.path().join("layout");
    oci::build(
        &output,
        &oci::Image {
            name: "zlib".to_string(),
            version: "1".to_string(),
            description: Some("test description".to_string()),
        },
        &[
            (amd64("v1"), vec![layer.clone()]),
            (amd64("v3"), vec![layer]),
        ],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap();
    let layout = super::support::Layout::at(output);
    for (_, manifest, _) in manifests(&layout) {
        assert_eq!(
            manifest["annotations"]["org.opencontainers.image.description"],
            "test description"
        );
    }
}
