//! 型付きビルド手順。
//!
//! `Recipe::Steps` はプリリュード（env/workdir）と型付きステップ列から
//! シェルスクリプトをレンダリングする。`info` 等でビルド設定を
//! 構造的に出力するための土台でもある。

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
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

    /// 成果物が動く Platform がビルドマシンと異なる
    /// （`$PACKAGE_HOST != $HOST_TRIPLE`。gcc の Canadian cross 判定）。
    pub fn canadian(&self) -> bool {
        self.package_host != self.host_triple
    }

    /// wrap_script が `/dep/{dep}` から生成するオーバーレイ prefix。
    /// 現在は `/tmp/{dep}-prefix`（gmp/mpfr の .la/.pc 書き換え先）。
    pub fn overlay(&self, dep: &str) -> String {
        format!("/tmp/{dep}-prefix")
    }
}

/// ビルド手順。
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub enum Recipe {
    /// ビルド不要（build-env）。
    Noop,
    /// 型付きステップ列。
    Steps(Steps),
}

/// C の -std 指定（`Env::Autotools` のプリリュードが付与）。
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
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
/// `wrap_script` のプリリュードと recipe 本体を束ねた計画。
/// bash backend では `prelude` を逐次 render して `recipe` の render に
/// 前置し、`runner::run_plan` は `prelude` を lax（`set -euxo` 前の
/// bash 相当: 失敗・未定義変数を許容して継続）に、`recipe` を
/// strict に実行する。
#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct BuildPlan {
    pub prelude: Vec<Step>,
    pub recipe: Recipe,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct Steps {
    /// フラグ/env プリリュード。
    pub env: Env,
    /// `CFLAGS` に付与する `-std`（`Env::Autotools`/`Env::Lto` で有効）。
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
    pub fn pgo(mut self, pgo: PgoTemplate) -> Self {
        self.pgo = Some(pgo);
        self
    }

    pub fn step(mut self, step: Step) -> Self {
        self.steps.push(step);
        self
    }

    pub fn steps(mut self, steps: Vec<Step>) -> Self {
        self.steps.extend(steps);
        self
    }
}

/// env プリリュード。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub enum PgoTemplate {
    /// configure → instrumented make → workload → `make distclean` →
    /// reconfigure → profile-use make → install。
    /// `Env::Autotools` は make 変数上書きで注入し、`Env::Lto` は
    /// configure env 経由で注入する（gmp 型）。
    Autotools(Vec<Step>),
    /// instrumented make → workload → `make clean` → profile-use make →
    /// install（configure を持たない in-source make 系。zstd/bzip2）。
    Make(Vec<Step>),
}

