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
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::ExitStatusExt;
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdout, Command, Stdio};
use std::sync::{Mutex, MutexGuard};

use super::recipe::{CROSS_CACHE_VARS, Env, Std};
use super::runner::{Error, Sandbox, glob_match};

/// `cd`/`Subshell` が process-global cwd を触るため、同時に生存する
/// `ContainerSandbox` は 1 つに直列化する。run-recipe はプロセス毎に
/// 1 個しか作らず、テストの並列実行ではここで待ち合わせる。
static CWD_LOCK: Mutex<()> = Mutex::new(());

/// 実環境 `Sandbox`。コンテナ内でそのまま動かす他、tempdir 上の
/// ホスト検証にも使える（外部依存は `PATH` 上のコマンドのみ）。
///
/// process-global cwd を共有資源として使うため Send ではない。
pub struct ContainerSandbox {
    /// `CWD_LOCK` の保持ガード。cwd を変える操作の競合を防ぐため
    /// 生存期間中はロックを持ち続ける。
    _cwd_guard: MutexGuard<'static, ()>,
    /// Drop 時に復帰する元 cwd（cd が process-global cwd を変えるため、
    /// 削除済み tempdir 等を指し残すと後続 spawn が getcwd で壊れる）。
    initial_cwd: PathBuf,
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
    /// `CWD_LOCK` を取得してから返す（cwd 競合の直列化。別の
    /// `ContainerSandbox` が生存中はブロックする）。
    pub fn new() -> Self {
        Self {
            // panic 済みのロックでも続行する（cwd は best-effort の
            // 直列化で、テストの panic で全体を wedged にしない）
            _cwd_guard: CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner()),
            initial_cwd: env::current_dir().unwrap_or_else(|_| PathBuf::from("/")),
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
                if matches!(rest.first().map(String::as_str), Some("-v") | Some("-V")) {
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
            return is_exec_file(Path::new(name)).then(|| name.to_string());
        }
        let path = self.get("PATH").unwrap_or_default();
        for dir in path.split(':') {
            // POSIX: 空の PATH 要素はカレントディレクトリ
            let dir = if dir.is_empty() { "." } else { dir };
            let candidate = format!("{dir}/{name}");
            if is_exec_file(Path::new(&candidate)) {
                return Some(candidate);
            }
        }
        None
    }
}

/// process-global cwd を復帰する。フィールドの drop は impl Drop の
/// 後に走るため、ここではまだ `CWD_LOCK` を保持したまま復帰できる。
impl Drop for ContainerSandbox {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.initial_cwd);
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

/// `command -v` 用: 実行ビットを持つ通常ファイルのみ
/// （dir は `command -v` のヒット対象にならない）。
fn is_exec_file(path: &Path) -> bool {
    fs::metadata(path).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
}

/// `command [-p|-V] cmd …` を `cmd …` へ書き換える
/// （builtin 経由の function bypass。`-v`/`-V` は builtin 側で処理）。
fn strip_command(argv: &[String]) -> &[String] {
    if argv.first().map(String::as_str) == Some("command") {
        match argv.get(1).map(String::as_str) {
            Some("-v") | Some("-V") => argv,
            Some("-p") => &argv[2..],
            _ => &argv[1..],
        }
    } else {
        argv
    }
}

/// `sanitize_flags()` の sed 相当: `-flto*`・LTO 系 `-Werror=*`・
/// `-f{,no-}hardened` を左から順に除去する（最長一致優先）。
fn sanitize_flags(value: &str) -> String {
    const PATTERNS: &[&str] = &[
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
        if ch == '-' {
            if let Some(rem) = rest.strip_prefix("-flto=") {
                // -flto=jobserver 等の未知の値も =値 全体を落とす
                let len = rem.find(char::is_whitespace).unwrap_or(rem.len());
                rest = &rem[len..];
                continue;
            }
            if let Some(pat) = PATTERNS.iter().find(|p| rest.starts_with(**p)) {
                rest = &rest[pat.len()..];
                continue;
            }
        }
        out.push(ch);
        rest = &rest[ch.len_utf8()..];
    }
    out
}

