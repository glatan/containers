//! コンテナ内実行 sandbox。`run-recipe` が IR を直接解釈する際の
//! 実環境実装。`std::process::Command` + `std::fs` で bash 相当の
//! 意味論を再現する:
//!
//! - 変数はスコープスタック + tombstone（`( unset X )` の shadow）
//! - 子プロセス env は `env_clear` + export 済み変数のみ + 呼出 override
//!   （`FOO=bar cmd` は export 無しでも子へ渡る bash 規則に合わせる）
//! - ENOENT は `Exit{127}`、`command`/`sanitize_flags` は builtin として
//!   intercept、`+ argv` を stderr へ出して `set -x` と揃える
//! - pipeline は pipefail（最右非ゼロコード）、末尾ステージに
//!   stdout/stderr リダイレクトを適用
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};

use super::recipe::{CROSS_CACHE_VARS, Env, Std};
use super::runner::{Error, Sandbox, glob_match};

/// 実環境 `Sandbox`。コンテナ内でそのまま動かす他、tempdir 上の
/// ホスト検証にも使える（外部依存は `PATH` 上のコマンドのみ）。
pub struct ContainerSandbox {
    /// 変数スコープ。値は `(値, export 済み)`。scope[0] は起動時の
    /// プロセス環境（bash の初期 env 全 export と同じ）。
    scopes: Vec<BTreeMap<String, (String, bool)>>,
    /// スコープ毎の unset tombstone（外側スコープの同名を shadow）。
    removed: Vec<BTreeSet<String>>,
    /// `Subshell` 用の cwd スタック。
    cwd_stack: Vec<PathBuf>,
}

impl ContainerSandbox {
    /// 現在のプロセス環境を scope[0]（全 export 済み）として作る。
    pub fn new() -> Self {
        Self {
            scopes: vec![env::vars().map(|(k, v)| (k, (v, true))).collect()],
            removed: vec![BTreeSet::new()],
            cwd_stack: Vec::new(),
        }
    }

    /// name の有効なスロット（内側スコープ優先、tombstone は None）。
    fn resolve(&self, name: &str) -> Option<&(String, bool)> {
        for (i, scope) in self.scopes.iter().enumerate().rev() {
            if self.removed[i].contains(name) {
                return None;
            }
            if let Some(slot) = scope.get(name) {
                return Some(slot);
            }
        }
        None
    }

    /// 子プロセスへ渡す env（export 済み変数のみ）。
    fn child_env(&self) -> Vec<(&String, &String)> {
        let mut names: BTreeSet<&String> = BTreeSet::new();
        for scope in &self.scopes {
            names.extend(scope.keys());
        }
        names
            .into_iter()
            .filter_map(|name| match self.resolve(name) {
                Some((value, true)) => Some((name, value)),
                _ => None,
            })
            .collect()
    }

    /// builtin（シェル関数・builtin 相当）の実行。`Some` は結果
    /// （出力バイト列 or Exit コード）。`command -v` と
    /// `sanitize_flags` を提供する。
    fn run_builtin(&self, argv: &[String]) -> Option<Result<Vec<u8>, i32>> {
        match argv.first().map(String::as_str) {
            Some("command") => {
                let rest = &argv[1..];
                if rest.first().map(String::as_str) == Some("-v") {
                    let mut out = Vec::new();
                    let mut code = 0;
                    for name in &rest[1..] {
                        match self.which(name) {
                            Some(path) => out.extend_from_slice(format!("{path}\n").as_bytes()),
                            // bash の `command -v` miss は exit 1
                            None => code = 1,
                        }
                    }
                    Some(if code == 0 { Ok(out) } else { Err(code) })
                } else {
                    // `command foo …` — function  bypass 相当で実コマンドを
                    // 呼ぶ（builtin 一覧に無いものは外部コマンドとして続行）。
                    None
                }
            }
            // `sanitize_flags`（configure probe 用の sed 相当を再現）
            Some("sanitize_flags") => {
                let input = argv.get(1).map(String::as_str).unwrap_or("");
                Some(Ok(sanitize_flags(input).into_bytes()))
            }
            _ => None,
        }
    }

