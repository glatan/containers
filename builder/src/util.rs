use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use sha2::{Digest, Sha256};
use sha3::Sha3_512;

use crate::error::{Error, Result};

pub fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

/// JoinHandle の panic payload から人間可読なメッセージを取り出す。
pub fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&'static str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "不明".to_string()
    }
}

struct SourceHashers {
    sha3: Sha3_512,
    blake3: blake3::Hasher,
}

impl Write for SourceHashers {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.sha3.update(buf);
        self.blake3.update(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub fn source_checksums(path: &Path) -> Result<(String, String)> {
    let mut file = File::open(path).map_err(|source| Error::io(path, source))?;
    let mut hashers = SourceHashers {
        sha3: Sha3_512::new(),
        blake3: blake3::Hasher::new(),
    };
    io::copy(&mut file, &mut hashers).map_err(|source| Error::io(path, source))?;
    Ok((
        hex_lower(&hashers.sha3.finalize()),
        blake3_512_hex(&hashers.blake3),
    ))
}

fn blake3_512_hex(hasher: &blake3::Hasher) -> String {
    let mut out = [0u8; 64];
    hasher.finalize_xof().fill(&mut out);
    hex_lower(&out)
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0xf) as usize] as char);
    }
    out
}

pub fn timestamp() -> String {
    utc_now().strftime("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// `SOURCE_DATE_EPOCH`（reproducible-builds の規約）を RFC 3339 に変換する。
/// 未設定・不正値のときは epoch 0 を返す。OCI イメージの `created` を
/// 決定的にして pack の digest を再現可能にするための値。
pub fn source_date_epoch() -> String {
    let epoch = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(0);
    jiff::Timestamp::from_second(epoch)
        .map(|ts| {
            ts.to_zoned(jiff::tz::TimeZone::UTC)
                .strftime("%Y-%m-%dT%H:%M:%SZ")
                .to_string()
        })
        .unwrap_or_else(|_| "1970-01-01T00:00:00Z".to_string())
}

/// パッケージング時点の HEAD コミット SHA。
/// `org.opencontainers.image.revision` 用。git が使えない環境では
/// None を返し、アノテーション自体を省略する。
pub fn revision() -> Option<String> {
    static REVISION: std::sync::OnceLock<Option<String>> = std::sync::OnceLock::new();
    REVISION
        .get_or_init(|| {
            // 呼出元の CWD に依存しないようリポジトリルートを明示する。
            std::process::Command::new("git")
                .args([
                    "-C",
                    concat!(env!("CARGO_MANIFEST_DIR"), "/.."),
                    "rev-parse",
                    "HEAD",
                ])
                .output()
                .ok()
                .filter(|output| output.status.success())
                .and_then(|output| String::from_utf8(output.stdout).ok())
                .map(|stdout| stdout.trim().to_string())
                .filter(|stdout| !stdout.is_empty())
        })
        .clone()
}

pub fn utc_date() -> String {
    utc_now().strftime("%Y%m%d").to_string()
}

fn utc_now() -> jiff::Zoned {
    jiff::Timestamp::now().to_zoned(jiff::tz::TimeZone::UTC)
}

pub fn write(path: &Path, contents: impl AsRef<[u8]>) -> Result<()> {
    File::create(path)
        .and_then(|mut file| file.write_all(contents.as_ref()))
        .map_err(|source| Error::io(path, source))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn revision_is_commit_sha_or_absent() {
        if let Some(revision) = revision() {
            assert_eq!(revision.len(), 40);
            assert!(
                revision
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            );
        }
    }

    #[test]
    fn source_checksums_are_512_bit_hex() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("blob");
        fs::write(&path, b"hello").unwrap();
        let (sha3, blake3) = source_checksums(&path).unwrap();
        assert_eq!(sha3.len(), 128);
        assert_eq!(blake3.len(), 128);
        assert!(
            sha3.bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        );
        assert!(
            blake3
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        );
        assert_eq!(
            sha3,
            "75d527c368f2efe848ecf6b073a36767800805e9eef2b1857d5f984f036eb6df891d75f72d9b154518c1cd58835286d1da9a38deba3de98b5a53e5ed78a84976"
        );
    }
}
