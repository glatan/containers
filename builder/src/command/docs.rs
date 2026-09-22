//! `docs` サブコマンド。`docs/package/definition/{name}.md` の
//! 生成ブロック（`<!-- GENERATED -->` マーカーで囲まれた部分）を
//! `PACKAGES` から再生成する。マーカー外の手書き記述は保持する。
//!
//! 生成内容は「`Package` trait + 評価済み Recipe の射影」が原則。
//! 構造情報の収集は `package::describe` に集約し、このモジュールは
//! markdown への薄いプリンタに留める。

use crate::error::{Error, Result};
use crate::flags::Pgo;
use crate::package::recipe::{BuildContext, Env, PgoTemplate, Recipe, Step, Steps, Workdir};
use crate::package::{self, Dependency, Package, Via, describe};
use crate::target::Target;
use std::fmt::Write as _;
use std::path::PathBuf;

/// 生成ブロックの開始マーカー。`BEGIN` と `END` の間は常に再生成され、
/// それ以外（マーカー外の補足・フラグ表等の手書き部分）は保持する。
const BEGIN: &str = "<!-- GENERATED -->";
const END: &str = "<!-- /GENERATED -->";

fn definition_docs_dir() -> PathBuf {
    PathBuf::from("docs").join("package").join("definition")
}

/// `docs` コマンド。各定義文書の生成ブロックと索引表を `PACKAGES`
/// から再生成する。`check` では書き込まず差分のある文書を報告する。
pub fn docs(check: bool) -> Result<()> {
    let dir = definition_docs_dir();
    let mut stale: Vec<String> = Vec::new();
    for defined in package::PACKAGES {
        let path = dir.join(format!("{}.md", defined.name()));
        let existing = std::fs::read_to_string(&path).unwrap_or_default();
        let updated = splice(&existing, &package_block(*defined));
        report(&path, &existing, &updated, check, &mut stale)?;
    }
    let index = dir.join("README.md");
    let existing = std::fs::read_to_string(&index).unwrap_or_default();
    let updated = splice(&existing, &index_block());
    report(&index, &existing, &updated, check, &mut stale)?;
    // 周辺文書の名前つき生成ブロック
    splice_blocks(
        &PathBuf::from("docs/package/README.md"),
        check,
        &mut stale,
        &[("tool-flags", tool_flags_block()), ("pgo", pgo_block())],
    )?;
    splice_blocks(
        &PathBuf::from("docs/package/host-tools.md"),
        check,
        &mut stale,
        &[
            ("env", host_env_block()),
            ("packages", host_packages_block()),
        ],
    )?;
    splice_blocks(
        &PathBuf::from("docs/package/toolchain.md"),
        check,
        &mut stale,
        &[
            ("supports", toolchain_supports_block()),
            ("upstream", toolchain_upstream_block()),
            ("stages", toolchain_stages_block()),
            (
                "stage-args",
                toolchain_args_block(&[
                    "binutils",
                    "gcc-stage1",
                    "glibc-headers",
                    "gcc-stage2",
                    "glibc",
                ]),
            ),
            ("gcc-args", toolchain_args_block(&["gcc"])),
        ],
    )?;
    // PACKAGES に載らない定義文書の取りこぼしを検出する
    let mut known: Vec<&str> = package::PACKAGES
        .iter()
        .map(|defined| defined.name())
        .collect();
    known.push("README");
    let mut orphan: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(&dir).map_err(|source| Error::io(&dir, source))? {
        let entry = entry.map_err(|source| Error::io(&dir, source))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(stem) = name.strip_suffix(".md")
            && !known.contains(&stem)
        {
            orphan.push(name);
        }
    }
    for name in &orphan {
        println!("PACKAGES に無い定義文書があります: {name}");
    }
    if !stale.is_empty() {
        return Err(Error::message(format!(
            "生成ブロックが古い文書があります（`docs` で再生成してください）: {}",
            stale.join(", ")
        )));
    }
    if !orphan.is_empty() {
        return Err(Error::message(format!(
            "PACKAGES に無い定義文書があります: {}",
            orphan.join(", ")
        )));
    }
    Ok(())
}

/// 差分があれば書き込み（check 時は stale リストへ記録）する。
fn report(
    path: &PathBuf,
    existing: &str,
    updated: &str,
    check: bool,
    stale: &mut Vec<String>,
) -> Result<()> {
    if existing == updated {
        return Ok(());
    }
    if check {
        stale.push(path.display().to_string());
        return Ok(());
    }
    std::fs::write(path, updated).map_err(|source| Error::io(path, source))
}

