use std::path::Path;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
use predicates::str::contains;

fn bin() -> Command {
    Command::cargo_bin("containers-builder").unwrap()
}

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap()
}

#[test]
fn env_status_prints_pinned_digest() {
    bin()
        .args(["env", "status"])
        .assert()
        .success()
        .stdout(contains("docker.io/library/archlinux"))
        .stdout(contains("base-devel"))
        .stdout(contains("sha256:"))
        .stdout(contains("シード参照:"));
}

#[test]
fn help_lists_flat_commands() {
    bin()
        .arg("--help")
        .assert()
        .success()
        .stdout(contains("ls"))
        .stdout(contains("info"))
        .stdout(contains("tree"))
        .stdout(contains("targets"))
        .stdout(contains("fetch"))
        .stdout(contains("build"))
        .stdout(contains("check"))
        .stdout(contains("pack"))
        .stdout(contains("compose"))
        .stdout(contains("clean"))
        .stdout(contains("env"))
        .stdout(predicates::str::contains("package\n").not());
}

#[test]
fn env_bootstrap_help_is_listed() {
    bin()
        .args(["env", "bootstrap", "--help"])
        .assert()
        .success()
        .stdout(contains("bootstrap"));
}

#[test]
fn compose_help_is_listed() {
    bin()
        .args(["compose", "--help"])
        .assert()
        .success()
        .stdout(contains("compose"))
        .stdout(contains("--target"));
}

#[test]
fn pack_help_lists_compression_flags() {
    bin()
        .args(["pack", "--help"])
        .assert()
        .success()
        .stdout(contains("--profile"))
        .stdout(contains("--zstd-level"))
        .stdout(contains("--with-build"));
}

