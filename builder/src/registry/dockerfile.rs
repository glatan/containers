//! Dockerfile の GENERATED ブロックの生成・書換え。
//!
//! ブロックは `# BEGIN GENERATED:containers ...` / `# END GENERATED` で
//! 囲まれ、ツールだけが中身を書き換える。ブロック外のユーザー記述は
//! そのまま保持する。

use crate::error::{Error, Result};

const BEGIN_PREFIX: &str = "# BEGIN GENERATED:containers";
pub const END_LINE: &str = "# END GENERATED";

pub fn begin_line(params: &str) -> String {
    if params.is_empty() {
        BEGIN_PREFIX.to_string()
    } else {
        format!("{BEGIN_PREFIX} {params}")
    }
}

/// `existing` の GENERATED ブロックを `block` で置き換える。
/// ブロックが無ければ最初の FROM 直前（無ければ末尾）に挿入する。
///
/// `requires_before_from` が真のとき（mode B の ARG ブロック）、
/// ブロックが最初の FROM より後に置かれていればエラーにする。
/// ARG は最初の FROM より前に宣言しないと `FROM ${ARG}` で使えないため。
pub fn splice(existing: &str, block: &str, requires_before_from: bool) -> Result<String> {
    let lines: Vec<&str> = existing.lines().collect();
    let begins: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| is_begin(line))
        .map(|(index, _)| index)
        .collect();
    let ends: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, line)| line.trim_end() == END_LINE)
        .map(|(index, _)| index)
        .collect();
    if begins.len() > 1 || ends.len() > 1 {
        return Err(Error::message(
            "GENERATED ブロックが複数あります。1つにまとめてください",
        ));
    }
    let first_from = lines.iter().position(|line| is_from(line));
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + 8);
    match (begins.first(), ends.first()) {
        (Some(&begin), Some(&end)) => {
            if end < begin {
                return Err(Error::message(
                    "GENERATED ブロックが壊れています（END が BEGIN より前です）",
                ));
            }
            if requires_before_from && first_from.is_some_and(|from| from < begin) {
                return Err(Error::message(
                    "GENERATED ブロックは最初の FROM より前に配置してください（ARG の宣言位置の制約です）",
                ));
            }
            out.extend(lines[..begin].iter().map(|line| line.to_string()));
            out.extend(block.lines().map(str::to_string));
            out.extend(lines[end + 1..].iter().map(|line| line.to_string()));
        }
        (None, None) => {
            let at = first_from.unwrap_or(lines.len());
            out.extend(lines[..at].iter().map(|line| line.to_string()));
            out.extend(block.lines().map(str::to_string));
            out.extend(lines[at..].iter().map(|line| line.to_string()));
        }
        _ => {
            return Err(Error::message(
                "GENERATED ブロックが壊れています（BEGIN/END の対応が取れていません）",
            ));
        }
    }
    let mut text = out.join("\n");
    text.push('\n');
    Ok(text)
}

fn is_begin(line: &str) -> bool {
    line.strip_prefix(BEGIN_PREFIX)
        .is_some_and(|rest| rest.is_empty() || rest.starts_with(' '))
}

fn is_from(line: &str) -> bool {
    let trimmed = line.trim_start();
    // バイト境界を避けるため get で取る（非 ASCII コメント行でパニックしない）。
    trimmed
        .get(..4)
        .is_some_and(|head| head.eq_ignore_ascii_case("FROM"))
        && trimmed
            .as_bytes()
            .get(4)
            .is_some_and(u8::is_ascii_whitespace)
}

/// mode A: pinned FROM + COPY 連結のブロックを生成する。
pub fn block_lines(
    platform: &str,
    index_digest: &crate::oci::Digest,
    stages: &[(String, String, String)], // (stage_name, reference, docker_platform)
) -> String {
    let mut lines = vec![begin_line(&format!(
        "platform={platform} index={}",
        index_digest.prefixed()
    ))];
    for (stage, reference, docker_platform) in stages {
        if docker_platform.is_empty() {
            lines.push(format!("FROM {reference} AS {stage}"));
        } else {
            lines.push(format!("FROM --platform={docker_platform} {reference} AS {stage}"));
        }
    }
    lines.push("FROM scratch AS containers-packages".to_string());
    for (stage, _, _) in stages {
        lines.push(format!("COPY --from={stage} / /"));
    }
    lines.push(END_LINE.to_string());
    lines.join("\n")
}

/// mode B: ARG pin のブロックを生成する。
pub fn arg_block(index_digest: &crate::oci::Digest, reference: &str) -> String {
    [
        begin_line(&format!("index={}", index_digest.prefixed())),
        format!("ARG CONTAINERS_BASE={reference}"),
        END_LINE.to_string(),
    ]
    .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splice_inserts_before_first_from() {
        let existing = "# syntax=docker/dockerfile:1\nFROM debian\nRUN true\n";
        let block = "# BEGIN GENERATED:containers platform=amd64/v4\nFROM x AS a\n# END GENERATED";
        let out = splice(existing, block, false).unwrap();
        assert!(out.contains("FROM x AS a\n# END GENERATED\nFROM debian"));
        assert!(out.contains("# syntax=docker/dockerfile:1"));
    }

    #[test]
    fn splice_replaces_existing_block() {
        let existing = "FROM a\n# BEGIN GENERATED:containers x\nFROM old AS old\n# END GENERATED\nRUN true\n";
        let block = "# BEGIN GENERATED:containers y\nFROM new AS new\n# END GENERATED";
        let out = splice(existing, block, false).unwrap();
        assert!(out.contains("FROM new AS new"));
        assert!(!out.contains("FROM old AS old"));
        assert!(out.contains("RUN true"));
    }

    #[test]
    fn splice_rejects_block_after_from_when_requires_before() {
        let existing = "FROM a\n# BEGIN GENERATED:containers\nARG X\n# END GENERATED\n";
        let error = splice(existing, "# BEGIN GENERATED:containers\nARG Y\n# END GENERATED", true)
            .unwrap_err();
        assert!(error.to_string().contains("FROM より前"));
    }

    #[test]
    fn splice_rejects_unpaired_markers() {
        let error = splice("FROM a\n# BEGIN GENERATED:containers\n", "x", false).unwrap_err();
        assert!(error.to_string().contains("壊れています"));
    }

    #[test]
    fn block_contains_copy_chain() {
        let digest = crate::oci::Digest::from_hex("a".repeat(64));
        let block = block_lines(
            "amd64/v4",
            &digest,
            &[(
                "pkg-zlib".to_string(),
                "ghcr.io/o/packages/zlib@sha256:ff".to_string(),
                "linux/amd64/v4".to_string(),
            )],
        );
        assert!(block.contains("FROM --platform=linux/amd64/v4 ghcr.io/o/packages/zlib@sha256:ff AS pkg-zlib"));
        assert!(block.contains("COPY --from=pkg-zlib / /"));
        assert!(block.contains("index=sha256:aa"));
    }
}
