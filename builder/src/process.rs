use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::thread;

use crate::error::{Error, Result};

pub fn status(command: &mut Command, name: &'static str) -> Result<()> {
    let status = command.status().map_err(|source| Error::CommandSpawn {
        command: name,
        source,
    })?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailed { command: name })
    }
}

pub fn output(command: &mut Command, name: &'static str) -> Result<Output> {
    let output = command.output().map_err(|source| Error::CommandSpawn {
        command: name,
        source,
    })?;
    if output.status.success() {
        Ok(output)
    } else {
        Err(Error::CommandFailed { command: name })
    }
}

pub fn tee_status(
    command: &mut Command,
    name: &'static str,
    log: &mut File,
    mut combined: Option<&mut File>,
    log_path: &Path,
) -> Result<()> {
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());
    let mut child = command.spawn().map_err(|source| Error::CommandSpawn {
        command: name,
        source,
    })?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::message(format!("{name} の標準出力を取得できません")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::message(format!("{name} の標準エラー出力を取得できません")))?;
    let (tx, rx) = mpsc::sync_channel::<Vec<u8>>(32);
    let tx_err = tx.clone();
    let out_thread = thread::spawn(move || copy_pipe(stdout, tx));
    let err_thread = thread::spawn(move || copy_pipe(stderr, tx_err));
    let mut stdout = io::stdout();
    let mut io_error = None;
    for chunk in rx {
        if let Err(error) = write_chunk(&chunk, &mut stdout, log, combined.as_deref_mut()) {
            io_error = Some(error);
            break;
        }
    }
    let _ = out_thread.join();
    let _ = err_thread.join();
    let status = child.wait().map_err(|source| Error::CommandSpawn {
        command: name,
        source,
    })?;
    if let Some(error) = io_error {
        return Err(Error::io(log_path, error));
    }
    if status.success() {
        Ok(())
    } else {
        Err(Error::CommandFailedWithLog {
            command: name,
            log: log_path.to_path_buf(),
        })
    }
}

fn copy_pipe(mut reader: impl Read, tx: mpsc::SyncSender<Vec<u8>>) {
    let mut buf = [0u8; 8192];
    loop {
        match reader.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                if tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

fn write_chunk(
    chunk: &[u8],
    stdout: &mut io::Stdout,
    log: &mut File,
    combined: Option<&mut File>,
) -> io::Result<()> {
    stdout.write_all(chunk)?;
    stdout.flush()?;
    log.write_all(chunk)?;
    log.flush()?;
    if let Some(combined) = combined {
        combined.write_all(chunk)?;
        combined.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;

    #[test]
    fn tee_status_copies_stdout_and_stderr_to_log() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("out.log");
        let mut log = File::create(&path).unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "printf 'out\\n'; printf 'err\\n' >&2"]);
        tee_status(&mut command, "sh", &mut log, None, &path).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("out"), "{body:?}");
        assert!(body.contains("err"), "{body:?}");
    }

    #[test]
    fn tee_status_failure_includes_log_path() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("fail.log");
        let mut log = File::create(&path).unwrap();
        let mut command = Command::new("sh");
        command.args(["-c", "printf fail; exit 1"]);
        let error = tee_status(&mut command, "sh", &mut log, None, &path).unwrap_err();
        let text = error.to_string();
        assert!(text.contains("sh"), "{text}");
        assert!(text.contains(&path.display().to_string()), "{text}");
        assert_eq!(fs::read_to_string(&path).unwrap(), "fail");
    }

    #[test]
    fn tee_status_drains_large_stdout_and_stderr() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("big.log");
        let mut log = File::create(&path).unwrap();
        let mut combined_file = File::create(tmp.path().join("combined.log")).unwrap();
        let mut command = Command::new("sh");
        command.args([
            "-c",
            "dd if=/dev/zero bs=1024 count=256 status=none; dd if=/dev/zero bs=1024 count=256 status=none >&2",
        ]);
        tee_status(
            &mut command,
            "sh",
            &mut log,
            Some(&mut combined_file),
            &path,
        )
        .unwrap();
        assert_eq!(fs::metadata(&path).unwrap().len(), 512 * 1024);
        assert_eq!(
            fs::metadata(tmp.path().join("combined.log")).unwrap().len(),
            512 * 1024
        );
    }
}
