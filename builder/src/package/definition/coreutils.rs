use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Bootstrap, Package, Source};

pub struct Coreutils;

impl Package for Coreutils {
    fn name(&self) -> &'static str {
        "coreutils"
    }
    fn version(&self) -> &'static str {
        "9.12"
    }
    fn license(&self) -> &'static str {
        "GPL-3.0-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/coreutils/coreutils-9.12.tar.xz",
            sha3_512: "d6f1fbd33b86bce8c0eec25875c04192e70a5a946561e2ed5a8fd9a4214e309cb07e0cc4ce1f86b228beeedc4f7fbf1c1218bd5dd3a45a308edd1fefd4e4d82b",
            blake3: "420edafa8b231145442d248c00412727651f20435e85dc80fbf18074fc249624c6ff75aaed836074f668767844667fe0ccc3ffa59975cb5bda8e29348dcb2779",
            subdir: "coreutils-9.12",
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
        crate::flags::Pgo::Profiled(PGO_WORKLOAD)
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::autotools_pgo(
            &[
                "--prefix=/usr",
                "--disable-acl",
                "--disable-xattr",
                "--without-selinux",
            ],
            PGO_WORKLOAD,
        )
    }
}

impl Bootstrap for Coreutils {}

const PGO_WORKLOAD: &str = r#"./src/ls -lR "$SOURCE" > /dev/null
./src/sort "$SOURCE"/src/*.c > /dev/null 2>&1 || true
./src/cp -a "$SOURCE" /tmp/coreutils-copy
./src/wc -l "$SOURCE"/src/*.c > /dev/null 2>&1 || true
./src/cat "$SOURCE"/configure | ./src/md5sum > /dev/null
find "$SOURCE" -name '*.h' -print0 | ./src/sort -z > /dev/null"#;
