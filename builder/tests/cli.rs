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
fn pack_rejects_jobs_zero() {
    bin()
        .args(["pack", "zlib", "--jobs", "0"])
        .assert()
        .failure()
        .stderr(contains("--jobs は 1 以上"));
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
        .stdout(contains("docs"))
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
        .stderr(contains("--all-packages"));
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
        .args(["clean", "zlib", "--packages-build"])
        .assert()
        .failure()
        .stderr(contains("--all-targets"));
}

#[test]
fn clean_requires_name_or_all() {
    bin()
        .args(["clean", "--all-targets", "--packages-build"])
        .assert()
        .failure()
        .stderr(contains("--all-packages"));
}

/// -t で platform を絞った pack が既存 layout に merge され、
/// --remove-platform で除去できること（pack の全再生成 → 増分 merge
/// への再設計の回帰テスト）。
#[test]
fn pack_merges_platform_and_remove_platform_reverts() {
    let tmp = tempfile::tempdir().unwrap();
    let make_artifact = |variant: &str| {
        let dir = tmp
            .path()
            .join(format!("packages/artifacts/zlib-1.3.2-amd64-{variant}"));
        std::fs::create_dir_all(dir.join("usr/lib")).unwrap();
        std::fs::write(dir.join(format!("usr/lib/libz-{variant}.so")), variant).unwrap();
    };
    make_artifact("v4");
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();
    let layout = tmp.path().join("oci-layouts/zlib-1.3.2");
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(layout.join("index.json")).unwrap()).unwrap();
    let manifests = index["manifests"].as_array().unwrap();
    assert_eq!(manifests.len(), 1);
    assert_eq!(
        manifests[0]["mediaType"], "application/vnd.oci.image.manifest.v1+json",
        "単一 platform は flat index"
    );

    // 別 platform の pack は既存 platform を保持して merge する。
    make_artifact("v3");
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v3"])
        .assert()
        .success();
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(layout.join("index.json")).unwrap()).unwrap();
    let top = &index["manifests"][0];
    assert_eq!(
        top["mediaType"], "application/vnd.oci.image.index.v1+json",
        "2 platform で nested index になる"
    );
    let nested: serde_json::Value = serde_json::from_slice(
        &std::fs::read(
            layout.join("blobs/sha256").join(
                top["digest"]
                    .as_str()
                    .unwrap()
                    .strip_prefix("sha256:")
                    .unwrap(),
            ),
        )
        .unwrap(),
    )
    .unwrap();
    let variants: Vec<_> = nested["manifests"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["platform"]["variant"].as_str().unwrap())
        .collect();
    assert_eq!(variants, ["v3", "v4"], "両 platform が保持される");

    // --remove-platform で v3 を除くと flat (v4) に戻る。
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "--remove-platform", "amd64/v3"])
        .assert()
        .success();
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(layout.join("index.json")).unwrap()).unwrap();
    let manifests = index["manifests"].as_array().unwrap();
    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0]["platform"]["variant"], "v4");
}

#[test]
fn pack_replace_conflicts_with_target_selection() {
    bin()
        .args(["pack", "zlib", "-t", "amd64/v4", "--replace"])
        .assert()
        .failure()
        .stderr(contains("cannot be used with"));
}

#[test]
fn pack_remove_last_platform_is_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(&artifact).unwrap();
    std::fs::write(artifact.join("file"), "x").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "--remove-platform", "amd64/v4"])
        .assert()
        .failure()
        .stderr(contains("platform が0件"));
}

/// layout の blob から成果物を復元する restore の回帰テスト。
#[test]
fn restore_extracts_artifact_from_layout() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(artifact.join("usr/lib")).unwrap();
    std::fs::write(artifact.join("usr/lib/libz.so"), "payload").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();
    // 成果物を消してから restore で復元する。
    std::fs::remove_dir_all(&artifact).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["restore", "zlib", "-t", "amd64/v4"])
        .assert()
        .success()
        .stdout(contains("復元"));
    assert_eq!(
        std::fs::read_to_string(artifact.join("usr/lib/libz.so")).unwrap(),
        "payload"
    );
}

#[test]
fn restore_missing_layout_without_from_registry_errors() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["restore", "zlib"])
        .assert()
        .failure()
        .stderr(contains("--from-registry"));
}

