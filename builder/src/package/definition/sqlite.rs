use crate::package::recipe::{BuildContext, Recipe, Step};
use crate::package::{Package, Source};

pub struct Sqlite;

impl Package for Sqlite {
    fn name(&self) -> &'static str {
        "sqlite"
    }
    fn version(&self) -> &'static str {
        "3.54.0"
    }
    fn license(&self) -> &'static str {
        "blessing"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://www.sqlite.org/2026/sqlite-autoconf-3540000.tar.gz",
            sha3_512: "27da7c186cd32c4812b2d30f54124c94483c13f6205fee2dfead17ab767907cf0428337aa228125ce60d818320132f63c962522183443fe2da6f921ca9863f8d",
            blake3: "d96be8c27a8f43b1905531d7f03bd7cfe9c9461eb9561ed6d63d66dbd364d6edfac5f1e7400b4b5993e2005d1b359acdfe66559cf9d42f060416e1ab784ad991",
            subdir: "sqlite-autoconf-3540000",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(target, &[])
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }
    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }
    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Profiled
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo(&["--prefix=/usr"], pgo_workload())
    }
}

fn pgo_workload() -> Vec<Step> {
    use crate::package::recipe::{Arg, Pipe, argv};

    let exec = |pipe: Pipe| Step::Exec { env: vec![], pipe };
    vec![
        exec(Pipe::cmd(argv(&[
            "./sqlite3",
            "/tmp/pgo.db",
            "\"CREATE TABLE t(a INTEGER, b TEXT); CREATE INDEX ti ON t(a);\"",
        ]))),
        Step::ForEach {
            var: "_i",
            items: vec![Arg::Subst(Box::new(Pipe::cmd(argv(&["seq", "1", "200"]))))],
            body: vec![exec(Pipe::cmd(argv(&[
                "./sqlite3",
                "/tmp/pgo.db",
                "\"INSERT INTO t VALUES(${_i}, 'val'); SELECT count(*), sum(a) FROM t; SELECT a FROM t t1 JOIN t t2 ON t1.a = t2.a LIMIT 100; BEGIN; UPDATE t SET b='x' WHERE a=1; COMMIT;\"",
            ])))],
            stdout: None,
        },
        exec(Pipe::cmd(argv(&[
            "./sqlite3",
            "/tmp/pgo.db",
            "\"EXPLAIN QUERY PLAN SELECT * FROM t WHERE a > 10 ORDER BY b; VACUUM;\"",
        ]))),
    ]
}
