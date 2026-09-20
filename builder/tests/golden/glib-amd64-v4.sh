set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
meson setup "${SOURCE}" --prefix=/usr --wrap-mode=nofallback -Dlibmount=disabled -Dnls=disabled -Dtests=false -Dsysprof=disabled -Dintrospection=disabled
meson compile -C . -j $(nproc)
meson install -C . --destdir "${OUT}"