#[test]
fn restore_rejects_build_env() {
    bin()
        .args(["restore", "build-env"])
        .assert()
        .failure()
        .stderr(contains("合成イメージ"));
}

#[test]
fn restore_skips_existing_and_force_restores() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(artifact.join("usr/lib")).unwrap();
    std::fs::write(artifact.join("usr/lib/libz.so"), "packed").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();

    // 既存成果物がある restore はスキップする。
    std::fs::write(artifact.join("usr/lib/libz.so"), "modified").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["restore", "zlib", "-t", "amd64/v4"])
        .assert()
        .success()
        .stdout(contains("スキップ(既存)"));
    assert_eq!(
        std::fs::read_to_string(artifact.join("usr/lib/libz.so")).unwrap(),
        "modified"
    );

    // --force は既存成果物をバックアップ付きで上書きする。
    bin()
        .current_dir(tmp.path())
        .args(["restore", "zlib", "-t", "amd64/v4", "--force"])
        .assert()
        .success()
        .stdout(contains("復元"));
    assert_eq!(
        std::fs::read_to_string(artifact.join("usr/lib/libz.so")).unwrap(),
        "packed"
    );
}

#[test]
fn restore_missing_platform_is_rejected() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(&artifact).unwrap();
    std::fs::write(artifact.join("f"), "x").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();
    bin()
        .current_dir(tmp.path())
        .args(["restore", "zlib", "-t", "amd64/v3"])
        .assert()
        .failure()
        .stderr(contains("platform amd64/v3 がありません"));
}

#[test]
fn pack_replace_requires_all_platform_artifacts() {
    // --replace は全再生成のため supported 全 platform の成果物を要求する。
    // 一部のみだと保持されない platform が黙って layout から落ちるため
    // 拒否する（外したい場合は --remove-platform 併用）。
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(&artifact).unwrap();
    std::fs::write(artifact.join("f"), "x").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "--replace"])
        .assert()
        .failure()
        .stderr(contains("--replace は全 platform の成果物が必要です"));
}

#[test]
fn pack_target_without_artifact_is_rejected() {
    // -t で絞った pack に対象成果物が無い場合はエラーになる
    // （merge の no-op で黙って何もしないことにならない）。
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .failure()
        .stderr(contains("成果物がありません"))
        .stderr(contains("amd64/v4"));
}

/// verify が健全な layout を OK とし、blob 喪失を検出する回帰テスト。
#[test]
fn verify_detects_missing_blob() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(artifact.join("usr/lib")).unwrap();
    std::fs::write(artifact.join("usr/lib/libz.so"), "x").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();
    bin()
        .current_dir(tmp.path())
        .args(["verify", "zlib"])
        .assert()
        .success()
        .stdout(contains("OK"));
    // layer blob を 1 つ消すと検出される。
    let blobs = tmp.path().join("oci-layouts/zlib-1.3.2/blobs/sha256");
    let victim = std::fs::read_dir(&blobs)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .max_by_key(|entry| entry.metadata().map(|m| m.len()).unwrap_or(0))
        .unwrap();
    std::fs::remove_file(victim.path()).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["verify", "zlib"])
        .assert()
        .failure()
        .stdout(contains("blob がありません"));
}

#[test]
fn verify_rejects_missing_layout() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["verify", "zlib"])
        .assert()
        .failure()
        .stderr(contains("layout がありません"));
}

/// nested index の blob 改竄（内容は存在するが digest 不一致）を
/// verify が検出する回帰テスト。
#[test]
fn verify_detects_corrupt_nested_blob() {
    let tmp = tempfile::tempdir().unwrap();
    for variant in ["v3", "v4"] {
        let artifact = tmp
            .path()
            .join(format!("packages/artifacts/zlib-1.3.2-amd64-{variant}"));
        std::fs::create_dir_all(&artifact).unwrap();
        std::fs::write(artifact.join("f"), variant).unwrap();
    }
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v3", "-t", "amd64/v4"])
        .assert()
        .success();
    // ネスト index blob を書き換える（index.json が指す digest の blob）。
    let layout = tmp.path().join("oci-layouts/zlib-1.3.2");
    let index: serde_json::Value =
        serde_json::from_slice(&std::fs::read(layout.join("index.json")).unwrap()).unwrap();
    let nested_digest = index["manifests"][0]["digest"]
        .as_str()
        .unwrap()
        .strip_prefix("sha256:")
        .unwrap();
    std::fs::write(layout.join("blobs/sha256").join(nested_digest), b"corrupt").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["verify", "zlib"])
        .assert()
        .failure()
        .stdout(contains("digest が一致しません"));
}

