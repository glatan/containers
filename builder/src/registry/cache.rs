//! 消費側の XDG キャッシュ配置。
//!
//! - `index/`: pull した index artifact
//! - `trees/{manifestDigest}`: 層を展開した rootfs ツリー
//! - `composed/`: mode B で合成した OCI layout
//!
//! blob は manifest digest でアドレスされるため世代ズレの心配がない。

use std::path::PathBuf;

use crate::oci::Digest;

/// `$XDG_CACHE_HOME/containers-builder`（既定 `~/.cache/containers-builder`）。
pub fn root() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(|| PathBuf::from(".cache"))
        .join("containers-builder")
}

/// index artifact の pull 先。`{repo-digest}` で分ける。
pub fn index_layout_dir(digest: &Digest) -> PathBuf {
    root().join("index").join(digest.encoded())
}

/// 展開済みパッケージツリー。manifest digest 単位で共有。
pub fn tree_dir(manifest_digest: &Digest) -> PathBuf {
    root().join("trees").join(manifest_digest.encoded())
}

/// mode B の合成 layout の既定出力先。
pub fn composed_dir(name: &str, tag: &str) -> PathBuf {
    root().join("composed").join(format!("{name}-{tag}"))
}
