//! 型付きビルド手順。生シェル `build_script()` の段階的な置き換え。
//!
//! `Recipe::Steps` はプリリュード（env/workdir）と型付きステップ列から
//! シェルスクリプトをレンダリングする。`info` 等でビルド設定を
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
    /// ネイティブビルド時の PGO 展開。クロスでは通常ステップへフォールバック。
    pub pgo: Option<PgoTemplate>,
    pub steps: Vec<Step>,
}

impl Steps {
    pub const fn new(env: Env) -> Self {
        Self {
            env,
            std: Std::Gnu17,
            sysroot: true,
            workdir: Workdir::Build,
            pgo: None,
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

    /// ネイティブのみ PGO フェーズへ展開するテンプレートを宣言する。
    pub const fn pgo(mut self, pgo: PgoTemplate) -> Self {
        self.pgo = Some(pgo);
        self
    }

    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }
}

/// env プリリュード。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Env {
    /// `-std` 付与 + クロス時の gnulib cache 変数 + `sanitize_flags` で
    /// configure のみ LTO/LTO系`-Werror=*`/`-f{,no-}hardened` を剥がす
    /// `autotools_configure`（config.log tail 付き）。`Step::Make` 系には
    /// `CFLAGS`/`CXXFLAGS`/`LDFLAGS` の var override を自動付与する。
    Autotools,
    /// フルフラグのままの `autotools_configure`（configure が CFLAGS を
    /// Makefile に焼き直す gmp/bash/ncurses 型。var override は付けない）。
    Lto,
    /// `set -euxo pipefail` のみ。
    None,
}

