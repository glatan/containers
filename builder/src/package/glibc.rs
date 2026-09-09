use crate::package::{Dependency, Package, Source};

pub struct Glibc;

impl Package for Glibc {
    fn name(&self) -> &'static str {
        "glibc"
    }

    fn version(&self) -> &'static str {
        "2.44"
    }

    fn license(&self) -> &'static str {
        "LGPL-2.1-or-later"
    }

    fn build_only(&self) -> bool {
        false
    }

    fn network(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz",
            sha3_512: "263dfb5e331b2033368c7455abcd9a9b15ab098ac963d5518ecf4bcb9eeb41298eb9976bf78a64af67d9471a50161bced2fa69d429c69759ea57f76b942ac521",
            blake3: "6d07bc16d8d066b1c0ae713d762175f261e27df9cab8015ec11e1c980c05e79b0fd5bab6cde6a90bf07651c432ca9039629376b57a96c9f3dcb3765f3745dbf4",
            subdir: "glibc-2.44",
        }
    }

    fn dependencies(&self) -> &'static [Dependency] {
        &[]
    }

    fn build_dependencies(&self) -> &'static [Dependency] {
        &[Dependency {
            name: "linux-headers",
            version: "6.12.109",
        }]
    }

    fn build_script(&self) -> &'static str {
        r#"set -euxo pipefail
if [ -n "$CROSS" ]; then
  pacman -Sy --noconfirm "${CROSS}gcc"
  export CC="$CROSS"gcc AR="$CROSS"ar RANLIB="$CROSS"ranlib
fi
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "$CROSS" ]; then
  "$SOURCE"/configure --prefix=/usr --disable-werror \
    --with-headers=/dep/linux-headers/usr/include \
    --host="$TRIPLE" --build="$HOST_TRIPLE" \
    libc_cv_forced_unwind=yes libc_cv_c_cleanup=yes libc_cv_ssp=no
else
  "$SOURCE"/configure --prefix=/usr --disable-werror \
    --with-headers=/dep/linux-headers/usr/include
fi
make -j"$(nproc)"
make install DESTDIR="$OUT"
if [ -n "$CROSS" ]; then
  mkdir -p "$OUT"/usr/"$TRIPLE"
  ln -sfn ../include "$OUT"/usr/"$TRIPLE"/include
  ln -sfn ../lib64 "$OUT"/usr/"$TRIPLE"/lib
  ln -sfn ../lib64 "$OUT"/usr/"$TRIPLE"/lib64
  LD=$(sed -n 's|.*AS_NEEDED ( /[^/]*/\(ld-[^ )]*\) ).*|\1|p' "$OUT"/usr/lib64/libc.so)
  case "$ARCH" in
    amd64|s390x|ppc64le) LDDIR=lib64 ;;
    *) LDDIR=lib ;;
  esac
  sed -i "s|GROUP (.*)|GROUP ( ../../lib64/libc.so.6 libc_nonshared.a AS_NEEDED ( ../../$LDDIR/$LD ) )|" "$OUT"/usr/lib64/libc.so
fi
"#
    }
}