/// マーカーが独立行として現れる位置。行頭〜行末がマーカーと一致する
/// 出現のみを認める（本文中のバッククォート付き言及と区別する）。
fn marker_pos(text: &str, marker: &str) -> Option<usize> {
    text.match_indices(marker)
        .map(|(index, _)| index)
        .find(|&index| {
            let start_ok = index == 0 || text.as_bytes()[index - 1] == b'\n';
            let after = &text[index + marker.len()..];
            start_ok && (after.is_empty() || after.starts_with('\n'))
        })
}

/// 既存文書に生成ブロックをはめ込む。マーカーがあれば置き換え、
/// なければ先頭に挿入する（既存のタイトル行は生成ブロックへ統合）。
fn splice(existing: &str, block: &str) -> String {
    if existing.is_empty() {
        return format!("{block}\n");
    }
    if let (Some(begin), Some(end)) = (marker_pos(existing, BEGIN), marker_pos(existing, END))
        && begin < end
    {
        let mut spliced = String::with_capacity(existing.len() + block.len());
        spliced.push_str(&existing[..begin]);
        spliced.push_str(block);
        spliced.push_str(&existing[end + END.len()..]);
        return spliced;
    }
    // マーカー無しの既存文書: 先頭のタイトル行は生成ブロックに置き換え、
    // 残りを手書き部分として保持する。タイトル行でない先頭行
    // （文中のマーカー言及など）は保持側に残す。
    let rest = if existing.starts_with("# ") {
        existing
            .split_once('\n')
            .map(|(_, rest)| rest)
            .unwrap_or("")
    } else {
        existing
    };
    format!("{block}\n{rest}")
}

/// パッケージ 1 件分の生成ブロック（タイトル・メタデータ・ソース・依存）。
fn package_block(defined: &'static dyn Package) -> String {
    let mut out = String::new();
    let module = defined.name().replace('-', "_");
    writeln!(out, "{BEGIN}").unwrap();
    writeln!(out, "# {}", defined.name()).unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| 項目 | 値 |").unwrap();
    writeln!(out, "|-|-|").unwrap();
    writeln!(out, "| パッケージ名 | `{}` |", defined.name()).unwrap();
    writeln!(out, "| バージョン | {} |", defined.version()).unwrap();
    writeln!(out, "| ライセンス | {} |", defined.license()).unwrap();
    writeln!(out, "| 分類 | {} |", classification(defined)).unwrap();
    writeln!(
        out,
        "| 対象 Platform | {} |",
        describe::supported_label(defined)
    )
    .unwrap();
    if !matches!(defined.pgo(), Pgo::None) {
        let kind = match defined.pgo() {
            Pgo::Profiled => "`Pgo::Profiled`",
            _ => "`Pgo::Custom`、レシピ内の個別手順",
        };
        let applied = describe::pgo_targets(defined);
        let supported = describe::supported_targets(defined);
        let scope = if applied.is_empty() {
            "現行条件では適用対象なし".to_string()
        } else if applied.len() == supported.len() {
            "全対象 Platform で適用".to_string()
        } else {
            format!(
                "{} で適用",
                describe::target_set_label(&applied, &supported)
            )
        };
        writeln!(out, "| PGO | {scope}（{kind}） |").unwrap();
    }
    if !defined.pacman().is_empty() {
        writeln!(out, "| シード用 pacman | {} |", defined.pacman().join(" ")).unwrap();
    }
    if let Some(triple) = defined.gnu_triple() {
        writeln!(out, "| コード生成 triple | `{triple}` |").unwrap();
    }
    if defined.runs_on_host() {
        writeln!(out, "| host 実行 | 依存として常に BUILD_ENV 側で実行 |").unwrap();
    }
    if defined.target_artifact() {
        writeln!(
            out,
            "| ターゲット成果物 | 依存マウント時に depender のターゲット成果物を選択 |"
        )
        .unwrap();
    }
    if !defined.uses_cc_wrapper() {
        writeln!(out, "| cc ラッパ | 付与しない |").unwrap();
    }
    writeln!(
        out,
        "| 定義 | [`builder/src/package/definition/{module}.rs`](../../../builder/src/package/definition/{module}.rs) |"
    )
    .unwrap();
    writeln!(out).unwrap();
    if !defined.compose_root() {
        let source = defined.source();
        writeln!(out, "## ソース").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| 項目 | 値 |").unwrap();
        writeln!(out, "|-|-|").unwrap();
        writeln!(out, "| URL | {} |", source.url).unwrap();
        writeln!(out, "| sha3-512 | `{}` |", source.sha3_512).unwrap();
        writeln!(out, "| blake3 | `{}` |", source.blake3).unwrap();
        writeln!(out, "| 展開ディレクトリ | `{}` |", source.subdir).unwrap();
        writeln!(out).unwrap();
    }
    writeln!(out, "## 依存").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{{triple}}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。"
    )
    .unwrap();
    writeln!(out).unwrap();
    // `flags` 由来の派生辺（FuseLdMold → mold の optional 辺）も含める
    // ため graph_deps 経由で評価する。
    let graph = crate::package::build::graph_deps(defined, Target::BUILD_ENV);
    dep_table(&mut out, "dependencies", &graph.runtime);
    dep_table(&mut out, "build-dependencies", &graph.build);
    dep_table(&mut out, "host-dependencies", &graph.host);
    if !graph.host.is_empty() {
        writeln!(
            out,
            "host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。"
        )
        .unwrap();
    }
    build_sections(&mut out, defined);
    // ブロックは END マーカーで終わる（末尾改行は保持領域側のもの）
    write!(out, "{END}").unwrap();
    out
}

