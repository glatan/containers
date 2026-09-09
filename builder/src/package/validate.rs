use std::collections::BTreeSet;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::process::Command;

use crate::error::{Error, Result};
use crate::process;
use crate::target::Target;

use super::Package;

pub fn check(package: &dyn Package) -> Result<()> {
    let mut validated = 0;
    for target in Target::all() {
        let dir = package.artifact_dir(target);
        if !dir.is_dir() {
            continue;
        }
        validated += 1;
        check_needed(package, &collect_basenames(&dir)?, &needed_in(&dir)?)?;
    }
    if validated == 0 {
        return Err(Error::message("パッケージがありません"));
    }
    Ok(())
}

pub(crate) fn check_needed(
    package: &dyn Package,
    files: &BTreeSet<String>,
    needed: &BTreeSet<String>,
) -> Result<()> {
    for soname in needed {
        if files.contains(soname.as_str()) {
            continue;
        }
        let Some(dependency) = soname_to_package(soname) else {
            return Err(Error::message(format!(
                "未知の動的依存: {soname} ({})",
                package.name()
            )));
        };
        if dependency == package.name() {
            return Err(Error::message(format!(
                "{} は {} を提供するべきですが、パッケージに見つかりません",
                package.name(),
                soname
            )));
        }
        if !package.depends_on(dependency) {
            return Err(Error::message(format!(
                "{} は {} に依存していますが、依存として宣言されていません",
                package.name(),
                soname
            )));
        }
    }
    Ok(())
}

fn collect_basenames(dir: &Path) -> Result<BTreeSet<String>> {
    let mut files = BTreeSet::new();
    walk(dir, &mut |path, is_symlink, is_dir| {
        if is_dir && !is_symlink {
            return Ok(Walk::Recurse);
        }
        if let Some(name) = path.file_name().and_then(|name| name.to_str()) {
            files.insert(name.to_string());
        }
        Ok(Walk::Skip)
    })?;
    Ok(files)
}

pub(crate) fn needed_in(dir: &Path) -> Result<BTreeSet<String>> {
    let mut needed = BTreeSet::new();
    walk(dir, &mut |path, is_symlink, is_dir| {
        if is_symlink {
            return Ok(Walk::Skip);
        }
        if is_dir {
            return Ok(Walk::Recurse);
        }
        if is_elf(path) {
            needed.extend(needed_of(path)?);
        }
        Ok(Walk::Skip)
    })?;
    Ok(needed)
}

enum Walk {
    Recurse,
    Skip,
}

fn walk(dir: &Path, visit: &mut dyn FnMut(&Path, bool, bool) -> Result<Walk>) -> Result<()> {
    for entry in fs::read_dir(dir).map_err(|source| Error::io(dir, source))? {
        let path = entry.map_err(|source| Error::io(dir, source))?.path();
        let metadata = fs::symlink_metadata(&path).map_err(|source| Error::io(&path, source))?;
        let is_symlink = metadata.file_type().is_symlink();
        let is_dir = metadata.is_dir();
        match visit(&path, is_symlink, is_dir)? {
            Walk::Recurse => walk(&path, visit)?,
            Walk::Skip => {}
        }
    }
    Ok(())
}

fn is_elf(path: &Path) -> bool {
    let Ok(mut file) = fs::File::open(path) else {
        return false;
    };
    let mut magic = [0u8; 4];
    file.read_exact(&mut magic).is_ok() && magic == [0x7f, b'E', b'L', b'F']
}

pub(crate) fn needed_of(file: &Path) -> Result<Vec<String>> {
    let output = process::output(Command::new("readelf").args(["-d"]).arg(file), "readelf")?;
    Ok(parse_needed(&String::from_utf8_lossy(&output.stdout)))
}

pub(crate) fn parse_needed(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| {
            let rest = line.split_once("(NEEDED)")?.1;
            let rest = rest.split_once("Shared library: [")?.1;
            Some(rest.split_once(']')?.0.to_string())
        })
        .collect()
}

pub(crate) fn soname_to_package(soname: &str) -> Option<&'static str> {
    if is_glibc_soname(soname) {
        return Some("glibc");
    }
    if soname.starts_with("libssl.so") || soname.starts_with("libcrypto.so") {
        return Some("openssl");
    }
    if soname.starts_with("libz.so") {
        return Some("zlib");
    }
    None
}

