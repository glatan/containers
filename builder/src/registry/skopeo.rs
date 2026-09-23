//! `skopeo` プロセスのラッパ。TLS・認証・プロキシは skopeo に委譲する。
//!
//! - push: `skopeo copy --all oci:{dir} docker://{ref}`
//!   `--all`（`--multi-arch=all`）が必須。既定の `system` だと
//!   ホスト platform の manifest 1件だけが送られてしまう。
//! - pull: `skopeo copy docker://{name}@{manifestDigest} oci:{dir}`
//!   manifest digest 指定なので platform 選択は発生しない。
//! - digest 照合: `skopeo inspect --raw | skopeo manifest-digest`
//!   通常の `inspect` は platform 選択で失敗し得るため使わない。

use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{Error, Result};
use crate::oci::Digest;

/// skopeo が使えるか。
pub fn available() -> bool {
    Command::new("skopeo")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub fn require() -> Result<()> {
    if available() {
        Ok(())
    } else {
        Err(Error::message(
            "skopeo が見つかりません。レジストリ操作には skopeo が必要です",
        ))
    }
}

/// OCI layout を `docker://{reference}`（`:tag` or `@digest`）へ push し、
/// リモートに格納された manifest/index の digest を返す。
pub fn push_layout(layout: &Path, reference: &str) -> Result<Digest> {
    let digestfile = tempfile::NamedTempFile::new()
        .map_err(|source| Error::message(format!("digestfile の作成に失敗: {source}")))?;
    run(&[
        "copy".to_string(),
        "--all".to_string(),
        "--digestfile".to_string(),
        digestfile.path().to_string_lossy().to_string(),
        format!("oci:{}", layout.display()),
        reference.to_string(),
    ])?;
    let text = std::fs::read_to_string(digestfile.path())
        .map_err(|source| Error::io(digestfile.path(), source))?;
    Digest::parse(text.trim())
}

/// `docker://{reference}`（または `oci:` 参照）のイメージ/artifact を
/// OCI layout として `dest` に落とす。
pub fn pull_layout(reference: &str, dest: &Path) -> Result<()> {
    run(&[
        "copy".to_string(),
        "--all".to_string(),
        reference.to_string(),
        format!("oci:{}", dest.display()),
    ])
}

/// `docker://{reference}` が指す manifest/index の digest を返す。
/// レジストリ未存在（404 等）や認証エラーは `Err` を返す。
/// `inspect --raw` の出力バイト列の sha256 が manifest digest に等しい
/// ため、ローカルで計算する（`skopeo manifest-digest` は stdin を読まない）。
pub fn remote_digest(reference: &str) -> Result<Digest> {
    let raw = capture(&["inspect".to_string(), "--raw".to_string(), reference.to_string()])?;
    Ok(Digest::from_hex(crate::util::sha256_hex(&raw)))
}

/// 既存タグとの digest 照合を経て push する（イミュータブルタグ運用）。
/// - 既存タグが同一 digest → 冪等にスキップ
/// - 別 digest を指す → 上書きせずエラー
/// - `inspect --raw` が「参照なし」（404 系）で失敗 → push 実行
/// - それ以外の失敗（認証・ネットワーク等） → push せずエラーを返す
pub fn push_immutable(layout: &Path, reference: &str, expected: &Digest) -> Result<()> {
    match remote_digest(reference) {
        Ok(remote) if remote == *expected => {
            println!("push 済み（同一 digest）: {reference}");
            return Ok(());
        }
        Ok(remote) => {
            return Err(Error::message(format!(
                "タグ {reference} は別の digest {} を指しています（イミュータブルタグのため上書きしません）",
                remote.prefixed()
            )));
        }
        Err(error) if is_not_found(&error) => {}
        Err(error) => return Err(error),
    }
    let digest = push_layout(layout, reference)?;
    if digest != *expected {
        return Err(Error::message(format!(
            "push 後の digest が一致しません: {} vs {}",
            digest.prefixed(),
            expected.prefixed()
        )));
    }
    println!("push: {reference} @ {}", digest.prefixed());
    Ok(())
}

/// `inspect --raw` の失敗が「参照なし」を示すか。
/// 404 / manifest unknown / name unknown 等の既知パターンのみを真とする。
fn is_not_found(error: &Error) -> bool {
    let text = error.to_string();
    [
        "manifest unknown",
        "name unknown",
        "404",
        "NotFound",
        "not found",
        "no such manifest",
    ]
    .iter()
    .any(|needle| text.contains(needle))
}

/// `docker://{repository}` のタグ一覧（`list-tags`）。`{"Tags":[...]}` をパースする。
pub fn list_tags(repository: &str) -> Result<Vec<String>> {
    let bytes = capture(&[
        "list-tags".to_string(),
        format!("docker://{repository}"),
    ])?;
    #[derive(serde::Deserialize)]
    struct Tags {
        #[serde(rename = "Tags", default)]
        tags: Vec<String>,
    }
    let parsed: Tags = serde_json::from_slice(&bytes)
        .map_err(|source| Error::message(format!("list-tags の出力が不正です: {source}")))?;
    Ok(parsed.tags)
}

fn run(args: &[String]) -> Result<()> {
    let status = Command::new("skopeo")
        .args(args)
        .status()
        .map_err(|source| Error::message(format!("skopeo の起動に失敗: {source}")))?;
    if !status.success() {
        return Err(Error::message(format!(
            "skopeo {} が失敗しました",
            args.join(" ")
        )));
    }
    Ok(())
}

fn capture(args: &[String]) -> Result<Vec<u8>> {
    let output = Command::new("skopeo")
        .args(args)
        .output()
        .map_err(|source| Error::message(format!("skopeo の起動に失敗: {source}")))?;
    if !output.status.success() {
        return Err(Error::message(format!(
            "skopeo {} が失敗しました: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(output.stdout)
}
