use crate::package::recipe::{BuildContext, Env, Recipe, Std, Step, Steps, Workdir};
use crate::package::{Package, Source};

pub struct Flex;

impl Package for Flex {
    fn name(&self) -> &'static str {
        "flex"
    }
    fn version(&self) -> &'static str {
        "2.6.4"
    }
    fn license(&self) -> &'static str {
        "BSD-2-Clause"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/westes/flex/files/981163/flex-2.6.4.tar.gz",
            sha3_512: "eac223beda4c1629dccfd29666eab9f510e05f5b606587496364ae59622c3bab8104a9ac52f4bb06098a628a4d4fba0c59f5a9d851d65cbc94ed0a660475e162",
            blake3: "d78b714ac38bd9de7f9b44a078efed82e96ed43e7cf9cd33944a7f379a2d09a4d024e6d3a3accb71ae72f36b4f028328f551c6271a231cdd3c307b542cdca143",
            subdir: "flex-2.6.4",
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
    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::Autotools)
                // gnulib の lib/malloc.c は K&R 宣言のため gnu17 で壊れる
                .std(Std::Gnu11)
                .workdir(Workdir::InSource)
                .step(Step::Shell(
                    r#"if [ -f lib/malloc.c ]; then
  cat > lib/malloc.c <<'EOF'
#include <stdlib.h>
void *rpl_malloc(size_t n) { if (n == 0) n = 1; return malloc(n); }
EOF
fi
if [ -f lib/realloc.c ]; then
  cat > lib/realloc.c <<'EOF'
#include <stdlib.h>
void *rpl_realloc(void *p, size_t n) { if (n == 0) n = 1; return realloc(p, n); }
EOF
fi"#,
                ))
                .step(Step::configure(&["--prefix=/usr"]))
                .step(Step::Shell(
                    "find . -name config.h -exec sed -i '/rpl_malloc/d;/rpl_realloc/d' {} +",
                ))
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }
}
