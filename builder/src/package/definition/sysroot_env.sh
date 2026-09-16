if [ -d /dep/${PACKAGE_HOST:-$TRIPLE}-glibc/usr/${PACKAGE_HOST:-$TRIPLE} ]; then
  SYSROOT=/dep/${PACKAGE_HOST:-$TRIPLE}-glibc/usr/${PACKAGE_HOST:-$TRIPLE}
elif [ -d /dep/${TRIPLE}-glibc/usr/${TRIPLE} ]; then
  SYSROOT=/dep/${TRIPLE}-glibc/usr/${TRIPLE}
elif [ -d /dep/${PACKAGE_HOST:-$TRIPLE}-glibc ]; then
  SYSROOT=/dep/${PACKAGE_HOST:-$TRIPLE}-glibc
elif [ -d /dep/${TRIPLE}-glibc ]; then
  SYSROOT=/dep/${TRIPLE}-glibc
fi
if [ -n "${SYSROOT:-}" ]; then
  export CFLAGS="${CFLAGS:-} --sysroot=$SYSROOT"
  export CXXFLAGS="${CXXFLAGS:-} --sysroot=$SYSROOT"
  export CPPFLAGS="${CPPFLAGS:-} --sysroot=$SYSROOT"
  export LDFLAGS="${LDFLAGS:-} --sysroot=$SYSROOT"
fi
if [ -d /dep/linux-headers/usr/${PACKAGE_HOST:-$TRIPLE}/include ]; then
  HDR=/dep/linux-headers/usr/${PACKAGE_HOST:-$TRIPLE}/include
elif [ -d /dep/linux-headers/usr/${TRIPLE}/include ]; then
  HDR=/dep/linux-headers/usr/${TRIPLE}/include
elif [ -d /dep/linux-headers/usr/include ]; then
  HDR=/dep/linux-headers/usr/include
fi
if [ -n "${HDR:-}" ]; then
  export CFLAGS="${CFLAGS:-} -idirafter $HDR"
  export CXXFLAGS="${CXXFLAGS:-} -idirafter $HDR"
  export CPPFLAGS="${CPPFLAGS:-} -idirafter $HDR"
fi
