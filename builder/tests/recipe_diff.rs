//! `Recipe` 本体を `runner::run` と描画した bash の両方で実行し、
//! ファイルシステム結果を比較する。`run_plan` の prelude は含めない。
use std::collections::BTreeMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::UNIX_EPOCH;

use containers_builder::package::recipe::{
    Arg, BuildContext, Cond, Env, Pipe, Recipe, Std, Step, Steps, Workdir,
};
use containers_builder::package::runner::{self, Sandbox};
use containers_builder::package::sandbox::ContainerSandbox;
use containers_builder::target::Target;

fn ctx() -> BuildContext {
    BuildContext::of(None, Target::BUILD_ENV)
}

fn recipe(workdir: Workdir, steps: Vec<Step>) -> Recipe {
    Recipe::Steps(Steps {
        env: Env::None,
        std: Std::Gnu17,
        sysroot: false,
        workdir,
        pgo: None,
        steps,
    })
}

fn fail_partial() -> Pipe {
    Pipe::cmd(vec![
        Arg::lit("sh"),
        Arg::lit("-c"),
        Arg::quoted(Arg::lit("printf partial; exit 1")),
    ])
}

fn write_text(name: &'static str, text: &'static str) -> Step {
    Step::Exec {
        env: Vec::new(),
        pipe: Pipe::cmd(vec![Arg::lit("printf"), Arg::lit("%s"), Arg::lit(text)])
            .stdout(Arg::lit(name)),
    }
}

fn touch_stamp(path: &Path) {
    let status = Command::new("touch")
        .args(["-d", "2020-01-02 03:04:05", &path.display().to_string()])
        .status()
        .unwrap();
    assert!(status.success(), "touch {path:?}");
}

fn touch_stamp_nofollow(path: &Path) {
    let status = Command::new("touch")
        .args([
            "-h",
            "-d",
            "2020-01-02 03:04:05",
            &path.display().to_string(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "touch -h {path:?}");
}

#[derive(Debug)]
struct Node {
    kind: &'static str,
    mode: u32,
    mtime_ns: u128,
    link: Option<PathBuf>,
    bytes: Option<Vec<u8>>,
}

fn snapshot(root: &Path) -> BTreeMap<String, Node> {
    let mut out = BTreeMap::new();
    walk(root, root, &mut out);
    out
}

fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Node>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let rel = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let meta = fs::symlink_metadata(&path).unwrap();
        let mtime_ns = meta
            .modified()
            .unwrap()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let mode = meta.permissions().mode() & 0o777;
        let (kind, link, bytes) = if meta.file_type().is_symlink() {
            ("link", Some(fs::read_link(&path).unwrap()), None)
        } else if meta.is_dir() {
            ("dir", None, None)
        } else {
            ("file", None, Some(fs::read(&path).unwrap()))
        };
        out.insert(
            rel,
            Node {
                kind,
                mode,
                mtime_ns,
                link,
                bytes,
            },
        );
        if meta.is_dir() && !meta.file_type().is_symlink() {
            walk(root, &path, out);
        }
    }
}

struct Outcome {
    ok: bool,
    stdout: String,
}

fn run_runner(
    recipe: &Recipe,
    root: &Path,
    source: Option<&Path>,
    build: Option<&Path>,
) -> Outcome {
    let mut sb = ContainerSandbox::new();
    sb.cd(root.to_str().unwrap()).unwrap();
    if let (Some(source), Some(build)) = (source, build) {
        sb.export("SOURCE", source.to_str().unwrap());
        sb.export("BUILD", build.to_str().unwrap());
    }
    let (result, stdout) = capture_stdout(|| runner::run(recipe, &ctx(), &mut sb));
    Outcome {
        ok: result.is_ok(),
        stdout,
    }
}

fn run_bash(recipe: &Recipe, root: &Path, source: Option<&Path>, build: Option<&Path>) -> Outcome {
    let script = recipe.render(&ctx());
    let mut command = Command::new("bash");
    command
        .current_dir(root)
        .env_clear()
        .env("PATH", std::env::var("PATH").unwrap())
        .arg("-c")
        .arg(&script)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
    if let (Some(source), Some(build)) = (source, build) {
        command.env("SOURCE", source);
        command.env("BUILD", build);
    }
    let output = command.output().unwrap();
    Outcome {
        ok: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    }
}

