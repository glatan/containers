use std::path::PathBuf;

use crate::flags::Flag;
use crate::target::Target;
use crate::workspace;

pub(crate) mod build;
pub mod definition;
pub(crate) mod env;
pub(crate) mod fetch;
pub mod recipe;
pub mod runner;
pub mod sandbox;
pub(crate) mod validate;

/// `Bootstrap` を実装したパッケージは bootstrap パッケージであり、
/// シード（Arch base-devel）イメージ上でもビルドできる。
/// 判定は `Package::as_bootstrap()` で行い、パッケージ名の別リストは持たない。
pub trait Bootstrap: Package {
    /// シード（Arch）上のコンパイラでビルドする中間ステージ。
    /// `false` のパッケージは自前の gcc / glibc を使い、予定 Platform 向けにビルドする。
    fn seed(&self) -> bool {
        false
    }
    /// 依存グラフの上書き。`Some` を返すと `Package` の依存宣言は
    /// 全ターゲットでこのペアに置き換わる。`{triple}-binutils` が
    /// glibc との循環を避けて実行時依存を空にするために使う。
    fn seed_dependencies(&self, _target: Target) -> Option<(Vec<Dependency>, Vec<Dependency>)> {
        None
    }
}

pub use build::{BuildOptions, build_bootstrap, build_package, build_packages};
pub use definition::{PACKAGES, all, find, images};
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
    /// ビルド手順。`ctx` で target・クロス判定を静的に解決する。
    fn recipe(&self, ctx: &recipe::BuildContext) -> recipe::Recipe;
    /// `target` 向けにレンダリングしたシェルスクリプト。
    fn rendered_script(&self, target: Target) -> String {
        let ctx = recipe::BuildContext::of(self.gnu_triple(), target);
        let recipe = self.recipe(&ctx);
        let errors = recipe.validate();
        assert!(
            errors.is_empty(),
            "{} の recipe が不変条件違反: {errors:?}",
            self.name()
        );
        recipe.render(&ctx)
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
    /// `Bootstrap` を実装している場合 `Some(self)`。
    /// bootstrap パッケージの判定はこのメソッドのみで行う。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        None
    }
    /// シード（Arch base-devel）イメージ上でビルドするときに
    /// pacman -Sy する追加のディストリパッケージ。
    fn pacman(&self) -> &'static [&'static str] {
        &[]
    }
    /// 公開対象のカタログパッケージ（query の表示分類）。
    fn catalog(&self) -> bool {
        false
    }
    /// 合成ルートの疑似パッケージ。fetch/build/pack/イメージレイヤの
    /// 対象外で、依存グラフのエントリポイントとしてのみ使う。
    fn compose_root(&self) -> bool {
        false
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
    /// コード生成 triple。`{triple}-` プレフィックスのツールチェーン
    /// 構成パッケージは自身の triple を返す。
    fn gnu_triple(&self) -> Option<&'static str> {
        None
    }
    /// `wrap_script` が cc ラッパ（`--sysroot`/`-idirafter` 注入）を
    /// 付与するか。ラッパが参照する gcc/glibc 自身は `false`。
    fn uses_cc_wrapper(&self) -> bool {
        true
    }
    /// 成果物がターゲットスコープか（ターゲット向けファイルを含む
    /// sysroot コンテンツ。glibc / glibc-headers）。依存として
    /// マウントされるとき BUILD_ENV ではなく depender のターゲット
    /// 成果物が選ばれる。
    fn target_artifact(&self) -> bool {
        false
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
