set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
if [ "${TRIPLE:-}" != "${HOST_TRIPLE:-}" ]; then
  export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
  export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
  export BUILD_CC="$CC_FOR_BUILD"
fi
autotools_configure() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"
}
