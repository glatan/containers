use crate::flags::Flag;
use crate::package::{Dependency, Package, Role, Source};
use crate::target::Target;

pub struct LinuxHeaders;

impl Package for LinuxHeaders {
    fn name(&self) -> &'static str {
        "linux-headers"
    }

    fn version(&self) -> &'static str {
        "6.18.33"
    }

    fn license(&self) -> &'static str {
        "GPL-2.0-only WITH Linux-syscall-note"
    }

    fn build_only(&self) -> bool {
        true
    }

    fn role(&self) -> Role {
        Role::LinuxHeaders
    }

    fn source(&self) -> Source {
        Source {
            url: "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.18.33.tar.xz",
            sha3_512: "ad87ec123afe83db49e304a4f32d1c529d525f2e55d8c78de208cff0de2b1306d1eaa35d0b49a3738ffa170977e252e1e85fa58654710f11de9f4267947258ca",
            blake3: "00d95e1098670a56ee94558692f1e63e0ac3d2113c9e46568e2da9423a37ad8855a530aba3e6f67bc43beacb5edd7a54834cb0f946112630143b7b7bd930f707",
            subdir: "linux-6.18.33",
        }
    }

    fn dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }

    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }

    fn build_script(&self) -> &'static str {
        r#"set -euxo pipefail
install_uapi() {
  local karch="$1" dest="$2"
  local o="$BUILD/$karch"
  mkdir -p "$o" "$dest"
  make -C "$SOURCE" O="$o" ARCH="$karch" headers
  cp -a "$o/usr/include/." "$dest/"
}
install_uapi x86_64 "$OUT/usr/x86_64-linux-gnu/include"
install_uapi arm64 "$OUT/usr/aarch64-linux-gnu/include"
install_uapi riscv "$OUT/usr/riscv64-linux-gnu/include"
install_uapi powerpc "$OUT/usr/powerpc64le-linux-gnu/include"
install_uapi s390 "$OUT/usr/s390x-linux-gnu/include"
mkdir -p "$OUT/usr/include"
cp -a "$OUT/usr/${HOST_TRIPLE}/include/." "$OUT/usr/include/"
"#
    }
}
