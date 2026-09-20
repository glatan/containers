set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
if [ -n "${SYSROOT:-}" ]; then
  export CC="${CC} --sysroot=${SYSROOT}"
fi
"${SOURCE}/Configure" linux-x86_64 --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
export PGODIR=/tmp/pgo
rm -rf "${PGODIR}"
mkdir -p "${PGODIR}"
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic" LDFLAGS="$LDFLAGS -fprofile-generate"
export LD_LIBRARY_PATH="${PWD}${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
export OPENSSL_MODULES="${PWD}/providers"
export OPENSSL_CONF=/dev/null
(
  ./apps/openssl speed -seconds 1 rsa2048 ecdsap256 ed25519
  ./apps/openssl speed -seconds 1 -evp aes-128-gcm -evp aes-256-gcm -evp chacha20-poly1305 -evp sha256 -evp sha512
  ./apps/openssl speed -seconds 1 aes-128-cbc aes-256-cbc sha256 sha512
) || true
make clean
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="$LDFLAGS -fprofile-use"
make install_sw DESTDIR="${OUT}"