    /// `command -v` 相当: sandbox の現在 PATH から実行可能ファイルを探す。
    fn which(&self, name: &str) -> Option<String> {
        if name.contains('/') {
            return is_exec(Path::new(name)).then(|| name.to_string());
        }
        let path = self.get("PATH").unwrap_or_default();
        for dir in path.split(':') {
            let candidate = format!("{dir}/{name}");
            if is_exec(Path::new(&candidate)) {
                return Some(candidate);
            }
        }
        None
    }
}

impl Default for ContainerSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// `path` が実行ビットを持つか（`[ -x ]` 相当。symlink 辿り・dir も真）。
fn is_exec(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|m| m.permissions().mode() & 0o111 != 0)
}

/// `sanitize_flags()` の sed 相当: `-flto*`・LTO 系 `-Werror=*`・
/// `-f{,no-}hardened` を左から順に除去する（最長一致優先）。
fn sanitize_flags(value: &str) -> String {
    const PATTERNS: &[&str] = &[
        "-flto=full",
        "-flto=auto",
        "-flto=thin",
        "-flto",
        "-Werror=lto-type-mismatch",
        "-Werror=strict-aliasing",
        "-Werror=odr",
        "-fno-hardened",
        "-fhardened",
    ];
    let mut out = String::with_capacity(value.len());
    let mut rest = value;
    while let Some(ch) = rest.chars().next() {
        if ch == '-'
            && let Some(pat) = PATTERNS.iter().find(|p| rest.starts_with(**p))
        {
            rest = &rest[pat.len()..];
            continue;
        }
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

/// builtin 出力の吐き先。`capture` なら返り値、stdout リダイレクトが
/// あればファイルへ、どちらも無ければ継承 stdout へ出す。
fn emit(output: Vec<u8>, stdout: Option<(&str, bool)>, capture: bool) -> Result<String, Error> {
    if let Some((path, append)) = stdout {
        write_lines(Path::new(path), &[output], append)?;
        return Ok(String::new());
    }
    if capture {
        return Ok(String::from_utf8_lossy(&output).into_owned());
    }
    let mut out = std::io::stdout();
    out.write_all(&output)
        .and_then(|()| out.flush())
        .map_err(|e| Error::Io(format!("stdout への書き込み失敗: {e}")))?;
    Ok(String::new())
}

/// `> path`/`>> path`/`2> path` 用のファイルを開く。
fn open_redirect(path: &str, append: bool) -> Result<File, Error> {
    OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(!append)
        .append(append)
        .open(path)
        .map_err(|_| {
            // bash のリダイレクト失敗はそのコマンドの exit 1
            Error::Exit {
                argv: vec![format!("> {path}")],
                code: 1,
            }
        })
}

/// 行を書き出す（`cat <<EOF`/`printf` 相当）。
fn write_lines(path: &Path, lines: &[Vec<u8>], append: bool) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(!append)
        .append(append)
        .open(path)
        .map_err(|e| exit_io("write", path, e))?;
    for line in lines {
        file.write_all(line)
            .and_then(|()| file.write_all(b"\n"))
            .map_err(|e| exit_io("write", path, e))?;
    }
    Ok(())
}

/// fs 操作失敗を bash の exit 1 相当にする。
fn exit_io(op: &str, path: &Path, e: std::io::Error) -> Error {
    Error::Io(format!("{op} {}: {e}", path.display()))
}

/// コマンドの exit code（シグナルは bash と同じく 128+signo）。
fn exit_code(status: std::process::ExitStatus) -> i32 {
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

/// `cp -a src/. dst/` 相当の再帰コピー（symlink 保存・perm 維持）。
fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let to = dst.join(entry.file_name());
        if ty.is_symlink() {
            let target = fs::read_link(entry.path())?;
            let _ = fs::remove_file(&to);
            std::os::unix::fs::symlink(target, &to)?;
        } else if ty.is_dir() {
            copy_tree(&entry.path(), &to)?;
        } else {
            fs::copy(entry.path(), &to)?;
            // `cp -a` parity: mtime を保持しないと Makefile.in 再生成系の
            // タイムスタンプ比較が壊れる（flex の automake 再実行等）。
            if let Ok(meta) = entry.metadata()
                && let Ok(mtime) = meta.modified()
                && let Ok(f) = File::options().write(true).open(&to)
            {
                let _ = f.set_modified(mtime);
            }
        }
    }
    Ok(())
}

