//! executor 非依存 runner。`Recipe`/`Steps` IR を bash へレンダリング
//! せず直接解釈する実行層。`Sandbox` が実行環境（コンテナ内・ホスト・
//! テスト用トレース）を抽象化し、将来的にはコンテナ内 runner が
//! このインタプリタを共有する。bash 生成（`render`）は移行期の
//! デバッグ backend として残る。
//!
//! bash の意味論との対応:
//! - `set -u` 相当: 未定義変数参照は `Error::UnboundVar`
//! - `set -e` 相当: `Error::Exit` がそのまま伝播する（`Or`・`Cond::Cmd`・
//!   `workload` の `( … ) || true` だけが Exit を捕捉する）
//! - `Subshell` は変数スコープを push/pop する
//! - `ForEach` の items は bash と同様に空白で word split する

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

use super::recipe::{
    Arg, BuildContext, BuildPlan, Cond, Env, PgoTemplate, Pipe, Recipe, Std, Step, Steps, Workdir,
};

/// 実行エラー。`Exit` は非ゼロ終了（`set -e` で伝播するもの）、
/// それ以外はインタプリタ・環境側の失敗。
#[derive(Debug)]
pub enum Error {
    /// argv/パイプラインが非ゼロ終了。
    Exit { argv: Vec<String>, code: i32 },
    /// 未定義変数の参照（`set -u` 相当）。
    UnboundVar(String),
    /// sandbox がその操作を持たない。
    Unsupported(&'static str),
    /// ファイル・プロセス操作の失敗。
    Io(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exit { argv, code } => write!(f, "{argv:?} が終了コード {code}"),
            Self::UnboundVar(name) => write!(f, "未定義変数 ${name}"),
            Self::Unsupported(what) => write!(f, "sandbox が {what} をサポートしない"),
            Self::Io(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Error {}

/// 実行環境の抽象化。コンテナ内 runner・ホスト実行・テスト用トレースが
/// 実装する。変数は「シェル変数」（`set`）と export 済みを区別し、
/// `push_scope`/`pop_scope` で `Subshell` の `( … )` を表現する。
pub trait Sandbox {
    fn get(&self, name: &str) -> Option<String>;
    /// シェル変数への代入（export しない `NAME=value`）。
    fn set(&mut self, name: &str, value: &str);
    /// `export NAME=value`。
    fn export(&mut self, name: &str, value: &str);
    /// `export NAME`（既存値の再 export）。
    fn reexport(&mut self, name: &str);
    /// `unset NAME`。Subshell 内では外側スコープの export を
    /// shadow しなければならない（bash の `( unset X )` 相当）。
    /// 実装はスコープ毎の tombstone で表現する。
    fn unset(&mut self, name: &str);
    /// `Subshell` 開始。以後の set/export/unset/cd は pop で破棄される。
    fn push_scope(&mut self);
    fn pop_scope(&mut self);
    fn nproc(&self) -> usize;

    /// argv 実行。`env` はそのコマンドだけの一時 override、
    /// `stdin` は `< file`、`stdout` は `> path`/`>> path`、
    /// `stderr` は `2> path`。`capture` が真で stdout リダイレクトが
    /// 無い場合は出力をキャプチャして返す。偽なら子の stdout は
    /// 継承（ログへ流れる）で戻り値は空。
    ///
    /// 実装の契約:
    /// - spawn 時の ENOENT（コマンド不在）は `Error::Exit{code:127}`
    ///   にマップする（bash の `command not found` 相当。
    ///   `||`/`Cond::Cmd`/`SubstOr` がこれを判定に使う）
    /// - `argv[0]` がシェル builtin（`command`/`sanitize_flags`）なら
    ///   実装側で intercept する（`command -v X` は sandbox の現在
    ///   PATH から解決して経路を返す／miss は Exit{1}）
    /// - `set -x` 相当: 実行 argv を stderr へ `+ argv` 形式で出すと
    ///   bash 経路のログと揃う
    fn exec(
        &mut self,
        argv: &[String],
        env: &[(String, String)],
        stdin: Option<&str>,
        stdout: Option<(&str, bool)>,
        stderr: Option<&str>,
        capture: bool,
    ) -> Result<String, Error>;

    /// argv ステージ列のパイプライン実行。`stdin` は `< file`、
    /// `stdout` は `> path`/`>> path`、`stderr` は `2> path`。
    /// `env` は先頭コマンドだけの一時 override（bash の
    /// `A=1 cmd | cmd2` と同じ）。`capture` が真で stdout
    /// リダイレクトが無い場合は出力をキャプチャして返す。偽なら
    /// 末尾ステージの stdout は継承される。
    /// `exec` と同じ builtin/ENOENT 契約を各ステージに適用する。
    /// いずれかのステージが失敗したら `Exit`（pipefail 相当）。
    fn pipeline(
        &mut self,
        stages: &[Vec<String>],
        env: &[(String, String)],
        stdin: Option<&str>,
        stdout: Option<(&str, bool)>,
        stderr: Option<&str>,
        capture: bool,
    ) -> Result<String, Error>;

    fn write_file(&mut self, path: &str, lines: &[String], append: bool) -> Result<(), Error>;
    fn mkdir_p(&mut self, path: &str) -> Result<(), Error>;
    /// `cp -a src/. dst/`（in-source コピー）。
    fn copy_into(&mut self, src: &str, dst: &str) -> Result<(), Error>;
    fn cd(&mut self, path: &str) -> Result<(), Error>;
    fn remove_dir(&mut self, path: &str) -> Result<(), Error>;
    /// `install -D -m mode src dst`。
    fn install(&mut self, src: &str, dst: &str, mode: &str) -> Result<(), Error>;
    /// `ln -s[f] target link`。
    fn symlink(&mut self, target: &str, link: &str, force: bool) -> Result<(), Error>;
    fn dir_exists(&self, path: &str) -> bool;
    fn file_exists(&self, path: &str) -> bool;
    /// `[ -x path ]`。既定は `file_exists` と同値（テスト用トレース向け）。
    fn is_executable(&self, path: &str) -> bool {
        self.file_exists(path)
    }

    /// `[ -e path ]`（シンボリックリンクを辿る）。fifo/socket も
    /// 真。既定は `file_exists || dir_exists` の近似。
    fn path_exists(&self, path: &str) -> bool {
        self.file_exists(path) || self.dir_exists(path)
    }

    /// パス展開（`*.o` 等）。既定は bash の non-nullglob と同じく
    /// パターンをそのまま返す。
    fn glob(&self, pattern: &str) -> Vec<String> {
        vec![pattern.to_string()]
    }

    /// `Env` プリリュード相当の環境準備（FORCE_UNSAFE_CONFIGURE・
    /// `-std` 付与・クロス時の CC_FOR_BUILD/cache 変数）。
    /// bash backend では prelude テキスト、直接 runner では Rust 実装。
    fn prepare_env(&mut self, env: Env, std: Std, cross: bool) -> Result<(), Error>;

    /// `autotools_configure` 相当（flag sanitize・--build/--host 付与・
    /// config.log tail）。`env` は sanitize 要否の判定用。
    fn configure(&mut self, args: &[String], env: Env) -> Result<(), Error>;

    /// instrumented ビルドの .libs を `LD_LIBRARY_PATH` へ載せる
    /// （`PGO_LD_LIBRARY_PATH` 相当。`find` が要るため sandbox 側の操作）。
    fn pgo_ld_library_path(&mut self) -> Result<(), Error>;
}

// ---------- Arg / Lit の評価 ----------

/// `Lit` の bash 風 word 展開。`'…'` は展開も分割もしない、
/// `"…"` は `$` を展開するが分割しない、非クォート `$VAR` /
/// `${VAR}` / `${VAR:-default}` / `${VAR:+word}` は展開して空白で
/// 分割する。glob 文字（`*?[`）を非クォートで含むワードは
/// `Sandbox::glob` でパス展開する。
fn lit_words(text: &str, sb: &mut dyn SandboxTrait) -> Result<Vec<String>, Error> {
    let mut words: Vec<(String, bool)> = Vec::new();
    let mut cur = String::new();
    let mut open = false;
    let mut globbable = false;
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\'' => {
                open = true;
                for c in it.by_ref() {
                    if c == '\'' {
                        break;
                    }
                    cur.push(c);
                }
            }
            '"' => {
                open = true;
                while let Some(c) = it.next() {
                    match c {
                        '"' => break,
                        // "…" 内の \ は $ ` " \ と改行の前のみ有効
                        '\\' => match it.peek().copied() {
                            Some('$') | Some('`') | Some('"') | Some('\\') => {
                                cur.push(it.next().unwrap());
                            }
                            _ => cur.push('\\'),
                        },
                        '$' => {
                            // クォート内なので split/glob しない
                            let token = take_dollar(&mut it);
                            cur.push_str(&expand_dollar(&token, sb)?);
                        }
                        c => cur.push(c),
                    }
                }
            }
            '$' => {
                let token = take_dollar(&mut it);
                let expanded = expand_dollar(&token, sb)?;
                let mut frags = expanded.split_ascii_whitespace();
                if let Some(first) = frags.next() {
                    cur.push_str(first);
                    open = true;
                    for frag in frags {
                        words.push((std::mem::take(&mut cur), globbable));
                        globbable = false;
                        cur = frag.to_string();
                    }
                }
            }
            '\\' => {
                // 非クォート部: \X → X（エスケープされた * は glob 化しない）
                open = true;
                if let Some(c) = it.next() {
                    cur.push(c);
                }
            }
            c if c.is_ascii_whitespace() => {
                if open {
                    words.push((std::mem::take(&mut cur), globbable));
                    globbable = false;
                    open = false;
                }
            }
            c => {
                if matches!(c, '*' | '?' | '[') {
                    globbable = true;
                }
                cur.push(c);
                open = true;
            }
        }
    }
    if open {
        words.push((cur, globbable));
    }
    let mut out = Vec::new();
    for (word, globbable) in words {
        if globbable {
            out.extend(sb.glob(&word));
        } else {
            out.push(word);
        }
    }
    Ok(out)
}

/// `it` は `$` の直後に位置する。`$VAR` / `${VAR…}` のトークンを
/// 切り出して返す（`$` 込み）。
fn take_dollar(it: &mut std::iter::Peekable<std::str::Chars>) -> String {
    let mut token = "$".to_string();
    if it.peek() == Some(&'{') {
        token.push(it.next().unwrap());
        for c in it.by_ref() {
            token.push(c);
            if c == '}' {
                break;
            }
        }
    } else {
        while let Some(&c) = it.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                token.push(c);
                it.next();
            } else {
                break;
            }
        }
    }
    token
}

