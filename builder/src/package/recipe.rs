//! 型付きビルド手順。生シェル `build_script()` の段階的な置き換え。
//!
//! `Recipe::Steps` はプリリュード（env/workdir）と型付きステップ列から
//! シェルスクリプトをレンダリングする。`package info` 等でビルド設定を
//! 構造的に出力するための土台でもある。表現しきれない手順は
//! `Recipe::Shell` / `Step::Shell` に逃がす。

use std::fmt::Write as _;

use crate::target::Target;

/// ビルド実行時のコンテキスト。`$TRIPLE`/`$PACKAGE_HOST`/クロス判定を
/// 静的に解決する。`compiler_triples` と同じ対応付け。
pub struct BuildContext {
    pub target: Target,
    /// `$TRIPLE` — コード生成 triple。toolchain は自身の triple。
    pub triple: String,
    /// `$HOST_TRIPLE`
    pub host_triple: String,
    /// `$PACKAGE_HOST` — 成果物ターゲットの triple。
    pub package_host: String,
    /// `$CROSS` — クロスツールチェーンのプリフィックス。
    pub cross_prefix: String,
}

impl BuildContext {
    /// `gnu_triple` は `Package::gnu_triple()` の戻り値
    /// （toolchain は自身の triple、それ以外は `None` → package_host）。
    pub fn of(gnu_triple: Option<&'static str>, target: Target) -> Self {
        let package_host = target.format_tuple();
        let triple = gnu_triple
            .map(str::to_string)
            .unwrap_or_else(|| package_host.clone());
        Self {
            target,
            triple,
            host_triple: Target::HOST_GNU_TRIPLE.to_string(),
            package_host,
            cross_prefix: target.cross_prefix(),
        }
    }

    /// 生成コードがホストと異なる triple を持つ（`$TRIPLE != $HOST_TRIPLE`）。
    pub fn cross(&self) -> bool {
        self.triple != self.host_triple
    }

    /// wrap_script が `/dep/{dep}` から生成するオーバーレイ prefix。
    /// 現在は `/tmp/{dep}-prefix`（gmp/mpfr の .la/.pc 書き換え先）。
    pub fn overlay(&self, dep: &str) -> String {
        format!("/tmp/{dep}-prefix")
    }
}

/// ビルド手順。
#[derive(Debug)]
pub enum Recipe {
    /// 生シェルスクリプト（`build_script()` の逃げ道。ctx 分岐のため
    /// 動的構築も可能）。
    Shell(std::borrow::Cow<'static, str>),
    /// ビルド不要（build-env）。
    Noop,
    /// 型付きステップ列。
    Steps(Steps),
}

/// C の -std 指定（`Env::Autotools` のプリリュードが付与）。
#[derive(Clone, Copy, Debug)]
pub enum Std {
    Gnu17,
    /// flex の gnulib 生成物のように K&R 宣言を含むもの。
    Gnu11,
}

impl Std {
    pub const fn as_str(self) -> &'static str {
        match self {
            Std::Gnu17 => "gnu17",
            Std::Gnu11 => "gnu11",
        }
    }
}

/// 型付きステップ列 + プリリュード。
#[derive(Debug)]
pub struct Steps {
    /// フラグ/env プリリュード。
    pub env: Env,
    /// `CFLAGS` に付与する `-std`（`Env::Autotools` のみ有効）。
    pub std: Std,
    /// `--sysroot` / `-idirafter` を付与するか。実際のパス検出は
    /// マウント情報を持つ `wrap_script` 側で行う。
    pub sysroot: bool,
    /// ビルドディレクトリの立て方。
    pub workdir: Workdir,
    pub steps: Vec<Step>,
}

impl Steps {
    pub const fn new(env: Env) -> Self {
        Self {
            env,
            std: Std::Gnu17,
            sysroot: true,
            workdir: Workdir::Build,
            steps: Vec::new(),
        }
    }

    pub const fn std(mut self, std: Std) -> Self {
        self.std = std;
        self
    }