#[test]
fn all_targets_conflicts_with_target() {
    bin()
        .args(["build", "zlib", "--all-targets", "-t", "amd64/v1"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn profile_conflicts_with_zstd_level() {
    bin()
        .args(["pack", "--profile", "release", "--zstd-level", "3", "zlib"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn zstd_level_out_of_range_is_rejected_by_clap() {
    bin()
        .args(["pack", "--zstd-level", "0", "zlib"])
        .assert()
        .failure()
        .stderr(contains("0"));
    bin()
        .args(["pack", "--zstd-level", "23", "zlib"])
        .assert()
        .failure()
        .stderr(contains("23"));
}

#[test]
fn pack_requires_name_or_all() {
    bin()
        .args(["pack"])
        .assert()
        .failure()
        .stderr(contains("--all"));
}

#[test]
fn unknown_package_prints_japanese_error_prefix() {
    bin()
        .args(["pack", "no-such-image"])
        .assert()
        .failure()
        .stderr(contains("エラー:"))
        .stderr(contains("パッケージが見つかりません"));
}

#[test]
fn pack_rejects_build_env() {
    bin()
        .args(["pack", "build-env"])
        .assert()
        .failure()
        .stderr(contains("env compose"));
}

#[test]
fn pack_rejects_build_only_package() {
    bin()
        .args(["pack", "linux-headers"])
        .assert()
        .failure()
        .stderr(contains("ビルド専用"));
}

#[test]
fn compose_rejects_build_env() {
    bin()
        .args(["compose", "build-env", "-t", "amd64/v4"])
        .assert()
        .failure()
        .stderr(contains("env compose"));
}

#[test]
fn compose_rejects_build_only_package() {
    bin()
        .args(["compose", "linux-headers", "-t", "amd64/v4"])
        .assert()
        .failure()
        .stderr(contains("ビルド専用"));
}

#[test]
fn check_reports_missing_artifacts() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["check", "linux-headers"])
        .assert()
        .failure()
        .stderr(contains("パッケージがありません"));
}

#[test]
fn clean_requires_target_selection() {
    bin()
        .args(["clean", "zlib"])
        .assert()
        .failure()
        .stderr(contains("--all-targets"));
}

#[test]
fn clean_requires_name_or_all() {
    bin()
        .args(["clean", "--all-targets"])
        .assert()
        .failure()
        .stderr(contains("--all"));
}

#[test]
fn clean_all_conflicts_with_package_name() {
    bin()
        .args(["clean", "zlib", "--all", "--all-targets"])
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
        .args(["clean", "zlib", "-t", "amd64/v4"])
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
        .args(["clean", "zlib", "-t", "amd64/v4", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("削除予定"));
    assert!(artifact.exists());
}

#[test]
fn tree_prints_dependencies() {
    bin()
        .args(["tree", "zlib", "-t", "amd64/v4"])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4)"))
        .stdout(contains("x86_64-linux-gnu-glibc 2.44"))
        .stdout(contains("linux-headers 6.18.33 [build]"));
}

#[test]
fn tree_rejects_unknown_target() {
    bin()
        .args(["tree", "zlib", "-t", "amd64/v9"])
        .assert()
        .failure()
        .stderr(contains("未対応のターゲット"));
}

#[test]
fn tree_requires_target() {
    bin().args(["tree", "zlib"]).assert().failure();
}

#[test]
fn info_prints_package_metadata() {
    bin()
        .args(["info", "zlib", "-t", "amd64/v4"])
        .assert()
        .success()
        .stdout(contains("名前: zlib"))
        .stdout(contains("ターゲット: amd64/v4"))
        .stdout(contains("依存: x86_64-linux-gnu-glibc 2.44"));
}

#[test]
fn info_requires_target() {
    bin().args(["info", "zlib"]).assert().failure();
}

#[test]
fn targets_lists_valid_targets() {
    bin()
        .args(["targets"])
        .assert()
        .success()
        .stdout(contains("amd64/v1\tx86_64-linux-gnu\tx86-64"))
        .stdout(contains("amd64/v4\tx86_64-linux-gnu\tx86-64-v4"))
        .stdout(contains("arm64/v8\taarch64-linux-gnu\tarmv8-a"));
}

#[test]
fn ls_lists_all_packages() {
    bin()
        .args(["ls"])
        .assert()
        .success()
        .stdout(contains("zlib"))
        .stdout(contains("linux-headers"));
}

#[test]
fn ls_images_filters_build_only() {
    bin()
        .args(["ls", "--images"])
        .assert()
        .success()
        .stdout(contains("zlib"))
        .stdout(contains("build-env"))
        .stdout(
            predicates::str::is_match("(?m)^linux-headers\t")
                .unwrap()
                .not(),
        );
}

#[test]
fn build_dry_run_prints_plan_markers() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["build", "zlib", "-t", "amd64/v4", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4) [will-build]"))
        .stdout(contains(
            "x86_64-linux-gnu-glibc 2.44 (amd64/v4) [will-build]",
        ))
        .stdout(contains("linux-headers 6.18.33 (amd64/v4) [will-build]"));
}

#[test]
fn build_dry_run_marks_cached_artifacts() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(artifact.join("usr")).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["build", "zlib", "-t", "amd64/v4", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4) [cached]"));
}

#[test]
fn build_force_rebuilds_only_root() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4/x")).unwrap();
    std::fs::create_dir_all(
        tmp.path()
            .join("packages/artifacts/x86_64-linux-gnu-glibc-2.44-amd64-v4/x"),
    )
    .unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["build", "zlib", "-t", "amd64/v4", "--force", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4) [will-build]"))
        .stdout(contains("x86_64-linux-gnu-glibc 2.44 (amd64/v4) [cached]"));
}

#[test]
fn build_no_deps_builds_only_root() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["build", "zlib", "-t", "amd64/v4", "--no-deps", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4) [will-build]"))
        .stdout(contains("glibc").not())
        .stdout(contains("linux-headers").not());
}

#[test]
fn build_no_deps_conflicts_with_all_and_keep_going() {
    bin()
        .args(["build", "--all", "-t", "amd64/v4", "--no-deps"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
    bin()
        .args([
            "build",
            "zlib",
            "-t",
            "amd64/v4",
            "--no-deps",
            "--keep-going",
        ])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn build_all_dry_run_deduplicates_shared_deps() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["build", "--all", "-t", "amd64/v4", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("x86_64-linux-gnu-glibc 2.44 (amd64/v4)").count(1));
}

#[test]
fn build_all_skips_unsupported_roots() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["build", "--all", "-t", "arm64/v8", "--dry-run"])
        .assert()
        .success()
        .stdout(contains(
            "x86_64-linux-gnu-glibc 2.44 (arm64/v8) [skip:unsupported]",
        ));
}

#[test]
fn build_repeated_target_is_deduplicated() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args([
            "build",
            "zlib",
            "-t",
            "amd64/v4",
            "-t",
            "amd64/v4",
            "--dry-run",
        ])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4)").count(1));
}

#[test]
fn build_single_unsupported_target_errors() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args([
            "build",
            "x86_64-linux-gnu-glibc",
            "-t",
            "arm64/v8",
            "--dry-run",
        ])
        .assert()
        .failure()
        .stderr(contains("この platform を提供しません"));
}

