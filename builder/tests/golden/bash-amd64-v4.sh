set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
mkdir -p "$BUILD"
cd "$BUILD"
export PGODIR=/tmp/pgo
rm -rf "$PGODIR"
mkdir -p "$PGODIR"
export _PGO_CFLAGS="${CFLAGS:-}" _PGO_CXXFLAGS="${CXXFLAGS:-}" _PGO_LDFLAGS="${LDFLAGS:-}"
export CFLAGS="$_PGO_CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-generate"
autotools_configure --prefix=/usr --without-bash-malloc
make -j"$(nproc)"
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
for _libdir in "$PWD"/lib "$PWD"/programs $(find "$PWD" -type d -name .libs 2>/dev/null); do
  [ -d "$_libdir" ] && export LD_LIBRARY_PATH="$_libdir:$LD_LIBRARY_PATH"
done
(
  ./bash -c '
  x=0; for i in $(seq 1 10000); do x=$((x+i)); done; echo "$x" > /dev/null
  s=""; for f in "$SOURCE"/*; do s="$s${f##*/}"; done; echo "${#s}" > /dev/null
  for i in $(seq 1 2000); do v="abc$i"; printf "%s" "${v%[0-9]*}${v##*[a-z]}" > /dev/null; done
  shopt -s nullglob; for f in "$SOURCE"/*.{c,h} "$SOURCE"/*/*.c; do echo "${#f}" > /dev/null; done
  '
) || true
make distclean || make clean
export CFLAGS="$_PGO_CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export CXXFLAGS="$_PGO_CXXFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"
export LDFLAGS="$_PGO_LDFLAGS -fprofile-use"
autotools_configure --prefix=/usr --without-bash-malloc
make -j"$(nproc)"
make install DESTDIR="$OUT"
ln -sf bash "$OUT"/usr/bin/sh
