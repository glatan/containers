use std::path::PathBuf;

use crate::flags::Flag;
use crate::target::Target;
use crate::workspace;

pub(crate) mod build;
pub mod definition;
pub(crate) mod env;
pub(crate) mod fetch;
pub mod recipe;
pub(crate) mod validate;

/// パッケージの役割。`Role::Toolchain` / `Role::LinuxHeaders` / `Role::BuildEnv`
/// は依存グラフ解決やコンテナ内環境構築で特別扱いされる。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// 公開対象のカタログパッケージ。
    Catalog,
    /// 通常パッケージ（ホストツール・ライブラリ）。
    Package,
    /// `{triple}-` プレフィックス付きツールチェーン。
    Toolchain {
        triple: &'static str,
        stage: Stage,
    },
    LinuxHeaders,
    BuildEnv,
}

/// ツールチェーンのビルドステージ。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Binutils,
    GccStage1,
    GlibcHeaders,
    GccStage2,
    Glibc,
    Gcc,
}

pub trait Bootstrap: Package {
    /// シード（Arch base-devel）上で pacman -Sy する追加パッケージ。
    fn pacman(&self) -> &'static [&'static str] {
        &[]
    }
    /// シード（Arch）上のコンパイラでビルドする中間ステージ。
    /// `false` のパッケージは自前の gcc / glibc を使い、予定 Platform 向けにビルドする。
    fn seed(&self) -> bool {
        false
    }
    /// シード相で使う依存グラフ。`None` は `Package` の依存をそのまま使う。
    /// toolchain が glibc との循環を避けるために使う。
    fn seed_dependencies(&self, _target: Target) -> Option<(Vec<Dependency>, Vec<Dependency>)> {
        None
    }
}

pub use build::{BuildOptions, build_bootstrap, build_package, build_packages};
pub use definition::{BOOTSTRAP, PACKAGES, all, find, find_bootstrap, images};
pub use fetch::fetch_package;
pub use recipe::{BuildContext, Recipe};

pub const LINUX_HEADERS: Dependency = Dependency {
    name: "linux-headers",
    version: "6.18.33",
};

pub const GLIBC_VERSION: &str = "2.44";

pub const GCC_VERSION: &str = "16.2.0";

pub const fn glibc_dep(target: Target) -> Dependency {
    Dependency {
        name: target.glibc_package(),
        version: GLIBC_VERSION,
    }
}

/// libstdc++ / libgcc_s / libatomic など、C++ ランタイムライブラリの提供元。
pub const fn gcc_dep(target: Target) -> Dependency {
    Dependency {
        name: target.gcc_package(),
        version: GCC_VERSION,
    }
}

pub fn with_libc(target: Target, extra: &[Dependency]) -> Vec<Dependency> {
    let mut dependencies = Vec::with_capacity(extra.len() + 1);
    dependencies.push(glibc_dep(target));
    dependencies.extend_from_slice(extra);
    dependencies
}

pub fn headers_dep() -> Vec<Dependency> {
    vec![LINUX_HEADERS]
}

pub trait Package: Sync {
    fn name(&self) -> &'static str;
    fn version(&self) -> &'static str;
    fn license(&self) -> &'static str;
    fn source(&self) -> Source;
    fn dependencies(&self, target: Target) -> Vec<Dependency>;
    fn build_dependencies(&self, target: Target) -> Vec<Dependency>;
    /// 生シェルのビルド手順。`recipe()` の逃げ道として残す。
    /// 新規・移行済みパッケージは `recipe()` を実装する。
    fn build_script(&self) -> &'static str {
        ""
    }
    /// ビルド手順。`ctx` で target・クロス判定を静的に解決する。
    fn recipe(&self, ctx: &recipe::BuildContext) -> recipe::Recipe {
        let _ = ctx;
        recipe::Recipe::Shell(self.build_script().into())
    }
    /// `target` 向けにレンダリングしたシェルスクリプト。
    fn rendered_script(&self, target: Target) -> String {
        let ctx = recipe::BuildContext::of(self.gnu_triple(), target);
        self.recipe(&ctx).render(&ctx)
    }

    fn flags(&self, target: Target) -> Flag;
    /// PGO ポリシー。
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::None
    }
    fn build_only(&self) -> bool {
        false
    }
    fn supports(&self, _target: Target) -> bool {
        true
    }
    /// パッケージの役割。
    fn role(&self) -> Role {
        Role::Package
    }
    /// 依存として使われるとき常に HOST (BUILD_ENV) 側で実行されるか。
    /// `true` なら成果物ターゲットが常に BUILD_ENV になる。
    fn runs_on_host(&self) -> bool {
        false
    }
    /// 依存宣言にない追加の HOST 実行ツール。成果物があればマウントされる。
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        Vec::new()
    }
    /// コード生成 triple。toolchain パッケージは自身の triple を返す。
    fn gnu_triple(&self) -> Option<&'static str> {
        match self.role() {
            Role::Toolchain { triple, .. } => Some(triple),
            _ => None,
        }
    }
    fn source_dir(&self) -> PathBuf {
        workspace::source_dir(self.name())
    }

    fn artifact_dir(&self, target: Target) -> PathBuf {
        workspace::artifact_dir(self.name(), self.version(), target)
    }

    fn depends_on(&self, target: Target, name: &str) -> bool {
        lists_name(&self.dependencies(target), name)
    }

    fn build_depends_on(&self, target: Target, name: &str) -> bool {
        lists_name(&self.build_dependencies(target), name)
    }
}

fn lists_name(dependencies: &[Dependency], name: &str) -> bool {
    dependencies
        .iter()
        .any(|dependency| dependency.name == name)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Source {
    pub url: &'static str,
    pub sha3_512: &'static str,
    pub blake3: &'static str,
    pub subdir: &'static str,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: &'static str,
    pub version: &'static str,
}

impl Dependency {
    pub fn artifact_dir(self, target: Target) -> PathBuf {
        workspace::artifact_dir(self.name, self.version, target)
    }
}
