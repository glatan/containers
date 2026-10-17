set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
mkdir -p "$BUILD/.cargo"
mkdir -p /build/ccw
cat > /build/ccw/mold-link <<'EOF'
#!/bin/sh
exec "$CC" --sysroot=/tmp/sysroot -L/tmp/sysroot/lib -L/tmp/sysroot/lib64 "$@"
EOF
chmod +x /build/ccw/mold-link
cat > $BUILD/.cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/rui314/mimalloc_rust?rev=3979460494f1cd1e7f936cb8e10f41e927c9f698"]
git = "https://github.com/rui314/mimalloc_rust"
rev = "3979460494f1cd1e7f936cb8e10f41e927c9f698"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/src/vendor"
EOF
CARGO_HOME="${BUILD}/cargo-home" CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/build/ccw/mold-link CC_aarch64_unknown_linux_gnu=${CC} CXX_aarch64_unknown_linux_gnu=${CXX} AR_aarch64_unknown_linux_gnu=${AR} CFLAGS_aarch64_unknown_linux_gnu=${CFLAGS} CC_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-gcc" CXX_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-g++" AR_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-ar" HOST_CC="${HOST_TRIPLE}-gcc" HOST_CXX="${HOST_TRIPLE}-g++" CFLAGS_x86_64_unknown_linux_gnu= CXXFLAGS_x86_64_unknown_linux_gnu= LDFLAGS_x86_64_unknown_linux_gnu= cargo build --release --locked --offline --target aarch64-unknown-linux-gnu
DESTDIR="${OUT}" PREFIX=/usr MOLD_LIBDIR=/usr/lib CARGO_TARGET_DIR="$BUILD/target/aarch64-unknown-linux-gnu" sh "$BUILD/install-mold.sh"