/// builtin 出力の吐き先。`capture` なら返り値、stdout リダイレクトが
/// あればファイルへ、どちらも無ければ継承 stdout へ出す。
/// パイプライン構築中の早期エラーで spawn 済みの子プロセスを回収する
/// （wait されない Child は zombie になる）。
fn reap_children(children: &mut [Option<Child>]) {
    for slot in children.iter_mut() {
        if let Some(mut child) = slot.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// builtin/空ステージが消費しない上流パイプの read end を drain する。
/// bash では builtin の subshell 終了時に read end が閉じて書き手は
/// SIGPIPE になる。ここでは drain 中は read end を開いたままにして
/// 書き手に猶予を与え、パイプバッファ相当量（64KiB）を超えたら閉じる
/// — 小さい producer は成功し、巨大/無限 producer は SIGPIPE で
/// 打ち止められてハングしない。
fn drain_pipe(out: ChildStdout) {
    std::thread::spawn(move || {
        let _ = out.take(64 * 1024).read_to_end(&mut Vec::new());
    });
}

fn emit(output: Vec<u8>, stdout: Option<(&str, bool)>, capture: bool) -> Result<String, Error> {
    if let Some((path, append)) = stdout {
        // builtin 出力は末尾改行込みの完成形なので行単位ではなく
        // そのまま書く（write_lines だと改行が重複する）
        let mut file = open_redirect(path, append)?;
        file.write_all(&output)
            .map_err(|e| fs_exit("write", Path::new(path), e))?;
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
                output: String::new(),
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
        .map_err(|e| fs_exit("write", path, e))?;
    for line in lines {
        file.write_all(line)
            .and_then(|()| file.write_all(b"\n"))
            .map_err(|e| fs_exit("write", path, e))?;
    }
    Ok(())
}

/// fs 操作失敗（spawn/wait 等の内部エラー）。
fn exit_io(op: &str, path: &Path, e: std::io::Error) -> Error {
    Error::Io(format!("{op} {}: {e}", path.display()))
}

/// bash で同名コマンドが失敗する fs 操作を `Exit{1}` にする
/// （`||`/`Cond::Cmd`/workload の catch に乗せるため `Io` ではなく
/// `Exit` にする。`cd` と同じ規則）。
fn fs_exit(op: &str, path: &Path, e: std::io::Error) -> Error {
    let _ = e;
    Error::Exit {
        argv: vec![format!("{op} {}", path.display())],
        code: 1,
        output: String::new(),
    }
}

/// コマンドの exit code（シグナルは bash と同じく 128+signo）。
fn exit_code(status: std::process::ExitStatus) -> i32 {
    status
        .code()
        .unwrap_or_else(|| 128 + status.signal().unwrap_or(0))
}

/// `cp -a src/. dst/` 相当の再帰コピー（symlink 保存・perm/mtime 維持）。
/// owner はコピーしない: コンテナ内では一様に root で実害が無く、
/// libc::chown 依存を増やすだけになるため。
fn copy_tree(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    let src_meta = fs::metadata(src).ok();
    // `cp -a` parity: ディレクトリの mode も保持する
    // （create_dir_all は umask 既定になる）
    if let Some(meta) = &src_meta {
        let _ = fs::set_permissions(dst, meta.permissions());
    }
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
    // dir の mtime は子を書き終えてから設定する（子の作成で更新される
    // ため。`cp -a` も同じ順で保持する）
    if let Some(mtime) = src_meta.and_then(|m| m.modified().ok())
        && let Ok(f) = File::open(dst)
    {
        let _ = f.set_modified(mtime);
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

/// glob 展開（`*`/`?`/`[...]`）。
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
        if comp.contains(['*', '?', '[']) {
            for base in &cur {
                let dir = if base.is_empty() {
                    Path::new(".")
                } else {
                    Path::new(base.as_str())
                };
                if let Ok(rd) = fs::read_dir(dir) {
                    for entry in rd.flatten() {
                        let name = entry.file_name().to_string_lossy().into_owned();
                        // bash: 先頭ドットの名前は `.` 始まりのパターン
                        // でしかマッチしない（dotfiles 規則）
                        if glob_match(comp, &name)
                            && (comp.starts_with('.') || !name.starts_with('.'))
                        {
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
        // subshell 内の `export X` は継承値を複製して現スコープで
        // マークする（外側スコープの export フラグを立てると
        // pop_scope 後に漏れる）
        if let Some((value, _)) = self.resolve(name) {
            let value = value.clone();
            self.scopes
                .last_mut()
                .unwrap()
                .insert(name.to_string(), (value, true));
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
        let argv = strip_command(argv);
        if argv.is_empty() {
            return Ok(String::new());
        }
        if let Some(result) = self.run_builtin(argv) {
            return match result {
                Ok(bytes) => emit(bytes, stdout, capture),
                Err(code) => Err(Error::Exit {
                    argv: argv.to_vec(),
                    code,
                    output: String::new(),
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
            // `<file` の open 失敗は bash ではそのコマンドの exit 1
            let file = File::open(path).map_err(|_| Error::Exit {
                argv: vec![format!("< {path}")],
                code: 1,
                output: String::new(),
            })?;
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
                    output: String::new(),
                });
            }
            // EACCES（実行ビットなし）は bash の 126
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                return Err(Error::Exit {
                    argv: argv.to_vec(),
                    code: 126,
                    output: String::new(),
                });
            }
            Err(e) => return Err(exit_io("spawn", Path::new(&argv[0]), e)),
            Ok(child) => child,
        };
        let output = child
            .wait_with_output()
            .map_err(|e| exit_io("wait", Path::new(&argv[0]), e))?;
        let status = output.status;
        // 失敗しても捕捉した stdout は残す（bash の `$(f)` は
        // 部分出力を代入・展開に使う）
        let captured = String::from_utf8_lossy(&output.stdout).into_owned();
        if !status.success() {
            return Err(Error::Exit {
                argv: argv.to_vec(),
                code: exit_code(status),
                output: captured,
            });
        }
        Ok(captured)
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
        let mut prev: Option<ChildStdout> = None;
        // builtin ステージの出力（次ステージの stdin へ書き込む）
        let mut feed: Option<Vec<u8>> = None;
        // 末尾が builtin の場合の emit 結果（capture 用）
        let mut last_builtin_out: Option<String> = None;
        // spawn 途中のエラーは spawn 済みの子を reap してから返す
        macro_rules! try_reap {
            ($e:expr) => {
                match $e {
                    Ok(value) => value,
                    Err(error) => {
                        reap_children(&mut children);
                        return Err(error);
                    }
                }
            };
        }
        for (i, stage) in stages.iter().enumerate() {
            let stage = strip_command(stage);
            if stage.is_empty() {
                // 裸の `command` — no-op 成功
                feed = Some(Vec::new());
                codes[i] = Some(0);
                children.push(None);
                if let Some(out) = prev.take() {
                    drain_pipe(out);
                }
                continue;
            }
            if let Some(result) = self.run_builtin(stage) {
                match result {
                    Ok(bytes) => {
                        if i == last {
                            // 末尾 builtin の出力は redirect/capture/
                            // inherit へ出す（feed には乗らない）
                            last_builtin_out = Some(emit(bytes, stdout, capture)?);
                        } else {
                            feed = Some(bytes);
                        }
                        codes[i] = Some(0);
                    }
                    Err(code) => {
                        if i == last {
                            last_builtin_out = Some(emit(Vec::new(), stdout, capture)?);
                        } else {
                            feed = Some(Vec::new());
                        }
                        codes[i] = Some(code);
                    }
                }
                children.push(None);
                if let Some(out) = prev.take() {
                    drain_pipe(out);
                }
                continue;
            }
            let mut cmd = Command::new(&stage[0]);
            cmd.args(&stage[1..]).env_clear();
            for (name, value) in self.child_env() {
                cmd.env(name, value);
            }
            // `A=1 c1 | c2` の env 前置は先頭ステージにだけ付く
            if i == 0 {
                for (name, value) in env {
                    cmd.env(name, value);
                }
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
                let file = try_reap!(File::open(path).map_err(|_| Error::Exit {
                    argv: vec![format!("< {path}")],
                    code: 1,
                    output: String::new(),
                }));
                cmd.stdin(Stdio::from(file));
            }
            // stdout: 末尾のみ redirect/capture/inherit、中間は piped
            if i == last {
                match stdout {
                    Some((path, append)) => {
                        cmd.stdout(Stdio::from(try_reap!(open_redirect(path, append))))
                    }
                    None if capture => cmd.stdout(Stdio::piped()),
                    None => cmd.stdout(Stdio::inherit()),
                };
                match stderr {
                    Some(path) => cmd.stderr(Stdio::from(try_reap!(open_redirect(path, false)))),
                    None => cmd.stderr(Stdio::inherit()),
                };
            } else {
                cmd.stdout(Stdio::piped());
            }
            let mut child = match cmd.spawn() {
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                    reap_children(&mut children);
                    return Err(Error::Exit {
                        argv: stage.to_vec(),
                        code: 127,
                        output: String::new(),
                    });
                }
                Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                    reap_children(&mut children);
                    return Err(Error::Exit {
                        argv: stage.to_vec(),
                        code: 126,
                        output: String::new(),
                    });
                }
                Err(e) => {
                    let error = exit_io("spawn", Path::new(&stage[0]), e);
                    reap_children(&mut children);
                    return Err(error);
                }
                Ok(child) => child,
            };
            if let Some(bytes) = feed_bytes
                && let Some(mut pipe) = child.stdin.take()
            {
                // bash ではパイプの書き手が先に読み手が終了して EPIPE で
                // 死んでも、パイプラインの結果は各ステージの終了コード
                // （pipefail）だけで決まる。ここでの write 失敗は同じ意味
                // なので握り潰す。pipe は drop で閉じて子に EOF を届ける。
                let _ = pipe.write_all(&bytes);
            }
            if i < last {
                prev = child.stdout.take();
            }
            children.push(Some(child));
        }
        let mut output = String::new();
        for (i, slot) in children.iter_mut().enumerate() {
            let Some(mut child) = slot.take() else {
                continue;
            };
            if i == last {
                let out = try_reap!(child.wait_with_output().map_err(|e| exit_io(
                    "wait",
                    Path::new(&stages[i][0]),
                    e
                )));
                if capture && stdout.is_none() {
                    output = String::from_utf8_lossy(&out.stdout).into_owned();
                }
                codes[i] = Some(exit_code(out.status));
            } else {
                let status = try_reap!(child.wait().map_err(|e| exit_io(
                    "wait",
                    Path::new(&stages[i][0]),
                    e
                )));
                codes[i] = Some(exit_code(status));
            }
        }
        let output = last_builtin_out.unwrap_or(output);
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
                output,
            });
        }
        Ok(output)
    }

    fn write_file(&mut self, path: &str, lines: &[String], append: bool) -> Result<(), Error> {
        let bytes: Vec<Vec<u8>> = lines.iter().map(|l| l.as_bytes().to_vec()).collect();
        write_lines(Path::new(path), &bytes, append)
    }

    fn mkdir_p(&mut self, path: &str) -> Result<(), Error> {
        fs::create_dir_all(path).map_err(|e| fs_exit("mkdir -p", Path::new(path), e))
    }

    fn copy_into(&mut self, src: &str, dst: &str) -> Result<(), Error> {
        copy_tree(Path::new(src), Path::new(dst)).map_err(|e| fs_exit("cp -a", Path::new(src), e))
    }

    fn cd(&mut self, path: &str) -> Result<(), Error> {
        env::set_current_dir(path).map_err(|e| {
            // bash の `cd` 失敗は exit 1
            let _ = e;
            Error::Exit {
                argv: vec!["cd".to_string(), path.to_string()],
                code: 1,
                output: String::new(),
            }
        })
    }

    fn cwd(&self) -> String {
        env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_default()
    }

    fn remove_dir(&mut self, path: &str) -> Result<(), Error> {
        match fs::remove_dir_all(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            result => result.map_err(|e| fs_exit("rm -rf", Path::new(path), e)),
        }
    }

    fn install(&mut self, src: &str, dst: &str, mode: &str) -> Result<(), Error> {
        let dst = Path::new(dst);
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|e| fs_exit("install -D", parent, e))?;
        }
        fs::copy(src, dst).map_err(|e| fs_exit("install -D", dst, e))?;
        // 不正な mode は bash の `install` 同様に失敗させる
        let mode = u32::from_str_radix(mode, 8).map_err(|_| Error::Exit {
            argv: vec![format!("install -D -m {mode}")],
            code: 1,
            output: String::new(),
        })?;
        fs::set_permissions(dst, fs::Permissions::from_mode(mode))
            .map_err(|e| fs_exit("install -D", dst, e))
    }

    fn symlink(&mut self, target: &str, link: &str, force: bool) -> Result<(), Error> {
        let mut link = PathBuf::from(link);
        // GNU ln の -T 無し既定: link が実在するディレクトリ（dir への
        // symlink 含む）なら、その中に target の basename でリンクを作る
        if link.is_dir()
            && let Some(name) = Path::new(target).file_name()
        {
            link.push(name);
        }
        if force && link.symlink_metadata().is_ok() {
            fs::remove_file(&link).map_err(|e| fs_exit("ln -sf", &link, e))?;
        }
        std::os::unix::fs::symlink(target, &link).map_err(|e| fs_exit("ln -s", &link, e))
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

    fn tempdir(_tag: &str) -> tempfile::TempDir {
        // TempDir は drop で自動削除される（/tmp に残骸を残さない）
        tempfile::tempdir().unwrap()
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
    fn pipeline_builtin_last_stage_captures() {
        // `producer | command -v sh` — 末尾 builtin の出力を拾う
        let mut sb = ContainerSandbox::new();
        let stages = vec![argv(&["echo", "x"]), argv(&["command", "-v", "sh"])];
        let out = sb.pipeline(&stages, &[], None, None, None, true).unwrap();
        assert!(out.ends_with("/sh\n"));
    }

    #[test]
    fn pipeline_builtin_last_stage_reports_exit() {
        // `command -v` の miss は bash と同じく exit 1
        let mut sb = ContainerSandbox::new();
        let stages = vec![argv(&["echo", "x"]), argv(&["command", "-v", "nope-xyz"])];
        let err = sb
            .pipeline(&stages, &[], None, None, None, true)
            .unwrap_err();
        assert!(matches!(err, Error::Exit { code: 1, .. }));
    }

    #[test]
    fn pipeline_env_applies_to_first_stage_only() {
        // `FOO=bar c1 | c2` — env 前置は先頭ステージにだけ付く
        let env = vec![("FOO".to_string(), "bar".to_string())];
        let mut sb = ContainerSandbox::new();
        let stages = vec![argv(&["sh", "-c", "echo $FOO"]), argv(&["cat"])];
        let out = sb.pipeline(&stages, &env, None, None, None, true).unwrap();
        assert_eq!(out, "bar\n");
        let stages = vec![argv(&["echo", "x"]), argv(&["sh", "-c", "echo \"$FOO\""])];
        let out = sb.pipeline(&stages, &env, None, None, None, true).unwrap();
        assert_eq!(out, "\n");
    }

    #[test]
    fn command_dispatches_to_real_command() {
        // `command foo …` は function bypass 相当で実コマンドを呼ぶ
        let mut sb = ContainerSandbox::new();
        let out = sb
            .exec(
                &argv(&["command", "echo", "hi"]),
                &[],
                None,
                None,
                None,
                true,
            )
            .unwrap();
        assert_eq!(out, "hi\n");
    }

    #[test]
    fn sanitize_flags_strips_arbitrary_flto_values() {
        // `-flto=jobserver` 等の未知の値も `=値` 全体を落とす
        assert_eq!(sanitize_flags("-O2 -flto=jobserver -g"), "-O2  -g");
        assert_eq!(sanitize_flags("-flto=thin -O2"), " -O2");
    }

    #[test]
    fn glob_expands_and_sorts() {
        let dir = tempdir("glob");
        let dir = dir.path();
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
    fn glob_char_class_and_dotfile_rule() {
        let dir = tempdir("glob-class");
        let dir = dir.path();
        fs::write(dir.join("a1.o"), "").unwrap();
        fs::write(dir.join("a2.o"), "").unwrap();
        fs::write(dir.join("ab.o"), "").unwrap();
        fs::write(dir.join(".hidden.o"), "").unwrap();
        let sb = ContainerSandbox::new();
        // `[12]` クラス
        let pat = format!("{}/a[12].o", dir.display());
        assert_eq!(
            sb.glob(&pat),
            vec![
                format!("{}/a1.o", dir.display()),
                format!("{}/a2.o", dir.display())
            ]
        );
        // `[!12]` 否定
        let pat = format!("{}/a[!12].o", dir.display());
        assert_eq!(sb.glob(&pat), vec![format!("{}/ab.o", dir.display())]);
        // `*` はドットファイルを拾わない（bash の dotfiles 規則）
        let pat = format!("{}/*.o", dir.display());
        assert!(!sb.glob(&pat).iter().any(|p| p.contains("hidden")));
        // `.*` 始まりなら拾う
        let pat = format!("{}/.*", dir.display());
        assert!(sb.glob(&pat).iter().any(|p| p.ends_with(".hidden.o")));
    }

    #[test]
    fn symlink_force_into_existing_dir_links_basename() {
        // GNU ln: `ln -sf t dir/` で dir が実在 dir なら dir/t を作る
        let dir = tempdir("ln");
        let dir = dir.path();
        let target = dir.join("t");
        fs::write(&target, "x").unwrap();
        let dest = dir.join("d");
        fs::create_dir(&dest).unwrap();
        let mut sb = ContainerSandbox::new();
        sb.symlink(
            target.to_str().unwrap(),
            &format!("{}/", dest.display()),
            true,
        )
        .unwrap();
        let made = dest.join("t");
        assert_eq!(
            fs::read_link(&made).unwrap(),
            target,
            "dir/t が t への symlink になる"
        );
        // 二度目の -f でも既存 symlink を置き換えて成功する
        sb.symlink(
            target.to_str().unwrap(),
            &format!("{}/", dest.display()),
            true,
        )
        .unwrap();
    }

    #[test]
    fn copy_into_preserves_dir_mtime() {
        // `cp -a` parity: ディレクトリの mtime も保持する
        // （タイムスタンプ比較で再生成されるビルド系への影響を防ぐ）
        let dir = tempdir("cp-mtime");
        let dir = dir.path();
        let src = dir.join("src");
        let dst = dir.join("dst");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("f"), "x").unwrap();
        let past = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        File::open(&src).unwrap().set_modified(past).unwrap();
        let mut sb = ContainerSandbox::new();
        sb.copy_into(src.to_str().unwrap(), dst.to_str().unwrap())
            .unwrap();
        let dst_mtime = fs::metadata(&dst).unwrap().modified().unwrap();
        assert_eq!(dst_mtime, past);
        assert_eq!(fs::read_to_string(dst.join("f")).unwrap(), "x");
    }

    #[test]
    fn write_file_and_append() {
        let dir = tempdir("write");
        let dir = dir.path();
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
        let dir = dir.path();
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

    #[test]
    fn cd_relative_normalizes_pwd() {
        // bash: `cd sub` 後の $PWD は正規化済みの絶対パスになる
        // （zlib/openssl/perl の `$PWD` 参照が相対名を拾わない）
        let dir = tempdir("cd-pwd");
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();
        let recipe = Recipe::Steps(Steps {
            env: Env::None,
            std: Std::Gnu17,
            sysroot: false,
            workdir: Workdir::None,
            pgo: None,
            steps: vec![
                Step::Cd(Arg::Lit(dir.path().display().to_string().into())),
                Step::Cd(Arg::lit("sub")),
            ],
        });
        let ctx = crate::package::recipe::BuildContext::of(None, Target::BUILD_ENV);
        let mut sb = ContainerSandbox::new();
        runner::run(&recipe, &ctx, &mut sb).unwrap();
        let want = sub.canonicalize().unwrap();
        assert_eq!(sb.get("PWD").as_deref(), Some(want.to_str().unwrap()));
    }

    #[test]
    fn pipeline_large_producer_to_builtin_gets_sigpipe() {
        // パイプバッファ（64KiB）超の producer | builtin — drain が
        // 打ち切られて書き手は SIGPIPE（pipefail で 141）。
        // ハングしないことも兼ねて検証する。
        let mut sb = ContainerSandbox::new();
        let stages = vec![
            argv(&["seq", "1", "100000"]),
            argv(&["command", "-v", "sh"]),
        ];
        let err = sb
            .pipeline(&stages, &[], None, None, None, true)
            .unwrap_err();
        assert!(matches!(err, Error::Exit { code: 141, .. }));
    }

    #[test]
    fn eacces_maps_to_exit_126() {
        // 実行ビットの無いファイルの spawn は bash と同じく 126
        let dir = tempdir("eacces");
        let dir = dir.path();
        let path = dir.join("noexec.sh");
        fs::write(&path, "echo hi\n").unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&path, perms).unwrap();
        let mut sb = ContainerSandbox::new();
        let err = sb
            .exec(
                &[path.to_str().unwrap().to_string()],
                &[],
                None,
                None,
                None,
                false,
            )
            .unwrap_err();
        assert!(matches!(err, Error::Exit { code: 126, .. }));
    }

    #[test]
    fn pipeline_writes_stdout_redirect() {
        // `cmd1 | cmd2 > file` — 末尾ステージのリダイレクト
        let dir = tempdir("redirect");
        let dir = dir.path();
        let out_path = dir.join("out.txt");
        let mut sb = ContainerSandbox::new();
        sb.pipeline(
            &[argv(&["echo", "hi"]), argv(&["cat"])],
            &[],
            None,
            Some((out_path.to_str().unwrap(), false)),
            None,
            false,
        )
        .unwrap();
        assert_eq!(fs::read_to_string(&out_path).unwrap(), "hi\n");
    }

    #[test]
    fn redirect_open_failure_is_exit_1() {
        // `cmd > /nonexistent/dir/f` — bash と同じく exit 1
        let mut sb = ContainerSandbox::new();
        let err = sb
            .pipeline(
                &[argv(&["echo", "hi"]), argv(&["cat"])],
                &[],
                None,
                Some(("/nonexistent-dir-xyz/f", false)),
                None,
                false,
            )
            .unwrap_err();
        assert!(matches!(err, Error::Exit { code: 1, .. }));
    }

    #[test]
    fn pipeline_mid_stage_enoent_is_127() {
        // 中間ステージの command-not-found も bash と同じく 127
        let mut sb = ContainerSandbox::new();
        let stages = vec![
            argv(&["echo", "x"]),
            argv(&["nope-xyz-missing"]),
            argv(&["cat"]),
        ];
        let err = sb
            .pipeline(&stages, &[], None, None, None, true)
            .unwrap_err();
        assert!(matches!(
            err,
            Error::Exit { code: 127, ref argv, .. } if argv[0] == "nope-xyz-missing"
        ));
    }

    #[test]
    fn command_v_flag_resolves_path() {
        // `command -V` も -v と同じく PATH から解決する
        let mut sb = ContainerSandbox::new();
        let out = sb
            .exec(&argv(&["command", "-V", "sh"]), &[], None, None, None, true)
            .unwrap();
        assert!(out.trim_end().ends_with("/sh"), "{out}");
    }

    #[test]
    fn pipeline_reads_stdin_redirect() {
        // `cmd < file | cmd2` — 先頭ステージの stdin リダイレクト
        let dir = tempdir("stdin");
        let in_path = dir.path().join("in.txt");
        fs::write(&in_path, "hello\n").unwrap();
        let mut sb = ContainerSandbox::new();
        let out = sb
            .pipeline(
                &[argv(&["cat"]), argv(&["cat"])],
                &[],
                Some(in_path.to_str().unwrap()),
                None,
                None,
                true,
            )
            .unwrap();
        assert_eq!(out, "hello\n");
    }

    #[test]
    fn pipeline_mid_builtin_feeds_next_stage() {
        // 中間 builtin の出力が次ステージの stdin を駆動する:
        // `sanitize_flags "$X" | cat`
        let mut sb = ContainerSandbox::new();
        let stages = vec![
            argv(&["sanitize_flags", "-O2 -flto=auto -g"]),
            argv(&["cat"]),
        ];
        let out = sb.pipeline(&stages, &[], None, None, None, true).unwrap();
        assert!(out.contains("-O2"), "{out}");
        assert!(!out.contains("flto"), "{out}");
        assert!(out.contains("-g"), "{out}");
    }
}
