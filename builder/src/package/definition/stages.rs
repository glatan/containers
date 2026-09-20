//! ツールチェーン構成ステージ（binutils/glibc/gcc 系）の共有ヘルパー。
//! 各 `{triple}-*.rs` パッケージが参照する定義側の共通部品。

use crate::package::Dependency;
use crate::package::recipe::Cond;
use crate::package::recipe::{Arg, Pipe, Step};

/// `Dependency` 構築の糖衣。
pub(crate) const fn dep(name: &'static str, version: &'static str) -> Dependency {
    Dependency { name, version }
}

/// `find dir -name '*.so'` の ld script（`GROUP ( //lib/libc.so.6 )`）を
/// `//` → `/` へ修正する共有ステップ。ld は `//` を sysroot しない。
/// glibc 自身のインストール後と、既存 glibc 成果物を取り込んだ gcc の
/// BUILD_SYSROOT の両方で必要。
pub(crate) fn ld_script_fixup(dir: Arg) -> Step {
    let cmd = |argv: Vec<Arg>| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(argv),
    };
    Step::ForEach {
        var: "_so",
        items: vec![Arg::Subst(Box::new(Pipe::cmd(vec![
            Arg::lit("find"),
            dir,
            Arg::lit("-type"),
            Arg::lit("f"),
            Arg::lit("-name"),
            Arg::quoted(Arg::lit("*.so")),
        ])))],
        body: vec![Step::If {
            cond: Cond::Cmd(Pipe::cmd(vec![
                Arg::lit("grep"),
                Arg::lit("-q"),
                Arg::quoted(Arg::lit("GNU ld script")),
                Arg::Var("_so"),
            ])),
            then: vec![cmd(vec![
                Arg::lit("sed"),
                Arg::lit("-i"),
                Arg::quoted(Arg::lit("s|//lib/|/lib/|g; s|//lib64/|/lib64/|g")),
                Arg::Var("_so"),
            ])],
            else_: vec![],
        }],
        stdout: None,
    }
}