/// 索引（definition/README.md）の生成ブロック。
fn index_block() -> String {
    let mut out = String::new();
    writeln!(out, "{BEGIN}").unwrap();
    writeln!(
        out,
        "| パッケージ名 | バージョン | 分類 | PGO | env | 文書 |"
    )
    .unwrap();
    writeln!(out, "|-|-|-|-|-|-|").unwrap();
    for defined in package::PACKAGES {
        let env = describe::recipe_env(*defined)
            .map(|env| format!("{env:?}"))
            .unwrap_or_else(|| "—".to_string());
        writeln!(
            out,
            "| {} | {} | {} | {:?} | {} | [{name}.md](./{name}.md) |",
            defined.name(),
            defined.version(),
            classification(*defined),
            defined.pgo(),
            env,
            name = defined.name()
        )
        .unwrap();
    }
    write!(out, "{END}").unwrap();
    out
}

/// 索引・各文書で使う分類表示。
fn classification(defined: &dyn Package) -> String {
    let mut label = if defined.compose_root() {
        "build-env（合成ルート）".to_string()
    } else if defined.build_only() {
        "ビルド専用".to_string()
    } else if defined.catalog() {
        "カタログ".to_string()
    } else {
        "パッケージ".to_string()
    };
    if let Some(bootstrap) = defined.as_bootstrap() {
        label.push_str(if bootstrap.seed() {
            "（Bootstrap・seed）"
        } else {
            "（Bootstrap）"
        });
    }
    label
}

/// `## ビルド設定`〜`## ビルド手順` の生成。`Recipe::Noop`
/// （compose_root）や `Steps` を持たないパッケージでは出さない。
fn build_sections(out: &mut String, defined: &dyn Package) {
    let ctx = BuildContext::of(defined.gnu_triple(), Target::BUILD_ENV);
    let Recipe::Steps(steps) = &defined.recipe(&ctx) else {
        return;
    };
    settings_section(out, steps);
    tool_flags_section(out, defined);
    configure_flags_section(out, defined);
    steps_section(out, defined);
}

/// `## ビルド設定` — env プリリュード・`-std`・sysroot・workdir・
/// PGO テンプレートと、builder が注入する値の注記。
fn settings_section(out: &mut String, steps: &Steps) {
    writeln!(out, "## ビルド設定").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| 項目 | 値 |").unwrap();
    writeln!(out, "|-|-|").unwrap();
    let env_label = match steps.env {
        Env::Autotools => {
            "`Env::Autotools`（configure のみ `-flto*`・LTO 系 `-Werror=*`・`-f{,no-}hardened` を `sanitize_flags` で除去。`make`/`make install` に `CFLAGS`/`CXXFLAGS`/`LDFLAGS` の変数上書きを自動付与）"
        }
        Env::Lto => {
            "`Env::Lto`（フルフラグのまま configure。configure が CFLAGS を Makefile へ焼く型）"
        }
        Env::None => "`Env::None`（前処理なし）",
    };
    writeln!(out, "| env プリリュード | {env_label} |").unwrap();
    if !matches!(steps.env, Env::None) {
        writeln!(
            out,
            "| `-std` 付与 | `-std={}`（CFLAGS 末尾） |",
            steps.std.as_str()
        )
        .unwrap();
    }
    writeln!(
        out,
        "| sysroot | {} |",
        if steps.sysroot {
            "あり（`--sysroot`/`-idirafter` は `wrap_script` が注入）"
        } else {
            "なし"
        }
    )
    .unwrap();
    let workdir_label = match steps.workdir {
        Workdir::Build => "`$BUILD`（out-of-tree）",
        Workdir::InSource => "`$BUILD`（`$SOURCE` をコピーして in-source）",
        Workdir::None => "なし",
    };
    writeln!(out, "| ビルドディレクトリ | {workdir_label} |").unwrap();
    match &steps.pgo {
        Some(PgoTemplate::Autotools(_)) => writeln!(
            out,
            "| PGO テンプレート | `autotools`（configure → instrumented make → workload → distclean → 再 configure → use make → install。ネイティブのみ） |"
        )
        .unwrap(),
        Some(PgoTemplate::Make(_)) => writeln!(
            out,
            "| PGO テンプレート | `make`（instrumented make → workload → clean → use make → install。ネイティブのみ） |"
        )
        .unwrap(),
        None => {}
    }
    writeln!(out).unwrap();
    if any_step(&steps.steps, &|s| matches!(s, Step::Configure(_))) {
        writeln!(
            out,
            "configure には `autotools_configure` が `--build=\"$HOST_TRIPLE\" --host=\"${{PACKAGE_HOST:-$TRIPLE}}\"` を自動付与します。"
        )
        .unwrap();
        writeln!(out).unwrap();
    }
    if any_step(&steps.steps, &|s| matches!(s, Step::MakeInstall { .. })) {
        writeln!(
            out,
            "`make install` には `DESTDIR=\"$OUT\"` を自動付与します。"
        )
        .unwrap();
        writeln!(out).unwrap();
    }
}

