use std::fs;

use containers_builder::oci;
use serde_json::Value;

use super::sample;
use super::support::{amd64, blob, manifests};

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
        },
        &[(amd64("v1"), vec![layer])],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap();
    let index: Value =
        serde_json::from_str(&fs::read_to_string(output.join("index.json")).unwrap()).unwrap();
    let nested_digest = index["manifests"][0]["digest"].as_str().unwrap();
    let nested: Value = serde_json::from_slice(
        &fs::read(
            output
                .join("blobs/sha256")
                .join(nested_digest.trim_start_matches("sha256:")),
        )
        .unwrap(),
    )
    .unwrap();
    let digest = nested["manifests"][0]["digest"].as_str().unwrap();
    let hex = digest.trim_start_matches("sha256:");
    let manifest: Value =
        serde_json::from_slice(&fs::read(output.join("blobs/sha256").join(hex)).unwrap()).unwrap();
    assert_eq!(
        manifest["annotations"]["org.opencontainers.image.title"],
        r#"glibc"quote"#
    );
}