/// status --local が artifacts/layout/index の状態を表示する回帰テスト。
#[test]
fn status_local_reports_layout_state() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(&artifact).unwrap();
    std::fs::write(artifact.join("file"), "x").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["pack", "zlib", "-t", "amd64/v4"])
        .assert()
        .success();
    bin()
        .current_dir(tmp.path())
        .args(["status", "zlib", "--local"])
        .assert()
        .success()
        .stdout(contains("layout: 1 platform"))
        .stdout(contains("artifacts: amd64/v4"))
        .stdout(contains("remote: -"));
}

/// index.json が壊れた layout を status が「破損」と表示する回帰テスト
/// （読み取りエラーを握り潰さずに可視化する）。
#[test]
fn status_local_reports_broken_layout() {
    let tmp = tempfile::tempdir().unwrap();
    let layout = tmp.path().join("oci-layouts/zlib-1.3.2");
    std::fs::create_dir_all(&layout).unwrap();
    std::fs::write(layout.join("index.json"), b"not json").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["status", "zlib", "--local"])
        .assert()
        .success()
        .stdout(contains("layout: 破損"));
}

/// 破損 layout が混在していても `index --all-issues` が全件集約して
/// 報告する回帰テスト（先頭1件で止まらない）。
#[test]
fn index_all_issues_aggregates_failures() {
    let tmp = tempfile::tempdir().unwrap();
    let broken = tmp.path().join("oci-layouts/zlib-1.3.2");
    std::fs::create_dir_all(&broken).unwrap();
    std::fs::write(broken.join("index.json"), b"not json").unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["index", "--all-issues", "--registry", "ghcr.io/example"])
        .assert()
        .failure()
        .stderr(contains("zlib"))
        .stderr(contains("件の問題"));
}

#[test]
fn clean_all_conflicts_with_package_name() {
    bin()
        .args([
            "clean",
            "zlib",
            "--all-packages",
            "--all-targets",
            "--packages-build",
        ])
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
            "clean",
            "zlib",
            "-t",
            "amd64/v4",
            "--packages-build",
            "--packages-artifacts",
            "--packages-logs",
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
            "clean",
            "zlib",
            "-t",
            "amd64/v4",
            "--packages-artifacts",
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
        .args(["tree", "zlib", "-t", "amd64/v4"])
        .assert()
        .success()
        .stdout(contains("zlib 1.3.2 (amd64/v4)"))
        .stdout(contains("x86_64-linux-gnu-glibc 2.44"))
        .stdout(contains("linux-headers 6.18.56 [build, bootstrap]"));
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
        .stdout(contains("linux-headers 6.18.56 (amd64/v4) [will-build]"));
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
        .args(["build", "--all-packages", "-t", "amd64/v4", "--no-deps"])
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
        .args(["build", "--all-packages", "-t", "amd64/v4", "--dry-run"])
        .assert()
        .success()
        .stdout(contains("x86_64-linux-gnu-glibc 2.44 (amd64/v4)").count(1));
}

#[test]
fn build_all_skips_unsupported_roots() {
    let tmp = tempfile::tempdir().unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["build", "--all-packages", "-t", "arm64/v8", "--dry-run"])
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
        .stdout(contains("linux-headers 6.18.56 (amd64/v4)"))
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
        .args(["clean", "zlib", "--oci-layouts", "--dry-run"])
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
        .args(["clean", "zlib", "-t", "amd64/v4", "--packages-artifacts"])
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
        .args(["clean", "zlib", "--oci-layouts"])
        .assert()
        .success();
    assert!(!layout.exists());
}

#[test]
fn clean_oci_layouts_rejects_target_selection() {
    bin()
        .args(["clean", "zlib", "-t", "amd64/v4", "--oci-layouts"])
        .assert()
        .failure()
        .stderr(contains("--all-targets"));
}