/// `$VAR` / `${VAR}` / `${VAR:-d}` / `${VAR:+w}` トークンを評価する。
/// トークンでない `$` はそのまま返す。
fn expand_dollar(token: &str, sb: &dyn SandboxTrait) -> Result<String, Error> {
    if let Some(inner) = token.strip_prefix("${").and_then(|t| t.strip_suffix('}')) {
        if let Some((name, default)) = inner.split_once(":-") {
            return Ok(match sb.get(name).filter(|v| !v.is_empty()) {
                Some(v) => v,
                None => expand_dollar_text(default, sb)?,
            });
        }
        if let Some((name, alt)) = inner.split_once(":+") {
            return Ok(match sb.get(name).filter(|v| !v.is_empty()) {
                Some(_) => expand_dollar_text(alt, sb)?,
                None => String::new(),
            });
        }
        return lookup(sb, inner);
    }
    if token.len() > 1 {
        return lookup(sb, &token[1..]);
    }
    Ok("$".to_string())
}

/// `${…:-…}` / `${…:+…}` の inner 部分の再帰展開（分割はしない）。
fn expand_dollar_text(text: &str, sb: &dyn SandboxTrait) -> Result<String, Error> {
    let mut out = String::new();
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        if c == '$' {
            let token = take_dollar(&mut it);
            out.push_str(&expand_dollar(&token, sb)?);
        } else {
            out.push(c);
        }
    }
    Ok(out)
}

/// 代入 RHS・リダイレクト先等の単一値コンテキスト: クォート除去と
/// `$` 展開はするが word split / glob はしない（bash の代入と同じ）。
fn lit_assign(text: &str, sb: &mut dyn SandboxTrait) -> Result<String, Error> {
    let mut out = String::new();
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            '\'' => {
                for c in it.by_ref() {
                    if c == '\'' {
                        break;
                    }
                    out.push(c);
                }
            }
            '"' => {
                while let Some(c) = it.next() {
                    match c {
                        '"' => break,
                        '\\' => match it.peek().copied() {
                            Some('$') | Some('`') | Some('"') | Some('\\') => {
                                out.push(it.next().unwrap());
                            }
                            _ => out.push('\\'),
                        },
                        '$' => {
                            let token = take_dollar(&mut it);
                            out.push_str(&expand_dollar(&token, sb)?);
                        }
                        c => out.push(c),
                    }
                }
            }
            '\\' => {
                if let Some(c) = it.next() {
                    out.push(c);
                }
            }
            '$' => {
                let token = take_dollar(&mut it);
                out.push_str(&expand_dollar(&token, sb)?);
            }
            c => out.push(c),
        }
    }
    Ok(out)
}

/// `eval_assign` の Arg 版。
fn eval_assign(arg: &Arg, sb: &mut dyn SandboxTrait) -> Result<String, Error> {
    match arg {
        Arg::Lit(text) => lit_assign(text, sb),
        Arg::Var(name) => lookup(sb, name),
        Arg::Concat(parts) => {
            let mut text = String::new();
            for part in parts {
                text.push_str(&eval_assign(part, sb)?);
            }
            Ok(text)
        }
        Arg::Opt(name, value) => Ok(format!("--{name}={}", eval_assign(value, sb)?)),
        Arg::Quoted(inner) => eval_assign(inner, sb),
        Arg::Subst(pipe) => Ok(eval_pipe(pipe, sb, None)?
            .trim_end_matches('\n')
            .to_string()),
        Arg::SubstOr(pipe) => Ok(match eval_pipe(pipe, sb, None) {
            Ok(out) => out.trim_end_matches('\n').to_string(),
            Err(Error::Exit { .. }) => String::new(),
            Err(e) => return Err(e),
        }),
    }
}

/// heredoc `<<EOF` 相当: `$`/`${…}` のみ展開し、クォート文字・空白・
/// インデントは全てリテラルに残す（分割・glob・クォート除去をしない）。
fn lit_verbatim(text: &str, sb: &mut dyn SandboxTrait) -> Result<String, Error> {
    let mut out = String::new();
    let mut it = text.chars().peekable();
    while let Some(c) = it.next() {
        match c {
            // bash の unquoted heredoc は `\$` `\\` `\`` と
            // 行継続 `\<newline>` のみエスケープとして処理する
            '\\' => match it.peek() {
                Some('$') | Some('`') | Some('\\') => {
                    out.push(it.next().unwrap());
                }
                Some('\n') => {
                    it.next();
                }
                _ => out.push('\\'),
            },
            '$' => {
                let token = take_dollar(&mut it);
                out.push_str(&expand_dollar(&token, sb)?);
            }
            _ => out.push(c),
        }
    }
    Ok(out)
}

/// `lit_verbatim` の Arg 版。`Quoted` は `"…"` の字面を残す
/// （heredoc 内ではクォートは普通の文字）。
fn eval_verbatim(arg: &Arg, sb: &mut dyn SandboxTrait) -> Result<String, Error> {
    match arg {
        Arg::Lit(text) => lit_verbatim(text, sb),
        Arg::Var(name) => lookup(sb, name),
        Arg::Concat(parts) => {
            let mut text = String::new();
            for part in parts {
                text.push_str(&eval_verbatim(part, sb)?);
            }
            Ok(text)
        }
        Arg::Opt(name, value) => Ok(format!("--{name}={}", eval_verbatim(value, sb)?)),
        Arg::Quoted(inner) => Ok(format!("\"{}\"", eval_verbatim(inner, sb)?)),
        Arg::Subst(pipe) | Arg::SubstOr(pipe) => Ok(match eval_pipe(pipe, sb, None) {
            Ok(out) => out.trim_end_matches('\n').to_string(),
            Err(Error::Exit { .. }) => String::new(),
            Err(e) => return Err(e),
        }),
    }
}

fn lookup(sb: &dyn SandboxTrait, name: &str) -> Result<String, Error> {
    sb.get(name)
        .ok_or_else(|| Error::UnboundVar(name.to_string()))
}

/// 代入・リダイレクト・heredoc コンテキスト: word split しない 1 値。
fn eval_value(arg: &Arg, sb: &mut dyn SandboxTrait) -> Result<String, Error> {
    match arg {
        Arg::Lit(text) => lit_assign(text, sb),
        Arg::Var(name) => lookup(sb, name),
        Arg::Concat(parts) => {
            let mut text = String::new();
            for part in parts {
                text.push_str(&eval_value(part, sb)?);
            }
            Ok(text)
        }
        Arg::Opt(name, value) => Ok(format!("--{name}={}", eval_value(value, sb)?)),
        Arg::Quoted(inner) => eval_value(inner, sb),
        Arg::Subst(pipe) | Arg::SubstOr(pipe) => {
            // `$( )` は末尾改行を除去する。失敗しても bash は外側の
            // コマンドを空展開で続行する（ステータスを継承するのは
            // `X=$(f)` 型の裸の代入だけ。そちらは eval_assign）。
            // 部分出力は Err 経路では取れないため空に倒す。
            Ok(match eval_pipe(pipe, sb, None) {
                Ok(out) => out.trim_end_matches('\n').to_string(),
                Err(Error::Exit { .. }) => String::new(),
                Err(e) => return Err(e),
            })
        }
    }
}

