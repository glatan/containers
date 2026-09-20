set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export BUILD_CC="$CC_FOR_BUILD"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
cat > meson-cross.ini <<EOF
[binaries]
c = '$CC'
cpp = '${CXX:-$CC}'
ar = '${AR:-${TRIPLE}-ar}'
strip = 'aarch64-linux-gnu-strip'
pkg-config = 'pkg-config'
[host_machine]
system = 'linux'
cpu_family = 'aarch64'
cpu = 'aarch64'
endian = 'little'
EOF
meson setup "${SOURCE}" --prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled --cross-file "${BUILD}/meson-cross.ini"
meson compile -C . -j $(nproc)
meson install -C . --destdir "${OUT}"
