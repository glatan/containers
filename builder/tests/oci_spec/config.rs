use std::io::Write;
use std::process::Command;

use super::sample;
use super::support::{blob, is_sha256_digest, manifests};

#[test]
fn config_required_fields() {
    let layout = sample();
    for (_, _, config) in manifests(&layout) {
        assert!(!config["architecture"].as_str().unwrap().is_empty());
        assert_eq!(config["os"], "linux");
        assert!(config["variant"].as_str().is_some());
        assert_eq!(config["rootfs"]["type"], "layers");
        assert!(!config["rootfs"]["diff_ids"].as_array().unwrap().is_empty());
    }
}

#[test]
fn config_created_is_rfc3339() {
    let layout = sample();
    let manifests = manifests(&layout);
    let created = manifests[0].2["created"].as_str().unwrap();
    assert!(
        chrono_like_rfc3339(created),
        "created が RFC 3339 ではありません: {created}"
    );
}

fn chrono_like_rfc3339(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 | 10 | 13 | 16 | 19 => true,
            _ => byte.is_ascii_digit(),
        })
}

#[test]
fn config_platform_matches_index() {
    let layout = sample();
    for (entry, _, config) in manifests(&layout) {
        assert_eq!(config["architecture"], entry["platform"]["architecture"]);
        assert_eq!(config["os"], entry["platform"]["os"]);
        assert_eq!(config["variant"], entry["platform"]["variant"]);
    }
}

#[test]
fn diff_id_is_uncompressed_tar() {
    let layout = sample();
    for (_, manifest, config) in manifests(&layout) {
        let layer = &manifest["layers"][0];
        let compressed = blob(&layout, layer["digest"].as_str().unwrap());
        let mut child = Command::new("zstd")
            .args(["-dc"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.take().unwrap().write_all(&compressed).unwrap();
        let uncompressed = child.wait_with_output().unwrap().stdout;
        assert_eq!(
            config["rootfs"]["diff_ids"][0].as_str().unwrap(),
            format!(
                "sha256:{}",
                containers_builder::util::sha256_hex(&uncompressed)
            )
        );
        assert!(is_sha256_digest(
            config["rootfs"]["diff_ids"][0].as_str().unwrap()
        ));
    }
}
