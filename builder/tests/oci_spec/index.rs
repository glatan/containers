use containers_builder::target::Target;
use serde_json::Value;

use super::documented_platforms;
use super::sample;
use super::support::{amd64, blob, build, index, manifests};

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
    let entries = index(&layout)["manifests"].as_array().unwrap().clone();
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
        "docs/targets.md と Target::all() が一致しません"
    );
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
