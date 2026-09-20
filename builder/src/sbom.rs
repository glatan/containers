use serde::Serialize;

use crate::error::Result;
use crate::package::Package;
use crate::util::timestamp;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    spdx_version: &'static str,
    data_license: &'static str,
    #[serde(rename = "SPDXID")]
    spdx_id: &'static str,
    name: String,
    document_namespace: String,
    creation_info: CreationInfo,
    packages: Vec<SpdxPackage>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CreationInfo {
    created: String,
    creators: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SpdxPackage {
    #[serde(rename = "SPDXID")]
    spdx_id: String,
    name: &'static str,
    version_info: &'static str,
    download_location: &'static str,
    checksums: Vec<Checksum>,
    license_declared: &'static str,
    license_concluded: &'static str,
    external_refs: Vec<ExternalRef>,
}

#[derive(Serialize)]
struct Checksum {
    algorithm: &'static str,
    #[serde(rename = "checksumValue")]
    checksum_value: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExternalRef {
    reference_category: &'static str,
    reference_type: &'static str,
    reference_locator: String,
}

pub fn document(package: &dyn Package) -> Result<Document> {
    let created = timestamp();
    let name = package.name();
    let version = package.version();
    Ok(Document {
        spdx_version: "SPDX-2.3",
        data_license: "CC0-1.0",
        spdx_id: "SPDXRef-DOCUMENT",
        name: format!("{name}-{version}"),
        document_namespace: format!(
            "https://spdx.org/spdxdocs/containers-{name}-{version}-{created}"
        ),
        creation_info: CreationInfo {
            created,
            creators: vec![format!(
                "Tool: containers-builder-{}",
                env!("CARGO_PKG_VERSION")
            )],
        },
        packages: vec![SpdxPackage {
            spdx_id: format!("SPDXRef-{name}"),
            name,
            version_info: version,
            download_location: package.source().url,
            checksums: vec![
                Checksum {
                    algorithm: "SHA3-512",
                    checksum_value: package.source().sha3_512,
                },
                Checksum {
                    algorithm: "BLAKE3",
                    checksum_value: package.source().blake3,
                },
            ],
            license_declared: package.license(),
            license_concluded: package.license(),
            external_refs: vec![ExternalRef {
                reference_category: "PACKAGE-MANAGER",
                reference_type: "purl",
                reference_locator: format!("pkg:generic/{name}@{version}"),
            }],
        }],
    })
}

pub fn json(package: &dyn Package) -> Result<Vec<u8>> {
    Ok(serde_json::to_vec(&document(package)?)?)
}

pub fn json_many(packages: &[&dyn Package]) -> Result<Vec<u8>> {
    if packages.is_empty() {
        return Err(crate::error::Error::message(
            "合成するパッケージがありません",
        ));
    }
    let created = timestamp();
    let names = packages
        .iter()
        .map(|package| package.name())
        .collect::<Vec<_>>()
        .join("+");
    let mut document = document(packages[0])?;
    document.name = names.clone();
    document.document_namespace = format!("https://spdx.org/spdxdocs/containers-{names}-{created}");
    document.packages = packages
        .iter()
        .map(|package| {
            let name = package.name();
            let version = package.version();
            SpdxPackage {
                spdx_id: format!("SPDXRef-{name}"),
                name,
                version_info: version,
                download_location: package.source().url,
                checksums: vec![
                    Checksum {
                        algorithm: "SHA3-512",
                        checksum_value: package.source().sha3_512,
                    },
                    Checksum {
                        algorithm: "BLAKE3",
                        checksum_value: package.source().blake3,
                    },
                ],
                license_declared: package.license(),
                license_concluded: package.license(),
                external_refs: vec![ExternalRef {
                    reference_category: "PACKAGE-MANAGER",
                    reference_type: "purl",
                    reference_locator: format!("pkg:generic/{name}@{version}"),
                }],
            }
        })
        .collect();
    Ok(serde_json::to_vec(&document)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::{Package, Source};

    struct Quoted;

    impl Package for Quoted {
        fn name(&self) -> &'static str {
            r#"pkg"quote"#
        }
        fn version(&self) -> &'static str {
            "1.0"
        }
        fn license(&self) -> &'static str {
            "MIT"
        }
        fn source(&self) -> Source {
            Source {
                url: r#"https://example.invalid/pkg"quote.tar.gz"#,
                sha3_512: "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                blake3: "00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
                subdir: "pkg",
            }
        }
        fn dependencies(&self, _target: crate::target::Target) -> Vec<crate::package::Dependency> {
            vec![]
        }
        fn build_dependencies(
            &self,
            _target: crate::target::Target,
        ) -> Vec<crate::package::Dependency> {
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
    }

    #[test]
    fn escapes_json_fields() {
        let raw = String::from_utf8(json(&Quoted).unwrap()).unwrap();
        assert!(raw.contains(r#"pkg\"quote"#), "{raw}");
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(value["spdxVersion"], "SPDX-2.3");
        assert_eq!(value["packages"][0]["name"], r#"pkg"quote"#);
        assert_eq!(
            value["packages"][0]["downloadLocation"],
            r#"https://example.invalid/pkg"quote.tar.gz"#
        );
        assert_eq!(
            value["packages"][0]["checksums"][0]["algorithm"],
            "SHA3-512"
        );
        assert_eq!(value["packages"][0]["checksums"][1]["algorithm"], "BLAKE3");
        assert!(
            value["documentNamespace"]
                .as_str()
                .unwrap()
                .contains(r#"pkg"quote-1.0"#)
        );
    }
}
