//! ツールチェーン構成ステージ（binutils/glibc/gcc 系）の共有ヘルパー。
//! 各 `{triple}-*.rs` パッケージが参照する定義側の共通部品。

use crate::package::recipe::Cond;
use crate::package::recipe::{Arg, Pipe, Step};

/// `Dependency` 構築の糖衣（`crate::package::dep` の再エクスポート）。
pub(crate) use crate::package::dep;

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

/// /usr/bin/{triple}-as および -ld へのシンボリックリンク。
/// stage1/stage2 gcc は --with-as/--with-ld=/usr/bin/{triple}-{as,ld}
/// でビルドされるため、それを呼び出すレシピ（glibc-headers, glibc）は
/// /dep/{triple}-binutils へのリンクを張る必要がある。
pub(crate) fn toolchain_links() -> Vec<Step> {
    let cmd = |argv: Vec<Arg>| Step::Exec {
        env: vec![],
        pipe: Pipe::cmd(argv),
    };
    let dep_binutils = |sub: &'static str| {
        Arg::Concat(vec![
            Arg::lit("/dep/"),
            Arg::Var("TRIPLE"),
            Arg::lit("-binutils/usr/bin/"),
            Arg::Var("TRIPLE"),
            Arg::lit(sub),
        ])
    };
    let usr_bin = |sub: &'static str| {
        Arg::Concat(vec![
            Arg::lit("/usr/bin/"),
            Arg::Var("TRIPLE"),
            Arg::lit(sub),
        ])
    };
    vec![
        cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            dep_binutils("-as"),
            usr_bin("-as"),
        ]),
        cmd(vec![
            Arg::lit("ln"),
            Arg::lit("-sfn"),
            dep_binutils("-ld"),
            usr_bin("-ld"),
        ]),
    ]
}
