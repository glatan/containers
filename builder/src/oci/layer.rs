use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;

use sha2::{Digest as _, Sha256};
use tempfile::NamedTempFile;

use crate::error::{Error, Result};
use crate::oci::model::{Digest, ZstdLevel};

pub struct Layer {
    pub diff_id: Digest,
    pub blob: Digest,
    pub size: u64,
}

/// diff_id（非圧縮 tar の digest）→ 圧縮済み blob のキャッシュ。
/// 内容が同一のレイヤーの再圧縮を省くため build 呼び出し内で共有する。
pub(crate) type BlobCache = Mutex<HashMap<Digest, (Digest, u64)>>;

fn tar(dir: &Path) -> Command {
    let mut command = Command::new("tar");
    command
        .args([
            "--sort=name",
            "--mtime=@0",
            "--owner=0",
            "--group=0",
            "--numeric-owner",
            "-C",
        ])
        .arg(dir)
        .args(["-cf", "-", "."])
        .stdout(Stdio::piped());
    command
}

fn zstd(zstd_level: ZstdLevel) -> Command {
    let mut command = Command::new("zstd");
    if zstd_level.needs_ultra() {
        command.arg("--ultra");
    }
    command
        .args([
            "--single-thread",
            "-q",
            "-c",
            &format!("-{}", zstd_level.get()),
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped());
    command
}

/// `input` を読み切り、あれば `sink` へ流しつつ sha256 を返す。
fn pump(input: &mut impl Read, mut sink: Option<&mut impl Write>) -> Result<Digest> {
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let read = input
            .read(&mut buf)
            .map_err(|source| Error::message(format!("ストリームの読み込みに失敗: {source}")))?;
        if read == 0 {
            break;
        }
        hasher.update(&buf[..read]);
        if let Some(sink) = sink.as_deref_mut() {
            sink.write_all(&buf[..read]).map_err(|source| {
                Error::message(format!("ストリームの書き込みに失敗: {source}"))
            })?;
        }
    }
    Ok(Digest::from_hex(format!("{:x}", hasher.finalize())))
}

/// `dir` の tar ストリームの sha256（diff_id）。
fn tar_digest(dir: &Path) -> Result<Digest> {
    let mut child = tar(dir).spawn().map_err(|source| Error::CommandSpawn {
        command: "tar",
        source,
    })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::message("tar の標準出力を取得できません"))?;
    // read 失敗時に stdout を保持したまま wait すると tar がパイプ一杯で
    // ブロックするため、pump のスコープで drop してから回収する
    // （compress() と同じ drop 順）。pump のエラーは exit status より
    // 詳しいので優先して返す。
    let digest = {
        let mut stdout = stdout;
        pump(&mut stdout, None::<&mut Vec<u8>>)
    };
    match digest {
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(error)
        }
        Ok(digest) => {
            let status = child.wait().map_err(|source| Error::CommandSpawn {
                command: "tar",
                source,
            })?;
            if !status.success() {
                return Err(Error::CommandFailed { command: "tar" });
            }
            Ok(digest)
        }
    }
}

