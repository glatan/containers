set -euxo pipefail
mkdir -p "$BUILD/x86_64" "$OUT/usr/x86_64-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/x86_64" ARCH="x86_64" headers
cp -a "$BUILD/x86_64/usr/include/." "$OUT/usr/x86_64-linux-gnu/include/"
mkdir -p "$BUILD/arm64" "$OUT/usr/aarch64-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/arm64" ARCH="arm64" headers
cp -a "$BUILD/arm64/usr/include/." "$OUT/usr/aarch64-linux-gnu/include/"
mkdir -p "$BUILD/riscv" "$OUT/usr/riscv64-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/riscv" ARCH="riscv" headers
cp -a "$BUILD/riscv/usr/include/." "$OUT/usr/riscv64-linux-gnu/include/"
mkdir -p "$BUILD/powerpc" "$OUT/usr/powerpc64le-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/powerpc" ARCH="powerpc" headers
cp -a "$BUILD/powerpc/usr/include/." "$OUT/usr/powerpc64le-linux-gnu/include/"
mkdir -p "$BUILD/s390" "$OUT/usr/s390x-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/s390" ARCH="s390" headers
cp -a "$BUILD/s390/usr/include/." "$OUT/usr/s390x-linux-gnu/include/"
mkdir -p "$OUT/usr/include"
cp -a "$OUT/usr/${HOST_TRIPLE}/include/." "$OUT/usr/include/"
