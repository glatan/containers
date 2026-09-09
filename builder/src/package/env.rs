/// パッケージビルドに使う環境イメージ。正本はこの定数です。
///
/// 実行時はタグではなく digest で固定します。更新するときは Docker Hub の
/// `archlinux:base-devel` の tag digest を取り、`digest` だけ差し替えます。
///
/// ```text
/// curl -sS https://hub.docker.com/v2/repositories/library/archlinux/tags/base-devel
/// ```
///
/// 記録時の Hub `last_updated`: 2026-08-31T21:48:27Z
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuildImage {
    pub name: &'static str,
    pub tag: &'static str,
    pub digest: &'static str,
}

impl BuildImage {
    pub fn reference(self) -> String {
        format!("{}:{}@{}", self.name, self.tag, self.digest)
    }
}

pub const BUILD_IMAGE: BuildImage = BuildImage {
    name: "docker.io/library/archlinux",
    tag: "base-devel",
    digest: "sha256:84cd9ef000b3cff245ec028e87965b84724f4bf1cc63fc2741ba927b88515ed6",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pins_sha256_digest() {
        let digest = BUILD_IMAGE.digest;
        let hex = digest.strip_prefix("sha256:").unwrap();
        assert_eq!(hex.len(), 64);
        assert!(
            hex.bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        );
        assert_eq!(
            BUILD_IMAGE.reference(),
            format!(
                "{}:{}@{}",
                BUILD_IMAGE.name, BUILD_IMAGE.tag, BUILD_IMAGE.digest
            )
        );
        assert!(!BUILD_IMAGE.name.is_empty());
        assert!(!BUILD_IMAGE.tag.is_empty());
    }
}
