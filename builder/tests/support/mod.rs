use std::fs;
use std::path::{Path, PathBuf};

use containers_builder::oci::{self, Platform};
use serde_json::Value;

pub struct Layout {
    dir: Option<tempfile::TempDir>,
    pub output: PathBuf,
}

impl Layout {
    pub fn tmp(&self) -> &Path {
        self.dir.as_ref().expect("layout tempdir").path()
    }
}

impl Drop for Layout {
    fn drop(&mut self) {
        if std::env::var("KEEP_LAYOUT").is_ok()
            && let Some(dir) = self.dir.take()
        {
            let _ = dir.keep();
        }
    }
}

pub fn build(platforms: &[Platform]) -> Layout {
    build_with_bytes(platforms, b"hello")
}

pub fn build_with_bytes(platforms: &[Platform], bytes: &[u8]) -> Layout {
    build_with_bytes_level(platforms, bytes, oci::DEFAULT_ZSTD_LEVEL)
}

pub fn build_with_bytes_level(platforms: &[Platform], bytes: &[u8], zstd_level: u8) -> Layout {
    let tmp = tempfile::tempdir().unwrap();
    let layer_src = tmp.path().join("layer");
    fs::create_dir_all(&layer_src).unwrap();
    fs::write(layer_src.join("file.txt"), bytes).unwrap();
    let output = tmp.path().join("layout");
    let image = oci::Image {
        name: "glibc".to_string(),
        version: "2.44".to_string(),
    };
    let layers = platforms
        .iter()
        .map(|platform| (*platform, vec![layer_src.clone()]))
        .collect::<Vec<_>>();
    oci::build(&output, &image, &layers, zstd_level).unwrap();
    Layout {
        dir: Some(tmp),
        output,
    }
}

pub fn amd64(variant: &'static str) -> Platform {
    Platform {
        architecture: "amd64",
        variant,
    }
}

pub fn index(layout: &Layout) -> Value {
    serde_json::from_str(&fs::read_to_string(layout.output.join("index.json")).unwrap()).unwrap()
}

pub fn blob(layout: &Layout, digest: &str) -> Vec<u8> {
    let hex = digest.strip_prefix("sha256:").unwrap_or(digest);
    fs::read(layout.output.join("blobs/sha256").join(hex)).unwrap()
}

pub fn blob_json(layout: &Layout, digest: &str) -> Value {
    serde_json::from_slice(&blob(layout, digest)).unwrap()
}

pub fn manifests(layout: &Layout) -> Vec<(Value, Value, Value)> {
    let index = index(layout);
    index["manifests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let manifest = blob_json(layout, entry["digest"].as_str().unwrap());
            let config = blob_json(layout, manifest["config"]["digest"].as_str().unwrap());
            (entry.clone(), manifest, config)
        })
        .collect()
}

pub fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64
        && hex
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

pub fn assert_descriptor(value: &Value) {
    let media_type = value["mediaType"].as_str().expect("mediaType");
    assert!(!media_type.is_empty());
    let digest = value["digest"].as_str().expect("digest");
    assert!(is_sha256_digest(digest), "digest が不正です: {digest}");
    assert!(
        value["size"].as_u64().is_some(),
        "size が数値ではありません"
    );
}

pub fn blobs_dir(layout: &Layout) -> PathBuf {
    layout.output.join("blobs/sha256")
}

pub fn walk_blobs(dir: &Path) -> Vec<(String, Vec<u8>)> {
    let mut blobs = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        let bytes = fs::read(entry.path()).unwrap();
        blobs.push((name, bytes));
    }
    blobs
}
