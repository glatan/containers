use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps};
use crate::package::{Package, Source};
use crate::target::Target;

pub struct Texinfo;

impl Package for Texinfo {
    fn name(&self) -> &'static str {
        "texinfo"
    }
    fn version(&self) -> &'static str {
        "7.3"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        vec!["perl"]
    }

    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/texinfo/texinfo-7.3.tar.xz",
            sha3_512: "04243a6f04eec6f208fecea57c16fc4a9f5fd0487c670370295d18edc6d266bbff3fa60d8a547687d8c17de19218c28434ac5359a3d29dde9720f53d86f33938",
            blake3: "01eae8c848df2fbd6903aeba43fef7ac45a019b4c72d4cd853f936940408102561499b58ecfc36753ea84e31dbbc79fc8c43d950380dc33f3bf8402c7d56aa49",
            subdir: "texinfo-7.3",
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
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let mut args = vec!["--prefix=/usr".to_owned()];
        if ctx.cross() {
            args.push("--disable-perl-xs".into());
        }
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .pgo(crate::package::recipe::PgoTemplate::Autotools(PGO_WORKLOAD))
                .step(Step::Configure(args))
                .step(Step::Shell(
                    "if [ ! -f tools/Makefile ]; then
  mkdir -p tools
  printf 'all:\ninstall:\ncheck:\n.PHONY: all install check\n' > tools/Makefile
fi",
                ))
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Profiled(PGO_WORKLOAD)
    }
}

const PGO_WORKLOAD: &str = r#"# texi2any は configure が生成するラッパー。out-of-tree で
# 見つからない場合に備えてビルドツリーから解決する。
_texi2any="$BUILD/tp/texi2any"
if [ ! -x "$_texi2any" ]; then
  _texi2any=$(find "$BUILD" -name texi2any -type f -print -quit)
fi
for _texi in "$SOURCE"/doc/*.texi; do
  [ -f "$_texi" ] && "$_texi2any" --plaintext "$_texi" > /dev/null 2>&1
done
find "$SOURCE" -name '*.texi' | head -5 | while read -r _t; do
  "$_texi2any" --html "$_t" -o /tmp/texi-out 2>/dev/null
done"#;