/// `## ツールチェーンフラグ` — `flags()` を `tool_flags` との差分で表示。
fn tool_flags_section(out: &mut String, defined: &dyn Package) {
    writeln!(out, "## ツールチェーンフラグ").unwrap();
    writeln!(out).unwrap();
    let diff = describe::flag_diff(defined, Target::BUILD_ENV);
    if diff.empty {
        writeln!(out, "`CFLAGS`/`LDFLAGS` は設定しません。").unwrap();
    } else if diff.same {
        writeln!(
            out,
            "`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "`tool_flags` の既定セットとの差分です（[README.md](../README.md) の既定フラグ表）。"
        )
        .unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| 対象 | 追加 | 除外 |").unwrap();
        writeln!(out, "|-|-|-|").unwrap();
        writeln!(
            out,
            "| CFLAGS | {} | {} |",
            token_cell(&diff.added_cflags),
            token_cell(&diff.removed_cflags)
        )
        .unwrap();
        writeln!(
            out,
            "| LDFLAGS | {} | {} |",
            token_cell(&diff.added_ldflags),
            token_cell(&diff.removed_ldflags)
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    writeln!(
        out,
        "`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。"
    )
    .unwrap();
    writeln!(out).unwrap();
}

/// `## ビルドフラグ` — configure 系呼出（`Step::Configure`/`Cmake`/
/// `Meson`/`ConfigureScript`）を全 supported target で評価し、呼出単位で
/// 表にする。条件列は引数が出現するターゲット集合。
/// Exec ベースの configure 呼出は表にしない（手順セクションを参照させる）。
fn configure_flags_section(out: &mut String, defined: &dyn Package) {
    let invocations = describe::configure_invocations(defined);
    if invocations.is_empty() {
        if describe::has_untyped_configure(defined) {
            writeln!(out, "## ビルドフラグ").unwrap();
            writeln!(out).unwrap();
            writeln!(
                out,
                "configure 相当の呼出は `Step::Exec` ベースのため表には未転記です。`## ビルド手順` のレンダリングを参照してください。"
            )
            .unwrap();
            writeln!(out).unwrap();
        }
        return;
    }
    let universe = describe::supported_targets(defined);
    writeln!(out, "## ビルドフラグ").unwrap();
    writeln!(out).unwrap();
    writeln!(
        out,
        "対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。"
    )
    .unwrap();
    writeln!(out).unwrap();
    let multi = invocations.len() > 1;
    for (index, inv) in invocations.iter().enumerate() {
        // 同名ラベルの呼出が複数あるときは連番で区別する
        let mut label = format!("`{}`", md(&inv.label));
        if invocations.iter().filter(|o| o.label == inv.label).count() > 1 {
            let ordinal = invocations[..=index]
                .iter()
                .filter(|o| o.label == inv.label)
                .count();
            label = format!("{label}（{ordinal}回目の呼出）");
        }
        if multi {
            writeln!(out, "### {label}").unwrap();
            writeln!(out).unwrap();
        }
        let scoped = !universe.iter().all(|t| inv.targets.contains(t));
        if scoped {
            writeln!(
                out,
                "- 対象: {}",
                describe::target_set_label(&inv.targets, &universe)
            )
            .unwrap();
        }
        for cond in &inv.conds {
            writeln!(out, "- 実行時条件: `{}`", md(cond)).unwrap();
        }
        if !inv.env.is_empty() {
            writeln!(
                out,
                "- env: {}",
                inv.env
                    .iter()
                    .map(|pair| format!("`{}`", md(pair)))
                    .collect::<Vec<_>>()
                    .join(" ")
            )
            .unwrap();
        }
        if multi && (scoped || !inv.conds.is_empty() || !inv.env.is_empty()) {
            writeln!(out).unwrap();
        }
        writeln!(out, "| フラグ | 設定値 | 条件 |").unwrap();
        writeln!(out, "|-|-|-|").unwrap();
        for (arg, targets) in &inv.args {
            let (flag, value) = match arg.split_once('=') {
                Some((flag, value)) => (flag, format!("`{}`", md(value))),
                None => (arg.as_str(), String::new()),
            };
            writeln!(
                out,
                "| `{}` | {} | {} |",
                md(flag),
                value,
                describe::target_set_label(targets, &universe)
            )
            .unwrap();
        }
        writeln!(out).unwrap();
    }
}

/// `## ビルド手順` — `rendered_script` の生成スクリプトを
/// `<details>` 折りたたみで埋め込む。代表 target は BUILD_ENV
/// （未対応なら先頭の supported target）と、生成結果が異なる
/// 最初の supported target。
fn steps_section(out: &mut String, defined: &dyn Package) {
    let variants = describe::script_variants(defined);
    if variants.is_empty() {
        return;
    }
    writeln!(out, "## ビルド手順").unwrap();
    writeln!(out).unwrap();
    if variants.len() == 1 {
        writeln!(
            out,
            "`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で差分はありません。"
        )
        .unwrap();
    } else {
        writeln!(
            out,
            "`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で {} 系統に分かれます。",
            variants.len()
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    let universe = describe::supported_targets(defined);
    for (targets, script) in &variants {
        script_block(out, defined, targets, script, &universe);
    }
}

fn script_block(
    out: &mut String,
    defined: &dyn Package,
    targets: &[Target],
    script: &str,
    universe: &[Target],
) {
    let cross = |t: Target| BuildContext::of(defined.gnu_triple(), t).cross();
    let canadian = |t: Target| BuildContext::of(defined.gnu_triple(), t).canadian();
    let mut tags = Vec::new();
    if targets.iter().all(|t| cross(*t)) {
        tags.push("クロス");
    } else if targets.iter().any(|t| cross(*t)) {
        tags.push("ネイティブ・クロス混在");
    }
    if targets.iter().all(|t| canadian(*t)) {
        tags.push("Canadian");
    } else if targets.iter().any(|t| canadian(*t)) {
        tags.push("Canadian 混在");
    }
    let label = format!(
        "{}{}",
        describe::target_set_label(targets, universe),
        if tags.is_empty() {
            String::new()
        } else {
            format!("（{}）", tags.join("・"))
        }
    );
    writeln!(out, "<details>").unwrap();
    writeln!(out, "<summary>{label}</summary>").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "```bash").unwrap();
    out.push_str(script);
    if !out.ends_with('\n') {
        writeln!(out).unwrap();
    }
    writeln!(out, "```").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "</details>").unwrap();
    writeln!(out).unwrap();
}

/// ステップ列の再帰走査（If/ForEach/Subshell/Or の内部も見る）。
fn any_step(steps: &[Step], pred: &impl Fn(&Step) -> bool) -> bool {
    steps.iter().any(|step| {
        pred(step)
            || match step {
                Step::If { then, else_, .. } => any_step(then, pred) || any_step(else_, pred),
                Step::ForEach { body, .. } | Step::Subshell(body) => any_step(body, pred),
                Step::Or { lhs, rhs } => any_step(lhs, pred) || any_step(rhs, pred),
                _ => false,
            }
    })
}

/// フラグ差分セルの表示（トークンを `<br>` 区切り）。
fn token_cell(tokens: &[String]) -> String {
    if tokens.is_empty() {
        "—".to_string()
    } else {
        tokens
            .iter()
            .map(|t| format!("`{}`", md(t)))
            .collect::<Vec<_>>()
            .join("<br>")
    }
}

/// markdown テーブルセル内の `|` をエスケープする。
fn md(text: &str) -> String {
    text.replace('|', "\\|")
}

/// `docs/package/README.md`・`host-tools.md` のように複数の
/// 名前つき生成ブロックを持つ文書へブロックをはめ込む。
/// マーカー（`<!-- GENERATED:{name} -->` … `<!-- /GENERATED:{name} -->`）
/// が無いブロックは stale 扱いにする。
fn splice_blocks(
    path: &PathBuf,
    check: bool,
    stale: &mut Vec<String>,
    blocks: &[(&str, String)],
) -> Result<()> {
    let existing = std::fs::read_to_string(path).unwrap_or_default();
    let mut updated = existing.clone();
    let mut applied = Vec::new();
    for (name, block) in blocks {
        match splice_named(&updated, name, block) {
            Some(next) => {
                updated = next;
                applied.push(*name);
            }
            None => {
                println!(
                    "{} に GENERATED:{name} マーカーがありません",
                    path.display()
                );
                stale.push(format!("{}:{name}", path.display()));
            }
        }
    }
    let _ = applied;
    report(path, &existing, &updated, check, stale)
}

/// 名前つきマーカーで囲まれた領域を `block`（マーカー行を含む）で
/// 置き換える。マーカーが無い・逆順なら `None`。
fn splice_named(existing: &str, name: &str, block: &str) -> Option<String> {
    let begin_marker = format!("<!-- GENERATED:{name} -->");
    let end_marker = format!("<!-- /GENERATED:{name} -->");
    let begin = marker_pos(existing, &begin_marker)?;
    let end = marker_pos(existing, &end_marker)?;
    if begin >= end {
        return None;
    }
    let mut spliced = String::with_capacity(existing.len() + block.len());
    spliced.push_str(&existing[..begin]);
    spliced.push_str(block);
    spliced.push_str(&existing[end + end_marker.len()..]);
    Some(spliced)
}

/// `docs/package/README.md` の `GENERATED:tool-flags` ブロック。
/// `tool_cflags` / `tool_ldflags` を `Target::all()` で評価し、
/// トークン × 出現対象の表を生成する。
fn tool_flags_block() -> String {
    use crate::package::describe;
    let mut out = String::new();
    writeln!(out, "<!-- GENERATED:tool-flags -->").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| トークン | 展開 | 種別 | 対象 |").unwrap();
    writeln!(out, "|-|-|-|-|").unwrap();
    for (name, expansion, kind, targets) in describe::tool_cflag_rows() {
        writeln!(out, "| `{name}` | {expansion} | {kind} | {targets} |").unwrap();
    }
    writeln!(out).unwrap();
    writeln!(out, "| トークン（Ldflag） | 展開 | 対象 |").unwrap();
    writeln!(out, "|-|-|-|").unwrap();
    for (name, expansion, targets) in describe::tool_ldflag_rows() {
        writeln!(out, "| `{name}` | {expansion} | {targets} |").unwrap();
    }
    let (cflags, ldflags) = describe::opt_in_flags();
    writeln!(out).unwrap();
    writeln!(out, "既定セットに含まれない opt-in トークン:").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| トークン | 変数 | 展開 |").unwrap();
    writeln!(out, "|-|-|-|").unwrap();
    for flag in cflags {
        let expansion = Vec::<String>::from(flag)
            .iter()
            .map(|f| format!("`{f}`"))
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(out, "| `{}` | CFLAGS | {expansion} |", flag.name()).unwrap();
    }
    for flag in ldflags {
        let expansion = Vec::<String>::from(flag)
            .iter()
            .map(|f| format!("`{f}`"))
            .collect::<Vec<_>>()
            .join(" ");
        writeln!(out, "| `{}` | LDFLAGS | {expansion} |", flag.name()).unwrap();
    }
    writeln!(out).unwrap();
    write!(out, "<!-- /GENERATED:tool-flags -->").unwrap();
    out
}

/// `docs/package/README.md` の `GENERATED:pgo` ブロック。
/// `Package::pgo()` で全パッケージを分類する。
fn pgo_block() -> String {
    use crate::package::describe;
    let mut out = String::new();
    writeln!(out, "<!-- GENERATED:pgo -->").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| `Pgo` | パッケージ |").unwrap();
    writeln!(out, "|-|-|").unwrap();
    // 表示順を固定（Profiled A → Profiled B → Custom → None）
    let order = [
        "Profiled（A（autotools））",
        "Profiled（B（in-source make））",
        "Profiled（テンプレートなし）",
        "Custom",
        "None",
    ];
    for label in order {
        let members: Vec<&dyn Package> = package::PACKAGES
            .iter()
            .copied()
            .filter(|p| describe::pgo_label(*p) == label)
            .collect();
        if members.is_empty() {
            continue;
        }
        writeln!(
            out,
            "| `{label}` | {} |",
            describe::package_list_label(&members)
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    write!(out, "<!-- /GENERATED:pgo -->").unwrap();
    out
}

/// `docs/package/host-tools.md` の `GENERATED:env` ブロック。
/// 全パッケージを `Steps::env`（`BUILD_ENV` 評価）で分類する。
fn host_env_block() -> String {
    use crate::package::describe;
    use crate::package::recipe::Env;
    let mut out = String::new();
    writeln!(out, "<!-- GENERATED:env -->").unwrap();
    writeln!(out).unwrap();
    for (label, pred) in [
        (
            "`Env::Autotools`",
            Box::new(|e: Option<Env>| e == Some(Env::Autotools))
                as Box<dyn Fn(Option<Env>) -> bool>,
        ),
        ("`Env::Lto`", Box::new(|e: Option<Env>| e == Some(Env::Lto))),
        (
            "`Env::None`",
            Box::new(|e: Option<Env>| e == Some(Env::None)),
        ),
        ("`Recipe::Noop`", Box::new(|e: Option<Env>| e.is_none())),
    ] {
        let members: Vec<&dyn Package> = package::PACKAGES
            .iter()
            .copied()
            .filter(|p| pred(describe::recipe_env(*p)))
            .collect();
        writeln!(out, "- {label}: {}", describe::package_list_label(&members)).unwrap();
    }
    writeln!(out).unwrap();
    write!(out, "<!-- /GENERATED:env -->").unwrap();
    out
}

/// `docs/package/host-tools.md` の `GENERATED:packages` ブロック。
/// Host tools 集合の索引表（個別設定は各定義文書の生成表を参照）。
fn host_packages_block() -> String {
    use crate::package::describe;
    let mut out = String::new();
    writeln!(out, "<!-- GENERATED:packages -->").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| パッケージ | バージョン | ライセンス | PGO | env |").unwrap();
    writeln!(out, "|-|-|-|-|-|").unwrap();
    for defined in describe::host_tools() {
        let env = describe::recipe_env(defined)
            .map(|env| format!("{env:?}"))
            .unwrap_or_else(|| "—".to_string());
        writeln!(
            out,
            "| [{name}](./definition/{name}.md) | {} | {} | {:?} | {} |",
            defined.version(),
            defined.license(),
            defined.pgo(),
            env,
            name = defined.name()
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    write!(out, "<!-- /GENERATED:packages -->").unwrap();
    out
}

/// `docs/package/toolchain.md` の `GENERATED:supports` ブロック。
/// ステージ種別ごとの `supports` 評価結果。
fn toolchain_supports_block() -> String {
    use crate::package::describe;
    let mut out = String::new();
    writeln!(out, "<!-- GENERATED:supports -->").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| パッケージ | `supports` |").unwrap();
    writeln!(out, "|-|-|").unwrap();
    for suffix in describe::TOOLCHAIN_STAGES {
        writeln!(
            out,
            "| `{{triple}}-{suffix}` | {} |",
            describe::toolchain_support_label(suffix)
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    write!(out, "<!-- /GENERATED:supports -->").unwrap();
    out
}

/// `docs/package/toolchain.md` の `GENERATED:upstream` ブロック。
/// 上流ソース（URL・バージョン・ハッシュ）は定義の `Source` が正本。
fn toolchain_upstream_block() -> String {
    use crate::package::describe;
    let mut out = String::new();
    writeln!(out, "<!-- GENERATED:upstream -->").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| 上流 | バージョン | URL | 展開ディレクトリ |").unwrap();
    writeln!(out, "|-|-|-|-|").unwrap();
    let rows = describe::upstream_rows();
    for (label, s) in &rows {
        writeln!(
            out,
            "| {label} | {} | {} | `{}` |",
            s.version, s.url, s.subdir
        )
        .unwrap();
    }
    writeln!(out).unwrap();
    for (label, s) in &rows {
        writeln!(out, "{label} sha3-512: `{}`  ", s.sha3).unwrap();
        writeln!(out, "{label} blake3: `{}`", s.blake3).unwrap();
        writeln!(out).unwrap();
    }
    write!(out, "<!-- /GENERATED:upstream -->").unwrap();
    out
}

/// `docs/package/toolchain.md` の `GENERATED:stages` ブロック。
fn toolchain_stages_block() -> String {
    use crate::package::describe;
    let mut out = String::new();
    writeln!(out, "<!-- GENERATED:stages -->").unwrap();
    writeln!(out).unwrap();
    writeln!(out, "| パッケージ | バージョン | パッケージイメージ化 |").unwrap();
    writeln!(out, "|-|-|-|").unwrap();
    for suffix in describe::TOOLCHAIN_STAGES {
        if let Some((name, version, image)) = describe::toolchain_stage_row(suffix) {
            writeln!(out, "| {name} | {version} | {image} |").unwrap();
        }
    }
    writeln!(out).unwrap();
    write!(out, "<!-- /GENERATED:stages -->").unwrap();
    out
}

/// `docs/package/toolchain.md` の `GENERATED:stage-args` /
/// `GENERATED:gcc-args` ブロック。`{triple}-{suffix}` の主 configure
/// 呼出引数を triple 集合でグルーピングして出す（各パッケージ文書の
/// 生成 `## ビルドフラグ` の統合ビュー）。
fn toolchain_args_block(suffixes: &[&str]) -> String {
    use crate::package::describe;
    let mut out = String::new();
    let name = if suffixes == &["gcc"] {
        "gcc-args"
    } else {
        "stage-args"
    };
    writeln!(out, "<!-- GENERATED:{name} -->").unwrap();
    for suffix in suffixes {
        writeln!(out).unwrap();
        writeln!(out, "### `{{triple}}-{suffix}`").unwrap();
        writeln!(out).unwrap();
        writeln!(out, "| 対象 | configure 引数 |").unwrap();
        writeln!(out, "|-|-|").unwrap();
        for (args, label) in describe::toolchain_configure_groups(suffix) {
            let cell = args
                .iter()
                .map(|a| format!("`{a}`"))
                .collect::<Vec<_>>()
                .join(" ");
            writeln!(out, "| {label} | {cell} |").unwrap();
        }
    }
    writeln!(out).unwrap();
    write!(out, "<!-- /GENERATED:{name} -->").unwrap();
    out
}

fn dep_table(out: &mut String, title: &str, dependencies: &[Dependency]) {
    writeln!(out, "### {title}").unwrap();
    writeln!(out).unwrap();
    if dependencies.is_empty() {
        writeln!(out, "なし。").unwrap();
        writeln!(out).unwrap();
        return;
    }
    writeln!(out, "| パッケージ | バージョン | via |").unwrap();
    writeln!(out, "|-|-|-|").unwrap();
    for dependency in dependencies {
        writeln!(
            out,
            "| [{name}](./{name}.md) | {} | {} |",
            dependency.version,
            via_label(dependency.via),
            name = dependency.name
        )
        .unwrap();
    }
    writeln!(out).unwrap();
}

fn via_label(via: Via) -> String {
    match via {
        Via::Ordered => "—".to_string(),
        Via::Seed(distro) if distro.is_empty() => "seed".to_string(),
        Via::Seed(distro) => format!("seed:{distro}"),
        Via::Artifact => "artifact".to_string(),
        Via::Optional => "optional".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLOCK: &str = "<!-- GENERATED -->\nnew\n<!-- /GENERATED -->";

    #[test]
    fn splice_replaces_block_and_keeps_surroundings() {
        let doc = "# 導入\n\n<!-- GENERATED -->\nold\n<!-- /GENERATED -->\n\n手書き\n";
        let out = splice(doc, BLOCK);
        assert_eq!(out, "# 導入\n\nBLOCK\n\n手書き\n".replace("BLOCK", BLOCK));
    }

    #[test]
    fn splice_is_idempotent() {
        let doc = "# 導入\n\n<!-- GENERATED -->\n\n手書き\n";
        let once = splice(doc, BLOCK);
        assert_eq!(splice(&once, BLOCK), once);
    }

    #[test]
    fn splice_ignores_inline_marker_mentions() {
        // 本文中でマーカーをバッククォート付きで言及していても
        // 独立行でなければ置換対象にしない。
        let doc = "ブロックは `<!-- GENERATED -->` で囲む。\n\n手書き\n";
        let out = splice(doc, BLOCK);
        assert!(out.contains("ブロックは `<!-- GENERATED -->` で囲む。"));
        assert!(out.contains(BLOCK));
        assert!(out.ends_with("手書き\n"));
    }

    #[test]
    fn splice_treats_one_sided_markers_as_no_marker() {
        // BEGIN のみ・END のみ・逆順は「マーカー無し」扱いにする。
        for doc in [
            "タイトル\n\n<!-- GENERATED -->\n本文\n",
            "タイトル\n\n<!-- /GENERATED -->\n本文\n",
            "タイトル\n\n<!-- /GENERATED -->\n本文\n<!-- GENERATED -->\n",
        ] {
            let out = splice(doc, BLOCK);
            assert!(out.starts_with(&format!("{BLOCK}\n")));
            assert!(out.contains("本文"));
        }
    }

    #[test]
    fn splice_reversed_markers_fall_back() {
        let doc = "タイトル\n\n<!-- /GENERATED -->\n本文\n<!-- GENERATED -->\n";
        let out = splice(doc, BLOCK);
        // 逆順マーカーは置換せず先頭挿動作に倒れる。
        assert!(out.starts_with(&format!("{BLOCK}\n")));
    }
}