#[test]
fn ls_prints_five_columns_with_role() {
    bin()
        .args(["ls"])
        .assert()
        .success()
        .stdout(contains(
            "zlib\t1.3.2\tcatalog\tx86_64-linux-gnu-glibc\tlinux-headers",
        ))
        .stdout(contains("x86_64-linux-gnu-binutils\t2.47\tpackage"))
        .stdout(contains("\tなし\t"));
}

#[test]
fn info_prints_supported_targets() {
    bin()
        .args(["info", "zlib", "-t", "amd64/v4"])
        .assert()
        .success()
        .stdout(contains("対応ターゲット: amd64:"))
        .stdout(contains("arm64:"));
}

#[test]
fn info_shows_artifact_target_when_different() {
    bin()
        .args(["info", "x86_64-linux-gnu-gcc", "-t", "arm64/v8"])
        .assert()
        .success()
        .stdout(contains("ターゲット: arm64/v8"))
        .stdout(contains("成果物ターゲット: amd64/v4"));
}

#[test]
fn tree_reverse_shows_dependers() {
    bin()
        .args(["tree", "linux-headers", "-t", "amd64/v4", "--reverse"])
        .assert()
        .success()
        .stdout(contains("linux-headers 6.18.33 (amd64/v4)"))
        .stdout(contains("[build"))
        .stdout(contains("zlib"));
}

#[test]
fn tree_reverse_preserves_tag_order() {
    bin()
        .args(["tree", "zlib", "-t", "amd64/v4", "--reverse"])
        .assert()
        .success()
        .stdout(
            predicates::str::is_match("\\[build, [a-z0-9]+/[a-zA-Z0-9._-]+, bootstrap\\]").unwrap(),
        );
}

#[test]
fn fetch_deps_flag_is_accepted() {
    bin()
        .args(["fetch", "--help"])
        .assert()
        .success()
        .stdout(contains("--deps"));
}

#[test]
fn fetch_deps_unknown_package_errors() {
    bin()
        .args(["fetch", "no-such-pkg", "--deps"])
        .assert()
        .failure()
        .stderr(contains("パッケージが見つかりません"));
}

#[test]
fn fetch_skips_sourceless_build_env() {
    bin()
        .args(["fetch", "build-env"])
        .assert()
        .success()
        .stdout(contains("ソースを持たないためスキップ"));
}

#[test]
fn compose_help_lists_with_build() {
    bin()
        .args(["compose", "--help"])
        .assert()
        .success()
        .stdout(contains("--with-build"));
}

#[test]
fn clean_images_dry_run_lists_oci_layout() {
    let tmp = tempfile::tempdir().unwrap();
    let layout = tmp.path().join("oci-layouts/zlib-1.3.2");
    std::fs::create_dir_all(&layout).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["clean", "zlib", "-t", "amd64/v4", "--images", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("削除予定"))
        .stdout(contains("oci-layouts"));
    assert!(layout.exists());
}

#[test]
fn clean_without_images_keeps_oci_layout() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4")).unwrap();
    let layout = tmp.path().join("oci-layouts/zlib-1.3.2");
    std::fs::create_dir_all(&layout).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["clean", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();
    assert!(layout.exists());
}

#[test]
fn clean_images_removes_oci_layout() {
    let tmp = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(tmp.path().join("packages")).unwrap();
    let layout = tmp.path().join("oci-layouts/zlib-1.3.2");
    std::fs::create_dir_all(&layout).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["clean", "zlib", "-t", "amd64/v4", "--images"])
        .assert()
        .success();
    assert!(!layout.exists());
}

#[test]
fn env_plan_lists_minimal_seed_set() {
    bin()
        .args(["env", "plan"])
        .assert()
        .success()
        .stdout(contains("Phase 1"))
        .stdout(contains("x86_64-linux-gnu-gcc 16.2.0"))
        .stdout(contains("env compose"));
}

#[test]
fn env_plan_for_bootstrap_member_shows_subset() {
    bin()
        .args(["env", "plan", "xz"])
        .assert()
        .success()
        .stdout(contains("xz 5.8.4"))
        .stdout(contains("x86_64-linux-gnu-glibc 2.44"));
}

#[test]
fn env_plan_for_catalog_package_shows_post_compose_phase() {
    bin()
        .args(["env", "plan", "openssl"])
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
        .args(["check", "x86_64-linux-gnu-glibc"])
        .assert()
        .success()
        .stdout(contains("x86_64-linux-gnu-glibc 2.44 の検証に成功しました"));
    bin()
        .current_dir(root)
        .args(["check", "openssl"])
        .assert()
        .success()
        .stdout(contains("openssl 4.0.2 の検証に成功しました"));
}
