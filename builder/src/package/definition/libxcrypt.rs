use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Dependency, Package, Source, dep};
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
    /// ビルドは tarball 事前生成物で自足するが、perl が
    /// あれば使う（任意）。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        deps.push(dep("perl", "5.42.3").via(crate::package::Via::Optional));
        deps
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
        crate::flags::Pgo::Profiled
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo(
            &["--prefix=/usr", "--disable-werror"],
            Recipe::make_check_workload(),
        )
    }
}