/// `find dir -type d -name .libs` 相当の収集。
fn find_named_dirs(dir: &Path, name: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(ty) = entry.file_type() else {
            continue;
        };
        if !ty.is_dir() {
            continue;
        }
        if entry.file_name() == name {
            out.push(entry.path());
        } else {
            find_named_dirs(&entry.path(), name, out);
        }
    }
}

/// glob 展開（`*`/`?`。文字クラスは未対応 — `glob_match` と同じ制約）。
/// bash の non-nullglob と同じくマッチ無しならパターンをそのまま返す。
fn expand_glob(pattern: &str) -> Vec<String> {
    let mut cur: Vec<String> = if pattern.starts_with('/') {
        vec!["/".to_string()]
    } else {
        vec![String::new()]
    };
    for comp in pattern.split('/') {
        if comp.is_empty() {
            continue;
        }
        let mut next = Vec::new();
        if comp.contains(['*', '?']) {
            for base in &cur {
                let dir = if base.is_empty() {
                    Path::new(".")
                } else {
                    Path::new(base.as_str())
                };
                if let Ok(rd) = fs::read_dir(dir) {
                    for entry in rd.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        if glob_match(comp, &name) {
                            next.push(join(base, &name));
                        }
                    }
                }
            }
        } else {
            for base in &cur {
                next.push(join(base, comp));
            }
        }
        cur = next;
        if cur.is_empty() {
            break;
        }
    }
    if cur.is_empty() {
        vec![pattern.to_string()]
    } else {
        cur.sort();
        cur
    }
}

fn join(base: &str, comp: &str) -> String {
    if base.is_empty() || base.ends_with('/') {
        format!("{base}{comp}")
    } else {
        format!("{base}/{comp}")
    }
}

impl Sandbox for ContainerSandbox {
    fn get(&self, name: &str) -> Option<String> {
        self.resolve(name).map(|(value, _)| value.clone())
    }