/// argv コンテキスト: word split・glob を適用したワード列。
/// `Quoted` と `'…'`/`"…"` で括られた部分は分割しない。
/// word の glob 文字を含む場合にパス展開する（split 後の各ワード）。
fn glob_word(w: String, sb: &mut dyn SandboxTrait) -> Vec<String> {
    if w.contains('*') || w.contains('?') || w.contains('[') {
        sb.glob(&w)
    } else {
        vec![w]
    }
}

fn eval_words(arg: &Arg, sb: &mut dyn SandboxTrait) -> Result<Vec<String>, Error> {
    match arg {
        Arg::Lit(text) => lit_words(text, sb),
        Arg::Var(name) => {
            let mut out = Vec::new();
            for w in split_words(&lookup(sb, name)?) {
                out.extend(glob_word(w, sb));
            }
            Ok(out)
        }
        Arg::Concat(parts) => {
            // 各パートの先頭ワードは前行の末尾に接着する
            // （`a$X`b で X が分割した場合の bash と同じ）。
            let mut out: Vec<String> = Vec::new();
            for part in parts {
                let mut ws = eval_words(part, sb)?;
                if ws.is_empty() {
                    continue;
                }
                let first = ws.remove(0);
                match out.last_mut() {
                    Some(last) => last.push_str(&first),
                    None => out.push(first),
                }
                out.extend(ws);
            }
            Ok(out)
        }
        Arg::Opt(name, value) => Ok(vec![format!("--{name}={}", eval_value(value, sb)?)]),
        Arg::Quoted(inner) => Ok(vec![eval_value(inner, sb)?]),
        Arg::Subst(pipe) | Arg::SubstOr(pipe) => {
            // `$(f)` が失敗しても外側コマンドは空展開で継続する
            // （bash）。`$UNSET` 等の非 Exit エラーは伝播する。
            let out = match eval_pipe(pipe, sb, None) {
                Ok(out) => out,
                Err(Error::Exit { .. }) => String::new(),
                Err(e) => return Err(e),
            };
            let mut words = Vec::new();
            for w in split_words(out.trim_end_matches('\n')) {
                words.extend(glob_word(w, sb));
            }
            Ok(words)
        }
    }
}

fn split_words(text: &str) -> Vec<String> {
    text.split_ascii_whitespace().map(str::to_string).collect()
}

/// `expand: false`（`<<'EOF'`）の heredoc 行はシェル評価されないので、
/// `Arg` をそのまま字面に戻す（render と同じ表記）。
fn literal(arg: &Arg) -> String {
    match arg {
        Arg::Lit(text) => text.to_string(),
        Arg::Var(name) => format!("${{{name}}}"),
        Arg::Concat(parts) => parts.iter().map(literal).collect(),
        Arg::Opt(name, value) => format!("--{name}={}", literal(value)),
        Arg::Quoted(inner) => format!("\"{}\"", literal(inner)),
        Arg::Subst(pipe) => format!("$({})", literal_pipe(pipe)),
        Arg::SubstOr(pipe) => format!("$({} || true)", literal_pipe(pipe)),
    }
}

fn literal_pipe(pipe: &Pipe) -> String {
    pipe.stages
        .iter()
        .map(|stage| stage.iter().map(literal).collect::<Vec<_>>().join(" "))
        .collect::<Vec<_>>()
        .join(" | ")
}

fn eval_pipe(
    pipe: &Pipe,
    sb: &mut dyn SandboxTrait,
    redirect: Option<(&str, bool)>,
) -> Result<String, Error> {
    let mut stages = Vec::with_capacity(pipe.stages.len());
    for stage in &pipe.stages {
        let mut argv = Vec::new();
        for arg in stage {
            argv.extend(eval_words(arg, sb)?);
        }
        if argv.is_empty() {
            // 展開結果が空のステージは bash の空コマンド相当で skip
            continue;
        }
        stages.push(argv);
    }
    let stdin = pipe
        .stdin
        .as_ref()
        .map(|arg| eval_value(arg, sb))
        .transpose()?;
    let stdout = match (&pipe.stdout, redirect) {
        (Some(r), _) => Some((eval_value(&r.target, sb)?, r.append)),
        (None, outer) => outer.map(|(path, append)| (path.to_string(), append)),
    };
    let stderr = pipe
        .stderr
        .as_ref()
        .map(|arg| eval_value(arg, sb))
        .transpose()?;
    if stages.is_empty() {
        // 空パイプライン: リダイレクトだけ適用して成功（bash と同じ）
        if let Some((path, append)) = &stdout {
            sb.write_file(path, &[], *append)?;
        }
        return Ok(String::new());
    }
    sb.pipeline(
        &stages,
        &[],
        stdin.as_deref(),
        stdout
            .as_ref()
            .map(|(path, append)| (path.as_str(), *append)),
        stderr.as_deref(),
        true,
    )
}

/// Sandbox trait object の別名（ジェネリクス境界を短くする）。
trait SandboxTrait: Sandbox {}
impl<T: Sandbox + ?Sized> SandboxTrait for T {}

// ---------- インタプリタ ----------

/// recipe を直接実行する。bash レンダリングと同じ意味論を
/// `Sandbox` 操作へ落とす。
pub fn run(recipe: &Recipe, ctx: &BuildContext, sb: &mut impl Sandbox) -> Result<(), Error> {
    match recipe {
        Recipe::Noop => Ok(()),
        Recipe::Steps(steps) => run_decl(steps, ctx, sb),
    }
}

/// `BuildPlan` を直接実行する。`prelude` は bash で `set -euxo
/// pipefail` の行より前に走る部分に相当するため lax に実行する
/// （各ステップの Exit/UnboundVar を握り潰して継続。bash は未定義
/// 変数を空文字としてコマンドを実行するが、ここではステップ全体を
/// スキップする近似）。`recipe` 本体は strict に実行する。
pub fn run_plan(plan: &BuildPlan, ctx: &BuildContext, sb: &mut impl Sandbox) -> Result<(), Error> {
    for step in &plan.prelude {
        let _ = exec_step(step, Env::None, sb, None);
    }
    run(&plan.recipe, ctx, sb)
}

fn run_decl(steps: &Steps, ctx: &BuildContext, sb: &mut impl Sandbox) -> Result<(), Error> {
    sb.prepare_env(steps.env, steps.std, ctx.cross())?;
    match steps.workdir {
        Workdir::Build => {
            let build = lookup_dyn(sb, "BUILD")?;
            sb.mkdir_p(&build)?;
            sb_cd(sb, &build)?;
        }
        Workdir::InSource => {
            let build = lookup_dyn(sb, "BUILD")?;
            let source = lookup_dyn(sb, "SOURCE")?;
            sb.mkdir_p(&build)?;
            sb.copy_into(&source, &build)?;
            sb_cd(sb, &build)?;
        }
        Workdir::None => {}
    }
    match (&steps.pgo, ctx.cross()) {
        (Some(PgoTemplate::Autotools(workload)), false) => {
            pgo_prelude(sb)?;
            run_pgo_autotools(steps, workload, sb)
        }
        (Some(PgoTemplate::Make(workload)), false) => {
            pgo_prelude(sb)?;
            run_pgo_make(steps, workload, sb)
        }
        _ => exec_steps(&steps.steps, steps.env, sb, None),
    }
}

fn lookup_dyn(sb: &mut impl Sandbox, name: &str) -> Result<String, Error> {
    sb.get(name)
        .ok_or_else(|| Error::UnboundVar(name.to_string()))
}

/// `cd` + `PWD` 更新。bash は cd 時に PWD を設定するため、
/// `Arg::Var("PWD")` を参照するレシピ（zlib/openssl/perl）用に追跡する。
fn sb_cd(sb: &mut impl Sandbox, dir: &str) -> Result<(), Error> {
    sb.cd(dir)?;
    sb.set("PWD", dir);
    // PWD は bash で export 済みが一般的なので export フラグを立てる
    sb.reexport("PWD");
    Ok(())
}

/// `export PGODIR=/tmp/pgo && rm -rf && mkdir -p`。
fn pgo_prelude(sb: &mut impl Sandbox) -> Result<(), Error> {
    sb.export("PGODIR", "/tmp/pgo");
    sb.remove_dir("/tmp/pgo")?;
    sb.mkdir_p("/tmp/pgo")
}

