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
CARGO_HOME="${BUILD}/cargo-home" CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/build/ccw/mold-link CC_x86_64_unknown_linux_gnu=${CC} CXX_x86_64_unknown_linux_gnu=${CXX} AR_x86_64_unknown_linux_gnu=${AR} CFLAGS_x86_64_unknown_linux_gnu=${CFLAGS} cargo build --release --locked --offline
DESTDIR="${OUT}" PREFIX=/usr MOLD_LIBDIR=/usr/lib sh "$BUILD/install-mold.sh"