fn is_glibc_soname(soname: &str) -> bool {
    const PREFIXES: &[&str] = &[
        "ld-linux",
        "ld64.so",
        "ld.so.",
        "libnss_",
        "libc.so",
        "libm.so",
        "libmvec.so",
        "libpthread.so",
        "libdl.so",
        "librt.so",
        "libutil.so",
        "libresolv.so",
        "libnsl.so",
        "libanl.so",
        "libcrypt.so",
        "libthread_db.so",
        "libBrokenLocale.so",
        "libcidn.so",
        "libmemusage.so",
        "libpcprofile.so",
        "libSegFault.so",
    ];
    PREFIXES.iter().any(|prefix| soname.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{Dependency, Source};

    struct Fake {
        name: &'static str,
        dependencies: &'static [Dependency],
        build_dependencies: &'static [Dependency],
    }

    impl Package for Fake {
        fn name(&self) -> &'static str {
            self.name
        }
        fn version(&self) -> &'static str {
            "1.0"
        }
        fn license(&self) -> &'static str {
            "MIT"
        }
        fn build_only(&self) -> bool {
            false
        }
        fn network(&self) -> bool {
            false
        }
        fn source(&self) -> Source {
            Source {
                url: "",
                sha3_512: "",
                blake3: "",
                subdir: "",
            }
        }
        fn dependencies(&self) -> &'static [Dependency] {
            self.dependencies
        }
        fn build_dependencies(&self) -> &'static [Dependency] {
            self.build_dependencies
        }
        fn build_script(&self) -> &'static str {
            ""
        }
        fn flags(&self) -> &'static [crate::flags::Var] {
            &[]
        }
    }

    fn needed(sonames: &[&str]) -> BTreeSet<String> {
        sonames.iter().map(|soname| (*soname).to_string()).collect()
    }

    #[test]
    fn collect_basenames_includes_symbolic_links() {
        let dir = tempfile::tempdir().unwrap();
        let lib = dir.path().join("usr/lib64");
        fs::create_dir_all(&lib).unwrap();
        fs::write(lib.join("libc-2.44.so"), b"real").unwrap();
        std::os::unix::fs::symlink("libc-2.44.so", lib.join("libc.so.6")).unwrap();
        let files = collect_basenames(dir.path()).unwrap();
        assert!(files.contains("libc.so.6"));
        assert!(files.contains("libc-2.44.so"));
    }

    #[test]
    fn glibc_self_sonames_are_satisfied_by_symlink_names() {
        let glibc = Fake {
            name: "glibc",
            dependencies: &[],
            build_dependencies: &[Dependency {
                name: "linux-headers",
                version: "1.0",
            }],
        };
        check_needed(
            &glibc,
            &BTreeSet::from([
                "libc.so.6".to_string(),
                "ld-linux-x86-64.so.2".to_string(),
                "libm.so.6".to_string(),
            ]),
            &needed(&["libc.so.6", "ld-linux-x86-64.so.2", "libm.so.6"]),
        )
        .unwrap();
    }

    #[test]
    fn glibc_missing_own_soname_is_an_error() {
        let glibc = Fake {
            name: "glibc",
            dependencies: &[],
            build_dependencies: &[],
        };
        let error = check_needed(&glibc, &BTreeSet::new(), &needed(&["libc.so.6"])).unwrap_err();
        assert!(error.to_string().contains("提供するべき"));
    }

    #[test]
    fn openssl_glibc_sonames_require_declared_glibc() {
        let openssl = Fake {
            name: "openssl",
            dependencies: &[],
            build_dependencies: &[Dependency {
                name: "linux-headers",
                version: "1.0",
            }],
        };
        let error = check_needed(
            &openssl,
            &BTreeSet::new(),
            &needed(&["libc.so.6", "libnss_files.so.2"]),
        )
        .unwrap_err();
        assert!(error.to_string().contains("依存として宣言されていません"));

        let openssl = Fake {
            name: "openssl",
            dependencies: &[Dependency {
                name: "glibc",
                version: "1.0",
            }],
            build_dependencies: &[Dependency {
                name: "linux-headers",
                version: "1.0",
            }],
        };
        check_needed(
            &openssl,
            &BTreeSet::new(),
            &needed(&["libc.so.6", "libm.so.6"]),
        )
        .unwrap();
    }

    #[test]
    fn unknown_soname_is_rejected() {
        let openssl = Fake {
            name: "openssl",
            dependencies: &[Dependency {
                name: "glibc",
                version: "1.0",
            }],
            build_dependencies: &[],
        };
        let error =
            check_needed(&openssl, &BTreeSet::new(), &needed(&["libfoo.so.1"])).unwrap_err();
        assert!(error.to_string().contains("未知の動的依存"));
    }

    #[test]
    fn maps_common_sonames() {
        assert_eq!(soname_to_package("libm.so.6"), Some("glibc"));
        assert_eq!(soname_to_package("libssl.so.3"), Some("openssl"));
        assert_eq!(soname_to_package("libz.so.1"), Some("zlib"));
        assert_eq!(soname_to_package("libfoo.so.1"), None);
    }

    #[test]
    fn parse_needed_extracts_sonames() {
        let output = " 0x0000000000000001 (NEEDED) Shared library: [libc.so.6]\n 0x0000000000000001 (NEEDED) Shared library: [libm.so.6]\n";
        assert_eq!(parse_needed(output), ["libc.so.6", "libm.so.6"]);
        assert!(parse_needed("").is_empty());
        assert!(
            parse_needed(" 0x000000000000000e (SONAME) Library soname: [libc.so.6]\n").is_empty()
        );
        assert!(parse_needed(" (NEEDED) Shared library: libc.so.6\n").is_empty());
    }

    #[test]
    fn needed_in_skips_symlinks() {
        let elf = ["/usr/bin/true", "/bin/true", "/usr/bin/cat"]
            .iter()
            .map(Path::new)
            .find(|path| path.is_file())
            .expect("ELF のサンプルが必要です");
        let dir = tempfile::tempdir().unwrap();
        std::os::unix::fs::symlink(elf, dir.path().join("libfoo.so.1")).unwrap();
        let files = collect_basenames(dir.path()).unwrap();
        assert!(files.contains("libfoo.so.1"));
        assert!(
            needed_in(dir.path()).unwrap().is_empty(),
            "symlink 自体は readelf しない"
        );
    }
}