fn capture_stdout<T>(body: impl FnOnce() -> T) -> (T, String) {
    use std::os::unix::io::AsRawFd;
    let (mut reader, writer) = std::io::pipe().unwrap();
    let saved = unsafe { dup(1) };
    assert!(saved >= 0, "dup");
    let swapped = unsafe { dup2(writer.as_raw_fd(), 1) };
    assert!(swapped >= 0, "dup2");
    drop(writer);
    let value = std::panic::catch_unwind(std::panic::AssertUnwindSafe(body));
    assert!(unsafe { dup2(saved, 1) } >= 0, "restore");
    unsafe { close(saved) };
    let mut stdout = String::new();
    std::io::Read::read_to_string(&mut reader, &mut stdout).unwrap();
    match value {
        Ok(value) => (value, stdout),
        Err(payload) => std::panic::resume_unwind(payload),
    }
}

unsafe extern "C" {
    fn dup(fd: i32) -> i32;
    fn dup2(oldfd: i32, newfd: i32) -> i32;
    fn close(fd: i32) -> i32;
}

fn assert_same(
    recipe: &Recipe,
    seed: impl Fn(&Path),
    source_rel: Option<&str>,
    expect_ok: bool,
    stdout: Option<&str>,
    pwd_rel: Option<&str>,
    compare_mtime: bool,
) {
    let runner_tmp = tempfile::tempdir().unwrap();
    let bash_tmp = tempfile::tempdir().unwrap();
    seed(runner_tmp.path());
    seed(bash_tmp.path());
    let paths = |root: &Path| {
        source_rel.map(|rel| (root.join(rel).join("src"), root.join(rel).join("build")))
    };
    let runner_paths = paths(runner_tmp.path());
    let bash_paths = paths(bash_tmp.path());
    let runner = run_runner(
        recipe,
        runner_tmp.path(),
        runner_paths.as_ref().map(|(s, _)| s.as_path()),
        runner_paths.as_ref().map(|(_, b)| b.as_path()),
    );
    let bash = run_bash(
        recipe,
        bash_tmp.path(),
        bash_paths.as_ref().map(|(s, _)| s.as_path()),
        bash_paths.as_ref().map(|(_, b)| b.as_path()),
    );
    assert_eq!(
        runner.ok,
        expect_ok,
        "runner ok={} script:\n{}",
        runner.ok,
        recipe.render(&ctx())
    );
    assert_eq!(bash.ok, expect_ok, "bash ok={}", bash.ok);
    if let Some(expected) = stdout {
        assert_eq!(runner.stdout, expected, "runner stdout");
        assert_eq!(bash.stdout, expected, "bash stdout");
    }
    let mut left = snapshot(runner_tmp.path());
    let mut right = snapshot(bash_tmp.path());
    if let Some(rel) = pwd_rel {
        let runner_pwd = fs::read_to_string(runner_tmp.path().join(rel)).unwrap();
        let bash_pwd = fs::read_to_string(bash_tmp.path().join(rel)).unwrap();
        let runner_expect = runner_tmp
            .path()
            .join(rel)
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap();
        let bash_expect = bash_tmp
            .path()
            .join(rel)
            .parent()
            .unwrap()
            .canonicalize()
            .unwrap();
        assert_eq!(runner_pwd, runner_expect.display().to_string());
        assert_eq!(bash_pwd, bash_expect.display().to_string());
        left.get_mut(rel).unwrap().bytes = None;
        right.get_mut(rel).unwrap().bytes = None;
    }
    assert_eq!(
        left.keys().collect::<Vec<_>>(),
        right.keys().collect::<Vec<_>>()
    );
    for (rel, node) in &left {
        let other = &right[rel];
        assert_eq!(node.kind, other.kind, "{rel}");
        assert_eq!(node.mode, other.mode, "{rel}");
        if compare_mtime {
            assert_eq!(node.mtime_ns, other.mtime_ns, "{rel}");
        }
        assert_eq!(node.link, other.link, "{rel}");
        assert_eq!(node.bytes, other.bytes, "{rel}");
    }
}

