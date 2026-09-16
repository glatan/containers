use std::fs;
use std::path::Path;
use std::process::Command;

use super::{Package, Role, Source};
use crate::error::{Error, Result};
use crate::process;
use crate::util::source_checksums;

pub fn fetch_package(package: &dyn Package) -> Result<Vec<String>> {
    if matches!(package.role(), Role::BuildEnv) {
        return Ok(vec![format!(
            "{} はソースを持たないためスキップします",
            package.name()
        )]);
    }
    let dir = package.source_dir();
    fs::create_dir_all(&dir).map_err(|source| Error::io(&dir, source))?;
    let filename = Path::new(package.source().url)
        .file_name()
        .ok_or_else(|| Error::message("ソース URL からファイル名を取得できません"))?;
    let tarball = dir.join(filename);
    let extracted = dir.join(package.source().subdir);
    let cached = tarball.exists() && verify(&tarball, package.source()).is_ok();
    let mut log = Vec::new();
    if cached {
        log.push(format!("{} は取得済みです", tarball.display()));
    } else if let Some(cache) = hashcache_hit(filename, package.source()) {
        fs::copy(&cache, &tarball).map_err(|source| Error::io(&tarball, source))?;
        log.push(format!("{} をキャッシュから複製しました", cache.display()));
    } else {
        log.push(format!("{} を取得します", package.source().url));
        process::status(
            Command::new("curl")
                .args(["-fL", "--retry", "3", "-o"])
                .arg(&tarball)
                .arg(package.source().url),
            "curl",
        )?;
        verify(&tarball, package.source())?;
    }
    if cached && extracted.is_dir() {
        log.push(format!("{} は展開済みです", extracted.display()));
        return Ok(log);
    }
    process::status(
        Command::new("tar")
            .args(["-xf"])
            .arg(&tarball)
            .args(["-C"])
            .arg(&dir),
        "tar",
    )?;
    log.push(format!("{} を展開しました", dir.display()));
    Ok(log)
}

fn hashcache_hit(filename: &std::ffi::OsStr, source: Source) -> Option<std::path::PathBuf> {
    let cache = Path::new("packages/sources/_hashcache").join(filename);
    if cache.is_file() && verify(&cache, source).is_ok() {
        Some(cache)
    } else {
        None
    }
}

pub(crate) fn verify(path: &Path, source: Source) -> Result<()> {
    let (sha3_512, blake3) = source_checksums(path)?;
    if sha3_512 == source.sha3_512 && blake3 == source.blake3 {
        Ok(())
    } else {
        Err(Error::message(format!(
            "checksum が一致しません: {} (sha3-512 期待値: {}, 実測: {sha3_512}; blake3 期待値: {}, 実測: {blake3})",
            path.display(),
            source.sha3_512,
            source.blake3
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::Dependency;
    use crate::util;

    struct Local {
        dir: std::path::PathBuf,
        sha3_512: &'static str,
        blake3: &'static str,
    }

    impl Package for Local {
        fn name(&self) -> &'static str {
            "fetch-local"
        }
        fn version(&self) -> &'static str {
            "1"
        }
        fn license(&self) -> &'static str {
            "MIT"
        }
        fn source(&self) -> Source {
            Source {
                url: "https://example.invalid/src.tar",
                sha3_512: self.sha3_512,
                blake3: self.blake3,
                subdir: "src",
            }
        }
        fn dependencies(&self, _target: crate::target::Target) -> Vec<Dependency> {
            vec![]
        }
        fn build_dependencies(&self, _target: crate::target::Target) -> Vec<Dependency> {
            vec![]
        }
        fn build_script(&self) -> &'static str {
            ""
        }
        fn flags(&self, _target: crate::target::Target) -> crate::flags::Flag {
            crate::flags::Flag::new(vec![], vec![])
        }
        fn source_dir(&self) -> std::path::PathBuf {
            self.dir.clone()
        }
    }

    fn make_tarball(dir: &Path) -> (&'static str, &'static str) {
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("file.txt"), "hello").unwrap();
        process::status(
            Command::new("tar")
                .args(["-cf", "src.tar", "src"])
                .current_dir(dir),
            "tar",
        )
        .unwrap();
        fs::remove_dir_all(&src).unwrap();
        let (sha3_512, blake3) = util::source_checksums(&dir.join("src.tar")).unwrap();
        (
            Box::leak(sha3_512.into_boxed_str()),
            Box::leak(blake3.into_boxed_str()),
        )
    }

    #[test]
    fn verify_rejects_mismatched_checksum() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("blob");
        fs::write(&path, b"hello").unwrap();
        let (sha3_512, blake3) = util::source_checksums(&path).unwrap();
        let error = verify(
            &path,
            Source {
                url: "https://example.invalid/blob",
                sha3_512: ZERO_512,
                blake3: ZERO_512,
                subdir: "blob",
            },
        )
        .unwrap_err();
        assert!(error.to_string().contains("checksum が一致しません"));
        verify(
            &path,
            Source {
                url: "https://example.invalid/blob",
                sha3_512: Box::leak(sha3_512.into_boxed_str()),
                blake3: Box::leak(blake3.into_boxed_str()),
                subdir: "blob",
            },
        )
        .unwrap();
    }

    const ZERO_512: &str = "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn fetch_package_skips_reextract_when_cached() {
        let tmp = tempfile::tempdir().unwrap();
        let (sha3_512, blake3) = make_tarball(tmp.path());
        let package = Local {
            dir: tmp.path().to_path_buf(),
            sha3_512,
            blake3,
        };
        fetch_package(&package).unwrap();
        let extracted = tmp.path().join("src/file.txt");
        fs::write(&extracted, "changed").unwrap();
        fetch_package(&package).unwrap();
        assert_eq!(fs::read_to_string(&extracted).unwrap(), "changed");
    }
}
