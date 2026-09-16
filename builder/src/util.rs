use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

use sha2::{Digest, Sha256};
use sha3::Sha3_512;

use crate::error::{Error, Result};

pub fn sha256_hex(data: &[u8]) -> String {
    format!("{:x}", Sha256::digest(data))
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = File::open(path).map_err(|source| Error::io(path, source))?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher).map_err(|source| Error::io(path, source))?;
    Ok(format!("{:x}", hasher.finalize()))
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
