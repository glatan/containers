use std::path::PathBuf;

use crate::flags::Flag;
use crate::target::Target;
use crate::workspace;

pub(crate) mod build;
pub mod definition;
pub(crate) mod describe;
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
}

pub use build::{BuildOptions, build_bootstrap, build_package, build_packages};
pub use definition::{PACKAGES, all, find, images};
pub use fetch::fetch_package;
pub use recipe::{BuildContext, Recipe};

pub const LINUX_HEADERS: Dependency = dep("linux-headers", "6.18.56");

pub const GLIBC_VERSION: &str = "2.44";

pub const GCC_VERSION: &str = "16.2.0";

pub const BINUTILS_VERSION: &str = "2.47";

pub const fn glibc_dep(target: Target) -> Dependency {
    dep(target.glibc_package(), GLIBC_VERSION)
}

/// libstdc++ / libgcc_s / libatomic など、C++ ランタイムライブラリの提供元。
pub const fn gcc_dep(target: Target) -> Dependency {
    dep(target.gcc_package(), GCC_VERSION)
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
    /// `cargo vendor` 等のローカル生成補助ソース。fetch が
    /// `packages/sources/{name}/{file}` を検証して `subdir` へ展開し、
    /// コンテナ内では `/src/{subdir}` から参照できる。
    fn aux_source(&self) -> Option<AuxSource> {
        None
    }
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
    /// HOST (BUILD_ENV) 側で実行するツールへの依存。`dep:false` で
    /// マウントされ、ターゲット向け `-I`/`-L` には載らない。
    /// 既定実装は `cc_deps`（C ツールチェーン依存）で、生成された辺は
    /// 通常の宣言辺として `tree` にも現れる。C ツールチェーンを
    /// 使わないパッケージは `Vec::new()` を返すよう上書きする。
    /// 追加の HOST 依存がある場合も `cc_deps` の戻り値に追記する
    /// こと（cc 辺の欠落はテストで検出する）。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        cc_deps(self, target)
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

/// ローカル生成の補助ソース（`cargo vendor` 成果物等）。
/// `packages/sources/{name}/{file}` に配置し、fetch が検証して
/// `subdir` へ展開する。配布元 URL を持たないため取得は手動。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AuxSource {
    pub file: &'static str,
    pub sha3_512: &'static str,
    pub blake3: &'static str,
    pub subdir: &'static str,
}

