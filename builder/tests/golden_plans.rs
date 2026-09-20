//! 計画レベルの golden テスト。
//! `recipe()` が構築する型付き計画（`Recipe` IR）を正準 JSON に
//! シリアライズし `tests/golden-plans/` と byte 比較する。
//! レンダリング（bash バックエンド）ではなく計画そのものの変更を
//! 検出するためのもので、将来の executor 非依存 runner が
//! 消費するのと同じデータ形を固定する。`UPDATE_GOLDEN=1` で再生成。
use containers_builder::package::definition::all;
use containers_builder::package::recipe::BuildContext;
use containers_builder::target::{Amd64Variant, Arm64Variant, Target};
use std::path::{Path, PathBuf};

const NATIVE: Target = Target::Amd64(Amd64Variant::V4);
const CROSS: Target = Target::Arm64(Arm64Variant::V8);

fn golden_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden-plans")
}

fn slug(target: Target) -> &'static str {
    match target {
        Target::Amd64(Amd64Variant::V4) => "amd64-v4",
        Target::Arm64(Arm64Variant::V8) => "arm64-v8",
        _ => unimplemented!("golden 対象外のターゲット"),
    }
}

/// (package name, target, canonical plan JSON) の全ケースを収集する。
fn cases() -> Vec<(String, Target, String)> {
    let mut cases = Vec::new();
    for package in all() {
        let targets: &[Target] = if package.gnu_triple().is_some() {
            &[NATIVE]
        } else {
            &[NATIVE, CROSS]
        };
        for target in targets {
            if !package.supports(*target) {
                continue;
            }
            let ctx = BuildContext::of(package.gnu_triple(), *target);
            let recipe = package.recipe(&ctx);
            // `Value`（BTreeMap）経由にしてオブジェクトキーを
            // ソートし、宣言順に依らない正準 JSON にする。
            let value = serde_json::to_value(&recipe)
                .unwrap_or_else(|_| panic!("{} のシリアライズに失敗", package.name()));
            let json = serde_json::to_string_pretty(&value).unwrap();
            cases.push((package.name().to_string(), *target, format!("{json}\n")));
        }
    }
    cases
}

/// golden ファイルは全て `cases()` の生成ケースに対応する必要がある。
/// パッケージ削除や `supports` 変更で残った孤立ファイルを検出する。
#[test]
fn no_orphan_golden_plans() {
    let expected: std::collections::BTreeSet<String> = cases()
        .iter()
        .map(|(name, target, _)| format!("{name}-{}.json", slug(*target)))
        .collect();
    let mut orphans = Vec::new();
    for entry in std::fs::read_dir(golden_dir()).unwrap() {
        let name = entry.unwrap().file_name().into_string().unwrap();
        if name.ends_with(".json") && !expected.contains(&name) {
            orphans.push(name);
        }
    }
    assert!(
        orphans.is_empty(),
        "孤立 golden（生成ケースなし）: {orphans:?}"
    );
}

/// 全ケースの計画 JSON を golden と byte 比較する。
/// UPDATE_GOLDEN=1 のときは比較せず golden を書き換える。
#[test]
fn plans_match_golden() {
    let dir = golden_dir();
    std::fs::create_dir_all(&dir).unwrap();
    let bless = std::env::var_os("UPDATE_GOLDEN").is_some();
    let mut mismatches = Vec::new();
    for (name, target, plan) in cases() {
        let path = dir.join(format!("{name}-{}.json", slug(target)));
        if bless {
            std::fs::write(&path, &plan).unwrap();
            continue;
        }
        let expected = std::fs::read(&path).unwrap_or_else(|_| {
            panic!(
                "golden が無い: {}（UPDATE_GOLDEN=1 で生成）",
                path.display()
            )
        });
        if expected != plan.as_bytes() {
            mismatches.push(format!("{name} ({})", slug(target)));
        }
    }
    assert!(
        mismatches.is_empty(),
        "plan golden と不一致: {mismatches:?}（意図的な変更なら UPDATE_GOLDEN=1 で再生成）"
    );
}

/// golden JSON が `Deserialize` で IR へ復元できること
/// （`run-recipe` が読む wire format の健全性）を検証する。
/// `&'static str` フィールドを含むため入力を `Box::leak` で 'static 化する。
#[test]
fn plans_deserialize_roundtrip() {
    use containers_builder::package::recipe::Recipe;
    for (name, _, plan) in cases() {
        let leaked: &'static str = Box::leak(plan.clone().into_boxed_str());
        let recipe: Recipe = serde_json::from_str(leaked)
            .unwrap_or_else(|e| panic!("{name} のデシリアライズに失敗: {e}"));
        let value = serde_json::to_value(&recipe).unwrap();
        let again = serde_json::to_string_pretty(&value).unwrap();
        assert_eq!(plan.trim_end(), again, "{name} のラウンドトリップが不一致");
    }
}

/// `run-recipe` が実際に読む `BuildPlan`（prelude+recipe）の
/// wire format 往復を prelude golden で検証する。
#[test]
fn build_plans_deserialize_roundtrip() {
    use containers_builder::package::recipe::BuildPlan;
    let dir = golden_dir().join("prelude");
    let mut count = 0;
    for entry in std::fs::read_dir(&dir).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name().into_string().unwrap();
        if !name.ends_with(".json") {
            continue;
        }
        count += 1;
        let original = std::fs::read_to_string(entry.path()).unwrap();
        let leaked: &'static str = Box::leak(original.clone().into_boxed_str());
        let plan: BuildPlan = serde_json::from_str(leaked)
            .unwrap_or_else(|e| panic!("{name} のデシリアライズに失敗: {e}"));
        let value = serde_json::to_value(&plan).unwrap();
        let again = format!("{}\n", serde_json::to_string_pretty(&value).unwrap());
        assert_eq!(
            original, again,
            "{name} の BuildPlan ラウンドトリップが不一致"
        );
    }
    assert!(count > 0, "prelude golden が 0 件（vacuous pass）");
}

/// 計画 JSON がラウンドトリップで安定であること（正準形式として
/// 2 度シリアライズして一致するか）を検証する。
#[test]
fn plans_serialize_deterministically() {
    for (name, _, plan) in cases() {
        let value: serde_json::Value =
            serde_json::from_str(&plan).unwrap_or_else(|_| panic!("{name} の JSON が壊れている"));
        let again = serde_json::to_string_pretty(&value).unwrap();
        assert_eq!(plan.trim_end(), again, "{name} の正準化が非決定的");
    }
}
