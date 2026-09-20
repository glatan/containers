//! 全パッケージの golden テスト。
//! `rendered_script` の全出力を `tests/golden/` のスナップショットと
//! byte 比較し、型付き IR への移行で生成スクリプトが変わったことを
//! 検出できるようにする。`UPDATE_GOLDEN=1` で再生成する。
use containers_builder::package::definition::all;
use containers_builder::target::{Amd64Variant, Arm64Variant, Target};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const NATIVE: Target = Target::Amd64(Amd64Variant::V4);
const CROSS: Target = Target::Arm64(Arm64Variant::V8);

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden")
}

fn slug(target: Target) -> &'static str {
    match target {
        Target::Amd64(Amd64Variant::V4) => "amd64-v4",
        Target::Arm64(Arm64Variant::V8) => "arm64-v8",
        _ => unimplemented!("golden 対象外のターゲット"),
    }
}

fn golden_path(name: &str, target: Target) -> PathBuf {
    golden_dir().join(format!("{name}-{}.sh", slug(target)))
}

/// (package name, target, rendered script) の全ケースを収集する。
fn cases() -> Vec<(String, Target, String)> {
    let mut cases = Vec::new();
    for package in all() {
        // toolchain は 6 ステージ × 5 triple。stage1/headers/stage2 は
        // BUILD_ENV でしかビルドされず、現行スクリプトはランタイム
        // ディスパッチで triple 非依存のため NATIVE で固定する。
        if package.gnu_triple().is_some() {
            cases.push((
                package.name().to_string(),
                NATIVE,
                package.rendered_script(NATIVE),
            ));
            continue;
        }
        // 非 toolchain はネイティブ/クロスの両面（PGO・host 分岐）で固定する。
        for target in [NATIVE, CROSS] {
            if package.supports(target) {
                cases.push((
                    package.name().to_string(),
                    target,
                    package.rendered_script(target),
                ));
            }
        }
    }
    cases
}

/// 全ケースの rendered_script を golden と byte 比較する。
/// UPDATE_GOLDEN=1 のときは比較せず golden を書き換える。
#[test]
fn rendered_scripts_match_golden() {
    let dir = golden_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let bless = std::env::var_os("UPDATE_GOLDEN").is_some();
    let mut mismatches = Vec::new();
    for (name, target, script) in cases() {
        let path = golden_path(&name, target);
        if bless {
            std::fs::write(&path, &script).unwrap();
            continue;
        }
        let expected = std::fs::read(&path).unwrap_or_else(|_| {
            panic!(
                "golden が無い: {}（UPDATE_GOLDEN=1 で生成）",
                path.display()
            )
        });
        if expected != script.as_bytes() {
            mismatches.push(format!("{name} ({})", slug(target)));
        }
    }
    assert!(
        mismatches.is_empty(),
        "golden と不一致: {mismatches:?}（意図的な変更なら UPDATE_GOLDEN=1 で再生成）"
    );
}

/// golden ファイルは全て `cases()` の生成ケースに対応する必要がある。
/// パッケージ削除や `supports` 変更で残った孤立ファイルを検出する。
#[test]
fn no_orphan_golden_scripts() {
    let expected: std::collections::BTreeSet<String> = cases()
        .iter()
        .map(|(name, target, _)| format!("{name}-{}.sh", slug(*target)))
        .collect();
    let mut orphans = Vec::new();
    for entry in std::fs::read_dir(golden_dir()).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        if name.ends_with(".sh") && !expected.contains(&name) {
            orphans.push(name);
        }
    }
    assert!(
        orphans.is_empty(),
        "孤立 golden（生成ケースなし）: {orphans:?}"
    );
}

/// 生成スクリプトが bash の構文として成立することを `bash -n` で検証する。
#[test]
fn rendered_scripts_pass_bash_n() {
    let mut failures = Vec::new();
    for (name, target, script) in cases() {
        let mut child = Command::new("bash")
            .arg("-n")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("bash -n の起動に失敗");
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(script.as_bytes())
            .unwrap();
        let output = child.wait_with_output().unwrap();
        if !output.status.success() {
            failures.push(format!(
                "{name} ({}): {}",
                slug(target),
                String::from_utf8_lossy(&output.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "bash -n 失敗: {failures:?}");
}