#[test]
fn recipe_matches_rendered_bash() {
    assert_same(
        &recipe(
            Workdir::None,
            vec![
                Step::Exec {
                    env: Vec::new(),
                    pipe: Pipe::cmd(vec![
                        Arg::lit("printf"),
                        Arg::lit("%s"),
                        Arg::quoted(Arg::Subst(Box::new(fail_partial()))),
                    ])
                    .stdout(Arg::lit("partial.txt")),
                },
                write_text("next.txt", "next"),
            ],
        ),
        |_| {},
        None,
        true,
        None,
        None,
        false,
    );

    assert_same(
        &recipe(
            Workdir::None,
            vec![
                Step::Set {
                    name: "X",
                    value: Arg::Subst(Box::new(fail_partial())),
                },
                write_text("after.txt", "after"),
            ],
        ),
        |_| {},
        None,
        false,
        None,
        None,
        false,
    );

    assert_same(
        &recipe(
            Workdir::None,
            vec![
                Step::Set {
                    name: "X",
                    value: Arg::SubstOr(Box::new(fail_partial())),
                },
                Step::Exec {
                    env: Vec::new(),
                    pipe: Pipe::cmd(vec![
                        Arg::lit("printf"),
                        Arg::lit("%s"),
                        Arg::quoted(Arg::Var("X")),
                    ])
                    .stdout(Arg::lit("kept.txt")),
                },
            ],
        ),
        |_| {},
        None,
        true,
        None,
        None,
        false,
    );

    assert_same(
        &recipe(
            Workdir::None,
            vec![Step::Symlink {
                target: "t".into(),
                link: "dir/".into(),
                force: true,
            }],
        ),
        |root| {
            fs::write(root.join("t"), "target").unwrap();
            fs::create_dir(root.join("dir")).unwrap();
        },
        None,
        true,
        None,
        None,
        false,
    );

    assert_same(
        &recipe(Workdir::InSource, Vec::new()),
        |root| {
            let src = root.join("src");
            let nested = src.join("sub");
            fs::create_dir_all(&nested).unwrap();
            let file = src.join("file");
            fs::write(&file, "body").unwrap();
            fs::set_permissions(&file, fs::Permissions::from_mode(0o640)).unwrap();
            let link = src.join("link");
            std::os::unix::fs::symlink("file", &link).unwrap();
            let inner = nested.join("inner");
            fs::write(&inner, "in").unwrap();
            fs::set_permissions(&nested, fs::Permissions::from_mode(0o750)).unwrap();
            touch_stamp(&file);
            touch_stamp(&inner);
            touch_stamp_nofollow(&link);
            touch_stamp(&nested);
            touch_stamp(&src);
        },
        Some(""),
        true,
        None,
        None,
        true,
    );

    assert_same(
        &recipe(
            Workdir::None,
            vec![Step::Exec {
                env: Vec::new(),
                pipe: Pipe::cmd(vec![Arg::lit("echo"), Arg::lit("*")]).stdout(Arg::lit("glob.txt")),
            }],
        ),
        |root| {
            fs::write(root.join("a.txt"), "").unwrap();
            fs::write(root.join("b.txt"), "").unwrap();
            fs::write(root.join(".hidden"), "").unwrap();
        },
        None,
        true,
        None,
        None,
        false,
    );

    assert_same(
        &recipe(
            Workdir::None,
            vec![
                Step::Cd(Arg::lit("sub")),
                Step::Exec {
                    env: Vec::new(),
                    pipe: Pipe::cmd(vec![
                        Arg::lit("printf"),
                        Arg::lit("%s"),
                        Arg::quoted(Arg::Var("PWD")),
                    ])
                    .stdout(Arg::lit("pwd.txt")),
                },
            ],
        ),
        |root| {
            fs::create_dir(root.join("sub")).unwrap();
        },
        None,
        true,
        None,
        Some("sub/pwd.txt"),
        false,
    );

    assert_same(
        &recipe(
            Workdir::None,
            vec![Step::If {
                cond: Cond::Cmd(Pipe::cmd(vec![Arg::lit("echo"), Arg::lit("probe")])),
                then: vec![write_text("then.txt", "yes")],
                else_: Vec::new(),
            }],
        ),
        |_| {},
        None,
        true,
        Some("probe\n"),
        None,
        false,
    );
}