/// `PGO_GENERATE`: ベース値を `_PGO_*` へ退避し `-fprofile-generate` を付与。
fn pgo_generate(sb: &mut impl Sandbox) {
    for name in ["CFLAGS", "CXXFLAGS", "LDFLAGS"] {
        let base = sb.get(name).unwrap_or_default();
        sb.export(&format!("_PGO_{name}"), &base);
    }
    let pgodir = sb.get("PGODIR").unwrap_or_default();
    let cflags = sb.get("_PGO_CFLAGS").unwrap_or_default();
    let cxxflags = sb.get("_PGO_CXXFLAGS").unwrap_or_default();
    let ldflags = sb.get("_PGO_LDFLAGS").unwrap_or_default();
    sb.export(
        "CFLAGS",
        &format!("{cflags} -fprofile-generate={pgodir} -fprofile-update=atomic"),
    );
    sb.export(
        "CXXFLAGS",
        &format!("{cxxflags} -fprofile-generate={pgodir} -fprofile-update=atomic"),
    );
    sb.export("LDFLAGS", &format!("{ldflags} -fprofile-generate"));
}

/// `PGO_USE`: `_PGO_*` から復元して `-fprofile-use` 系を付与。
fn pgo_use(sb: &mut impl Sandbox) {
    let pgodir = sb.get("PGODIR").unwrap_or_default();
    let cflags = sb.get("_PGO_CFLAGS").unwrap_or_default();
    let cxxflags = sb.get("_PGO_CXXFLAGS").unwrap_or_default();
    let ldflags = sb.get("_PGO_LDFLAGS").unwrap_or_default();
    let use_flags = format!(
        "-fprofile-use={pgodir} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
    );
    sb.export("CFLAGS", &format!("{cflags} {use_flags}"));
    sb.export("CXXFLAGS", &format!("{cxxflags} {use_flags}"));
    sb.export("LDFLAGS", &format!("{ldflags} -fprofile-use"));
}

/// `make distclean || make clean`（引数で順序を変える）。
fn make_clean(
    sb: &mut impl Sandbox,
    first: &'static str,
    second: &'static str,
) -> Result<(), Error> {
    let argv = |t: &'static str| vec!["make".to_string(), t.to_string()];
    match sb.exec(&argv(first), &[], None, None, None, false) {
        Err(Error::Exit { .. }) => sb
            .exec(&argv(second), &[], None, None, None, false)
            .map(|_| ()),
        result => result.map(|_| ()),
    }
}

/// `( workload ) || true` 相当。サブシェル相当のスコープで各ステップを
/// 試行し、失敗は全て飲み込む（bash の `|| true` + `set -e` 抑制と同じ）。
fn run_workload(steps: &Steps, workload: &[Step], sb: &mut impl Sandbox) {
    sb.push_scope();
    for step in workload {
        match exec_step(step, steps.env, sb, None) {
            // `( … ) || true` 内では set -e が効かないので Exit は継続。
            // 一方 set -u は有効なので UnboundVar 等は subshell を abort
            // する（残りステップを飛ばすのが bash の挙動）。
            Ok(()) | Err(Error::Exit { .. }) => {}
            Err(_) => break,
        }
    }
    sb.pop_scope();
}

/// テンプレート A: configure → instrumented make → workload →
/// distclean → reconfigure → use make → install。
/// `render_pgo_autotools` と同じフェーズ分割を `Sandbox` 操作で行う。
fn run_pgo_autotools(steps: &Steps, workload: &[Step], sb: &mut impl Sandbox) -> Result<(), Error> {
    let Some(configure) = steps
        .steps
        .iter()
        .position(|step| matches!(step, Step::Configure(_)))
    else {
        return exec_steps(&steps.steps, steps.env, sb, None);
    };
    let first_make = steps
        .steps
        .iter()
        .position(|step| matches!(step, Step::Make { .. }))
        .unwrap_or(steps.steps.len());
    let install = steps
        .steps
        .iter()
        .position(|step| matches!(step, Step::MakeInstall { .. }))
        .unwrap_or(steps.steps.len());
    if configure + 1 > first_make || first_make > install {
        return exec_steps(&steps.steps, steps.env, sb, None);
    }
    exec_steps(&steps.steps[..configure], steps.env, sb, None)?;
    if matches!(steps.env, Env::Lto) {
        pgo_generate(sb);
    }
    exec_step(&steps.steps[configure], steps.env, sb, None)?;
    exec_steps(&steps.steps[configure + 1..first_make], steps.env, sb, None)?;
    if matches!(steps.env, Env::Autotools) {
        pgo_generate(sb);
    }
    exec_steps(&steps.steps[first_make..install], steps.env, sb, None)?;
    sb.pgo_ld_library_path()?;
    run_workload(steps, workload, sb);
    make_clean(sb, "distclean", "clean")?;
    pgo_use(sb);
    exec_step(&steps.steps[configure], steps.env, sb, None)?;
    exec_steps(&steps.steps[configure + 1..first_make], steps.env, sb, None)?;
    exec_steps(&steps.steps[first_make..install], steps.env, sb, None)?;
    exec_steps(&steps.steps[install..], steps.env, sb, None)
}

/// テンプレート B: instrumented make → workload → clean →
/// use make → install（configure を持たない in-source make 系）。
fn run_pgo_make(steps: &Steps, workload: &[Step], sb: &mut impl Sandbox) -> Result<(), Error> {
    let install = steps
        .steps
        .iter()
        .position(|step| matches!(step, Step::MakeInstall { .. }))
        .unwrap_or(steps.steps.len());
    pgo_generate(sb);
    exec_steps(&steps.steps[..install], steps.env, sb, None)?;
    sb.pgo_ld_library_path()?;
    run_workload(steps, workload, sb);
    make_clean(sb, "clean", "distclean")?;
    pgo_use(sb);
    exec_steps(&steps.steps[..install], steps.env, sb, None)?;
    exec_steps(&steps.steps[install..], steps.env, sb, None)
}

/// `Env::Autotools` が make 系に付与する変数上書きを argv 語
/// `NAME=value` として返す。ステップ引数が既に同名変数を持つ場合は
/// 付けない（render_flag_vars と同じ）。
/// コマンドライン変数は makefile の `CFLAGS = @CFLAGS@` を打ち消す
/// が、env にすると負けるため argv として渡す。
fn flag_overrides(sb: &mut impl Sandbox, env: Env, args: &[String]) -> Vec<String> {
    if !matches!(env, Env::Autotools) {
        return Vec::new();
    }
    ["CFLAGS", "CXXFLAGS", "LDFLAGS"]
        .into_iter()
        .filter(|name| {
            let prefix = format!("{name}=");
            !args.iter().any(|arg| arg.starts_with(&prefix))
        })
        .map(|name| format!("{name}={}", sb.get(name).unwrap_or_default()))
        .collect()
}

fn exec_steps(
    steps: &[Step],
    env: Env,
    sb: &mut impl Sandbox,
    redirect: Option<(&str, bool)>,
) -> Result<(), Error> {
    for step in steps {
        exec_step(step, env, sb, redirect)?;
    }
    Ok(())
}

/// `||` の側を実行する。bash は `||` 側内で `set -e` が抑制される
/// ため、途中のコマンド失敗では残りを止めず、側全体の結果は最後の
/// コマンドの終了コードになる（`UnboundVar` 等の非 Exit エラーは
/// `set -u` 相当で即時伝播する）。
fn exec_or_side(
    steps: &[Step],
    env: Env,
    sb: &mut impl Sandbox,
    redirect: Option<(&str, bool)>,
) -> Result<(), Error> {
    let mut result = Ok(());
    for (i, step) in steps.iter().enumerate() {
        result = exec_step(step, env, sb, redirect);
        if let Err(Error::Exit { .. }) = &result
            && i + 1 < steps.len()
        {
            // `||` 側内の途中失敗は続行する
            continue;
        }
        if result.is_err() {
            return result;
        }
    }
    result
}

