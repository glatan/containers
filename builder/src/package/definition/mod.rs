use crate::error::{Error, Result};
use crate::package::Package;

mod glibc;
mod linux_headers;
mod openssl;

pub static PACKAGES: &[&dyn Package] = &[
    &glibc::Glibc,
    &linux_headers::LinuxHeaders,
    &openssl::Openssl,
];

pub fn all() -> &'static [&'static dyn Package] {
    PACKAGES
}

pub fn images() -> impl Iterator<Item = &'static dyn Package> {
    PACKAGES
        .iter()
        .copied()
        .filter(|package| !package.build_only())
}

pub fn find(name: &str) -> Result<&'static dyn Package> {
    PACKAGES
        .iter()
        .copied()
        .find(|package| package.name() == name)
        .ok_or_else(|| Error::message(format!("パッケージが見つかりません: {name}")))
}

pub fn find_image(name: &str) -> Result<&'static dyn Package> {
    let package = find(name)?;
    if package.build_only() {
        return Err(Error::message(format!(
            "{} はビルド専用のパッケージです",
            package.name()
        )));
    }
    Ok(package)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_are_unique() {
        let mut names = PACKAGES
            .iter()
            .map(|package| package.name())
            .collect::<Vec<_>>();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), PACKAGES.len());
    }

    #[test]
    fn finds_known_packages() {
        assert_eq!(find("glibc").unwrap().version(), "2.44");
        assert!(find("missing").is_err());
        assert!(find_image("linux-headers").is_err());
        assert_eq!(images().count(), 2);
    }

    #[test]
    fn openssl_declares_runtime_and_build_deps() {
        let openssl = find("openssl").unwrap();
        assert!(openssl.depends_on("glibc"));
        assert!(!openssl.depends_on("linux-headers"));
        assert!(openssl.build_depends_on("linux-headers"));
        assert!(find("glibc").unwrap().build_depends_on("linux-headers"));
        assert!(!find("glibc").unwrap().depends_on("linux-headers"));
    }

    #[test]
    fn each_package_exposes_static_metadata() {
        for package in PACKAGES {
            assert!(!package.name().is_empty());
            assert!(!package.version().is_empty());
            assert!(!package.license().is_empty());
            assert!(!package.source().url.is_empty());
            assert_eq!(package.source().sha3_512.len(), 128);
            assert_eq!(package.source().blake3.len(), 128);
            assert!(
                package
                    .source()
                    .sha3_512
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            );
            assert!(
                package
                    .source()
                    .blake3
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            );
            let mut names = std::collections::BTreeSet::new();
            for dependency in package
                .dependencies()
                .iter()
                .chain(package.build_dependencies())
            {
                assert!(
                    names.insert(dependency.name),
                    "{} の依存名が重複しています: {}",
                    package.name(),
                    dependency.name
                );
                let declared = find(dependency.name).unwrap();
                assert_eq!(declared.version(), dependency.version);
            }
            for dependency in package.dependencies() {
                assert!(
                    !find(dependency.name).unwrap().build_only(),
                    "dependencies にビルド専用パッケージは置けません: {}",
                    dependency.name
                );
            }
            assert!(!package.source().subdir.is_empty());
            assert!(!package.build_script().is_empty());
        }
        let headers = find("linux-headers").unwrap();
        assert!(headers.build_only());
        assert!(!find("glibc").unwrap().build_only());
        assert!(find("glibc").unwrap().network());
        assert!(find("openssl").unwrap().network());
        assert!(
            find("linux-headers")
                .unwrap()
                .build_script()
                .contains("riscv64")
        );
        assert!(
            find("openssl")
                .unwrap()
                .build_script()
                .contains("linux64-riscv64")
        );
    }
}