    fn set(&mut self, name: &str, value: &str) {
        self.removed.last_mut().unwrap().remove(name);
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), (value.to_string(), false));
    }

    fn export(&mut self, name: &str, value: &str) {
        self.removed.last_mut().unwrap().remove(name);
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), (value.to_string(), true));
    }

    fn reexport(&mut self, name: &str) {
        self.removed.last_mut().unwrap().remove(name);
        if let Some((_, exported)) = self
            .scopes
            .iter_mut()
            .rev()
            .find_map(|scope| scope.get_mut(name))
        {
            *exported = true;
        }
    }

    fn unset(&mut self, name: &str) {
        self.scopes.last_mut().unwrap().remove(name);
        self.removed.last_mut().unwrap().insert(name.to_string());
    }

    fn push_scope(&mut self) {
        self.scopes.push(BTreeMap::new());
        self.removed.push(BTreeSet::new());
        self.cwd_stack.push(env::current_dir().unwrap_or_default());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
        self.removed.pop();
        if let Some(cwd) = self.cwd_stack.pop() {
            let _ = env::set_current_dir(cwd);
        }
    }

    fn nproc(&self) -> usize {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
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
        if argv.is_empty() {
            return Ok(String::new());
        }
        if let Some(result) = self.run_builtin(argv) {
            return match result {
                Ok(bytes) => emit(bytes, stdout, capture),
                Err(code) => Err(Error::Exit {
                    argv: argv.to_vec(),
                    code,
                }),
            };
        }
        eprintln!("+ {}", argv.join(" "));
        let mut cmd = Command::new(&argv[0]);
        cmd.args(&argv[1..]).env_clear();
        for (name, value) in self.child_env() {
            cmd.env(name, value);
        }
        for (name, value) in env {
            cmd.env(name, value);
        }
        if let Some(path) = stdin {
            let file = File::open(path).map_err(|e| exit_io("open", Path::new(path), e))?;
            cmd.stdin(Stdio::from(file));
        }
        let capture_stdout = capture && stdout.is_none();
        match stdout {
            Some((path, append)) => cmd.stdout(Stdio::from(open_redirect(path, append)?)),
            None if capture_stdout => cmd.stdout(Stdio::piped()),
            None => cmd.stdout(Stdio::inherit()),
        };
        match stderr {
            Some(path) => cmd.stderr(Stdio::from(open_redirect(path, false)?)),
            None => cmd.stderr(Stdio::inherit()),
        };
        let child = match cmd.spawn() {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Err(Error::Exit {
                    argv: argv.to_vec(),
                    code: 127,
                });
            }
            Err(e) => return Err(exit_io("spawn", Path::new(&argv[0]), e)),
            Ok(child) => child,
        };
        let output = child
            .wait_with_output()
            .map_err(|e| exit_io("wait", Path::new(&argv[0]), e))?;
        let status = output.status;
        if !status.success() {
            return Err(Error::Exit {
                argv: argv.to_vec(),
                code: exit_code(status),
            });
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
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
        if stages.len() == 1 {
            return self.exec(&stages[0], env, stdin, stdout, stderr, capture);
        }
        eprintln!(
            "+ {}",
            stages
                .iter()
                .map(|stage| stage.join(" "))
                .collect::<Vec<_>>()
                .join(" | ")
        );
        let last = stages.len() - 1;
        // 各ステージの終了コード（None=未実行/実行中）
        let mut codes: Vec<Option<i32>> = vec![None; stages.len()];
        let mut children: Vec<Option<Child>> = Vec::new();
        // 前ステージの stdout（次ステージの stdin へ繋ぐ）
        let mut prev: Option<Stdio> = None;
        // builtin ステージの出力（次ステージの stdin へ書き込む）
        let mut feed: Option<Vec<u8>> = None;
        for (i, stage) in stages.iter().enumerate() {
            if let Some(result) = self.run_builtin(stage) {
                match result {
                    Ok(bytes) => {
                        feed = Some(bytes);
                        codes[i] = Some(0);
                    }
                    Err(code) => {
                        feed = Some(Vec::new());
                        codes[i] = Some(code);
                    }
                }
                children.push(None);
                prev = None;
                continue;
            }
            let mut cmd = Command::new(&stage[0]);
            cmd.args(&stage[1..]).env_clear();
            for (name, value) in self.child_env() {
                cmd.env(name, value);
            }
            // stdin: builtin feed > 前ステージ stdout > i==0 の <file
            let feed_bytes = feed.take();
            if feed_bytes.is_some() {
                cmd.stdin(Stdio::piped());
            } else if let Some(stdio) = prev.take() {
                cmd.stdin(stdio);
            } else if i == 0
                && let Some(path) = stdin
            {
                let file = File::open(path).map_err(|e| exit_io("open", Path::new(path), e))?;
                cmd.stdin(Stdio::from(file));
            }
            // stdout: 末尾のみ redirect/capture/inherit、中間は piped
            if i == last {
                match stdout {
                    Some((path, append)) => cmd.stdout(Stdio::from(open_redirect(path, append)?)),
                    None if capture => cmd.stdout(Stdio::piped()),
                    None => cmd.stdout(Stdio::inherit()),
                };
                match stderr {
                    Some(path) => cmd.stderr(Stdio::from(open_redirect(path, false)?)),
                    None => cmd.stderr(Stdio::inherit()),
                };
            } else {
                cmd.stdout(Stdio::piped());
            }
            let mut child = match cmd.spawn() {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    return Err(Error::Exit {
                        argv: stage.clone(),
                        code: 127,
                    });
                }
                Err(e) => return Err(exit_io("spawn", Path::new(&stage[0]), e)),
                Ok(child) => child,
            };
            if let Some(bytes) = feed_bytes
                && let Some(mut pipe) = child.stdin.take()
            {
                let _ = pipe.write_all(&bytes);
            }
            if i < last {
                prev = child.stdout.take().map(Stdio::from);
            }
            children.push(Some(child));
        }
        let mut output = String::new();
        for (i, slot) in children.iter_mut().enumerate() {
            let Some(mut child) = slot.take() else {
                continue;
            };
            if i == last {
                let out = child
                    .wait_with_output()
                    .map_err(|e| exit_io("wait", Path::new(&stages[i][0]), e))?;
                if capture && stdout.is_none() {
                    output = String::from_utf8_lossy(&out.stdout).into_owned();
                }
                codes[i] = Some(exit_code(out.status));
            } else {
                let status = child
                    .wait()
                    .map_err(|e| exit_io("wait", Path::new(&stages[i][0]), e))?;
                codes[i] = Some(exit_code(status));
            }
        }
        // pipefail: 最右の非ゼロコードを返す
        if let Some((i, code)) = codes
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, c)| c.filter(|c| *c != 0).map(|c| (i, c)))
        {
            return Err(Error::Exit {
                argv: stages[i].clone(),
                code,
            });
        }
        Ok(output)
    }

    fn write_file(&mut self, path: &str, lines: &[String], append: bool) -> Result<(), Error> {
        let bytes: Vec<Vec<u8>> = lines.iter().map(|l| l.as_bytes().to_vec()).collect();
        write_lines(Path::new(path), &bytes, append)
    }

    fn mkdir_p(&mut self, path: &str) -> Result<(), Error> {
        fs::create_dir_all(path).map_err(|e| exit_io("mkdir -p", Path::new(path), e))
    }

    fn copy_into(&mut self, src: &str, dst: &str) -> Result<(), Error> {
        copy_tree(Path::new(src), Path::new(dst)).map_err(|e| exit_io("cp -a", Path::new(src), e))
    }

    fn cd(&mut self, path: &str) -> Result<(), Error> {
        env::set_current_dir(path).map_err(|e| {
            // bash の `cd` 失敗は exit 1
            let _ = e;
            Error::Exit {
                argv: vec!["cd".to_string(), path.to_string()],
                code: 1,
            }
        })
    }

    fn remove_dir(&mut self, path: &str) -> Result<(), Error> {
        match fs::remove_dir_all(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result.map_err(|e| exit_io("rm -rf", Path::new(path), e)),
        }
    }

    fn install(&mut self, src: &str, dst: &str, mode: &str) -> Result<(), Error> {
        let dst = Path::new(dst);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| exit_io("install -D", parent, e))?;
        }
        fs::copy(src, dst).map_err(|e| exit_io("install -D", dst, e))?;
        let mode = u32::from_str_radix(mode, 8).unwrap_or(0o755);
        fs::set_permissions(dst, fs::Permissions::from_mode(mode))
            .map_err(|e| exit_io("install -D", dst, e))
    }

    fn symlink(&mut self, target: &str, link: &str, force: bool) -> Result<(), Error> {
        let link = Path::new(link);
        if force && link.symlink_metadata().is_ok() {
            fs::remove_file(link).map_err(|e| exit_io("ln -sf", link, e))?;
        }
        std::os::unix::fs::symlink(target, link).map_err(|e| exit_io("ln -s", link, e))
    }

    fn dir_exists(&self, path: &str) -> bool {
        Path::new(path).is_dir()
    }

    fn file_exists(&self, path: &str) -> bool {
        Path::new(path).is_file()
    }

    fn is_executable(&self, path: &str) -> bool {
        is_exec(Path::new(path))
    }

    fn path_exists(&self, path: &str) -> bool {
        // `[ -e ]` 相当（シンボリックリンクを辿る）
        Path::new(path).try_exists().unwrap_or(false)
    }

    fn glob(&self, pattern: &str) -> Vec<String> {
        expand_glob(pattern)
    }

    fn prepare_env(&mut self, env: Env, std: Std, cross: bool) -> Result<(), Error> {
        if matches!(env, Env::None) {
            return Ok(());
        }
        self.export("FORCE_UNSAFE_CONFIGURE", "1");
        if matches!(env, Env::Autotools | Env::Lto) {
            let cflags = self.get("CFLAGS").unwrap_or_default();
            self.export("CFLAGS", &format!("{cflags} -std={}", std.as_str()));
        }
        if cross {
            let host = self.get("HOST_TRIPLE").unwrap_or_default();
            self.export("CC_FOR_BUILD", &format!("{host}-gcc"));
            self.export("CXX_FOR_BUILD", &format!("{host}-g++"));
            let cc = self.get("CC_FOR_BUILD").unwrap_or_default();
            self.export("BUILD_CC", &cc);
            if matches!(env, Env::Autotools) {
                for (name, value) in CROSS_CACHE_VARS {
                    self.export(name, value);
                }
            }
        }
        Ok(())
    }

    fn configure(&mut self, args: &[String], env: Env) -> Result<(), Error> {
        let source = self
            .get("SOURCE")
            .ok_or_else(|| Error::UnboundVar("SOURCE".to_string()))?;
        let host_triple = self
            .get("HOST_TRIPLE")
            .ok_or_else(|| Error::UnboundVar("HOST_TRIPLE".to_string()))?;
        let host = self
            .get("PACKAGE_HOST")
            .filter(|v| !v.is_empty())
            .or_else(|| self.get("TRIPLE"))
            .ok_or_else(|| Error::UnboundVar("TRIPLE".to_string()))?;
        // Workdir::InSource はソースを BUILD へコピーしてからビルドする。
        // srcdir が read-only の $SOURCE だと version.texi 再生成のような
        // srcdir 書き込み rule で失敗するため、コピー済みの ./configure を優先。
        let configure = if Path::new("configure").is_file() {
            "./configure".to_owned()
        } else {
            format!("{source}/configure")
        };
        let mut argv = vec![
            configure,
            format!("--build={host_triple}"),
            format!("--host={host}"),
        ];
        argv.extend(args.iter().cloned());
        // Env::Autotools は configure probe 用にフラグを剥がす
        // （sanitize_flags 相当）。Env::Lto はフルフラグのまま渡す。
        let overrides: Vec<(String, String)> = if matches!(env, Env::Autotools) {
            ["CFLAGS", "CXXFLAGS", "LDFLAGS"]
                .into_iter()
                .map(|name| {
                    (
                        name.to_string(),
                        sanitize_flags(&self.get(name).unwrap_or_default()),
                    )
                })
                .collect()
        } else {
            Vec::new()
        };
        match self.exec(&argv, &overrides, None, None, None, false) {
            Err(e @ Error::Exit { .. }) => {
                // bash: `if [ -f config.log ]; then echo…; tail -n 80; fi`
                if let Ok(content) = fs::read_to_string("config.log") {
                    let lines: Vec<&str> = content.lines().collect();
                    let tail = &lines[lines.len().saturating_sub(80)..];
                    println!("===== config.log (last 80 lines) =====");
                    println!("{}", tail.join("\n"));
                }
                Err(e)
            }
            result => result.map(|_| ()),
        }
    }

    fn pgo_ld_library_path(&mut self) -> Result<(), Error> {
        let pwd = env::current_dir().map_err(|e| Error::Io(format!("getcwd: {e}")))?;
        // `export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"`
        let mut ld = match self.get("LD_LIBRARY_PATH") {
            Some(prev) if !prev.is_empty() => format!("{}:{prev}", pwd.display()),
            _ => pwd.display().to_string(),
        };
        let mut dirs = vec![pwd.join("lib"), pwd.join("programs")];
        find_named_dirs(&pwd, ".libs", &mut dirs);
        for dir in dirs {
            if dir.is_dir() {
                ld = format!("{}:{ld}", dir.display());
            }
        }
        self.export("LD_LIBRARY_PATH", &ld);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::package::recipe::{Arg, Pipe, Recipe, Step, Steps, Workdir};
    use crate::package::runner;
    use crate::target::Target;

    fn argv(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| w.to_string()).collect()
    }

    fn tempdir(tag: &str) -> PathBuf {
        let dir = env::temp_dir().join(format!("sandbox-test-{}-{tag}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn exec_captures_stdout() {
        let mut sb = ContainerSandbox::new();
        let out = sb
            .exec(&argv(&["echo", "hello"]), &[], None, None, None, true)
            .unwrap();
        assert_eq!(out, "hello\n");
    }

    #[test]
    fn enoent_maps_to_exit_127() {
        let mut sb = ContainerSandbox::new();
        let err = sb
            .exec(
                &argv(&["definitely-not-a-command-xyz"]),
                &[],
                None,
                None,
                None,
                false,
            )
            .unwrap_err();
        assert!(matches!(err, Error::Exit { code: 127, .. }));
    }

    #[test]
    fn exit_code_propagates() {
        let mut sb = ContainerSandbox::new();
        let err = sb
            .exec(&argv(&["false"]), &[], None, None, None, false)
            .unwrap_err();
        assert!(matches!(err, Error::Exit { code: 1, .. }));
    }

    #[test]
    fn child_env_has_only_exported_vars() {
        let mut sb = ContainerSandbox::new();
        sb.export("SB_EXPORTED", "yes");
        sb.set("SB_LOCAL", "no");
        let out = sb
            .exec(
                &argv(&[
                    "sh",
                    "-c",
                    "printf %s ${SB_EXPORTED-unset}:${SB_LOCAL-unset}",
                ]),
                &[],
                None,
                None,
                None,
                true,
            )
            .unwrap();
        assert_eq!(out, "yes:unset");
    }

    #[test]
    fn per_call_env_reaches_child_without_export() {
        // `FOO=bar cmd` は export 無しでも子へ渡る
        let mut sb = ContainerSandbox::new();
        let out = sb
            .exec(
                &argv(&["sh", "-c", "printf %s ${SB_OVERRIDE-unset}"]),
                &[("SB_OVERRIDE".to_string(), "yes".to_string())],
                None,
                None,
                None,
                true,
            )
            .unwrap();
        assert_eq!(out, "yes");
    }

    #[test]
    fn command_v_resolves_in_path() {
        let mut sb = ContainerSandbox::new();
        let out = sb
            .exec(&argv(&["command", "-v", "sh"]), &[], None, None, None, true)
            .unwrap();
        assert!(out.trim_end().ends_with("/sh"), "{out:?}");
        let err = sb
            .exec(
                &argv(&["command", "-v", "definitely-not-here-xyz"]),
                &[],
                None,
                None,
                None,
                true,
            )
            .unwrap_err();
        assert!(matches!(err, Error::Exit { code: 1, .. }));
    }

    #[test]
    fn sanitize_flags_builtin_strips_lto() {
        let mut sb = ContainerSandbox::new();
        let out = sb
            .exec(
                &argv(&["sanitize_flags", "-O2 -flto -Werror=odr -fhardened -pipe"]),
                &[],
                None,
                None,
                None,
                true,
            )
            .unwrap();
        assert_eq!(out, "-O2    -pipe");
    }

    #[test]
    fn pipeline_chains_and_captures() {
        let mut sb = ContainerSandbox::new();
        let stages = vec![argv(&["echo", "hi"]), argv(&["cat"])];
        let out = sb.pipeline(&stages, &[], None, None, None, true).unwrap();
        assert_eq!(out, "hi\n");
    }

    #[test]
    fn pipeline_pipefail_returns_failure() {
        let mut sb = ContainerSandbox::new();
        let stages = vec![argv(&["false"]), argv(&["cat"])];
        let err = sb
            .pipeline(&stages, &[], None, None, None, true)
            .unwrap_err();
        assert!(matches!(err, Error::Exit { .. }));
    }

    #[test]
    fn glob_expands_and_sorts() {
        let dir = tempdir("glob");
        fs::write(dir.join("b.o"), "").unwrap();
        fs::write(dir.join("a.o"), "").unwrap();
        fs::write(dir.join("c.txt"), "").unwrap();
        let sb = ContainerSandbox::new();
        let pattern = format!("{}/*.o", dir.display());
        assert_eq!(
            sb.glob(&pattern),
            vec![
                format!("{}/a.o", dir.display()),
                format!("{}/b.o", dir.display())
            ]
        );
        // マッチ無しはパターンをそのまま返す（non-nullglob）
        let none = format!("{}/*.xyz", dir.display());
        assert_eq!(sb.glob(&none), vec![none]);
    }

    #[test]
    fn write_file_and_append() {
        let dir = tempdir("write");
        let path = format!("{}/f", dir.display());
        let mut sb = ContainerSandbox::new();
        sb.write_file(&path, &["a".to_string()], false).unwrap();
        sb.write_file(&path, &["b".to_string()], true).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "a\nb\n");
    }

    #[test]
    fn unset_shadows_outer_scope() {
        let mut sb = ContainerSandbox::new();
        sb.export("X", "outer");
        sb.push_scope();
        sb.unset("X");
        assert_eq!(sb.get("X"), None);
        sb.pop_scope();
        assert_eq!(sb.get("X").as_deref(), Some("outer"));
    }

    #[test]
    fn recipe_runs_end_to_end() {
        // run() 経由で WriteFile が実 FS と結合することを確認
        let dir = tempdir("e2e");
        let marker = dir.join("marker");
        let recipe = Recipe::Steps(Steps {
            env: Env::None,
            std: Std::Gnu17,
            sysroot: false,
            workdir: Workdir::None,
            pgo: None,
            steps: vec![
                Step::WriteFile {
                    path: Arg::Lit(marker.display().to_string().into()),
                    lines: vec![Arg::lit("ok")],
                    append: false,
                    expand: false,
                },
                Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![Arg::Lit(format!("cat {}", marker.display()).into())]),
                },
            ],
        });
        let ctx = crate::package::recipe::BuildContext::of(None, Target::BUILD_ENV);
        let mut sb = ContainerSandbox::new();
        runner::run(&recipe, &ctx, &mut sb).unwrap();
        assert_eq!(fs::read_to_string(&marker).unwrap(), "ok\n");
    }
}