fn exec_step(
    step: &Step,
    env: Env,
    sb: &mut impl Sandbox,
    redirect: Option<(&str, bool)>,
) -> Result<(), Error> {
    match step {
        Step::Configure(args) => {
            // 生文字列引数は bash と同じく未クォート部分を
            // word split/glob して複数 argv にする
            let mut expanded = Vec::new();
            for arg in args {
                expanded.extend(lit_words(arg, sb)?);
            }
            sb.configure(&expanded, env)
        }
        Step::Make { jobs, args } => {
            let mut argv = vec!["make".to_string()];
            if *jobs {
                argv.push(format!("-j{}", sb.nproc()));
            }
            for arg in args {
                argv.extend(lit_words(arg, sb)?);
            }
            // コマンドライン変数（makefile の `CFLAGS = …` を打ち消す）
            // として argv に追加する。env にすると負ける。
            argv.extend(flag_overrides(sb, env, args));
            sb.exec(&argv, &[], None, redirect, None, false).map(|_| ())
        }
        Step::MakeInstall { vars } => {
            let out = lookup_dyn(sb, "OUT")?;
            let mut argv = vec!["make".to_string(), "install".to_string()];
            for var in vars {
                argv.extend(lit_words(var, sb)?);
            }
            argv.extend(flag_overrides(sb, env, vars));
            argv.push(format!("DESTDIR={out}"));
            sb.exec(&argv, &[], None, redirect, None, false).map(|_| ())
        }
        Step::Install { src, dst, mode } => {
            let src = lit_words(src, sb)?;
            let dst = lit_words(dst, sb)?;
            if src.len() != 1 || dst.len() != 1 {
                return Err(Error::Exit {
                    argv: vec!["install".to_string()],
                    code: 1,
                });
            }
            sb.install(&src[0], &dst[0], mode)
        }
        Step::Symlink {
            target,
            link,
            force,
        } => {
            let target = lit_words(target, sb)?;
            let link = lit_words(link, sb)?;
            if target.len() != 1 || link.len() != 1 {
                return Err(Error::Exit {
                    argv: vec!["ln".to_string()],
                    code: 1,
                });
            }
            sb.symlink(&target[0], &link[0], *force)
        }
        Step::Exec { env: vars, pipe } => {
            let overrides = vars
                .iter()
                .map(|(name, value)| Ok(((*name).to_string(), eval_value(value, sb)?)))
                .collect::<Result<Vec<(String, String)>, Error>>()?;
            sb.push_scope();
            let result = eval_pipe_env(pipe, &overrides, sb, redirect);
            sb.pop_scope();
            result.map(|_| ())
        }
        Step::Comment(_) => Ok(()),
        Step::Cd(dir) => {
            let dir = eval_value(dir, sb)?;
            sb_cd(sb, &dir)
        }
        Step::Export { name, value } => {
            let value = eval_value(value, sb)?;
            sb.export(name, &value);
            Ok(())
        }
        Step::ExportBare(name) => {
            sb.reexport(name);
            Ok(())
        }
        Step::Set { name, value } => {
            let value = eval_assign(value, sb)?;
            sb.set(name, &value);
            Ok(())
        }
        Step::Unset(name) => {
            sb.unset(name);
            Ok(())
        }
        Step::If { cond, then, else_ } => {
            if eval_cond(cond, sb)? {
                exec_steps(then, env, sb, redirect)
            } else {
                exec_steps(else_, env, sb, redirect)
            }
        }
        Step::ForEach {
            var,
            items,
            body,
            stdout,
        } => {
            let mut words = Vec::new();
            for item in items {
                words.extend(eval_words(item, sb)?);
            }
            let inner_redirect = match stdout {
                Some(r) => {
                    // `done > f` はループ先頭で 1 度だけ開く
                    // （毎反復 truncate しない）。先に truncate してから
                    // 各反復は append 扱いにする。
                    let path = eval_value(&r.target, sb)?;
                    if !r.append {
                        sb.write_file(&path, &[], false)?;
                    }
                    Some((path, true))
                }
                None => redirect.map(|(p, a)| (p.to_string(), a)),
            };
            for word in words {
                sb.set(var, &word);
                exec_steps(
                    body,
                    env,
                    sb,
                    inner_redirect
                        .as_ref()
                        .map(|(path, append)| (path.as_str(), *append)),
                )?;
            }
            Ok(())
        }
        Step::Or { lhs, rhs } => {
            // render の side() 規則: 複数ステップ、または render で
            // 複数行になる単一ステップ（If/ForEach/Subshell/WriteFile）
            // は ( … ) に包まれて subshell 化するため、変数/cd の
            // 副作用をスコープで閉じる。
            let scoped = |steps: &[Step]| {
                steps.len() > 1
                    || matches!(
                        steps.first(),
                        Some(
                            Step::If { .. }
                                | Step::ForEach { .. }
                                | Step::Subshell(_)
                                | Step::WriteFile { .. }
                        )
                    )
            };
            if scoped(lhs) {
                sb.push_scope();
            }
            let result = exec_or_side(lhs, env, sb, redirect);
            if scoped(lhs) {
                sb.pop_scope();
            }
            match result {
                Err(Error::Exit { .. }) => {
                    if scoped(rhs) {
                        sb.push_scope();
                    }
                    let result = exec_or_side(rhs, env, sb, redirect);
                    if scoped(rhs) {
                        sb.pop_scope();
                    }
                    result
                }
                result => result,
            }
        }
        Step::Subshell(body) => {
            sb.push_scope();
            let result = exec_steps(body, env, sb, redirect);
            sb.pop_scope();
            result
        }
        Step::WriteFile {
            path,
            lines,
            append,
            expand,
        } => {
            let path = eval_value(path, sb)?;
            let body = lines
                .iter()
                .map(|line| {
                    if *expand {
                        // `<<EOF`: `$`/`${}`/`$(…)` のみ展開し、
                        // クォート・空白はリテラルに残す
                        eval_verbatim(line, sb)
                    } else {
                        Ok(literal(line))
                    }
                })
                .collect::<Result<Vec<_>, _>>()?;
            sb.write_file(&path, &body, *append)
        }
    }
}

/// `Exec` の `NAME=value cmd` はコマンド実行時のみの env override だが、
/// bash では変数展開の結果がそのまま argv に影響するため、パイプ評価の
/// スコープとして閉じ込める（親スコープを汚染しない）。
fn eval_pipe_env(
    pipe: &Pipe,
    overrides: &[(String, String)],
    sb: &mut impl Sandbox,
    redirect: Option<(&str, bool)>,
) -> Result<String, Error> {
    let mut stages = Vec::with_capacity(pipe.stages.len());
    for stage in &pipe.stages {
        let mut argv = Vec::new();
        for arg in stage {
            argv.extend(eval_words(arg, sb)?);
        }
        stages.push(argv);
    }
    let stdin = pipe
        .stdin
        .as_ref()
        .map(|arg| eval_value(arg, sb))
        .transpose()?;
    let stdout = match (&pipe.stdout, redirect) {
        (Some(r), _) => Some((eval_value(&r.target, sb)?, r.append)),
        (None, outer) => outer.map(|(path, append)| (path.to_string(), append)),
    };
    let stderr = pipe
        .stderr
        .as_ref()
        .map(|arg| eval_value(arg, sb))
        .transpose()?;
    // 展開結果が空のステージは bash の空コマンド相当で skip
    stages.retain(|argv| !argv.is_empty());
    if stages.is_empty() {
        if let Some((path, append)) = &stdout {
            sb.write_file(path, &[], *append)?;
        }
        return Ok(String::new());
    }
    if stages.len() == 1 {
        sb.exec(
            &stages[0],
            overrides,
            stdin.as_deref(),
            stdout
                .as_ref()
                .map(|(path, append)| (path.as_str(), *append)),
            stderr.as_deref(),
            false,
        )
    } else {
        // env override はパイプライン先頭コマンドにのみ適用する
        // （bash の `A=1 cmd | cmd2` と同じく cmd にだけ付く）。
        sb.pipeline(
            &stages,
            overrides,
            stdin.as_deref(),
            stdout
                .as_ref()
                .map(|(path, append)| (path.as_str(), *append)),
            stderr.as_deref(),
            false,
        )
    }
}

