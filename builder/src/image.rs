use std::path::PathBuf;

use crate::error::{Error, Result};
use crate::oci;
use crate::package::{self, Package};
use crate::sbom;
use crate::target::Target;
use crate::workspace;

pub fn build(package: &dyn Package, zstd_level: u8, with_packages: bool) -> Result<PathBuf> {
    if with_packages {
        let targets: Vec<_> = Target::all().collect();
        package::build_package(package, &targets, true)?;
    }
    let mut layers = Vec::new();
    for target in Target::all() {
        let dir = package.artifact_dir(target);
        if dir.is_dir() {
            layers.push((target, vec![dir]));
        } else {
            eprintln!("スキップ(パッケージなし): {}", dir.display());
        }
    }
    if layers.is_empty() {
        return Err(Error::message(format!(
            "パッケージがありません: {} (package build を実行してください)",
            package.name()
        )));
    }
    let output = workspace::layout_dir(package.name(), package.version());
    let image = oci::Image {
        name: package.name().to_string(),
        version: package.version().to_string(),
    };
    oci::build(&output, &image, &layers, zstd_level)?;
    crate::util::write(&output.join("sbom.spdx.json"), &sbom::json(package)?)?;
    Ok(output)
}
