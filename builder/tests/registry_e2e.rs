//! skopeo 経由の e2e 検証。ゲート:
//! - skopeo バイナリが無ければ各テストは早期 return（スキップ）
//! - `CONTAINERS_E2E_REGISTRY`（例: `localhost:5000`）が無ければ
//!   docker:// 往復はスキップ
//!
//! `skopeo copy --all oci:… oci:…` はレジストリ不要で、
//! containers/image の oci: トランスポートが本プロジェクトの
//! ネスト index レイアウトを受理することを直接検証できる。

mod support;

use std::process::Command;

use serde_json::Value;
use support::{Layout, amd64, build};

fn has_skopeo() -> bool {
    Command::new("skopeo")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

fn skopeo_copy(src: &str, dest: &str) {
    let output = Command::new("skopeo")
        .args(["copy", "--all", src, dest])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "skopeo copy が失敗しました: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn nested_manifest_digests(layout_dir: &std::path::Path) -> Vec<String> {
    let top: Value =
        serde_json::from_slice(&std::fs::read(layout_dir.join("index.json")).unwrap()).unwrap();
    let entries = top["manifests"].as_array().unwrap();
    assert_eq!(entries.len(), 1, "index.json は単一 descriptor のはずです");
    let nested_digest = entries[0]["digest"].as_str().unwrap();
    let hex = nested_digest.strip_prefix("sha256:").unwrap();
    let nested: Value = serde_json::from_slice(
        &std::fs::read(layout_dir.join("blobs/sha256").join(hex)).unwrap(),
    )
    .unwrap();
    nested["manifests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["digest"].as_str().unwrap().to_string())
        .collect()
}

/// containers/image の oci: トランスポートがネスト index を受理し、
/// platform 別 manifest を丸ごとコピーできること（ErrMoreThanOneImage
/// のリグレッション検知）。
#[test]
fn skopeo_copy_all_preserves_multi_platform_index() {
    if !has_skopeo() {
        eprintln!("skopeo が無いためスキップします");
        return;
    }
    let layout: Layout = build(&[amd64("v1"), amd64("v4")]);
    let dest = tempfile::tempdir().unwrap();
    let dest_dir = dest.path().join("copied");
    skopeo_copy(
        &format!("oci:{}", layout.output.display()),
        &format!("oci:{}", dest_dir.display()),
    );
    let source = nested_manifest_digests(&layout.output);
    let copied = nested_manifest_digests(&dest_dir);
    assert_eq!(source.len(), 2);
    assert_eq!(source, copied, "manifest digest が保存されていません");
}

/// docker:// 往復。`CONTAINERS_E2E_REGISTRY` にローカルレジストリ
/// （`docker run -p 5000:5000 registry:2` 等）を指定したときだけ実行。
#[test]
fn skopeo_push_pull_roundtrip_via_registry() {
    if !has_skopeo() {
        eprintln!("skopeo が無いためスキップします");
        return;
    }
    let Ok(registry) = std::env::var("CONTAINERS_E2E_REGISTRY") else {
        eprintln!("CONTAINERS_E2E_REGISTRY 未設定のためスキップします");
        return;
    };
    let layout = build(&[amd64("v1"), amd64("v4")]);
    let reference = format!("docker://{registry}/e2e/containers-builder:test");
    skopeo_copy(
        &format!("oci:{}", layout.output.display()),
        &reference,
    );
    // push 後にタグが指す digest（ネスト index blob）を raw inspect で取得。
    let raw = Command::new("skopeo")
        .args(["inspect", "--raw", &reference])
        .output()
        .unwrap();
    assert!(raw.status.success(), "inspect --raw が失敗しました");
    let index: Value = serde_json::from_slice(&raw.stdout).unwrap();
    assert_eq!(
        index["mediaType"],
        "application/vnd.oci.image.index.v1+json",
        "index として push されているはずです"
    );
    assert_eq!(index["manifests"].as_array().unwrap().len(), 2);
    // platform 別 manifest を digest pin で個別に pull できること。
    let pulled = tempfile::tempdir().unwrap();
    let dest_dir = pulled.path().join("manifest");
    let manifest_digest = index["manifests"][0]["digest"].as_str().unwrap();
    // `localhost:5000/...` のようなポート付きでもタグ部分だけ落とす。
    let repo = reference
        .trim_start_matches("docker://")
        .rsplit_once(':')
        .map(|(repo, _)| repo)
        .unwrap();
    skopeo_copy(
        &format!("docker://{repo}@{manifest_digest}"),
        &format!("oci:{}", dest_dir.display()),
    );
    assert!(dest_dir.join("index.json").exists());
}