fn eval_cond(cond: &Cond, sb: &mut impl Sandbox) -> Result<bool, Error> {
    match cond {
        Cond::DirExists(path) => {
            let path = eval_value(path, sb)?;
            Ok(sb.dir_exists(&path))
        }
        Cond::FileExists(path) => {
            let path = eval_value(path, sb)?;
            Ok(sb.file_exists(&path))
        }
        Cond::Exists(path) => {
            let path = eval_value(path, sb)?;
            Ok(sb.path_exists(&path))
        }
        Cond::Executable(path) => {
            let path = eval_value(path, sb)?;
            Ok(sb.is_executable(&path))
        }
        Cond::NonEmpty(value) => Ok(!eval_value(value, sb)?.is_empty()),
        Cond::EnvSet(name) => Ok(sb.get(name).is_some_and(|v| !v.is_empty())),
        Cond::Cmd(pipe) => match eval_pipe(pipe, sb, None) {
            Ok(_) => Ok(true),
            Err(Error::Exit { .. }) => Ok(false),
            Err(e) => Err(e),
        },
        Cond::And(conds) => {
            for cond in conds {
                if !eval_cond(cond, sb)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        Cond::Not(inner) => Ok(!eval_cond(inner, sb)?),
    }
}

/// 簡易 fnmatch（`*`=`[^/]*`、`?`=1 文字、文字クラスなし）。
pub(crate) fn glob_match(pattern: &str, path: &str) -> bool {
    fn inner(p: &[u8], s: &[u8]) -> bool {
        match p.first() {
            None => s.is_empty(),
            Some(b'*') => (0..=s.len()).any(|i| inner(&p[1..], &s[i..])),
            Some(b'?') => !s.is_empty() && inner(&p[1..], &s[1..]),
            Some(&c) => s.first() == Some(&c) && inner(&p[1..], &s[1..]),
        }
    }
    inner(pattern.as_bytes(), path.as_bytes())
}

// ---------- テスト/デバッグ用トレース sandbox ----------

/// 副作用を記録するだけの sandbox。実コマンドは実行せず、
/// `results` キューで `Subst`/`Cond::Cmd` の結果を供給する。
/// runner の意味論テストと、計画→操作列の可視化に使う。
#[derive(Default)]
pub struct Trace {
    /// 記録した操作（`exec …`/`export NAME=…`/`cd …` 等）。
    pub ops: Vec<String>,
    /// `pipeline`/`exec` が返す結果のキュー（順に消費、空なら成功）。
    pub results: VecDeque<Result<String, i32>>,
    /// 存在するディレクトリとして振る舞うパス。
    pub dirs: BTreeSet<String>,
    /// 存在するファイルとして振る舞うパス。
    pub present_files: BTreeSet<String>,
    /// `write_file` で書かれた内容。
    pub files: BTreeMap<String, String>,
    /// 現在の作業ディレクトリ（`cd` で記録）。
    pub cwd: Option<String>,
    pub nproc: usize,
    vars: Vec<BTreeMap<String, (String, bool)>>,
    /// スコープ毎の unset 済み名（外側スコープの同名を shadow）。
    removed: Vec<BTreeSet<String>>,
    cwd_stack: Vec<Option<String>>,
}

impl Trace {
    pub fn new() -> Self {
        Self {
            nproc: 8,
            vars: vec![BTreeMap::new()],
            removed: vec![BTreeSet::new()],
            ..Self::default()
        }
    }

    pub fn with_var(mut self, name: &str, value: &str) -> Self {
        self.vars[0].insert(name.to_string(), (value.to_string(), false));
        self
    }
}

impl Sandbox for Trace {
    fn get(&self, name: &str) -> Option<String> {
        for (i, scope) in self.vars.iter().enumerate().rev() {
            // 内側スコープで unset された名は外側を shadow する
            if self.removed.get(i).is_some_and(|r| r.contains(name)) {
                return None;
            }
            if let Some((v, _)) = scope.get(name) {
                return Some(v.clone());
            }
        }
        None
    }

    fn set(&mut self, name: &str, value: &str) {
        self.ops.push(format!("set {name}={value}"));
        self.removed.last_mut().unwrap().remove(name);
        self.vars
            .last_mut()
            .unwrap()
            .insert(name.to_string(), (value.to_string(), false));
    }

    fn export(&mut self, name: &str, value: &str) {
        self.ops.push(format!("export {name}={value}"));
        self.removed.last_mut().unwrap().remove(name);
        self.vars
            .last_mut()
            .unwrap()
            .insert(name.to_string(), (value.to_string(), true));
    }

    fn reexport(&mut self, name: &str) {
        self.ops.push(format!("export {name}"));
        self.removed.last_mut().unwrap().remove(name);
        if let Some((_, exported)) = self
            .vars
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
        {
            *exported = true;
        }
    }

    fn unset(&mut self, name: &str) {
        self.ops.push(format!("unset {name}"));
        self.vars.last_mut().unwrap().remove(name);
        // 外側スコープの同名を shadow する tombstone
        self.removed.last_mut().unwrap().insert(name.to_string());
    }

    fn push_scope(&mut self) {
        self.ops.push("(".to_string());
        self.vars.push(BTreeMap::new());
        self.removed.push(BTreeSet::new());
        self.cwd_stack.push(self.cwd.clone());
    }

    fn pop_scope(&mut self) {
        self.ops.push(")".to_string());
        self.vars.pop();
        self.removed.pop();
        if let Some(cwd) = self.cwd_stack.pop() {
            self.cwd = cwd;
        }
    }

    fn nproc(&self) -> usize {
        self.nproc
    }

    fn exec(
        &mut self,
        argv: &[String],
        env: &[(String, String)],
        stdin: Option<&str>,
        stdout: Option<(&str, bool)>,
        stderr: Option<&str>,
        capture: bool,
    ) -> Result<String, Error> {
        let env = env
            .iter()
            .map(|(n, v)| format!("{n}={v}"))
            .collect::<Vec<_>>()
            .join(" ");
        let io = format!(
            "{}{}{}",
            stdin.map(|p| format!(" <{p}")).unwrap_or_default(),
            stdout
                .map(|(p, a)| format!(" {} {}", if a { ">>" } else { ">" }, p))
                .unwrap_or_default(),
            stderr.map(|p| format!(" 2>{p}")).unwrap_or_default(),
        );
        let cap = if capture { " capture" } else { "" };
        self.ops
            .push(format!("exec {env} {}{io}{cap}", argv.join(" ")));
        match self.results.pop_front() {
            Some(Err(code)) => Err(Error::Exit {
                argv: argv.to_vec(),
                code,
            }),
            Some(Ok(out)) => Ok(out),
            None => Ok(String::new()),
        }
    }

    fn pipeline(
        &mut self,
        stages: &[Vec<String>],
        env: &[(String, String)],
        stdin: Option<&str>,
        stdout: Option<(&str, bool)>,
        stderr: Option<&str>,
        capture: bool,
    ) -> Result<String, Error> {
        let env = env
            .iter()
            .map(|(n, v)| format!("{n}={v}"))
            .collect::<Vec<_>>()
            .join(" ");
        let pipe = stages
            .iter()
            .map(|stage| stage.join(" "))
            .collect::<Vec<_>>()
            .join(" | ");
        let io = format!(
            "{}{}{}",
            stdin.map(|p| format!(" <{p}")).unwrap_or_default(),
            stdout
                .map(|(p, a)| format!(" {} {}", if a { ">>" } else { ">" }, p))
                .unwrap_or_default(),
            stderr.map(|p| format!(" 2>{p}")).unwrap_or_default(),
        );
        let cap = if capture { " capture" } else { "" };
        self.ops.push(format!("pipeline {env} {pipe}{io}{cap}"));
        match self.results.pop_front() {
            Some(Err(code)) => Err(Error::Exit {
                argv: stages[0].clone(),
                code,
            }),
            Some(Ok(out)) => Ok(out),
            None => Ok(String::new()),
        }
    }

    fn write_file(&mut self, path: &str, lines: &[String], append: bool) -> Result<(), Error> {
        self.ops.push(format!("write {path} ({}行)", lines.len()));
        let body = if lines.is_empty() {
            String::new()
        } else {
            format!("{}\n", lines.join("\n"))
        };
        if append {
            self.files
                .entry(path.to_string())
                .or_default()
                .push_str(&body);
        } else {
            self.files.insert(path.to_string(), body);
        }
        Ok(())
    }

    fn mkdir_p(&mut self, path: &str) -> Result<(), Error> {
        self.ops.push(format!("mkdir -p {path}"));
        self.dirs.insert(path.to_string());
        Ok(())
    }

    fn copy_into(&mut self, src: &str, dst: &str) -> Result<(), Error> {
        self.ops.push(format!("cp -a {src}/. {dst}/"));
        Ok(())
    }

    fn cd(&mut self, path: &str) -> Result<(), Error> {
        self.ops.push(format!("cd {path}"));
        self.cwd = Some(path.to_string());
        Ok(())
    }

    fn remove_dir(&mut self, path: &str) -> Result<(), Error> {
        self.ops.push(format!("rm -rf {path}"));
        self.dirs.remove(path);
        Ok(())
    }

    fn install(&mut self, src: &str, dst: &str, mode: &str) -> Result<(), Error> {
        self.ops.push(format!("install -D -m {mode} {src} {dst}"));
        self.present_files.insert(dst.to_string());
        Ok(())
    }

    fn symlink(&mut self, target: &str, link: &str, force: bool) -> Result<(), Error> {
        self.ops.push(format!(
            "ln -s{} {target} {link}",
            if force { "f" } else { "" }
        ));
        Ok(())
    }

    fn dir_exists(&self, path: &str) -> bool {
        self.dirs.contains(path)
    }

    fn file_exists(&self, path: &str) -> bool {
        self.present_files.contains(path)
    }

    fn glob(&self, pattern: &str) -> Vec<String> {
        let mut matches: Vec<String> = self
            .present_files
            .iter()
            .chain(self.dirs.iter())
            .filter(|path| glob_match(pattern, path))
            .cloned()
            .collect();
        if matches.is_empty() {
            matches.push(pattern.to_string());
        }
        matches
    }

    fn prepare_env(&mut self, env: Env, std: Std, cross: bool) -> Result<(), Error> {
        self.ops.push(format!(
            "prepare_env {env:?} std={} cross={cross}",
            std.as_str()
        ));
        Ok(())
    }

    fn configure(&mut self, args: &[String], env: Env) -> Result<(), Error> {
        self.ops
            .push(format!("configure {env:?} {}", args.join(" ")));
        Ok(())
    }

    fn pgo_ld_library_path(&mut self) -> Result<(), Error> {
        self.ops.push("pgo_ld_library_path".to_string());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::recipe::{Redirect, argv};
    use crate::target::Target;

    fn ctx() -> BuildContext {
        BuildContext::of(None, Target::BUILD_ENV)
    }

    fn steps(steps: Vec<Step>) -> Recipe {
        Recipe::Steps(Steps {
            env: Env::None,
            std: Std::Gnu17,
            sysroot: false,
            workdir: Workdir::None,
            pgo: None,
            steps,
        })
    }

    fn pipe(argv: &[&str]) -> Pipe {
        Pipe::cmd(argv.iter().map(|s| Arg::owned((*s).to_string())).collect())
    }

    #[test]
    fn exec_records_evaluated_argv() {
        let recipe = steps(vec![
            Step::Set {
                name: "DIR",
                value: Arg::lit("/tmp/x"),
            },
            Step::Exec {
                env: Vec::new(),
                pipe: pipe(&["make", "-C", "${DIR}"]),
            },
        ]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"set DIR=/tmp/x".to_string()));
        assert!(trace.ops.contains(&"exec  make -C /tmp/x".to_string()));
    }

    #[test]
    fn unbound_var_is_an_error() {
        let recipe = steps(vec![Step::Exec {
            env: Vec::new(),
            pipe: pipe(&["echo", "$MISSING"]),
        }]);
        let mut trace = Trace::new();
        let err = run(&recipe, &ctx(), &mut trace).unwrap_err();
        assert!(matches!(err, Error::UnboundVar(name) if name == "MISSING"));
    }

    #[test]
    fn or_falls_back_on_exit() {
        let recipe = steps(vec![Step::Or {
            lhs: vec![Step::Exec {
                env: Vec::new(),
                pipe: pipe(&["false"]),
            }],
            rhs: vec![Step::Exec {
                env: Vec::new(),
                pipe: pipe(&["echo", "fallback"]),
            }],
        }]);
        let mut trace = Trace::new();
        trace.results.push_back(Err(1));
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"exec  echo fallback".to_string()));
    }

    #[test]
    fn or_side_continues_after_intermediate_failure() {
        // bash: `(false; echo ok) || echo fallback` — `||` 側内では
        // set -e が抑制されるので false の後も echo ok が走り、
        // 側の結果は最後のコマンドの状態（成功）。rhs は走らない。
        let recipe = steps(vec![Step::Or {
            lhs: vec![
                Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["false"]),
                },
                Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["echo", "ok"]),
                },
            ],
            rhs: vec![Step::Exec {
                env: Vec::new(),
                pipe: pipe(&["echo", "fallback"]),
            }],
        }]);
        let mut trace = Trace::new();
        trace.results.push_back(Err(1));
        trace.results.push_back(Ok(String::new()));
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"exec  echo ok".to_string()));
        assert!(!trace.ops.contains(&"exec  echo fallback".to_string()));
    }

    #[test]
    fn or_side_scopes_single_multi_line_step() {
        // 単一でも複数行に render されるステップ（If/ForEach 等）は
        // render 側で ( … ) に包まれる → 変数をリークさせない
        let recipe = steps(vec![
            Step::Or {
                lhs: vec![Step::If {
                    cond: Cond::NonEmpty(Arg::lit("x")),
                    then: vec![Step::Set {
                        name: "SCOPED",
                        value: Arg::lit("1"),
                    }],
                    else_: Vec::new(),
                }],
                rhs: vec![Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["true"]),
                }],
            },
            Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("SCOPED")]),
            },
        ]);
        let mut trace = Trace::new();
        let err = run(&recipe, &ctx(), &mut trace).unwrap_err();
        assert!(matches!(err, Error::UnboundVar(name) if name == "SCOPED"));
    }

    #[test]
    fn failing_subst_in_args_expands_empty() {
        // bash: `echo "$(false)"` — $( ) の失敗は外側を止めず
        // 空文字に展開される
        let recipe = steps(vec![Step::Exec {
            env: Vec::new(),
            pipe: Pipe::cmd(vec![
                Arg::lit("echo"),
                Arg::quoted(Arg::Subst(Box::new(pipe(&["false"])))),
            ]),
        }]);
        let mut trace = Trace::new();
        trace.results.push_back(Err(1));
        run(&recipe, &ctx(), &mut trace).unwrap();
        // argv = ["echo", ""]（空文字 1 ワード = bash の "$(f)" と同じ）
        assert!(trace.ops.contains(&"exec  echo ".to_string()));
    }

    #[test]
    fn failing_subst_in_set_propagates_exit() {
        // bash: `X=$(false)` — 裸の代入は $( ) の終了コードを継承し
        // set -e で中断する
        let recipe = steps(vec![Step::Set {
            name: "X",
            value: Arg::Subst(Box::new(pipe(&["false"]))),
        }]);
        let mut trace = Trace::new();
        trace.results.push_back(Err(1));
        let err = run(&recipe, &ctx(), &mut trace).unwrap_err();
        assert!(matches!(err, Error::Exit { code: 1, .. }));
    }

    #[test]
    fn subshell_does_not_leak_vars() {
        let recipe = steps(vec![
            Step::Subshell(vec![Step::Set {
                name: "INNER",
                value: Arg::lit("x"),
            }]),
            Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("INNER")]),
            },
        ]);
        let mut trace = Trace::new();
        let err = run(&recipe, &ctx(), &mut trace).unwrap_err();
        assert!(matches!(err, Error::UnboundVar(_)));
    }

    #[test]
    fn foreach_iterates_split_words() {
        let recipe = steps(vec![Step::ForEach {
            var: "D",
            items: vec![Arg::lit("a b c")],
            body: vec![Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("D")]),
            }],
            stdout: None,
        }]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        let echoes: Vec<_> = trace
            .ops
            .iter()
            .filter(|op| op.starts_with("exec"))
            .collect();
        assert_eq!(echoes.len(), 3, "{echoes:?}");
    }

    #[test]
    fn pgo_autotools_runs_phases() {
        let recipe = Recipe::Steps(Steps {
            env: Env::Autotools,
            std: Std::Gnu17,
            sysroot: false,
            workdir: Workdir::Build,
            pgo: Some(PgoTemplate::Autotools(vec![Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["pgo-workload-cmd"])),
            }])),
            steps: vec![
                Step::Configure(vec!["--enable-x".to_string()]),
                Step::make(&[]),
                Step::make_install(&[]),
            ],
        });
        let mut trace = Trace::new()
            .with_var("BUILD", "/build")
            .with_var("OUT", "/out");
        run(&recipe, &ctx(), &mut trace).unwrap();
        let ops = trace.ops.join("\n");
        // configure → generate → make → workload → distclean →
        // use → configure → make → install の順で記録される。
        let configure = ops.match_indices("configure").count();
        let makes = ops.match_indices("make -j8").count();
        assert_eq!(configure, 2, "{ops}");
        assert_eq!(makes, 2, "{ops}");
        assert!(ops.contains("pgo-workload-cmd"), "{ops}");
        assert!(ops.contains("CFLAGS"), "{ops}");
    }

    #[test]
    fn cross_skips_pgo() {
        let target = Target::Arm64(crate::target::Arm64Variant::V8);
        let ctx = BuildContext::of(None, target);
        let recipe = Recipe::Steps(Steps {
            env: Env::None,
            std: Std::Gnu17,
            sysroot: false,
            workdir: Workdir::None,
            pgo: Some(PgoTemplate::Make(vec![Step::Exec {
                env: vec![],
                pipe: Pipe::cmd(argv(&["pgo-workload-cmd"])),
            }])),
            steps: vec![Step::make(&[])],
        });
        let mut trace = Trace::new();
        run(&recipe, &ctx, &mut trace).unwrap();
        assert!(!trace.ops.join("\n").contains("pgo-workload-cmd"));
        assert!(trace.ops.iter().any(|op| op.contains("make -j")));
    }

    #[test]
    fn colon_plus_expands_when_set() {
        let recipe = steps(vec![Step::Exec {
            env: Vec::new(),
            pipe: pipe(&["echo", "x${FOO:+:$FOO}y", "${BAR:+missing}z"]),
        }]);
        let mut trace = Trace::new().with_var("FOO", "v");
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"exec  echo x:vy z".to_string()));
    }

    #[test]
    fn comment_is_noop() {
        let recipe = steps(vec![
            Step::Comment("note".into()),
            Step::Set {
                name: "A",
                value: Arg::lit("1"),
            },
        ]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"set A=1".to_string()));
        assert!(
            trace.ops.iter().all(|op| !op.contains("note")),
            "{:?}",
            trace.ops
        );
    }

    #[test]
    fn subst_or_swallows_failure() {
        // `$(command -v x || true)`: 失敗しても空文字で続行する
        let recipe = steps(vec![
            Step::Set {
                name: "SRC",
                value: Arg::SubstOr(Box::new(pipe(&["command", "-v", "x"]))),
            },
            Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("SRC")]),
            },
        ]);
        let mut trace = Trace::new();
        trace.results.push_back(Err(1));
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"set SRC=".to_string()));
        assert!(trace.ops.contains(&"exec  echo".to_string()));
    }

    #[test]
    fn cond_variants_drive_branches() {
        let recipe = steps(vec![
            Step::Install {
                src: "a".into(),
                dst: "/out/a".into(),
                mode: "755".into(),
            },
            Step::If {
                cond: Cond::And(vec![
                    Cond::FileExists(Arg::lit("/out/a")),
                    Cond::Executable(Arg::lit("/out/a")),
                    Cond::NonEmpty(Arg::lit("x")),
                    Cond::Not(Box::new(Cond::DirExists(Arg::lit("/missing")))),
                ]),
                then: vec![Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["echo", "hit"]),
                }],
                else_: vec![Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["echo", "miss"]),
                }],
            },
            Step::If {
                cond: Cond::Exists(Arg::lit("/out/a")),
                then: vec![Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["echo", "exists"]),
                }],
                else_: vec![],
            },
        ]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(
            trace.ops.contains(&"exec  echo hit".to_string()),
            "{:?}",
            trace.ops
        );
        assert!(
            trace.ops.contains(&"exec  echo exists".to_string()),
            "{:?}",
            trace.ops
        );
        assert!(!trace.ops.contains(&"exec  echo miss".to_string()));
    }

    #[test]
    fn write_file_records_and_expands() {
        let recipe = steps(vec![
            Step::Set {
                name: "V",
                value: Arg::lit("world"),
            },
            Step::WriteFile {
                path: Arg::lit("/out/f"),
                lines: vec![Arg::lit("hello"), Arg::lit("$V")],
                append: false,
                expand: true,
            },
        ]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert_eq!(
            trace.files.get("/out/f").map(String::as_str),
            Some("hello\nworld\n"),
            "{:?}",
            trace.files
        );
    }

    #[test]
    fn double_quoted_section_preserves_spaces() {
        // `"$CFLAGS -fpic"`: クォート内は分割しないがスペースは残る
        let recipe = steps(vec![Step::Exec {
            env: Vec::new(),
            pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::lit("\"$CFLAGS -fpic\"")]),
        }]);
        let mut trace = Trace::new().with_var("CFLAGS", "-O2 -g");
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(
            trace.ops.contains(&"exec  echo -O2 -g -fpic".to_string()),
            "{:?}",
            trace.ops
        );
    }

    #[test]
    fn make_flag_overrides_are_argv_words() {
        // `make CFLAGS=…` はコマンドライン変数（argv）であって env ではない
        let recipe = Recipe::Steps(Steps {
            env: Env::Autotools,
            std: Std::Gnu17,
            sysroot: false,
            workdir: Workdir::None,
            pgo: None,
            steps: vec![Step::Make {
                jobs: false,
                args: vec![],
            }],
        });
        let mut trace = Trace::new().with_var("CFLAGS", "-O2");
        run(&recipe, &ctx(), &mut trace).unwrap();
        // env 欄（`exec ` と argv の間）に CFLAGS が来ないこと
        assert!(
            trace
                .ops
                .contains(&"exec  make CFLAGS=-O2 CXXFLAGS= LDFLAGS=".to_string()),
            "{:?}",
            trace.ops
        );
    }

    #[test]
    fn exec_passes_stdin_and_stderr_redirects() {
        let recipe = steps(vec![Step::Exec {
            env: Vec::new(),
            pipe: Pipe::cmd(vec![Arg::lit("cc"), Arg::lit("-")])
                .stdin(Arg::lit("/tmp/in.c"))
                .stderr(Arg::lit("/dev/null")),
        }]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(
            trace
                .ops
                .contains(&"exec  cc - </tmp/in.c 2>/dev/null".to_string()),
            "{:?}",
            trace.ops
        );
    }

    #[test]
    fn write_file_expand_keeps_verbatim_text() {
        // `<<EOF` 相当: `$` のみ展開、クォート・空白・インデントは維持
        let recipe = steps(vec![Step::WriteFile {
            path: Arg::lit("/out/f"),
            lines: vec![Arg::lit("  c = '$CC'")],
            append: false,
            expand: true,
        }]);
        let mut trace = Trace::new().with_var("CC", "x86_64-linux-gnu-gcc");
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert_eq!(
            trace.files.get("/out/f").map(String::as_str),
            Some("  c = 'x86_64-linux-gnu-gcc'\n"),
            "{:?}",
            trace.files
        );
    }

    #[test]
    fn subshell_unset_shadows_outer_export() {
        let recipe = steps(vec![
            Step::Export {
                name: "CFLAGS",
                value: Arg::lit("-O2"),
            },
            Step::Subshell(vec![
                Step::Unset("CFLAGS"),
                Step::Exec {
                    env: Vec::new(),
                    pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("CFLAGS")]),
                },
            ]),
        ]);
        let mut trace = Trace::new();
        let err = run(&recipe, &ctx(), &mut trace).unwrap_err();
        assert!(matches!(err, Error::UnboundVar(n) if n == "CFLAGS"));
    }

    #[test]
    fn multi_step_or_scopes_side_effects() {
        let recipe = steps(vec![
            Step::Or {
                lhs: vec![
                    Step::Set {
                        name: "LEAK",
                        value: Arg::lit("x"),
                    },
                    Step::Exec {
                        env: Vec::new(),
                        pipe: pipe(&["false"]),
                    },
                ],
                rhs: vec![Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["echo", "rhs"]),
                }],
            },
            Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("LEAK")]),
            },
        ]);
        let mut trace = Trace::new();
        trace.results.push_back(Err(1));
        let err = run(&recipe, &ctx(), &mut trace).unwrap_err();
        // lhs の Set は ( … ) 相当のスコープに閉じるため外から見えない
        assert!(matches!(err, Error::UnboundVar(n) if n == "LEAK"));
    }

    #[test]
    fn foreach_stdout_redirect_truncates_once() {
        // `for … done > f`: ループ先頭で一度だけ切り詰め、
        // 各反復のコマンド出力は追記される
        let recipe = steps(vec![Step::ForEach {
            var: "I",
            items: vec![Arg::lit("a b")],
            body: vec![Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("I")]),
            }],
            stdout: Some(Redirect {
                target: Arg::lit("/tmp/f"),
                append: false,
            }),
        }]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(
            trace.ops.contains(&"write /tmp/f (0行)".to_string()),
            "{:?}",
            trace.ops
        );
        assert!(
            trace.ops.contains(&"exec  echo a >> /tmp/f".to_string()),
            "{:?}",
            trace.ops
        );
        assert!(
            trace.ops.contains(&"exec  echo b >> /tmp/f".to_string()),
            "{:?}",
            trace.ops
        );
    }

    #[test]
    fn cd_updates_pwd() {
        let recipe = steps(vec![
            Step::Exec {
                env: Vec::new(),
                pipe: pipe(&["mkdir", "-p", "/work"]),
            },
            Step::Cd(Arg::lit("/work")),
            Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("PWD")]),
            },
        ]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"cd /work".to_string()));
        assert!(trace.ops.contains(&"set PWD=/work".to_string()));
        assert!(trace.ops.contains(&"exec  echo /work".to_string()));
    }

    #[test]
    fn run_plan_prelude_is_lax() {
        // prelude の失敗・未定義変数参照は握り潰して recipe 本体へ進む
        let plan = BuildPlan {
            prelude: vec![
                Step::Exec {
                    env: Vec::new(),
                    pipe: pipe(&["false"]),
                },
                Step::Exec {
                    env: Vec::new(),
                    pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::Var("MISSING")]),
                },
            ],
            recipe: steps(vec![Step::Exec {
                env: Vec::new(),
                pipe: pipe(&["echo", "body"]),
            }]),
        };
        let mut trace = Trace::new();
        trace.results.push_back(Err(1));
        run_plan(&plan, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.contains(&"exec  echo body".to_string()));
    }

    #[test]
    fn empty_pipeline_is_noop() {
        let recipe = steps(vec![Step::Exec {
            env: Vec::new(),
            pipe: Pipe::cmd(vec![Arg::Var("EMPTY")]),
        }]);
        let mut trace = Trace::new().with_var("EMPTY", "");
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(trace.ops.iter().all(|op| !op.starts_with("exec ")));
    }

    #[test]
    fn make_args_are_expanded() {
        let recipe = steps(vec![
            Step::Set {
                name: "EXTRA",
                value: Arg::lit("-O0"),
            },
            Step::Make {
                jobs: false,
                args: vec!["CFLAGS=$EXTRA -Wall".to_string()],
            },
        ]);
        let mut trace = Trace::new();
        run(&recipe, &ctx(), &mut trace).unwrap();
        assert!(
            trace
                .ops
                .contains(&"exec  make CFLAGS=-O0 -Wall".to_string()),
            "{:?}",
            trace.ops
        );
    }
}
