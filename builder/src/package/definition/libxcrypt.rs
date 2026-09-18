use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Package, Source};
use crate::target::Target;

pub struct Libxcrypt;

impl Package for Libxcrypt {
    fn name(&self) -> &'static str {
        "libxcrypt"
    }
    fn version(&self) -> &'static str {
        "4.5.2"
    }
    fn license(&self) -> &'static str {
        "LGPL-2.1-or-later"
    }
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        vec!["perl"]
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/besser82/libxcrypt/releases/download/v4.5.2/libxcrypt-4.5.2.tar.xz",
            sha3_512: "7c5eeccd7d7706048d73109937aba4624d9641f22965b154ba23ec8373c6c2bc939d182704c214f7a1a8d7600f5d0f61720d4e4a57913ab500d050d647a3ffe8",
            blake3: "8294e7735d240876f4c4c38a1370e1e9e9e2da4856fcfcb81168ea20ab83f33d1cc591380e2f0c00eb45ff4ac999b47c54d0d20b05b29757b1d147559227a9c9",
            subdir: "libxcrypt-4.5.2",
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
        crate::flags::Pgo::Profiled(MAKE_CHECK_WORKLOAD)
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo(&["--prefix=/usr", "--disable-werror"], MAKE_CHECK_WORKLOAD)
    }
}

/// make check へフラグ変数を伝播（テストバイナリも instrumented 化）。
const MAKE_CHECK_WORKLOAD: &str =
    "make check CFLAGS=\"$CFLAGS\" CXXFLAGS=\"${CXXFLAGS:-}\" LDFLAGS=\"$LDFLAGS\"";