/// PGO テンプレート。workload は `( … ) || true` で囲んで emit する
/// コンテナ内シェル片。
#[derive(Clone, Copy, Debug)]
pub enum PgoTemplate {
    /// configure → instrumented make → workload → `make distclean` →
    /// reconfigure → profile-use make → install。
    /// `Env::Autotools` は make 変数上書きで注入し、`Env::Lto` は
    /// configure env 経由で注入する（gmp 型）。
    Autotools(&'static str),
    /// instrumented make → workload → `make clean` → profile-use make →
    /// install（configure を持たない in-source make 系。zstd/bzip2）。
    Make(&'static str),
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

    /// 定型 autotools + PGO テンプレート（ネイティブのみ展開）。
    /// `workload` は `( … ) || true` で囲んで emit されるシェル片。
    pub fn autotools_pgo(args: &[&'static str], workload: &'static str) -> Self {
        Self::autotools_pgo_env(Env::Autotools, args, workload)
    }

    /// env を選べる定型 autotools + PGO テンプレート。
    pub fn autotools_pgo_env(env: Env, arguments: &[&'static str], workload: &'static str) -> Self {
        Self::autotools_pgo_args(env, args(arguments), workload)
    }

    /// 実行時 configure 引数 + PGO テンプレート（gmp の ABI 等）。
    pub fn autotools_pgo_args(env: Env, args: Vec<String>, workload: &'static str) -> Self {
        Self::Steps(
            Steps::new(env)
                .pgo(PgoTemplate::Autotools(workload))
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

/// configure probe 用の env 剥がし（`Flag::sanitized` と同じ集合）。
/// `-flto*`・LTO 系 `-Werror=*`・`-f{,no-}hardened` を落とす。
/// `-Werror=format-security` は残す（Fedora と同じ）。
const SANITIZE_FLAGS: &str = r#"sanitize_flags() {
  printf '%s' "$1" | sed -E 's/-flto(=full|=auto|=thin)?//g; s/-Werror=(odr|lto-type-mismatch|strict-aliasing)//g; s/-f(no-)?hardened//g'
}
"#;

/// instrumented ビルドを実行した成果物を workload が実行できるよう
/// `$PWD` と `*/.libs` を `LD_LIBRARY_PATH` へ載せる。
const PGO_LD_LIBRARY_PATH: &str = r#"export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
"#;

/// `-fprofile-generate` への env 切り替え（ベース値を `_PGO_*` へ退避）。
const PGO_GENERATE: &str = concat!(
    "export _PGO_CFLAGS=\"${CFLAGS:-}\" _PGO_CXXFLAGS=\"${CXXFLAGS:-}\" _PGO_LDFLAGS=\"${LDFLAGS:-}\"\n",
    "export CFLAGS=\"$_PGO_CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic\"\n",
    "export CXXFLAGS=\"$_PGO_CXXFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic\"\n",
    "export LDFLAGS=\"$_PGO_LDFLAGS -fprofile-generate\"\n",
);

/// `-fprofile-use` への env 切り替え。
const PGO_USE: &str = concat!(
    "export CFLAGS=\"$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch\"\n",
    "export CXXFLAGS=\"$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch\"\n",
    "export LDFLAGS=\"$_PGO_LDFLAGS -fprofile-use\"\n",
);

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
                out.push_str(SANITIZE_FLAGS);
                out.push_str(
                    r#"autotools_configure() {
  if ! CFLAGS="$(sanitize_flags "${CFLAGS:-}")" \
       CXXFLAGS="$(sanitize_flags "${CXXFLAGS:-}")" \
       LDFLAGS="$(sanitize_flags "${LDFLAGS:-}")" \
       "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"; then
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
        match (self.pgo, ctx.cross()) {
            (Some(PgoTemplate::Autotools(workload)), false) => {
                out.push_str(PGODIR_PRELUDE);
                self.render_pgo_autotools(&mut out, workload);
            }
            (Some(PgoTemplate::Make(workload)), false) => {
                out.push_str(PGODIR_PRELUDE);
                self.render_pgo_make(&mut out, workload);
            }
            _ => self.render_steps(&mut out),
        }
        out
    }

    fn render_steps(&self, out: &mut String) {
        for step in &self.steps {
            step.render(out, self.env);
        }
    }

    /// テンプレート A: configure → instrumented make → workload →
    /// distclean → reconfigure → use make → install。
    /// `Env::Lto` は configure が env を Makefile へ焼くため profile フラグを
    /// configure 前に export する。`Env::Autotools` は make 変数上書きが
    /// env を拾うため configure 後に export する。
    fn render_pgo_autotools(&self, out: &mut String, workload: &str) {
        let Some(configure) = self
            .steps
            .iter()
            .position(|step| matches!(step, Step::Configure(_)))
        else {
            return self.render_steps(out);
        };
        let first_make = self
            .steps
            .iter()
            .position(|step| matches!(step, Step::Make { .. }))
            .unwrap_or(self.steps.len());
        let install = self
            .steps
            .iter()
            .position(|step| matches!(step, Step::MakeInstall { .. }))
            .unwrap_or(self.steps.len());
        // Configure→Make→MakeInstall の順序が崩れた手組み Steps では
        // スライスが逆転して panic するため、PGO なしへフォールバックする。
        if configure + 1 > first_make || first_make > install {
            return self.render_steps(out);
        }
        for step in &self.steps[..configure] {
            step.render(out, self.env);
        }
        if matches!(self.env, Env::Lto) {
            out.push_str(PGO_GENERATE);
        }
        self.steps[configure].render(out, self.env);
        for step in &self.steps[configure + 1..first_make] {
            step.render(out, self.env);
        }
        if matches!(self.env, Env::Autotools) {
            out.push_str(PGO_GENERATE);
        }
        for step in &self.steps[first_make..install] {
            step.render(out, self.env);
        }
        out.push_str(PGO_LD_LIBRARY_PATH);
        render_workload(out, workload);
        out.push_str("make distclean || make clean\n");
        out.push_str(PGO_USE);
        self.steps[configure].render(out, self.env);
        for step in &self.steps[configure + 1..first_make] {
            step.render(out, self.env);
        }
        for step in &self.steps[first_make..install] {
            step.render(out, self.env);
        }
        for step in &self.steps[install..] {
            step.render(out, self.env);
        }
    }

    /// テンプレート B: instrumented make → workload → clean →
    /// use make → install（configure 無しの in-source make 系）。
    fn render_pgo_make(&self, out: &mut String, workload: &str) {
        let install = self
            .steps
            .iter()
            .position(|step| matches!(step, Step::MakeInstall { .. }))
            .unwrap_or(self.steps.len());
        out.push_str(PGO_GENERATE);
        for step in &self.steps[..install] {
            step.render(out, self.env);
        }
        out.push_str(PGO_LD_LIBRARY_PATH);
        render_workload(out, workload);
        out.push_str("make clean || make distclean\n");
        out.push_str(PGO_USE);
        for step in &self.steps[..install] {
            step.render(out, self.env);
        }
        for step in &self.steps[install..] {
            step.render(out, self.env);
        }
    }
}

/// `PGODIR` は絶対パス必須（相対だと実行時 cwd に依存する）。
const PGODIR_PRELUDE: &str = "export PGODIR=/tmp/pgo\nrm -rf \"$PGODIR\"\nmkdir -p \"$PGODIR\"\n";

/// workload シェル片を `( … ) || true` で囲む。1 テストの失敗で
/// プロファイル生成全体を捨てないため。
fn render_workload(out: &mut String, workload: &str) {
    out.push_str("(\n");
    out.push_str(workload);
    if !workload.ends_with('\n') {
        out.push('\n');
    }
    out.push_str(") || true\n");
}

impl Step {
    fn render(&self, out: &mut String, env: Env) {
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
                if matches!(env, Env::Autotools) {
                    render_flag_vars(out, args);
                }
                out.push('\n');
            }
            Step::MakeInstall { vars } => {
                out.push_str("make install");
                for var in vars {
                    let _ = write!(out, " {var}");
                }
                if matches!(env, Env::Autotools) {
                    render_flag_vars(out, vars);
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

/// autotools が `@CFLAGS@` を Makefile へ焼き込むのを打ち消す変数上書き。
/// ステップが既に同名変数を明示している場合（bzip2 の `CFLAGS=` 等）は
/// そちらを尊重して付与しない。
fn render_flag_vars(out: &mut String, args: &[String]) {
    for (name, value) in [
        ("CFLAGS", "\"${CFLAGS:-}\""),
        ("CXXFLAGS", "\"${CXXFLAGS:-}\""),
        ("LDFLAGS", "\"${LDFLAGS:-}\""),
    ] {
        let owned = format!("{name}=");
        if !args.iter().any(|arg| arg.starts_with(&owned)) {
            let _ = write!(out, " {name}={value}");
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
        // configure のみ sanitize し、make/install は var override でフルフラグ
        assert!(script.contains("sanitize_flags"), "{script}");
        assert!(
            script.contains("CFLAGS=\"$(sanitize_flags \"${CFLAGS:-}\")\""),
            "{script}"
        );
        assert!(script.contains("-std=gnu17"), "{script}");
        // sysroot 検出は wrap_script 側（マウント情報が必要なため）
        assert!(!script.contains("--sysroot=$SYSROOT"), "{script}");
        assert!(
            script.contains("autotools_configure --prefix=/usr --disable-acl"),
            "{script}"
        );
        assert!(
            script.contains(
                "make -j\"$(nproc)\" CFLAGS=\"${CFLAGS:-}\" CXXFLAGS=\"${CXXFLAGS:-}\" LDFLAGS=\"${LDFLAGS:-}\""
            ),
            "{script}"
        );
        assert!(
            script.contains("make install CFLAGS=\"${CFLAGS:-}\" CXXFLAGS=\"${CXXFLAGS:-}\" LDFLAGS=\"${LDFLAGS:-}\" DESTDIR=\"$OUT\""),
            "{script}"
        );
    }

    #[test]
    fn autotools_make_skips_explicit_flag_vars() {
        let recipe = Recipe::Steps(
            Steps::new(Env::Autotools)
                .workdir(Workdir::InSource)
                .step(Step::Make {
                    jobs: false,
                    args: args(&[
                        "CFLAGS=\"$CFLAGS -fpic -D_FILE_OFFSET_BITS=64\"",
                        "LDFLAGS=\"$LDFLAGS\"",
                    ]),
                })
                .step(Step::make_install(&["PREFIX=/usr"])),
        );
        let script = recipe.render(&ctx());
        // 明示 CFLAGS=/LDFLAGS= は残り、CXXFLAGS のみ自動付与される
        assert!(
            script.contains(
                "make CFLAGS=\"$CFLAGS -fpic -D_FILE_OFFSET_BITS=64\" LDFLAGS=\"$LDFLAGS\" CXXFLAGS=\"${CXXFLAGS:-}\""
            ),
            "{script}"
        );
    }

    #[test]
    fn lto_env_keeps_lto_flags() {
        let recipe = Recipe::autotools_env(Env::Lto, &["--prefix=/usr"]);
        let script = recipe.render(&ctx());
        assert!(!script.contains("sanitize_flags"), "{script}");
        assert!(script.contains("autotools_configure"), "{script}");
    }

    #[test]
    fn autotools_pgo_native_expands_phases() {
        let recipe = Recipe::autotools_pgo(&["--prefix=/usr"], "./tool --bench");
        let script = recipe.render(&ctx());
        assert!(script.contains("export PGODIR=/tmp/pgo"), "{script}");
        assert!(script.contains("-fprofile-generate=$PGODIR"), "{script}");
        assert!(script.contains("-fprofile-use=$PGODIR"), "{script}");
        assert!(script.contains("-fprofile-correction"), "{script}");
        assert!(script.contains("make distclean || make clean"), "{script}");
        assert!(script.contains("(\n./tool --bench\n) || true"), "{script}");
        // configure → gen make → workload → distclean → reconfigure → use make
        let configure = script.find("autotools_configure --prefix=/usr").unwrap();
        let second = script[configure + 1..]
            .find("autotools_configure --prefix=/usr")
            .map(|pos| configure + 1 + pos);
        assert!(second.is_some(), "再 configure がありません: {script}");
    }

    #[test]
    fn autotools_pgo_cross_falls_back_to_plain() {
        let target = Target::Arm64(crate::target::Arm64Variant::V8);
        let ctx = BuildContext::of(None, target);
        let recipe = Recipe::autotools_pgo(&["--prefix=/usr"], "./tool --bench");
        let script = recipe.render(&ctx);
        assert!(!script.contains("-fprofile-generate"), "{script}");
        assert!(!script.contains("PGODIR"), "{script}");
        assert!(
            script.contains("autotools_configure --prefix=/usr"),
            "{script}"
        );
    }

    #[test]
    fn autotools_pgo_lto_env_exports_before_configure() {
        let recipe =
            Recipe::autotools_pgo_env(Env::Lto, &["--prefix=/usr", "ABI=64"], "make check");
        let script = recipe.render(&ctx());
        let gen_at = script.find("-fprofile-generate=$PGODIR").unwrap();
        let configure = script.find("autotools_configure --prefix").unwrap();
        // Env::Lto は configure env で注入するため export が先
        assert!(gen_at < configure, "{script}");
        // Env::Lto では make 変数上書きを付けない
        assert!(!script.contains("CFLAGS=\"$CFLAGS\" DESTDIR"), "{script}");
    }

    #[test]
    fn make_pgo_replays_build_and_install() {
        let recipe = Recipe::Steps(
            Steps::new(Env::Autotools)
                .workdir(Workdir::InSource)
                .pgo(PgoTemplate::Make("./zstd -b3 input"))
                .step(Step::make(&["PREFIX=/usr"]))
                .step(Step::make_install(&["PREFIX=/usr"])),
        );
        let script = recipe.render(&ctx());
        assert_eq!(script.matches("make -j\"$(nproc)\" PREFIX=/usr").count(), 2);
        assert_eq!(script.matches("make install PREFIX=/usr").count(), 1);
        assert!(script.contains("make clean || make distclean"), "{script}");
        assert!(
            script.contains("(\n./zstd -b3 input\n) || true"),
            "{script}"
        );
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
        assert!(script.contains("make install PREFIX=/usr"), "{script}");
        assert!(script.contains("DESTDIR=\"$OUT\""), "{script}");
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
