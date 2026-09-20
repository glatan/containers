use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

use tempfile::NamedTempFile;

use crate::error::{Error, Result};
use crate::oci::model::{Digest, ZstdLevel};
use crate::process;
use crate::util::sha256_file;

pub struct Layer {
    pub diff_id: Digest,
    pub blob: Digest,
    pub size: u64,
}

pub fn build(blobs: &Path, dir: &Path, zstd_level: ZstdLevel) -> Result<Layer> {
    if !dir.is_dir() {
        return Err(Error::message(format!(
            "レイヤーのディレクトリが見つかりません: {}",
            dir.display()
        )));
    }
    let parent = blobs
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("/tmp"));
    let tar = NamedTempFile::new_in(parent).map_err(|source| Error::io(parent, source))?;
    let zst = NamedTempFile::new_in(parent).map_err(|source| Error::io(parent, source))?;
    process::status(
        Command::new("tar")
            .args([
                "--sort=name",
                "--mtime=@0",
                "--owner=0",
                "--group=0",
                "--numeric-owner",
                "-C",
            ])
            .arg(dir)
            .args(["-cf"])
            .arg(tar.path())
            .arg("."),
        "tar",
    )?;
    let diff_id = Digest::from_hex(sha256_file(tar.path())?);
    let mut command = Command::new("zstd");
    if zstd_level.needs_ultra() {
        command.arg("--ultra");
    }
    let file = fs::File::create(zst.path()).map_err(|source| Error::io(zst.path(), source))?;
    process::status(
        command
            .args([
                "--single-thread",
                "-q",
                "-c",
                &format!("-{}", zstd_level.get()),
            ])
            .arg(tar.path())
            .stdout(Stdio::from(file)),
        "zstd",
    )?;
    let blob = Digest::from_hex(sha256_file(zst.path())?);
    let size = fs::metadata(zst.path())
        .map_err(|source| Error::io(zst.path(), source))?
        .len();
    let dest = blobs.join(blob.encoded());
    fs::copy(zst.path(), &dest).map_err(|source| Error::io(&dest, source))?;
    Ok(Layer {
        diff_id,
        blob,
        size,
    })
}
