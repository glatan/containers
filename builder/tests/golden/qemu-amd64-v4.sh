set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
export CFLAGS="$(printf '%s' "${CFLAGS:-}" | sed -E 's/--sysroot=[^ ]+//g') -flto=auto"
export CXXFLAGS="$(printf '%s' "${CXXFLAGS:-}" | sed -E 's/--sysroot=[^ ]+//g') -flto=auto"
export LDFLAGS="$(printf '%s' "${LDFLAGS:-}" | sed -E 's/--sysroot=[^ ]+//g') -flto=auto"
if [ -d /dep/${TRIPLE}-glibc ]; then
  _glibc=/dep/${TRIPLE}-glibc
  export LDFLAGS="${LDFLAGS} -B${_glibc}/usr/lib64 -L${_glibc}/usr/lib64 -L${_glibc}/lib64"
fi
if [ -d /dep/linux-headers/usr/include ]; then
  export CFLAGS="${CFLAGS} -idirafter /dep/linux-headers/usr/include"
  export CXXFLAGS="${CXXFLAGS} -idirafter /dep/linux-headers/usr/include"
fi
PKG_CONFIG_LIBDIR=
for _dep in glib pcre2 zlib libffi; do
  for _pc in /dep/"${_dep}"/usr/lib/pkgconfig /dep/"${_dep}"/usr/lib64/pkgconfig; do
    if [ -d "${_pc}" ]; then
      PKG_CONFIG_LIBDIR="${PKG_CONFIG_LIBDIR:+$PKG_CONFIG_LIBDIR:}${_pc}"
    fi
  done
done
if [ -n "${PKG_CONFIG_LIBDIR:-}" ]; then
  export PKG_CONFIG_LIBDIR
  export PKG_CONFIG_PATH=
fi
if [ -d /dep/glib ]; then
  export PKG_CONFIG_SYSROOT_DIR=/dep/glib
fi
"$SOURCE"/configure --prefix=/usr --target-list=x86_64-linux-user,aarch64-linux-user,riscv64-linux-user,ppc64le-linux-user,s390x-linux-user --enable-tcg --disable-system --disable-tools --disable-docs --disable-guest-agent --disable-slirp --disable-vnc --disable-sdl --disable-gtk --disable-curses --disable-brlapi --disable-spice --disable-libnfs --disable-libusb --disable-vde --disable-nettle --disable-gnutls --disable-gcrypt --disable-auth-pam
make -j"$(nproc)"
make install DESTDIR="$OUT"
if command -v qemu-s390x >/dev/null; then
  qemu-s390x -cpu help | grep -E 'z900|z990|z9EC' || true
fi
