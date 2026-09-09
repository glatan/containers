use crate::package::{Dependency, Package, Source};

pub struct LinuxHeaders;

impl Package for LinuxHeaders {
    fn name(&self) -> &'static str {
        "linux-headers"
    }

    fn version(&self) -> &'static str {
        "6.12.109"
    }

    fn license(&self) -> &'static str {
        "GPL-2.0-only WITH Linux-syscall-note"
    }

    fn build_only(&self) -> bool {
        true
    }

    fn network(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.109.tar.xz",
            sha3_512: "e8a40f7302e8c93710001876941c49744e8120fbb5094c7fa1b08cfc360e2fa61c73b3d1b144b0fb4933ae5286d5b833377535567547a6167dac56a081c474f2",
            blake3: "9abb3b5ba08f97c241bb995571e927fc13e6b2c946081f23c9a28e231aef576bf9d0d2e770b963129b59efc9ac4ce950f2909a44f718f43e3fa033b3adafa069",
            subdir: "linux-6.12.109",
        }
    }

    fn dependencies(&self) -> &'static [Dependency] {
        &[]
    }

    fn build_dependencies(&self) -> &'static [Dependency] {
        &[]
    }

    fn build_script(&self) -> &'static str {
        r#"set -euxo pipefail
case "$ARCH" in
  amd64) KARCH=x86_64 ;;
  arm64) KARCH=arm64 ;;
  *) echo "未対応の architecture です: $ARCH"; exit 1 ;;
esac
pacman -Sy --noconfirm rsync
mkdir -p "$BUILD"
make -C "$SOURCE" O="$BUILD" headers_install ARCH="$KARCH" INSTALL_HDR_PATH="$OUT"/usr
"#
    }
}
