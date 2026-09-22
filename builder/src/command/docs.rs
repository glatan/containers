//! `docs` サブコマンド。`docs/package/definition/{name}.md` の
//! 生成ブロック（`<!-- GENERATED -->` マーカーで囲まれた部分）を
//! `PACKAGES` から再生成する。マーカー外の手書き記述は保持する。

use crate::error::{Error, Result};
use crate::package::{self, Dependency, Package, Via};
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
    // ブロックは END マーカーで終わる（末尾改行は保持領域側のもの）
    write!(out, "{END}").unwrap();
    out
}

/// 索引（definition/README.md）の生成ブロック。
fn index_block() -> String {
    let mut out = String::new();
    writeln!(out, "{BEGIN}").unwrap();
    writeln!(out, "| パッケージ名 | バージョン | 分類 | 文書 |").unwrap();
    writeln!(out, "|-|-|-|-|").unwrap();
    for defined in package::PACKAGES {
        writeln!(
            out,
            "| {} | {} | {} | [{name}.md](./{name}.md) |",
            defined.name(),
            defined.version(),
            classification(*defined),
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
