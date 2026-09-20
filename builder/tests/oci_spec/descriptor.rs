use containers_builder::util;
use serde_json::Value;

use super::sample;
use super::support::{assert_descriptor, blob, blobs_dir, is_sha256_digest, manifests, walk_blobs};

#[test]
fn blob_bytes_match_digest() {
    let layout = sample();
    for (name, bytes) in walk_blobs(&blobs_dir(&layout)) {
        assert_eq!(
            util::sha256_hex(&bytes),
            name,
            "blobs/sha256/{name} の内容が digest と一致しません"
        );
    }
}

#[test]
fn blob_names_match_digest_grammar() {
    let layout = sample();
    for (name, _) in walk_blobs(&blobs_dir(&layout)) {
        assert!(
            is_sha256_digest(&format!("sha256:{name}")),
            "blob 名が sha256 encoded の文法に合いません: {name}"
        );
    }
}

#[test]
fn descriptors_have_required_fields() {
    let layout = sample();
    for (entry, manifest, _) in manifests(&layout) {
        assert_descriptor(&entry);
        assert_descriptor(&manifest["config"]);
        for layer in manifest["layers"].as_array().unwrap() {
            assert_descriptor(layer);
        }
    }
}

#[test]
fn descriptors_use_sha256_digest() {
    let layout = sample();
    for (entry, manifest, _) in manifests(&layout) {
        assert!(is_sha256_digest(entry["digest"].as_str().unwrap()));
        assert!(is_sha256_digest(
            manifest["config"]["digest"].as_str().unwrap()
        ));
        for layer in manifest["layers"].as_array().unwrap() {
            assert!(is_sha256_digest(layer["digest"].as_str().unwrap()));
        }
    }
}

#[test]
fn descriptor_size_matches_blob() {
    let layout = sample();
    let check = |descriptor: &Value| {
        let digest = descriptor["digest"].as_str().unwrap();
        let size = descriptor["size"].as_u64().unwrap();
        assert_eq!(blob(&layout, digest).len() as u64, size);
    };
    for (entry, manifest, _) in manifests(&layout) {
        check(&entry);
        check(&manifest["config"]);
        for layer in manifest["layers"].as_array().unwrap() {
            check(layer);
        }
    }
}