/// 依存エッジの充足方法。`Ordered` 以外は循環を断つ切断辺で、
/// スケジューリング（ビルド順）には参加しない。いずれの辺も宣言済みの
/// 依存であり、バージョン検証・マウント解決・`tree` 表示の対象になる。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Via {
    /// 通常辺。依存先を先にビルドする。
    Ordered,
    /// 切断辺。シード相では引数の distro パッケージが担保する
    /// （空文字列はベースイメージ同梱＝pacman 追加不要を意味する）。
    /// 合成済み env 上では成果物が必須。
    Seed(&'static str),
    /// 切断辺。常に成果物が必須（distro で賄えない自己参照等）。
    Artifact,
    /// 切断辺。成果物があればマウントし、無ければ使わない（任意）。
    Optional,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dependency {
    pub name: &'static str,
    pub version: &'static str,
    /// 充足方法。既定は `Via::Ordered`（`dep()` で構築）。
    pub via: Via,
}

impl Dependency {
    /// スケジューリングに参加する通常辺か。
    pub fn ordered(&self) -> bool {
        matches!(self.via, Via::Ordered)
    }
    /// `via` を付与する糖衣（`dep("x", "1").via(Via::Seed("x"))`）。
    pub const fn via(self, via: Via) -> Dependency {
        Dependency { via, ..self }
    }
    /// マウントされる成果物ディレクトリ。
    pub fn artifact_dir(self, target: Target) -> PathBuf {
        workspace::artifact_dir(self.name, self.version, target)
    }
}

/// `Dependency` 構築の糖衣（`Via::Ordered` 辺）。
pub const fn dep(name: &'static str, version: &'static str) -> Dependency {
    Dependency {
        name,
        version,
        via: Via::Ordered,
    }
}

/// `target` がシード相か（`Bootstrap::seed` パッケージの BUILD_ENV
/// ビルド）。シード相はシードイメージ（Arch base-devel）の toolchain
/// と distro パッケージでビルドする。
pub(crate) fn is_seed<P: Package + ?Sized>(package: &P, target: Target) -> bool {
    target == Target::BUILD_ENV && package.as_bootstrap().is_some_and(|b| b.seed())
}

/// C/C++ ツールチェーン依存を生成する共有コンストラクタ。
/// `Package::host_dependencies` の既定実装から呼ばれ、生成された辺は
/// `host_dependencies` の戻り値に実在する通常の宣言辺である
/// （グラフ外の暗黙チャネルではない）。
///
/// - `{target}-gcc` / `{target}-binutils`: コード生成側 toolchain。
///   `via(Seed)` —— シード相は distro の cross パッケージ、env では
///   成果物必須。Ordered にすると全 C パッケージ→gcc で glibc→gcc→glibc
///   等の循環になるため切断辺。
/// - `{HOST}-gcc` / `{HOST}-binutils`: 非 amd64 向けビルドや非 HOST
///   triple の toolchain でビルド機側の codegen が必要な場合に追加
///   （CC_FOR_BUILD / canadian ビルド）。ベースイメージが担保。
///
/// 除外:
/// - シード相パッケージ（`is_seed`）: シードイメージの無印 cc を使う。
/// - ターゲット libc 提供者（`{triple}-glibc` / `-glibc-headers`）:
///   stage コンパイラを Ordered の build_dep で宣言済み。
pub fn cc_deps<P: Package + ?Sized>(package: &P, target: Target) -> Vec<Dependency> {
    if is_seed(package, target) {
        return Vec::new();
    }
    // ターゲット libc 提供者（{triple}-glibc / {triple}-glibc-headers）は
    // stage コンパイラを Ordered の build_dep で宣言済みのため、
    // ターゲット toolchain 辺は付けない（ホスト側辺のみ）。
    let libc_provider = package.gnu_triple().is_some() && package.target_artifact();
    // amd64 向けはベースイメージ（env / シードとも gcc を同梱）が担保する
    // ため distro 名は空。非 amd64 は distro の cross toolchain パッケージ
    // （aarch64-linux-gnu-gcc 等。自前パッケージと同名）。
    let distro = |name: &'static str| {
        if matches!(target, Target::Amd64(_)) {
            ""
        } else {
            name
        }
    };
    let mut deps = Vec::new();
    if !libc_provider {
        deps.push(
            dep(target.gcc_package(), GCC_VERSION).via(Via::Seed(distro(target.gcc_package()))),
        );
        deps.push(
            dep(target.binutils_package(), BINUTILS_VERSION)
                .via(Via::Seed(distro(target.binutils_package()))),
        );
    }
    // ビルド機側 codegen（CC_FOR_BUILD / canadian）が要るのは、非 HOST
    // triple の toolchain か、非 amd64 向けビルドのみ。通常パッケージの
    // amd64 向けビルドはターゲット toolchain = ホスト toolchain のため
    // 追加しない（同一パッケージの重複辺になる）。
    if package
        .gnu_triple()
        .is_some_and(|triple| triple != Target::HOST_GNU_TRIPLE)
        || !matches!(target, Target::Amd64(_))
    {
        // 非 HOST triple の toolchain を amd64 向けにビルドすると、
        // ターゲット toolchain 辺と同一パッケージになるため重複を除く。
        for cc in [
            dep(Target::BUILD_ENV.gcc_package(), GCC_VERSION).via(Via::Seed("")),
            dep(Target::BUILD_ENV.binutils_package(), BINUTILS_VERSION).via(Via::Seed("")),
        ] {
            if !deps.iter().any(|dep| dep.name == cc.name) {
                deps.push(cc);
            }
        }
    }
    deps
}
