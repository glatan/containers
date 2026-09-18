use crate::package::recipe::{BuildContext, LTO_ENV, Recipe, SYSROOT_ENV};
use crate::package::{Package, Source};

pub struct Glib;

impl Package for Glib {
    fn name(&self) -> &'static str {
        "glib"
    }
    fn version(&self) -> &'static str {
        "2.90.0"
    }
    fn license(&self) -> &'static str {
        "LGPL-2.1-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://download.gnome.org/sources/glib/2.90/glib-2.90.0.tar.xz",
            sha3_512: "c896df2b43beb87254843d355e2fc389d9d62e01a49c58e078f3736286c0f0f16eedcfb611b19edb13ebf38ec8714ab1639b7b52b1ad02301eb1365cddaa205d",
            blake3: "0cd33e9b83b0fbc9b8396e99a59504909ced4e2f233f445e6127dacbcf17c0d00cc425186477da98e7f37cacfc8a27f8957f28b6b1cbd94ba1e98deef2570866",
            subdir: "glib-2.90.0",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[
                crate::package::Dependency {
                    name: "libffi",
                    version: "3.8.0",
                },
                crate::package::Dependency {
                    name: "pcre2",
                    version: "10.48",
                },
                crate::package::Dependency {
                    name: "zlib",
                    version: "1.3.2",
                },
            ],
        )
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        let mut deps = crate::package::headers_dep();
        deps.extend([
            crate::package::Dependency {
                name: "meson",
                version: "1.12.0",
            },
            crate::package::Dependency {
                name: "ninja",
                version: "1.13.2",
            },
            crate::package::Dependency {
                name: "python",
                version: "3.14.7",
            },
            crate::package::Dependency {
                name: "pkgconf",
                version: "2.5.1",
            },
        ]);
        deps
    }
    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }
    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        let mut script = String::from(LTO_ENV);
        script.push_str(SYSROOT_ENV);
        script.push_str(
            r#"mkdir -p "$BUILD"
cd "$BUILD"
meson_args=(--prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled)
if [ "$TRIPLE" != "$HOST_TRIPLE" ]; then
  case "$TRIPLE" in
    aarch64-*) cpu_family=aarch64; cpu=aarch64 ;;
    riscv64-*) cpu_family=riscv64; cpu=riscv64 ;;
    x86_64-*) cpu_family=x86_64; cpu=x86_64 ;;
    powerpc64le-*) cpu_family=ppc64; cpu=ppc64le ;;
    s390x-*) cpu_family=s390x; cpu=s390x ;;
    *) echo "未対応の meson cpu: $TRIPLE"; exit 1 ;;
  esac
  cat > meson-cross.ini <<EOF
[binaries]
c = '$CC'
cpp = '${CXX:-$CC}'
ar = '${AR:-${TRIPLE}-ar}'
strip = '${TRIPLE}-strip'
pkg-config = 'pkg-config'
[host_machine]
system = 'linux'
cpu_family = '$cpu_family'
cpu = '$cpu'
endian = 'little'
EOF
  meson_args+=(--cross-file "$BUILD/meson-cross.ini")
fi
meson setup "$SOURCE" "${meson_args[@]}"
meson compile -C . -j "$(nproc)"
meson install -C . --destdir "$OUT"
"#,
        );
        Recipe::Shell(script.into())
    }
}
