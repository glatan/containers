use std::fs;
use std::path::Path;
use std::process::Command;

use super::{Package, Source};
use crate::error::{Error, Result};
use crate::process;
use crate::util::source_checksums;

pub fn fetch_package(package: &dyn Package) -> Result<Vec<String>> {
    if package.compose_root() {
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
    } else {
        process::status(
            Command::new("tar")
                .args(["-xf"])
                .arg(&tarball)
                .args(["-C"])
                .arg(&dir),
            "tar",
        )?;
        log.push(format!("{} を展開しました", dir.display()));
    }
    // 補助ソース（cargo vendor 等）は配布元 URL を持たないため、
    // packages/sources/{name}/{file} に手動配置されたものを検証して展開する。
    if let Some(aux) = package.aux_source() {
        let path = dir.join(aux.file);
        if !path.is_file() {
            return Err(Error::message(format!(
                "補助ソースが見つかりません: {} (vendor tarball を配置してください)",
                path.display()
            )));
        }
        verify_checksums(&path, aux.sha3_512, aux.blake3)?;
        let aux_dir = dir.join(aux.subdir);
        // aux.subdir はバージョン非依存（"vendor" 等）なので、同名で
        // 再生成された tarball に対して古い展開済みツリーが残り得る。
        // 検証済みハッシュのマーカーで展開の鮮度を判定する。
        let marker = dir.join(format!(".{}.extracted", aux.subdir));
        let fresh = aux_dir.is_dir()
            && fs::read_to_string(&marker)
                .map(|content| content.trim() == aux.sha3_512)
                .unwrap_or(false);
        if !fresh {
            if aux_dir.exists() {
                fs::remove_dir_all(&aux_dir).map_err(|source| Error::io(&aux_dir, source))?;
            }
            process::status(
                Command::new("tar")
                    .args(["-xf"])
                    .arg(&path)
                    .args(["-C"])
                    .arg(&dir),
                "tar",
            )?;
            fs::write(&marker, format!("{}\n", aux.sha3_512))
                .map_err(|source| Error::io(&marker, source))?;
            log.push(format!("{} を展開しました", aux_dir.display()));
        }
    }
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
    verify_checksums(path, source.sha3_512, source.blake3)
}

fn verify_checksums(path: &Path, sha3_512: &str, blake3: &str) -> Result<()> {
    let (actual_sha3, actual_blake3) = source_checksums(path)?;
    if actual_sha3 == sha3_512 && actual_blake3 == blake3 {
        Ok(())
    } else {
        Err(Error::message(format!(
            "checksum が一致しません: {} (sha3-512 期待値: {}, 実測: {actual_sha3}; blake3 期待値: {}, 実測: {actual_blake3})",
            path.display(),
            sha3_512,
            blake3
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
        fn recipe(
            &self,
            _ctx: &crate::package::recipe::BuildContext,
        ) -> crate::package::recipe::Recipe {
            crate::package::recipe::Recipe::Noop
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