    pub const fn sysroot(mut self, on: bool) -> Self {
        self.sysroot = on;
        self
    }

    pub const fn workdir(mut self, workdir: Workdir) -> Self {
        self.workdir = workdir;
        self
    }

    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }
}

/// env プリリュード。
#[derive(Clone, Copy, Debug)]
pub enum Env {
    /// `-std` 付与 + クロス時の gnulib cache 変数 +
    /// `autotools_configure`（config.log tail 付き）。LTO/Werror 剥がしは
    /// host 側の `Flag::sanitized` で行う。
    Autotools,
    /// LTO を保持する最小 env + `autotools_configure`。
    Lto,
    /// `set -euxo pipefail` のみ。
    None,
}

/// ビルドディレクトリの立て方。
#[derive(Clone, Copy, Debug)]
pub enum Workdir {
    /// `mkdir -p "$BUILD" && cd "$BUILD"`（out-of-tree）。
    Build,
    /// `mkdir -p "$BUILD" && cp -a "$SOURCE"/. "$BUILD"/ && cd "$BUILD"`
    /// （in-source コピー。zstd/bzip2/ninja/flex 型）。
    InSource,
    /// cd しない（`$OUT` への直接インストール、pip install 等）。
    None,
}

/// 型付きビルドステップ。
#[derive(Debug)]
pub enum Step {
    /// `autotools_configure {args}`（`$SOURCE`/configure --build=… --host=…
    /// は env プリリュードの関数が付与）。Env::Autotools / Env::Lto が必要。
    Configure(Vec<String>),
    /// `make [-j"$(nproc)"] {args}`。
    Make { jobs: bool, args: Vec<String> },
    /// `make install {vars} DESTDIR="$OUT"`。
    MakeInstall { vars: Vec<String> },
    /// `install -D -m {mode} {src} {dst}`。
    Install {
        src: &'static str,
        dst: &'static str,
        mode: &'static str,
    },
    /// `ln -s[-f] {target} {link}`。
    Symlink {
        target: &'static str,
        link: &'static str,
        force: bool,
    },
    /// 生シェル片（複雑な手順の逃げ道）。
    Shell(&'static str),
}

/// `&[&'static str]` → `Vec<String>`。`Step`/`Steps` 構築の糖衣。
pub fn args(args: &[&'static str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}

impl Step {
    pub fn configure(arguments: &[&'static str]) -> Self {
        Self::Configure(args(arguments))
    }

    /// `make -j"$(nproc)" {args}`。
    pub fn make(arguments: &[&'static str]) -> Self {
        Self::Make {
            jobs: true,
            args: args(arguments),
        }
    }

    /// `make install {vars} DESTDIR="$OUT"`。
    pub fn make_install(vars: &[&'static str]) -> Self {
        Self::MakeInstall { vars: args(vars) }
    }
}

impl Recipe {
    /// 定型 autotools: configure + `make -j` + `make install DESTDIR=$OUT`。
    pub fn autotools(args: &[&'static str]) -> Self {
        Self::autotools_env(Env::Autotools, args)
    }

    /// env プリリュードを選べる定型 autotools。
    pub fn autotools_env(env: Env, args: &[&'static str]) -> Self {
        Self::Steps(
            Steps::new(env)
                .step(Step::configure(args))
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }

    /// configure 引数に実行時値（overlay パス・ABI 等）を含む定型 autotools。
    pub fn autotools_args(args: Vec<String>) -> Self {
        Self::Steps(
            Steps::new(Env::Autotools)
                .step(Step::Configure(args))
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }

    /// レンダリングしたシェルスクリプト。
    pub fn render(&self, ctx: &BuildContext) -> String {
        match self {
            Recipe::Shell(script) => script.to_string(),
            Recipe::Noop => "true".to_owned(),
            Recipe::Steps(steps) => steps.render(ctx),
        }
    }
}

/// クロスビルド時の `CC_FOR_BUILD` 系（autotools_env.sh/lto_env.sh 共通）。
const CROSS_BUILD_CC: &str = r#"export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export BUILD_CC="$CC_FOR_BUILD"
"#;

/// クロスビルド時の gnulib/autoconf 推測値キャッシュ（autotools_env.sh 由来）。
/// Autoconf 2.72 は AC_RUN_IFELSE の cross 分岐が空だとエラーにする。
const CROSS_CACHE: &str = r#"export gl_cv_func_strcasecmp_works=yes
export gl_cv_func_strncasecmp_works=yes
export gl_cv_func_memchr_works=yes
export gl_cv_func_working_mktime=yes
export gl_cv_func_working_utimes=yes
export gl_cv_func_getcwd_null=yes
export gl_cv_func_getcwd_path_max=yes
export gl_cv_func_getcwd_abort_bug=no
export gl_cv_func_chown_follows_symlink=yes
export gl_cv_func_fchownat_nofollow_works=yes
export gl_cv_func_lstat_dereferences_slashed_symlink=yes
export gl_cv_func_stat_file_slash=yes
export gl_cv_func_strerror_r_works=yes
export gl_cv_func_strerror_0_works=yes
export gl_cv_func_strtod_works=yes
export gl_cv_func_nanosleep=yes
export gl_cv_func_printf_positions=yes
export gl_cv_func_printf_infinite=yes
export gl_cv_func_printf_enomem=yes
export gl_cv_func_malloc_0_nonnull=yes
export gl_cv_func_malloc_posix=yes
export ac_cv_func_malloc_0_nonnull=yes
export ac_cv_func_realloc_0_nonnull=yes
export ac_cv_func_memcmp_working=yes
"#;

impl Steps {
    fn render(&self, ctx: &BuildContext) -> String {
        let mut out = String::new();
        match self.env {
            Env::Autotools => {
                out.push_str("set -euxo pipefail\nexport FORCE_UNSAFE_CONFIGURE=1\n");
                let _ = writeln!(
                    out,
                    "export CFLAGS=\"${{CFLAGS:-}} -std={}\"",
                    self.std.as_str()
                );
                if ctx.cross() {
                    out.push_str(CROSS_BUILD_CC);
                    out.push_str(CROSS_CACHE);
                }
                out.push_str(
                    r#"autotools_configure() {
  if ! "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"; then
    if [ -f config.log ]; then
      echo "===== config.log (last 80 lines) ====="
      tail -n 80 config.log
    fi
    return 1
  fi
}
"#,
                );
            }
            Env::Lto => {
                out.push_str("set -euxo pipefail\nexport FORCE_UNSAFE_CONFIGURE=1\n");
                if ctx.cross() {
                    out.push_str(CROSS_BUILD_CC);
                }
                out.push_str(
                    r#"autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
"#,
                );
            }
            Env::None => out.push_str("set -euxo pipefail\n"),
        }
        match self.workdir {
            Workdir::Build => out.push_str("mkdir -p \"$BUILD\"\ncd \"$BUILD\"\n"),
            Workdir::InSource => out
                .push_str("mkdir -p \"$BUILD\"\ncp -a \"$SOURCE\"/. \"$BUILD\"/\ncd \"$BUILD\"\n"),
            Workdir::None => {}
        }
        for step in &self.steps {
            step.render(&mut out);
        }
        out
    }
}

impl Step {
    fn render(&self, out: &mut String) {
        match self {
            Step::Configure(args) => {
                out.push_str("autotools_configure");
                for arg in args {
                    let _ = write!(out, " {arg}");
                }
                out.push('\n');
            }
            Step::Make { jobs, args } => {
                out.push_str("make");
                if *jobs {
                    out.push_str(" -j\"$(nproc)\"");
                }
                for arg in args {
                    let _ = write!(out, " {arg}");
                }
                out.push('\n');
            }
            Step::MakeInstall { vars } => {
                out.push_str("make install");
                for var in vars {
                    let _ = write!(out, " {var}");
                }
                out.push_str(" DESTDIR=\"$OUT\"\n");
            }
            Step::Install { src, dst, mode } => {
                let _ = writeln!(out, "install -D -m {mode} {src} {dst}");
            }
            Step::Symlink {
                target,
                link,
                force,
            } => {
                let _ = writeln!(
                    out,
                    "ln -s{force} {target} {link}",
                    force = if *force { "f" } else { "" }
                );
            }
            Step::Shell(script) => {
                out.push_str(script);
                if !script.ends_with('\n') {
                    out.push('\n');
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> BuildContext {
        BuildContext::of(None, crate::target::Target::BUILD_ENV)
    }

    #[test]
    fn autotools_renders_prelude_configure_make_install() {
        let recipe = Recipe::autotools(&["--prefix=/usr", "--disable-acl"]);
        let script = recipe.render(&ctx());
        // LTO/Werror 剥がしは host 側の Flag::sanitized が担う
        assert!(!script.contains("sanitize_flags"), "{script}");
        assert!(script.contains("-std=gnu17"), "{script}");
        // sysroot 検出は wrap_script 側（マウント情報が必要なため）
        assert!(!script.contains("--sysroot=$SYSROOT"), "{script}");
        assert!(
            script.contains("autotools_configure --prefix=/usr --disable-acl"),
            "{script}"
        );
        assert!(script.contains("make -j\"$(nproc)\""), "{script}");
        assert!(script.contains("make install DESTDIR=\"$OUT\""), "{script}");
    }

    #[test]
    fn lto_env_keeps_lto_flags() {
        let recipe = Recipe::autotools_env(Env::Lto, &["--prefix=/usr"]);
        let script = recipe.render(&ctx());
        assert!(!script.contains("sanitize_flags"), "{script}");
        assert!(script.contains("autotools_configure"), "{script}");
    }

    #[test]
    fn in_source_copies_tree_before_build() {
        let recipe = Recipe::Steps(
            Steps::new(Env::Autotools)
                .workdir(Workdir::InSource)
                .step(Step::make(&["PREFIX=/usr"]))
                .step(Step::make_install(&["PREFIX=/usr"])),
        );
        let script = recipe.render(&ctx());
        assert!(
            script.contains("cp -a \"$SOURCE\"/. \"$BUILD\"/"),
            "{script}"
        );
        assert!(
            script.contains("make install PREFIX=/usr DESTDIR=\"$OUT\""),
            "{script}"
        );
    }

    #[test]
    fn install_symlink_and_shell_steps_render_verbatim() {
        let recipe = Recipe::Steps(
            Steps::new(Env::None)
                .sysroot(false)
                .workdir(Workdir::None)
                .step(Step::Install {
                    src: "\"$SOURCE\"/cacert.pem",
                    dst: "\"$OUT\"/etc/ssl/certs/ca.crt",
                    mode: "644",
                })
                .step(Step::Symlink {
                    target: "ca.crt",
                    link: "\"$OUT\"/etc/ssl/ca-bundle.crt",
                    force: false,
                }),
        );
        let script = recipe.render(&ctx());
        assert_eq!(
            script,
            "set -euxo pipefail\n\
             install -D -m 644 \"$SOURCE\"/cacert.pem \"$OUT\"/etc/ssl/certs/ca.crt\n\
             ln -s ca.crt \"$OUT\"/etc/ssl/ca-bundle.crt\n"
        );
    }

    #[test]
    fn build_context_resolves_cross_and_overlay() {
        let target = Target::Arm64(crate::target::Arm64Variant::V8);
        let ctx = BuildContext::of(None, target);
        assert!(ctx.cross());
        assert_eq!(ctx.triple, "aarch64-linux-gnu");
        assert_eq!(ctx.package_host, "aarch64-linux-gnu");
        assert_eq!(ctx.overlay("gmp"), "/tmp/gmp-prefix");
        let ctx = BuildContext::of(Some("aarch64-linux-gnu"), Target::BUILD_ENV);
        assert_eq!(ctx.triple, "aarch64-linux-gnu");
        assert_eq!(ctx.package_host, "x86_64-linux-gnu");
        assert!(ctx.cross());
    }
}
