//! レジストリ配布・消費側の実装。
//!
//! - `index`: ローカルの `oci-layouts/` からリポジトリ index を生成する
//! - `push`: `skopeo` 経由でパッケージイメージと index を公開する
//! - `sync`: 消費側リポジトリの設定ファイルを読み、Dockerfile の
//!   GENERATED ブロックを同期する

pub(crate) mod cache;
pub(crate) mod config;
pub(crate) mod dockerfile;
pub(crate) mod index;
pub(crate) mod layout_io;
pub(crate) mod lockfile;
pub(crate) mod pull;
pub(crate) mod resolve;
pub(crate) mod skopeo;

use crate::oci::Digest;

/// 既定のレジストリ前置詞（`{registry}/packages/{name}:{version}`）。
/// `--registry` または `CONTAINERS_REGISTRY` で上書きする。
/// `ghcr.io/{org}` の org 確定まではホスト名のみを既定とする。
pub const DEFAULT_REGISTRY: &str = "ghcr.io";

/// パッケージイメージを置く名前空間。
pub const PACKAGES_NAMESPACE: &str = "packages";

/// index artifact のリポジトリ名。パッケージ名との衝突を避けるため
/// `index` は PACKAGES の予約名（テストで保証する）。
pub const INDEX_REPOSITORY: &str = "index";

/// 合成ベースイメージ（mode B）を置く名前空間。
pub const IMAGES_NAMESPACE: &str = "images";

/// 既定の index 参照（digest pin）。index は頻繁に変わるため、
/// 初回公開後に実 digest を焼き込むまでは `None` で、利用側は
/// `--index` / `CONTAINERS_INDEX` / 設定ファイルの `"index"` で指定する。
pub const INDEX_IMAGE: Option<&'static str> = None;

pub fn package_repo(registry: &str, name: &str) -> String {
    format!("{registry}/{PACKAGES_NAMESPACE}/{name}")
}

pub fn index_repo(registry: &str) -> String {
    format!("{registry}/{PACKAGES_NAMESPACE}/{INDEX_REPOSITORY}")
}

/// mode B の合成イメージのリポジトリ。
pub fn image_repo(registry: &str, name: &str) -> String {
    format!("{registry}/{IMAGES_NAMESPACE}/{name}")
}

/// `docker://{repo}:{tag}` 参照。
pub fn tag_ref(repository: &str, tag: &str) -> String {
    format!("docker://{repository}:{tag}")
}

/// `docker://{repo}@{digest}` 参照（digest pin）。
pub fn digest_ref(repository: &str, digest: &Digest) -> String {
    format!("docker://{repository}@{}", digest.prefixed())
}

/// レジストリ前置詞の解決: `--registry` > `CONTAINERS_REGISTRY` > 既定値。
pub fn registry_prefix(flag: Option<&str>) -> String {
    flag.map(str::to_string)
        .or_else(|| std::env::var("CONTAINERS_REGISTRY").ok())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_REGISTRY.to_string())
}

/// index 参照の解決: `--index` > `CONTAINERS_INDEX` > 設定 > 内蔵 pin。
/// `"auto"` は呼び出し側で `skopeo list-tags` による最新タグ解決に置き換える。
pub fn index_ref(flag: Option<&str>, configured: Option<&str>) -> Option<String> {
    flag.map(str::to_string)
        .or_else(|| std::env::var("CONTAINERS_INDEX").ok())
        .or_else(|| configured.map(str::to_string))
        .or_else(|| INDEX_IMAGE.map(str::to_string))
        .filter(|value| !value.is_empty())
}