/// ビルドディレクトリの立て方。
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
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
#[derive(Clone, Debug, Serialize, Deserialize)]
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
        src: Cow<'static, str>,
        dst: Cow<'static, str>,
        mode: Cow<'static, str>,
    },
    /// `ln -s[-f] {target} {link}`。
    Symlink {
        target: Cow<'static, str>,
        link: Cow<'static, str>,
        force: bool,
    },
    /// `{env} {argv} [| {argv}]` — 汎用コマンド/パイプラインの実行。
    Exec {
        env: Vec<(&'static str, Arg)>,
        pipe: Pipe,
    },
    /// `cd {dir}` — 作業ディレクトリの変更（builtin のため専用 variant。
    /// `Subshell` 内では子スコープに閉じる）。
    Cd(Arg),
    /// `export NAME=value`。
    Export { name: &'static str, value: Arg },
    /// `export NAME`（値のない再 export）。
    ExportBare(&'static str),
    /// `NAME=value`（export しない代入）。
    Set { name: &'static str, value: Arg },
    /// `unset NAME`。
    Unset(&'static str),
    /// `if cond; then … [else …] fi`。
    If {
        cond: Cond,
        then: Vec<Step>,
        else_: Vec<Step>,
    },
    /// `for VAR in items; do … done`（`done > out` のリダイレクト可）。
    ForEach {
        var: &'static str,
        items: Vec<Arg>,
        body: Vec<Step>,
        stdout: Option<Redirect>,
    },
    /// `{lhs} || {rhs}`（フォールバック・`|| true` 等）。複数ステップの側は
    /// `( … )` で囲む。
    Or { lhs: Vec<Step>, rhs: Vec<Step> },
    /// `( … )` サブシェル。export/cd を子スコープに閉じ込める
    /// （perl のホスト miniperl ビルド等）。
    Subshell(Vec<Step>),
    /// `# comment`（生成スクリプト内の注記。runner では no-op）。
    Comment(Cow<'static, str>),
    /// `printf`/`cat` によるファイル生成（heredoc 代替）。
    /// `expand: true` は `<<EOF`（変数展開あり）、false は `<<'EOF'`。
    WriteFile {
        path: Arg,
        lines: Vec<Arg>,
        append: bool,
        expand: bool,
    },
}

/// `&[&'static str]` → `Vec<String>`。`Step`/`Steps` 構築の糖衣。
pub fn args(args: &[&'static str]) -> Vec<String> {
    args.iter().map(|arg| (*arg).to_owned()).collect()
}

/// コマンド引数・値のトークン。1 ワード内の結合は `Concat` で表し、
/// 引用は `Quoted` で明示する。`Lit` は変数参照を含む生テキストで、
/// 段階移行の受け皿。新規は `Var`/`Concat`/`Opt` を優先する。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Arg {
    /// ベリbatim テキスト（`$VAR` 等を含む生文字列）。
    Lit(std::borrow::Cow<'static, str>),
    /// `${NAME}` 参照。
    Var(&'static str),
    /// 連結して 1 ワードにする。
    Concat(Vec<Arg>),
    /// `--name=value`。
    Opt(&'static str, Box<Arg>),
    /// `"…"` で囲む。
    Quoted(Box<Arg>),
    /// `$( … )` コマンド置換。
    Subst(Box<Pipe>),
    /// `$(… || true)` — 失敗しても空文字で続くコマンド置換
    /// （`command -v` プローブ等）。
    SubstOr(Box<Pipe>),
}

impl Arg {
    /// `$(nproc)` のコマンド置換。
    pub fn nproc() -> Arg {
        Arg::Subst(Box::new(Pipe::cmd(vec![Arg::lit("nproc")])))
    }

    /// `-j"$(nproc)"` 相当: `prefix` + `nproc` の出力を結合する。
    pub fn nproc_jobs(prefix: &'static str) -> Arg {
        Arg::Concat(vec![Arg::lit(prefix), Self::nproc()])
    }

    pub const fn lit(text: &'static str) -> Self {
        Self::Lit(std::borrow::Cow::Borrowed(text))
    }

    pub const fn owned(text: String) -> Self {
        Self::Lit(std::borrow::Cow::Owned(text))
    }

    pub fn quoted(arg: Arg) -> Self {
        Self::Quoted(Box::new(arg))
    }

    fn render(&self) -> String {
        match self {
            Self::Lit(text) => text.to_string(),
            Self::Var(name) => format!("${{{name}}}"),
            Self::Concat(parts) => parts.iter().map(Self::render).collect(),
            Self::Opt(name, value) => format!("--{name}={}", value.render()),
            Self::Quoted(arg) => format!("\"{}\"", arg.render()),
            Self::Subst(pipe) => format!("$({})", pipe.render()),
            Self::SubstOr(pipe) => format!("$({} || true)", pipe.render()),
        }
    }
}

impl From<&'static str> for Arg {
    fn from(text: &'static str) -> Self {
        Self::lit(text)
    }
}

impl From<String> for Arg {
    fn from(text: String) -> Self {
        Self::owned(text)
    }
}

/// `&[&'static str]` → `Vec<Arg>`（全て `Lit`）。
pub fn argv(args: &[&'static str]) -> Vec<Arg> {
    args.iter().map(|arg| Arg::lit(arg)).collect()
}

/// `>`/`>>` リダイレクト先。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct Redirect {
    pub target: Arg,
    pub append: bool,
}

/// argv 列を `|` で接続したパイプライン。1 要素なら素のコマンド。
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound(deserialize = "'de: 'static"))]
pub struct Pipe {
    pub stages: Vec<Vec<Arg>>,
    /// 先頭コマンドの `< input`。
    pub stdin: Option<Arg>,
    /// 末尾の `> target` / `>> target`。
    pub stdout: Option<Redirect>,
    /// 末尾の `2> target`。
    pub stderr: Option<Arg>,
}

impl Pipe {
    /// 単一コマンドのパイプライン。
    pub fn cmd(argv: Vec<Arg>) -> Self {
        Self {
            stages: vec![argv],
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    /// 複数コマンドのパイプライン。
    pub const fn new(stages: Vec<Vec<Arg>>) -> Self {
        Self {
            stages,
            stdin: None,
            stdout: None,
            stderr: None,
        }
    }

    pub fn stdin(mut self, input: Arg) -> Self {
        self.stdin = Some(input);
        self
    }

    pub fn stdout(mut self, target: Arg) -> Self {
        self.stdout = Some(Redirect {
            target,
            append: false,
        });
        self
    }

    pub fn append_stdout(mut self, target: Arg) -> Self {
        self.stdout = Some(Redirect {
            target,
            append: true,
        });
        self
    }

    pub fn stderr(mut self, target: Arg) -> Self {
        self.stderr = Some(target);
        self
    }

    fn render(&self) -> String {
        let mut out = String::new();
        for (index, argv) in self.stages.iter().enumerate() {
            if index > 0 {
                out.push_str(" | ");
            }
            out.push_str(&argv.iter().map(Arg::render).collect::<Vec<_>>().join(" "));
            if index == 0
                && let Some(stdin) = &self.stdin
            {
                let _ = write!(out, " < {}", stdin.render());
            }
        }
        if let Some(stdout) = &self.stdout {
            let op = if stdout.append { " >> " } else { " > " };
            out.push_str(op);
            out.push_str(&stdout.target.render());
        }
        if let Some(stderr) = &self.stderr {
            out.push_str(" 2> ");
            out.push_str(&stderr.render());
        }
        out
    }
}

/// `Step::If` の条件。
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cond {
    /// `[ -d path ]`
    DirExists(Arg),
    /// `[ -f path ]`
    FileExists(Arg),
    /// `[ -e path ]`（シンボリックリンク・デバイス等も含む存在検査）。
    Exists(Arg),
    /// `[ -x path ]`
    Executable(Arg),
    /// `[ -n "value" ]`（値が空でない）。
    NonEmpty(Arg),
    /// `[ -n "$NAME" ]`
    EnvSet(&'static str),
    /// `{pipe} >/dev/null` の成功（コマンド存在・実行検査）。
    /// `pipe.stdout` を明示した場合はそちらへ流す（pacman 等）。
    Cmd(Pipe),
    /// `{a} && {b} && …` の全条件。
    And(Vec<Cond>),
    /// `! {cond}` の否定。
    Not(Box<Cond>),
}

impl Cond {
    fn render(&self) -> String {
        match self {
            Self::DirExists(path) => format!("[ -d {} ]", path.render()),
            Self::FileExists(path) => format!("[ -f {} ]", path.render()),
            Self::Exists(path) => format!("[ -e {} ]", path.render()),
            Self::Executable(path) => format!("[ -x {} ]", path.render()),
            Self::NonEmpty(value) => format!("[ -n {} ]", value.render()),
            Self::EnvSet(name) => format!("[ -n \"${{{name}:-}}\" ]"),
            Self::Cmd(pipe) => match &pipe.stdout {
                Some(_) => pipe.render(),
                None => format!("{} >/dev/null", pipe.render()),
            },
            Self::And(conds) => conds
                .iter()
                .map(Cond::render)
                .collect::<Vec<_>>()
                .join(" && "),
            // `! a && b` は `(! a) && b` にパースされるため、
            // And の否定は `! ( … )` に包む
            Self::Not(inner) => match inner.as_ref() {
                Self::And(_) => format!("! ( {} )", inner.render()),
                _ => format!("! {}", inner.render()),
            },
        }
    }
}

impl Recipe {
    /// IR の構造的不変条件を検査し、違反リストを返す。
    /// `rendered_script`/`wrap_script` が実行・レンダリング前に呼び、
    /// 違反があれば panic する（レシピの記述ミスはバグであり
    /// 実行可能なスクリプトへ変換できないため）。
    pub fn validate(&self) -> Vec<String> {
        match self {
            Self::Steps(steps) => steps.validate(),
            Self::Noop => Vec::new(),
        }
    }
}

/// `Arg`・生文字列内の `$SYSROOT` 参照を検査する。`sysroot(false)` の
/// 計画では `wrap_script` が SYSROOT を注入しないため、未ガードの参照は
/// `set -u` 下で未定義変数エラーになる。`If EnvSet("SYSROOT")` の
/// then ブランチ内、または先行する `Set`/`Export` で定義済みなら安全。
fn check_sysroot(text: &str, sysroot_defined: bool, errors: &mut Vec<String>, what: &str) {
    if !sysroot_defined && (text.contains("$SYSROOT") || text.contains("${SYSROOT}")) {
        errors.push(format!(
            "{what}: sysroot(false) の計画で $SYSROOT を参照している\
             （`If EnvSet(\"SYSROOT\")` ガードか先行する `Set`/`Export` が必要）"
        ));
    }
}

/// `'…'` スパンを除去する（シングルクォート内はシェル展開されない）。
fn strip_single_quotes(text: &str) -> String {
    let mut out = String::new();
    let mut it = text.chars();
    while let Some(c) = it.next() {
        if c == '\'' {
            for c in it.by_ref() {
                if c == '\'' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// `\X` エスケープ（エスケープされた文字そのもの）を取り除く。
/// `"\`…\`"` や `\$(` は bash が展開しないため検査対象外にする。
fn strip_escapes(text: &str) -> String {
    let mut out = String::new();
    let mut it = text.chars();
    while let Some(c) = it.next() {
        if c == '\\' {
            it.next();
        } else {
            out.push(c);
        }
    }
    out
}

/// `Arg::Lit`・生文字列ステップ引数に共通するテキスト検査。
/// `heredoc` が真なら unquoted heredoc 意味論（`'…'` はクォートに
/// ならない）で、偽ならコマンドラインワード意味論（`'…'` の内側は
/// 展開対象外）で検査する。
fn check_lit_text(text: &str, heredoc: bool, errors: &mut Vec<String>, what: &str) {
    // `'…'` の内側と `\X` エスケープはシェル展開対象外（sed 式中の
    // `$(CC)` や `"…\`…\`…"` のエスケープ済みバッククォートは
    // そのままコマンドへ渡る字面）。heredoc では `'…'` はクォート
    // にならないので除去しない。
    let unquoted = if heredoc {
        strip_escapes(text)
    } else {
        strip_escapes(&strip_single_quotes(text))
    };
    if unquoted.contains("$(") {
        errors.push(format!("{what}: `$(...)` は `Arg::Subst` を使う"));
    }
    if unquoted.contains('`') {
        errors.push(format!("{what}: バッククォートは `Arg::Subst` を使う"));
    }
    // `${VAR}` / `${VAR:-x}` / `${VAR:+x}` のみサポート。`#` `%` `/` `!`
    // `@` `^` `,` `[` 等の演算子・`${#}`・位置パラメータは runner が
    // 未対応なので拒否する。
    let mut rest = unquoted.as_str();
    while let Some(i) = rest.find("${") {
        rest = &rest[i + 2..];
        let name_len = rest
            .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .unwrap_or(rest.len());
        let name = &rest[..name_len];
        let after = &rest[name_len..];
        let bad = if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) {
            true
        } else {
            match after.chars().next() {
                Some('}') => false,
                Some(':') => !matches!(after.chars().nth(1), Some('-') | Some('+')),
                _ => true,
            }
        };
        if bad {
            errors.push(format!(
                "{what}: 未対応の `${{…}}` 形式（`${{VAR}}` / `:-` / `:+` のみサポート）"
            ));
        }
    }
    // 裸の特殊変数 `$?` `$$` `$#` `$!` `$@` `$*` `$-` `$0-9` と
    // `$'…'`/`$"…"`（ANSI-C/ロケールクォート）は runner 未対応で
    // bash とは別の意味になるため拒否する。
    for window in unquoted.as_bytes().windows(2) {
        if window[0] != b'$' {
            continue;
        }
        let c = window[1];
        if matches!(c, b'?' | b'$' | b'#' | b'!' | b'@' | b'*' | b'-')
            || c.is_ascii_digit()
            || c == b'\''
            || c == b'"'
        {
            errors.push(format!(
                "{what}: 未対応の特殊変数 `$…`（IR で表現してください）"
            ));
            break;
        }
    }
}

fn validate_arg(
    arg: &Arg,
    sysroot_defined: bool,
    errors: &mut Vec<String>,
    what: &str,
    heredoc: bool,
) {
    match arg {
        Arg::Lit(text) => {
            check_sysroot(text, sysroot_defined, errors, what);
            check_lit_text(text, heredoc, errors, what);
        }
        Arg::Var(name) => {
            if name.is_empty() {
                errors.push(format!("{what}: `Var` の変数名が空"));
            }
            if *name == "SYSROOT" {
                check_sysroot("$SYSROOT", sysroot_defined, errors, what);
            }
        }
        Arg::Concat(parts) => {
            if parts.is_empty() {
                errors.push(format!("{what}: `Arg::Concat` が空"));
            }
            for part in parts {
                validate_arg(part, sysroot_defined, errors, what, heredoc);
            }
        }
        Arg::Opt(name, value) => {
            if name.is_empty() {
                errors.push(format!("{what}: `Opt` の名前が空"));
            }
            validate_arg(value, sysroot_defined, errors, what, heredoc);
        }
        Arg::Quoted(inner) => validate_arg(inner, sysroot_defined, errors, what, heredoc),
        Arg::Subst(pipe) | Arg::SubstOr(pipe) => validate_pipe(pipe, sysroot_defined, errors, what),
    }
}

fn validate_pipe(pipe: &Pipe, sysroot_defined: bool, errors: &mut Vec<String>, what: &str) {
    if pipe.stages.is_empty() {
        errors.push(format!("{what}: `Pipe` のステージが空"));
    }
    // 外部コマンドとして存在しない純粋シェル builtin。
    // runner が spawn を試みて ENOENT=127 になるため plan 時に拒否する。
    // `command`/`sanitize_flags` は runner builtin として処理され、
    // `true`/`false`/`echo`/`printf`/`[` 等は外部バイナリが存在する。
    const PURE_BUILTINS: &[&str] = &[
        ".", ":", "break", "cd", "continue", "declare", "dirs", "eval", "exec", "exit", "export",
        "local", "popd", "pushd", "read", "readonly", "return", "set", "shift", "shopt", "source",
        "trap", "typeset", "umask", "unset", "wait",
    ];
    for stage in &pipe.stages {
        if stage.is_empty() {
            errors.push(format!("{what}: `Pipe` に空の argv ステージがある"));
        }
        if let Some(Arg::Lit(cmd)) = stage.first()
            && PURE_BUILTINS.contains(&cmd.as_ref())
        {
            errors.push(format!(
                "{what}: `{cmd}` は外部コマンドが存在しないシェル builtin（runner で実行不可）"
            ));
        }
        for arg in stage {
            validate_arg(arg, sysroot_defined, errors, what, false);
        }
    }
    if let Some(arg) = &pipe.stdin {
        validate_arg(arg, sysroot_defined, errors, what, false);
    }
    if let Some(redirect) = &pipe.stdout {
        validate_arg(&redirect.target, sysroot_defined, errors, what, false);
    }
    if let Some(arg) = &pipe.stderr {
        validate_arg(arg, sysroot_defined, errors, what, false);
    }
}

fn validate_cond(cond: &Cond, sysroot_defined: bool, errors: &mut Vec<String>) {
    match cond {
        Cond::DirExists(arg)
        | Cond::FileExists(arg)
        | Cond::Exists(arg)
        | Cond::Executable(arg)
        | Cond::NonEmpty(arg) => validate_arg(arg, sysroot_defined, errors, "Cond", false),
        Cond::EnvSet(name) => {
            if name.is_empty() {
                errors.push("`EnvSet` の変数名が空".to_string());
            }
        }
        Cond::Cmd(pipe) => validate_pipe(pipe, sysroot_defined, errors, "Cond::Cmd"),
        Cond::And(conds) => {
            if conds.is_empty() {
                errors.push("`And` の条件列が空".to_string());
            }
            for cond in conds {
                validate_cond(cond, sysroot_defined, errors);
            }
        }
        Cond::Not(inner) => validate_cond(inner, sysroot_defined, errors),
    }
}

/// `sysroot_defined`: この位置で `SYSROOT` が定義済みか。
/// `Set`/`Export` は同じシェルなので後続へ伝播するが、`Subshell`・
/// `If` ブランチ内での定義は外へ伝播しない（bash のスコープ規則に一致）。
pub(crate) fn validate_steps(
    steps: &[Step],
    env: Env,
    sysroot_defined: &mut bool,
    errors: &mut Vec<String>,
) {
    for step in steps {
        match step {
            Step::Configure(args) => {
                if !matches!(env, Env::Autotools | Env::Lto) {
                    errors.push(
                        "`Step::Configure` は Env::Autotools/Env::Lto が必要\
                         （`autotools_configure` が未定義になる）"
                            .to_string(),
                    );
                }
                for arg in args {
                    check_sysroot(arg, *sysroot_defined, errors, "Configure");
                    check_lit_text(arg, false, errors, "Configure");
                }
            }
            Step::Make { args, .. } => {
                for arg in args {
                    check_sysroot(arg, *sysroot_defined, errors, "Make");
                    check_lit_text(arg, false, errors, "Make");
                }
            }
            Step::MakeInstall { vars } => {
                for var in vars {
                    check_sysroot(var, *sysroot_defined, errors, "MakeInstall");
                    check_lit_text(var, false, errors, "MakeInstall");
                }
            }
            Step::Install { src, dst, mode } => {
                check_sysroot(src, *sysroot_defined, errors, "Install");
                check_sysroot(dst, *sysroot_defined, errors, "Install");
                check_lit_text(src, false, errors, "Install");
                check_lit_text(dst, false, errors, "Install");
                if mode.is_empty() || !mode.chars().all(|c| matches!(c, '0'..='7')) {
                    errors.push("`Install` の mode が 8 進数ではない".to_string());
                }
            }
            Step::Symlink { target, link, .. } => {
                check_sysroot(target, *sysroot_defined, errors, "Symlink");
                check_sysroot(link, *sysroot_defined, errors, "Symlink");
                check_lit_text(target, false, errors, "Symlink");
                check_lit_text(link, false, errors, "Symlink");
            }
            Step::Exec { env: vars, pipe } => {
                for (name, value) in vars {
                    if name.is_empty() {
                        errors.push("`Exec` の env 名が空".to_string());
                    }
                    validate_arg(value, *sysroot_defined, errors, "Exec", false);
                }
                validate_pipe(pipe, *sysroot_defined, errors, "Exec");
            }
            Step::Cd(dir) => {
                validate_arg(dir, *sysroot_defined, errors, "Cd", false);
            }
            Step::Export { name, value } | Step::Set { name, value } => {
                if name.is_empty() {
                    errors.push("変数名が空".to_string());
                }
                validate_arg(value, *sysroot_defined, errors, "Export/Set", false);
                if *name == "SYSROOT" {
                    *sysroot_defined = true;
                }
            }
            Step::ExportBare(name) | Step::Unset(name) => {
                if name.is_empty() {
                    errors.push("変数名が空".to_string());
                }
                // `export SYSROOT` は変数を作らない（値は未定義のまま）、
                // `unset SYSROOT` は未定義に戻す
                if *name == "SYSROOT" && matches!(step, Step::Unset(_)) {
                    *sysroot_defined = false;
                }
            }
            Step::If { cond, then, else_ } => {
                if then.is_empty() {
                    errors.push(
                        "`If` の then ブランチが空（`if x; then fi` は構文エラー）".to_string(),
                    );
                }
                validate_cond(cond, *sysroot_defined, errors);
                // `If EnvSet("SYSROOT")` の then 内は定義済みとして扱う。
                let saved = *sysroot_defined;
                if matches!(cond, Cond::EnvSet("SYSROOT")) {
                    *sysroot_defined = true;
                }
                validate_steps(then, env, sysroot_defined, errors);
                *sysroot_defined = saved;
                // else_ 内は SYSROOT 未設定側。ブランチ内の Set も外に伝播しない。
                let mut else_state = saved;
                validate_steps(else_, env, &mut else_state, errors);
            }
            Step::ForEach {
                var,
                items,
                body,
                stdout,
            } => {
                if var.is_empty() {
                    errors.push("`ForEach` の変数名が空".to_string());
                }
                if items.is_empty() {
                    errors.push("`ForEach` の items が空".to_string());
                }
                if body.is_empty() {
                    errors.push("`ForEach` の body が空".to_string());
                }
                for item in items {
                    validate_arg(item, *sysroot_defined, errors, "ForEach", false);
                }
                // body は同じシェルで走るため定義は伝播する。
                validate_steps(body, env, sysroot_defined, errors);
                if let Some(redirect) = stdout {
                    validate_arg(&redirect.target, *sysroot_defined, errors, "ForEach", false);
                }
            }
            Step::Or { lhs, rhs } => {
                if lhs.is_empty() || rhs.is_empty() {
                    errors.push("`Or` の lhs/rhs が空".to_string());
                }
                // `||` のどちらが走るか不定なので定義は伝播させない。
                for side in [lhs, rhs] {
                    let mut inner = *sysroot_defined;
                    validate_steps(side, env, &mut inner, errors);
                }
            }
            Step::Subshell(body) => {
                if body.is_empty() {
                    errors.push("`Subshell` が空".to_string());
                }
                // サブシェル内の定義は外へ漏れない。
                let mut inner = *sysroot_defined;
                validate_steps(body, env, &mut inner, errors);
            }
            Step::Comment(_) => {}
            Step::WriteFile {
                path,
                lines,
                expand,
                ..
            } => {
                validate_arg(path, *sysroot_defined, errors, "WriteFile", false);
                for line in lines {
                    if matches!(line, Arg::Lit(text) if text.trim_end() == "EOF") {
                        errors.push("`WriteFile` の行が heredoc デリミタ `EOF` と衝突".to_string());
                    }
                    // `expand: false`（<<'EOF'）の行はシェル評価されない。
                    if *expand {
                        validate_arg(line, *sysroot_defined, errors, "WriteFile", true);
                    }
                }
            }
        }
    }
}

impl Steps {
    /// この計画の不変条件を検査し、違反リストを返す。
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        let mut sysroot_defined = self.sysroot;
        validate_steps(&self.steps, self.env, &mut sysroot_defined, &mut errors);
        // PGO workload は `( … ) || true` のサブシェル相当で走るため
        // SYSROOT 定義状態は外へ伝播しない独立スコープとして検査する。
        let workload = match &self.pgo {
            Some(PgoTemplate::Autotools(w) | PgoTemplate::Make(w)) => Some(w.as_slice()),
            None => None,
        };
        if let Some(workload) = workload {
            let mut inner = self.sysroot;
            validate_steps(workload, self.env, &mut inner, &mut errors);
        }
        errors
    }
}

/// ネストしたステップを 2 スペースインデントで出力する。
/// `WriteFile` の heredoc は body がリテラルかつ `EOF` が行頭必須のため、
/// `cat` 行のみインデントし本体・デリミタはそのまま出す。
fn render_nested(out: &mut String, env: Env, steps: &[Step]) {
    for step in steps {
        let mut buf = String::new();
        step.render(&mut buf, env);
        // heredoc の本体・デリミタは行頭必須なのでインデントしない。
        let mut in_heredoc = false;
        for line in buf.lines() {
            if in_heredoc {
                out.push_str(line);
                out.push('\n');
                if line == "EOF" {
                    in_heredoc = false;
                }
                continue;
            }
            let _ = writeln!(out, "  {line}");
            if line.contains("<<EOF") || line.contains("<<'EOF'") {
                in_heredoc = true;
            }
        }
    }
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
    /// `workload` は `( … ) || true` で囲んで emit される型付きステップ列。
    pub fn autotools_pgo(args: &[&'static str], workload: Vec<Step>) -> Self {
        Self::autotools_pgo_env(Env::Autotools, args, workload)
    }

    /// env を選べる定型 autotools + PGO テンプレート。
    pub fn autotools_pgo_env(env: Env, arguments: &[&'static str], workload: Vec<Step>) -> Self {
        Self::autotools_pgo_args(env, args(arguments), workload)
    }

    /// `make check` にビルドフラグを明示引き渡しする共有 workload
    /// （gmp/isl/mpc/mpfr/libffi/libxcrypt 型）。
    pub fn make_check_workload() -> Vec<Step> {
        vec![Step::Make {
            jobs: false,
            args: vec![
                "check".into(),
                "CFLAGS=\"$CFLAGS\"".into(),
                "CXXFLAGS=\"${CXXFLAGS:-}\"".into(),
                "LDFLAGS=\"$LDFLAGS\"".into(),
            ],
        }]
    }

    /// 実行時 configure 引数 + PGO テンプレート（gmp の ABI 等）。
    pub fn autotools_pgo_args(env: Env, args: Vec<String>, workload: Vec<Step>) -> Self {
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
            Recipe::Noop => "true".to_owned(),
            Recipe::Steps(steps) => steps.render(ctx),
        }
    }
}

/// クロスビルド時の `CC_FOR_BUILD` 系（`AUTOTOOLS_ENV`/`LTO_ENV` 共通）。
const CROSS_BUILD_CC: &str = r#"export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export BUILD_CC="$CC_FOR_BUILD"
"#;

/// クロスビルド時の gnulib/autoconf 推測値キャッシュ（`AUTOTOOLS_ENV` 由来）。
/// Autoconf 2.72 は AC_RUN_IFELSE の cross 分岐が空だとエラーにする。
/// `prepare_env`（直接 runner）も同じ一覧を使う単一ソース。
pub(crate) const CROSS_CACHE_VARS: &[(&str, &str)] = &[
    ("gl_cv_func_strcasecmp_works", "yes"),
    ("gl_cv_func_strncasecmp_works", "yes"),
    ("gl_cv_func_memchr_works", "yes"),
    ("gl_cv_func_working_mktime", "yes"),
    ("gl_cv_func_working_utimes", "yes"),
    ("gl_cv_func_getcwd_null", "yes"),
    ("gl_cv_func_getcwd_path_max", "yes"),
    ("gl_cv_func_getcwd_abort_bug", "no"),
    ("gl_cv_func_chown_follows_symlink", "yes"),
    ("gl_cv_func_fchownat_nofollow_works", "yes"),
    ("gl_cv_func_lstat_dereferences_slashed_symlink", "yes"),
    ("gl_cv_func_stat_file_slash", "yes"),
    ("gl_cv_func_strerror_r_works", "yes"),
    ("gl_cv_func_strerror_0_works", "yes"),
    ("gl_cv_func_strtod_works", "yes"),
    ("gl_cv_func_nanosleep", "yes"),
    ("gl_cv_func_printf_positions", "yes"),
    ("gl_cv_func_printf_infinite", "yes"),
    ("gl_cv_func_printf_enomem", "yes"),
    ("gl_cv_func_malloc_0_nonnull", "yes"),
    ("gl_cv_func_malloc_posix", "yes"),
    ("ac_cv_func_malloc_0_nonnull", "yes"),
    ("ac_cv_func_realloc_0_nonnull", "yes"),
    ("ac_cv_func_memcmp_working", "yes"),
];

/// `CROSS_CACHE_VARS` の `export NAME=value` スクリプト。
fn cross_cache_script() -> String {
    let mut out = String::new();
    for (name, value) in CROSS_CACHE_VARS {
        let _ = writeln!(out, "export {name}={value}");
    }
    out
}

/// configure probe 用の env 剥がし（`Flag::sanitized` と同じ集合）。
/// `-flto*`・LTO 系 `-Werror=*`・`-f{,no-}hardened` を落とす。
/// `-Werror=format-security` は残す（Fedora と同じ）。
const SANITIZE_FLAGS: &str = r#"sanitize_flags() {
  printf '%s' "$1" | sed -E 's/-flto(=[^[:space:]]*)?//g; s/-Werror=(odr|lto-type-mismatch|strict-aliasing)//g; s/-f(no-)?hardened//g'
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
        // InSource は $SOURCE を $BUILD へコピーしてからビルドするため、
        // configure はコピー側（writable）の ./configure を使う。
        // "$SOURCE"/configure だと srcdir が read-only マウントのままになり、
        // flex の version.texi 再生成のように srcdir へ書き込む rule で失敗する。
        let configure_cmd = if matches!(self.workdir, Workdir::InSource) {
            "./configure"
        } else {
            "\"$SOURCE\"/configure"
        };
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
                    out.push_str(&cross_cache_script());
                }
                out.push_str(SANITIZE_FLAGS);
                let _ = write!(
                    out,
                    r#"autotools_configure() {{
  if ! CFLAGS="$(sanitize_flags "${{CFLAGS:-}}")" \
       CXXFLAGS="$(sanitize_flags "${{CXXFLAGS:-}}")" \
       LDFLAGS="$(sanitize_flags "${{LDFLAGS:-}}")" \
       {configure_cmd} --build="$HOST_TRIPLE" --host="${{PACKAGE_HOST:-$TRIPLE}}" "$@"; then
    if [ -f config.log ]; then
      echo "===== config.log (last 80 lines) ====="
      tail -n 80 config.log
    fi
    return 1
  fi
}}
"#,
                );
            }
            Env::Lto => {
                out.push_str("set -euxo pipefail\nexport FORCE_UNSAFE_CONFIGURE=1\n");
                let _ = writeln!(
                    out,
                    "export CFLAGS=\"${{CFLAGS:-}} -std={}\"",
                    self.std.as_str()
                );
                if ctx.cross() {
                    out.push_str(CROSS_BUILD_CC);
                }
                let _ = write!(
                    out,
                    r#"autotools_configure() {{
  {configure_cmd} --build="$HOST_TRIPLE" --host="${{PACKAGE_HOST:-$TRIPLE}}" "$@"
}}
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
        match (&self.pgo, ctx.cross()) {
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
    fn render_pgo_autotools(&self, out: &mut String, workload: &[Step]) {
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
        render_workload(out, self.env, workload);
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
    fn render_pgo_make(&self, out: &mut String, workload: &[Step]) {
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
        render_workload(out, self.env, workload);
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
fn render_workload(out: &mut String, env: Env, workload: &[Step]) {
    out.push_str("(\n");
    render_nested(out, env, workload);
    out.push_str(") || true\n");
}

impl Step {
    pub(crate) fn render(&self, out: &mut String, env: Env) {
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
            Step::Exec { env, pipe } => {
                for (name, value) in env {
                    let _ = write!(out, "{name}={} ", value.render());
                }
                let _ = writeln!(out, "{}", pipe.render());
            }
            Step::Cd(dir) => {
                let _ = writeln!(out, "cd {}", dir.render());
            }
            Step::Export { name, value } => {
                let _ = writeln!(out, "export {name}={}", value.render());
            }
            Step::ExportBare(name) => {
                let _ = writeln!(out, "export {name}");
            }
            Step::Set { name, value } => {
                let _ = writeln!(out, "{name}={}", value.render());
            }
            Step::Unset(name) => {
                let _ = writeln!(out, "unset {name}");
            }
            Step::If { cond, then, else_ } => {
                let _ = writeln!(out, "if {}; then", cond.render());
                render_nested(out, env, then);
                if !else_.is_empty() {
                    out.push_str("else\n");
                    render_nested(out, env, else_);
                }
                out.push_str("fi\n");
            }
            Step::ForEach {
                var,
                items,
                body,
                stdout,
            } => {
                let items = items.iter().map(Arg::render).collect::<Vec<_>>().join(" ");
                let _ = writeln!(out, "for {var} in {items}; do");
                render_nested(out, env, body);
                match stdout {
                    Some(redirect) => {
                        let op = if redirect.append { " >> " } else { " > " };
                        let _ = writeln!(out, "done{op}{}", redirect.target.render());
                    }
                    None => out.push_str("done\n"),
                }
            }
            Step::Or { lhs, rhs } => {
                let side = |steps: &[Step]| -> String {
                    if let [step] = steps {
                        let mut buf = String::new();
                        step.render(&mut buf, env);
                        let trimmed = buf.trim_end_matches('\n');
                        if !trimmed.contains('\n') {
                            return trimmed.to_string();
                        }
                    }
                    let mut buf = String::from("(\n");
                    render_nested(&mut buf, env, steps);
                    buf.push(')');
                    buf
                };
                let _ = writeln!(out, "{} || {}", side(lhs), side(rhs));
            }
            Step::Subshell(steps) => {
                out.push_str("(\n");
                render_nested(out, env, steps);
                out.push_str(")\n");
            }
            Step::Comment(text) => {
                for line in text.lines() {
                    let _ = writeln!(out, "# {line}");
                }
            }
            Step::WriteFile {
                path,
                lines,
                append,
                expand,
            } => {
                let redirect = if *append { ">>" } else { ">" };
                let delimiter = if *expand { "EOF" } else { "'EOF'" };
                let _ = writeln!(out, "cat {redirect} {} <<{delimiter}", path.render());
                for line in lines {
                    let _ = writeln!(out, "{}", line.render());
                }
                out.push_str("EOF\n");
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
        let recipe = Recipe::autotools_pgo(
            &["--prefix=/usr"],
            vec![Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["./tool", "--bench"])),
            }],
        );
        let script = recipe.render(&ctx());
        assert!(script.contains("export PGODIR=/tmp/pgo"), "{script}");
        assert!(script.contains("-fprofile-generate=$PGODIR"), "{script}");
        assert!(script.contains("-fprofile-use=$PGODIR"), "{script}");
        assert!(script.contains("-fprofile-correction"), "{script}");
        assert!(script.contains("make distclean || make clean"), "{script}");
        assert!(
            script.contains("(\n  ./tool --bench\n) || true"),
            "{script}"
        );
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
        let recipe = Recipe::autotools_pgo(
            &["--prefix=/usr"],
            vec![Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["./tool", "--bench"])),
            }],
        );
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
        let recipe = Recipe::autotools_pgo_env(
            Env::Lto,
            &["--prefix=/usr", "ABI=64"],
            Recipe::make_check_workload(),
        );
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
                .pgo(PgoTemplate::Make(vec![Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(argv(&["./zstd", "-b3", "input"])),
                }]))
                .step(Step::make(&["PREFIX=/usr"]))
                .step(Step::make_install(&["PREFIX=/usr"])),
        );
        let script = recipe.render(&ctx());
        assert_eq!(script.matches("make -j\"$(nproc)\" PREFIX=/usr").count(), 2);
        assert_eq!(script.matches("make install PREFIX=/usr").count(), 1);
        assert!(script.contains("make clean || make distclean"), "{script}");
        assert!(
            script.contains("(\n  ./zstd -b3 input\n) || true"),
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
                    src: "\"$SOURCE\"/cacert.pem".into(),
                    dst: "\"$OUT\"/etc/ssl/certs/ca.crt".into(),
                    mode: "644".into(),
                })
                .step(Step::Symlink {
                    target: "ca.crt".into(),
                    link: "\"$OUT\"/etc/ssl/ca-bundle.crt".into(),
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
    fn validate_catches_structural_violations() {
        // sysroot(false) で未ガードの $SYSROOT 参照 → 違反。
        let bad = Steps::new(Env::None).sysroot(false).step(Step::Exec {
            env: Vec::new(),
            pipe: Pipe::cmd(vec![Arg::lit("cc"), Arg::Var("SYSROOT")]),
        });
        let errors = Recipe::Steps(bad).validate();
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert!(errors[0].contains("SYSROOT"), "{errors:?}");

        // $SYSROOT ガード内なら合法。
        let guarded = Steps::new(Env::None).sysroot(false).step(Step::If {
            cond: Cond::EnvSet("SYSROOT"),
            then: vec![Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("cc"), Arg::Var("SYSROOT")]),
            }],
            else_: Vec::new(),
        });
        assert!(Recipe::Steps(guarded).validate().is_empty());

        // 先行する Set で定義済みなら合法、Subshell 内の Set は外に伝播しない。
        let scoped = Steps::new(Env::None)
            .sysroot(false)
            .step(Step::Subshell(vec![Step::Set {
                name: "SYSROOT",
                value: Arg::lit("/x"),
            }]))
            .step(Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("cc"), Arg::Var("SYSROOT")]),
            });
        assert_eq!(Recipe::Steps(scoped).validate().len(), 1);

        // Env::None での Configure・空 then・空 Pipe も違反。
        let misc = Steps::new(Env::None)
            .step(Step::Configure(vec!["--x".into()]))
            .step(Step::If {
                cond: Cond::EnvSet("FOO"),
                then: Vec::new(),
                else_: Vec::new(),
            })
            .step(Step::WriteFile {
                path: Arg::lit("/tmp/f"),
                lines: vec![Arg::lit("EOF")],
                append: false,
                expand: false,
            });
        let errors = Recipe::Steps(misc).validate();
        assert_eq!(errors.len(), 3, "{errors:?}");
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

    #[test]
    fn write_file_heredoc_stays_at_column_zero_in_nested_if() {
        // ネストした If 内でも heredoc 本体・デリミタは行頭必須
        // （インデントすると bash が終端 `EOF` を認識しない）。
        let mut out = String::new();
        Step::If {
            cond: Cond::EnvSet("X"),
            then: vec![Step::If {
                cond: Cond::EnvSet("Y"),
                then: vec![Step::WriteFile {
                    path: Arg::lit("/tmp/f"),
                    lines: vec![Arg::lit("body")],
                    append: false,
                    expand: false,
                }],
                else_: vec![],
            }],
            else_: vec![],
        }
        .render(&mut out, Env::None);
        assert!(out.contains("\nbody\nEOF\n"), "{out}");
        assert!(out.lines().any(|line| line == "EOF"), "{out}");
    }

    #[test]
    fn validate_rejects_runner_unsupported_syntax() {
        // 外部コマンドの無い純粋 builtin の argv[0] は runner で
        // 実行不能なので拒否する
        let builtin = Steps::new(Env::None).step(Step::Exec {
            env: Vec::new(),
            pipe: Pipe::cmd(vec![Arg::lit("eval"), Arg::lit("true")]),
        });
        let errors = Recipe::Steps(builtin).validate();
        assert!(errors.iter().any(|e| e.contains("builtin")), "{errors:?}");

        // 裸の特殊変数は runner 未対応（bash とは別の意味になる）
        for text in ["echo $?", "echo $$", "echo $1", "echo $@"] {
            let s = Steps::new(Env::None).step(Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::lit(text)]),
            });
            let errors = Recipe::Steps(s).validate();
            assert!(
                errors.iter().any(|e| e.contains("特殊変数")),
                "{text}: {errors:?}"
            );
        }

        // `${VAR%…}` `${VAR#…}` `${#VAR}` 等は `}` `:-` `:+` のみ許可
        for text in ["${X%/*}", "${X#*/}", "${#X}"] {
            let s = Steps::new(Env::None).step(Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::lit(text)]),
            });
            let errors = Recipe::Steps(s).validate();
            assert!(!errors.is_empty(), "{text} を拒否できていない");
        }

        // Lit 内の `$(`・バッククォートは Arg::Subst を使う
        let subst = Steps::new(Env::None).step(Step::Exec {
            env: Vec::new(),
            pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::lit("$(id)")]),
        });
        let errors = Recipe::Steps(subst).validate();
        assert!(errors.iter().any(|e| e.contains("Subst")), "{errors:?}");
    }
}