/// `tar | zstd` をパイプで繋いで圧縮し、blob digest・サイズ・diff_id
/// （feed 側が tar ストリームから計算した sha256）とともに一時ファイルへ
/// 書き出す（中間 tar を置かず、読み直しも行わない）。
fn compress(
    dir: &Path,
    zstd_level: ZstdLevel,
    staging: &Path,
) -> Result<(NamedTempFile, Digest, u64, Digest)> {
    let mut tar_child = tar(dir).spawn().map_err(|source| Error::CommandSpawn {
        command: "tar",
        source,
    })?;
    // 早期 return する経路でも起動済みの子プロセスは必ず回収する
    // （ゾンビを残さない）。
    let mut zstd_child = match zstd(zstd_level).spawn() {
        Ok(child) => child,
        Err(source) => {
            let _ = tar_child.kill();
            let _ = tar_child.wait();
            return Err(Error::CommandSpawn {
                command: "zstd",
                source,
            });
        }
    };
    let mut tmp = match NamedTempFile::new_in(staging) {
        Ok(tmp) => tmp,
        Err(source) => {
            let _ = tar_child.kill();
            let _ = tar_child.wait();
            let _ = zstd_child.kill();
            let _ = zstd_child.wait();
            return Err(Error::io(staging, source));
        }
    };

    let outcome = std::thread::scope(|scope| -> Result<(Digest, Digest)> {
        let feed = scope.spawn(|| -> Result<Digest> {
            let mut input = tar_child
                .stdout
                .take()
                .ok_or_else(|| Error::message("tar の標準出力を取得できません"))?;
            let mut output = zstd_child
                .stdin
                .take()
                .ok_or_else(|| Error::message("zstd の標準入力を取得できません"))?;
            // パイプは tar→zstd→file の直列で、相互に詰まる経路はない。
            pump(&mut input, Some(&mut output))
        });
        // feed の join より先に input（zstd stdout の読み側）を drop する。
        // pump が早期失敗してもパイプが閉じて zstd は終了し、feed の
        // write は EPIPE で返るため join が詰まらない。
        let blob = {
            let mut input = zstd_child
                .stdout
                .take()
                .ok_or_else(|| Error::message("zstd の標準出力を取得できません"))?;
            pump(&mut input, Some(&mut tmp.as_file_mut()))
        };
        let diff_id = feed.join().map_err(|payload| {
            Error::message(format!(
                "tar→zstd のパイプスレッドがパニックしました: {}",
                crate::util::panic_message(&*payload)
            ))
        })?;
        Ok((blob?, diff_id?))
    });
    if outcome.is_err() {
        let _ = tar_child.kill();
        let _ = zstd_child.kill();
    }
    let tar_status = tar_child.wait().map_err(|source| Error::CommandSpawn {
        command: "tar",
        source,
    })?;
    let zstd_status = zstd_child.wait().map_err(|source| Error::CommandSpawn {
        command: "zstd",
        source,
    })?;
    let (blob, diff_id) = outcome?;
    if !tar_status.success() {
        return Err(Error::CommandFailed { command: "tar" });
    }
    if !zstd_status.success() {
        return Err(Error::CommandFailed { command: "zstd" });
    }
    let size = tmp
        .as_file()
        .metadata()
        .map_err(|source| Error::io(tmp.path(), source))?
        .len();
    Ok((tmp, blob, size, diff_id))
}

pub fn build(blobs: &Path, dir: &Path, zstd_level: ZstdLevel, cache: &BlobCache) -> Result<Layer> {
    if !dir.is_dir() {
        return Err(Error::message(format!(
            "レイヤーのディレクトリが見つかりません: {}",
            dir.display()
        )));
    }
    // blobs の2階層上（= staging dir、同一 FS）に temp を置き、rename で
    // 確定する。FS を跨ぐコピーを避けるため /tmp 既定にはしない。
    let parent = blobs
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| Path::new("/tmp"));

    // キャッシュ判定用に diff_id だけ先に計算する。ヒットした場合は
    // 圧縮を丸ごと省略できる。ミス時は tar を2回流すことになるが、
    // ページキャッシュが効くため実害は小さい（圧縮の方が支配的）。
    // compress の feed スレッドでも同じ digest を取り直し、食い違い
    // （圧縮中に内容が変化）を検査する。
    let expected = tar_digest(dir)?;
    if let Some((blob, size)) = cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(&expected)
    {
        return Ok(Layer {
            diff_id: expected,
            blob: blob.clone(),
            size: *size,
        });
    }

    let (tmp, blob, size, diff_id) = compress(dir, zstd_level, parent)?;
    if diff_id != expected {
        return Err(Error::message(format!(
            "レイヤーの内容が圧縮中に変化しました: {}",
            dir.display()
        )));
    }
    let dest = blobs.join(blob.encoded());
    tmp.persist(&dest).map_err(|source| {
        Error::message(format!(
            "{} を {} へ配置できません: {}",
            source.file.path().display(),
            dest.display(),
            source.error
        ))
    })?;
    cache
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(diff_id.clone(), (blob.clone(), size));
    Ok(Layer {
        diff_id,
        blob,
        size,
    })
}
