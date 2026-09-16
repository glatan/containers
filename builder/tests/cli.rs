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
        .stdout(contains("シード参照:"));
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
fn package_bootstrap_help_is_listed() {
    bin()
        .args(["package", "bootstrap", "--help"])
        .assert()
        .success()
        .stdout(contains("bootstrap"));
}

#[test]
fn image_compose_help_is_listed() {
    bin()
        .args(["image", "compose", "--help"])
        .assert()
        .success()
        .stdout(contains("compose"));
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
fn clean_requires_target_selection() {
    bin()
        .args(["package", "clean", "zlib"])
        .assert()
        .failure()
        .stderr(contains("--all-target"));
}

#[test]
fn clean_requires_name_or_all() {
    bin()
        .args(["package", "clean", "--all-target"])
        .assert()
        .failure()
        .stderr(contains("パッケージ名か --all"));
}

#[test]
fn clean_all_conflicts_with_package_name() {
    bin()
        .args(["package", "clean", "zlib", "--all", "--all-target"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn clean_removes_only_the_named_package() {
    let tmp = tempfile::tempdir().unwrap();
    let artifacts = tmp.path().join("packages/artifacts");
    let artifact = artifacts.join("zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(artifact.join("usr/lib")).unwrap();
    std::fs::write(artifact.join("usr/lib/libz.so"), "x").unwrap();
    std::fs::create_dir_all(artifacts.join("keep-1-amd64-v4")).unwrap();
    let build = tmp.path().join("packages/build/amd64-v4");
    std::fs::create_dir_all(build.join("zlib-1.3.2")).unwrap();
    let logs = tmp.path().join("packages/logs");
    std::fs::create_dir_all(&logs).unwrap();
    std::fs::write(logs.join("zlib-1.3.2-amd64-v4.log"), "log").unwrap();
    bin()
        .current_dir(tmp.path())
        .args([
            "package",
            "clean",
            "zlib",
            "--architecture",
            "amd64",
            "--variant",
            "v4",
        ])
        .assert()
        .success();
    assert!(!artifact.exists());
    assert!(artifacts.join("keep-1-amd64-v4").exists());
    assert!(!build.join("zlib-1.3.2").exists());
    assert!(!logs.join("zlib-1.3.2-amd64-v4.log").exists());
}

#[test]
fn clean_dry_run_removes_nothing() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(&artifact).unwrap();
    bin()
        .current_dir(tmp.path())
        .args([
            "package",
            "clean",
            "zlib",
            "--architecture",
            "amd64",
            "--variant",
            "v4",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(contains("削除予定"));
    assert!(artifact.exists());
}

#[test]
fn tree_prints_dependencies() {
    bin()
        .args(["package", "tree", "zlib"])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4)"))
        .stdout(contains("x86_64-linux-gnu-glibc 2.44"))
        .stdout(contains("linux-headers 6.18.33 [build]"));
}

#[test]
fn tree_rejects_unknown_target() {
    bin()
        .args([
            "package",
            "tree",
            "zlib",
            "--architecture",
            "amd64",
            "--variant",
            "v9",
        ])
        .assert()
        .failure()
        .stderr(contains("未対応のターゲット"));
}

#[test]
fn bootstrap_path_lists_minimal_seed_set() {
    bin()
        .args(["package", "bootstrap-path"])
        .assert()
        .success()
        .stdout(contains("Phase 1"))
        .stdout(contains("x86_64-linux-gnu-gcc 16.2.0"))
        .stdout(contains("image compose build-env"));
}

#[test]
fn bootstrap_path_for_bootstrap_member_shows_subset() {
    bin()
        .args(["package", "bootstrap-path", "xz"])
        .assert()
        .success()
        .stdout(contains("xz 5.8.4"))
        .stdout(contains("x86_64-linux-gnu-glibc 2.44"));
}

#[test]
fn bootstrap_path_for_catalog_package_shows_post_compose_phase() {
    bin()
        .args(["package", "bootstrap-path", "openssl"])
        .assert()
        .success()
        .stdout(contains("Phase 3"))
        .stdout(contains("openssl 4.0.2"));
}

#[test]
fn validates_real_glibc_and_openssl_artifacts() {
    let root = repo_root();
    let artifacts = root.join("packages/artifacts");
    let glibc = artifacts.join("x86_64-linux-gnu-glibc-2.44-amd64-v4");
    let openssl = artifacts.join("openssl-4.0.2-amd64-v4");
    if !glibc.is_dir() || !openssl.is_dir() {
        eprintln!("skip: パッケージがありません ({glibc:?}, {openssl:?})");
        return;
    }
    bin()
        .current_dir(root)
        .args(["image", "validate", "x86_64-linux-gnu-glibc"])
        .assert()
        .success()
        .stdout(contains("x86_64-linux-gnu-glibc 2.44 の検証に成功しました"));
    bin()
        .current_dir(root)
        .args(["image", "validate", "openssl"])
        .assert()
        .success()
        .stdout(contains("openssl 4.0.2 の検証に成功しました"));
}