#[test]
fn clean_packages_build_keeps_artifact_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp.path().join("packages/artifacts/zlib-1.3.2-amd64-v4");
    std::fs::create_dir_all(artifact.join("usr")).unwrap();
    let build = tmp.path().join("packages/build/amd64-v4/zlib-1.3.2");
    std::fs::create_dir_all(&build).unwrap();
    let tmp_artifact = tmp
        .path()
        .join("packages/artifacts/zlib-1.3.2-amd64-v4.tmp");
    std::fs::create_dir_all(&tmp_artifact).unwrap();
    bin()
        .current_dir(tmp.path())
        .args(["clean", "zlib", "-t", "amd64/v4", "--packages-build"])
        .assert()
        .success();
    assert!(artifact.exists());
    assert!(!build.exists());
    assert!(!tmp_artifact.exists());
}

#[test]
fn clean_packages_artifacts_refuses_while_active_lock_is_held() {
    let tmp = tempfile::tempdir().unwrap();
    let artifact = tmp
        .path()
        .join("packages/artifacts/zlib-1.3.2-amd64-v4/usr");
    std::fs::create_dir_all(&artifact).unwrap();
    let lock_path = tmp.path().join("packages/artifacts/.locks/active.lock");
    let held = containers_builder::workspace::ExclusiveLock::try_acquire(&lock_path).unwrap();
    assert!(held.is_some());
    bin()
        .current_dir(tmp.path())
        .args(["clean", "zlib", "-t", "amd64/v4", "--packages-artifacts"])
        .assert()
        .failure()
        .stderr(contains("packages/artifacts/"));
    assert!(artifact.exists());
}

#[test]
fn clean_packages_build_skips_busy_cell() {
    let tmp = tempfile::tempdir().unwrap();
    let build = tmp.path().join("packages/build/amd64-v4/zlib-1.3.2");
    std::fs::create_dir_all(&build).unwrap();
    let lock_path = tmp
        .path()
        .join("packages/artifacts/.locks/zlib-1.3.2-amd64-v4.lock");
    let held = containers_builder::workspace::ExclusiveLock::try_acquire(&lock_path).unwrap();
    assert!(held.is_some());
    bin()
        .current_dir(tmp.path())
        .args(["clean", "zlib", "-t", "amd64/v4", "--packages-build"])
        .assert()
        .success()
        .stdout(contains("実行中のためスキップ"));
    assert!(build.exists());
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
        .stdout(contains("openssl 4.0.3"));
}

#[test]
fn validates_real_glibc_and_openssl_artifacts() {
    let root = repo_root();
    let artifacts = root.join("packages/artifacts");
    let glibc = artifacts.join("x86_64-linux-gnu-glibc-2.44-amd64-v4");
    let openssl = artifacts.join("openssl-4.0.3-amd64-v4");
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
        .stdout(contains("openssl 4.0.3 の検証に成功しました"));
}

#[test]
fn index_check_requires_existing_artifact() {
    // テストの CWD は builder/ で oci-layouts/ は存在しない。
    bin()
        .args(["index", "--check", "--registry", "ghcr.io/example"])
        .assert()
        .failure()
        .stderr(contains("index が生成されていません"));
}

#[test]
fn push_dry_run_without_layouts_is_a_noop() {
    bin()
        .args([
            "push",
            "--all-packages",
            "--dry-run",
            "--registry",
            "ghcr.io/example",
        ])
        .assert()
        .success()
        .stdout(contains("push 対象がありません"));
}

#[test]
fn registry_prefix_is_required_for_push_and_index() {
    bin()
        .env_remove("CONTAINERS_REGISTRY")
        .args(["push", "--all-packages", "--dry-run"])
        .assert()
        .failure()
        .stderr(contains("レジストリ前置詞を指定してください"));
    bin()
        .env_remove("CONTAINERS_REGISTRY")
        .args(["index"])
        .assert()
        .failure()
        .stderr(contains("レジストリ前置詞を指定してください"));
    bin()
        .env_remove("CONTAINERS_REGISTRY")
        .args(["restore", "zlib", "--from-registry"])
        .assert()
        .failure()
        .stderr(contains("レジストリ前置詞を指定してください"));
}

#[test]
fn sync_requires_existing_config() {
    bin()
        .args(["sync", "nonexistent-containers.json"])
        .assert()
        .failure();
}
