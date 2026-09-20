set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
"${SOURCE}/Configure" linux-aarch64 --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
make -j"$(nproc)"
make install_sw DESTDIR="${OUT}"
