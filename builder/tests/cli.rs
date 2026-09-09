use std::path::Path;

use assert_cmd::Command;
use predicates::str::contains;

fn bin() -> Command {
    Command::cargo_bin("containers-builder").unwrap()
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

#[test]
fn package_env_prints_pinned_digest() {
    bin()
        .args(["package", "env"])
        .assert()
        .success()
        .stdout(contains("docker.io/library/archlinux"))
        .stdout(contains("base-devel"))
        .stdout(contains("sha256:"))
        .stdout(contains("@sha256:"));
}

#[test]
fn help_lists_package_and_image() {
    bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("package"))
        .stdout(contains("image"));
}

#[test]
fn image_build_help_lists_compression_flags() {
    bin()
        .args(["image", "build", "--help"])
        .assert()
        .success()
        .stdout(contains("--publish"))
        .stdout(contains("--zstd-level"));
}

#[test]
fn all_target_conflicts_with_architecture() {
    bin()
        .args([
            "package",
            "build",
            "--all-target",
            "--architecture",
            "amd64",
            "--variant",
            "v1",
            "glibc",
        ])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn publish_conflicts_with_zstd_level() {
    bin()
        .args(["image", "build", "--publish", "--zstd-level", "3", "glibc"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn zstd_level_out_of_range_is_rejected_by_clap() {
    bin()
        .args(["image", "build", "--zstd-level", "0", "glibc"])
        .assert()
        .failure()
        .stderr(contains("0"));
    bin()
        .args(["image", "build", "--zstd-level", "23", "glibc"])
        .assert()
        .failure()
        .stderr(contains("23"));
}

#[test]
fn image_build_requires_name_or_all() {
    bin()
        .args(["image", "build"])
        .assert()
        .failure()
        .stderr(contains("エラー:"))
        .stderr(contains("パッケージ名か --all"));
}

#[test]
fn unknown_package_prints_japanese_error_prefix() {
    bin()
        .args(["image", "build", "no-such-image"])
        .assert()
        .failure()
        .stderr(contains("エラー:"))
        .stderr(contains("パッケージが見つかりません"));
}

#[test]
fn image_validate_rejects_build_only_package() {
    bin()
        .args(["image", "validate", "linux-headers"])
        .assert()
        .failure()
        .stderr(contains("ビルド専用"));
}

#[test]
fn validates_real_glibc_and_openssl_artifacts() {
    let root = repo_root();
    let artifacts = root.join("packages/artifacts");
    let glibc = artifacts.join("glibc-2.44-amd64-v3");
    let openssl = artifacts.join("openssl-3.6.4-amd64-v3");
    if !glibc.is_dir() || !openssl.is_dir() {
        eprintln!("skip: パッケージがありません ({glibc:?}, {openssl:?})");
        return;
    }
    bin()
        .current_dir(root)
        .args(["image", "validate", "glibc"])
        .assert()
        .success()
        .stdout(contains("glibc 2.44 の検証に成功しました"));
    bin()
        .current_dir(root)
        .args(["image", "validate", "openssl"])
        .assert()
        .success()
        .stdout(contains("openssl 3.6.4 の検証に成功しました"));
}
