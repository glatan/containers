use std::fs;

use containers_builder::oci;
use containers_builder::target::Target;
use serde_json::Value;

use super::sample;
use super::support::{amd64, assert_descriptor, blob, build, index, manifests, nested_index};
use super::{documented_platforms, platform_statuses};

#[test]
fn index_is_image_index() {
    let layout = sample();
    let index = index(&layout);
    assert_eq!(index["schemaVersion"], 2);
    assert_eq!(
        index["mediaType"],
        "application/vnd.oci.image.index.v1+json"
    );
    assert!(!index["manifests"].as_array().unwrap().is_empty());
}

/// containers/image の oci: トランスポート互換（buildx 出力と同型）:
/// index.json は ref.name 注釈付きの単一 descriptor でネスト index を指す。
#[test]
fn index_json_points_to_nested_index() {
    let layout = sample();
    let index = index(&layout);
    let entries = index["manifests"].as_array().unwrap();
    assert_eq!(entries.len(), 1);
    let entry = &entries[0];
    assert_descriptor(entry);
    assert_eq!(
        entry["mediaType"],
        "application/vnd.oci.image.index.v1+json"
    );
    assert_eq!(entry["platform"]["os"], "unknown");
    assert_eq!(entry["platform"]["architecture"], "unknown");
    assert_eq!(
        entry["annotations"]["org.opencontainers.image.ref.name"],
        "2.44"
    );
    let nested = nested_index(&layout);
    assert_eq!(nested["schemaVersion"], 2);
    assert_eq!(
        nested["mediaType"],
        "application/vnd.oci.image.index.v1+json"
    );
    assert_eq!(nested["manifests"].as_array().unwrap().len(), 2);
}

#[test]
fn index_platform_required_fields() {
    let layout = sample();
    for (entry, _, _) in manifests(&layout) {
        let platform = &entry["platform"];
        assert!(!platform["architecture"].as_str().unwrap().is_empty());
        assert_eq!(platform["os"], "linux");
        assert!(platform["variant"].as_str().is_some());
    }
}

#[test]
fn implementation_defined_variant_is_accepted() {
    let layout = build(&[Target::lookup("s390x", "z17").unwrap()]);
    let (entry, _, config) = &manifests(&layout)[0];
    assert_eq!(entry["platform"]["architecture"], "s390x");
    assert_eq!(entry["platform"]["variant"], "z17");
    assert_eq!(config["architecture"], "s390x");
    assert_eq!(config["variant"], "z17");
}

#[test]
fn multi_platform_index_lists_each_manifest() {
    let layout = sample();
    let entries = nested_index(&layout)["manifests"]
        .as_array()
        .unwrap()
        .clone();
    assert_eq!(entries.len(), 2);
    for entry in entries {
        let digest = entry["digest"].as_str().unwrap();
        let manifest: Value = serde_json::from_slice(&blob(&layout, digest)).unwrap();
        assert_eq!(manifest["layers"].as_array().unwrap().len(), 1);
    }
}

#[test]
fn documented_platforms_match_targets() {
    let documented = documented_platforms();
    let targets = Target::all()
        .map(|target| {
            (
                target.architecture().to_string(),
                target.variant().to_string(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        documented, targets,
        "docs/user/targets.md と Target::all() が一致しません"
    );
    let md = include_str!("../../../docs/user/targets.md");
    assert!(
        md.contains("すべてのパッケージで公開されている"),
        "公開範囲の限定が targets.md にありません"
    );
    let statuses = platform_statuses();
    assert_eq!(
        statuses
            .iter()
            .map(|(architecture, variant, _)| (architecture.clone(), variant.clone()))
            .collect::<Vec<_>>(),
        targets,
        "docs/dev/oci.md の status 表と Target::all() が一致しません"
    );
    for (architecture, _, status) in statuses {
        let expected = if architecture == "s390x" {
            "defined"
        } else {
            "listed"
        };
        assert_eq!(status, expected, "{architecture}");
    }
}

#[test]
fn amd64_v1_is_a_listed_platform_variant() {
    let layout = build(&[amd64("v1")]);
    let (entry, _, config) = &manifests(&layout)[0];
    assert_eq!(entry["platform"]["architecture"], "amd64");
    assert_eq!(entry["platform"]["variant"], "v1");
    assert_eq!(config["variant"], "v1");
    assert!(
        documented_platforms()
            .iter()
            .any(|(architecture, variant)| architecture == "amd64" && variant == "v1")
    );
}

/// GHCR は multi-arch タグの description/source をイメージ index の
/// annotations から読む。トップ index とネスト index の両方に付く。
#[test]
fn index_carries_publish_annotations() {
    let layout = sample();
    for value in [index(&layout), nested_index(&layout)] {
        let annotations = value["annotations"]
            .as_object()
            .expect("index に annotations が必要です");
        assert_eq!(
            annotations["org.opencontainers.image.source"].as_str(),
            Some("https://github.com/glatan/containers")
        );
    }
}

/// ネスト index blob のフィールド順が Go の Index marshal 順
/// （manifests → annotations）と一致すること。containers/image の
/// 再シリアライズとバイト一致しないと digest が食い違う。
#[test]
fn nested_index_field_order_matches_go_marshal() {
    let layout = sample();
    let entry = &index(&layout)["manifests"][0];
    let raw = String::from_utf8(blob(&layout, entry["digest"].as_str().unwrap())).unwrap();
    assert!(
        raw.find("\"manifests\"").unwrap() < raw.find("\"annotations\"").unwrap(),
        "{raw}"
    );
}

#[test]
fn flat_index_follows_image_description() {
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
        &[(amd64("v1"), vec![layer])],
        oci::DEFAULT_ZSTD_LEVEL,
    )
    .unwrap();
    let layout = super::support::Layout::at(output);
    let top = index(&layout);
    let annotations = top["annotations"].as_object().unwrap();
    assert_eq!(
        annotations["org.opencontainers.image.description"],
        "test description"
    );
}
