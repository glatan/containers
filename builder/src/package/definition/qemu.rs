use crate::flags::Flag;
use crate::package::{Dependency, Package, Source};
use crate::target::Target;

pub struct Qemu;

impl Package for Qemu {
    fn name(&self) -> &'static str {
        "qemu"
    }
    fn version(&self) -> &'static str {
        "11.1.1"
    }
    fn license(&self) -> &'static str {
        "GPL-2.0-only"
    }
    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://download.qemu.org/qemu-11.1.1.tar.xz",
            sha3_512: "8b53993040fe8ce2e6c69e2d616a952fcd3da00ddcd114b64dea2d5b1cc0a4f6df03bc899ba12437b6166559be8f2379b97a737608659f6b2c6ab56605bd20a2",
            blake3: "9cda9512c997d0ec176add30ff4f4531d4882681eb12e98ba5c7d11cb4cf610ca079689e5def5ae36c1c1a173dee7eb001ce8fffeba77f418048e4b4773dff67",
            subdir: "qemu-11.1.1",
        }
    }
    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(
            target,
            &[
                Dependency {
                    name: "glib",
                    version: "2.90.0",
                },
                Dependency {
                    name: "zlib",
                    version: "1.3.2",
                },
                Dependency {
                    name: "zstd",
                    version: "1.5.7",
                },
            ],
        )
    }
    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![
            crate::package::LINUX_HEADERS,
            Dependency {
                name: "pcre2",
                version: "10.48",
            },
            Dependency {
                name: "libffi",
                version: "3.8.0",
            },
        ]
    }
    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }
    fn supports(&self, target: Target) -> bool {
        target == Target::BUILD_ENV
    }
    fn build_script(&self) -> &'static str {
        r#"set -euxo pipefail
# meson の sizeof 検査はリンクもする。--sysroot は HOST glibc に /usr/include が無く
# sizeof(void*)=-1 になり、64-bit linux-user の TCG が全部落ちる。
strip_sysroot() { printf '%s' "$1" | sed -E 's/--sysroot=[^ ]+//g'; }
export CFLAGS="$(strip_sysroot "${CFLAGS:-}") -flto=auto"
export CXXFLAGS="$(strip_sysroot "${CXXFLAGS:-}") -flto=auto"
export LDFLAGS="$(strip_sysroot "${LDFLAGS:-}") -flto=auto"
if [ -d /dep/${TRIPLE}-glibc ]; then
  _glibc=/dep/${TRIPLE}-glibc
  export LDFLAGS="$LDFLAGS -B$_glibc/usr/lib64 -L$_glibc/usr/lib64 -L$_glibc/lib64"
fi
if [ -d /dep/linux-headers/usr/include ]; then
  export CFLAGS="$CFLAGS -idirafter /dep/linux-headers/usr/include"
  export CXXFLAGS="$CXXFLAGS -idirafter /dep/linux-headers/usr/include"
fi
PKG_CONFIG_LIBDIR=
for _dep in glib pcre2 zlib libffi; do
  for _pc in /dep/"$_dep"/usr/lib/pkgconfig /dep/"$_dep"/usr/lib64/pkgconfig; do
    if [ -d "$_pc" ]; then
      PKG_CONFIG_LIBDIR="${PKG_CONFIG_LIBDIR:+$PKG_CONFIG_LIBDIR:}$_pc"
    fi
  done
done
if [ -n "$PKG_CONFIG_LIBDIR" ]; then
  export PKG_CONFIG_LIBDIR
  export PKG_CONFIG_PATH=
fi
if [ -d /dep/glib ]; then
  export PKG_CONFIG_SYSROOT_DIR=/dep/glib
fi
mkdir -p "$BUILD"
cd "$BUILD"
"$SOURCE"/configure --prefix=/usr --target-list=x86_64-linux-user,aarch64-linux-user,riscv64-linux-user,ppc64le-linux-user,s390x-linux-user \
  --enable-tcg --disable-system --disable-tools --disable-docs --disable-guest-agent --disable-slirp --disable-vnc --disable-sdl --disable-gtk --disable-curses --disable-brlapi --disable-spice --disable-libnfs --disable-libusb --disable-vde --disable-nettle --disable-gnutls --disable-gcrypt --disable-auth-pam
make -j"$(nproc)"
make install DESTDIR="$OUT"
if command -v qemu-s390x >/dev/null; then
  qemu-s390x -cpu help | grep -E 'z900|z990|z9EC' || true
fi
"#
    }
}
