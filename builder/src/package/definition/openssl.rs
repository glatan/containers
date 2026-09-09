use crate::package::{Dependency, Package, Source};

pub struct Openssl;

impl Package for Openssl {
    fn name(&self) -> &'static str {
        "openssl"
    }

    fn version(&self) -> &'static str {
        "3.6.4"
    }

    fn license(&self) -> &'static str {
        "Apache-2.0"
    }

    fn build_only(&self) -> bool {
        false
    }

    fn network(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/openssl/openssl/releases/download/openssl-3.6.4/openssl-3.6.4.tar.gz",
            sha3_512: "ae8c16f7fa9003fe7dac1fc538a8e6805ccafd39beb05a9b3d5a4d0adbf7a3a278de34b04779337c0bea99b98458e485463e6ce2cc71e36dd78f279433664b15",
            blake3: "ce6ba56d9f75a9e6965cd98385a85b57aa7d1a001dbb9c638c3ce50d391554c6a4cb4e336bbc22c7d3111b199ac4d893bb66174d23159c9ebcbb0b9aa496a050",
            subdir: "openssl-3.6.4",
        }
    }

    fn dependencies(&self) -> &'static [Dependency] {
        &[Dependency {
            name: "glibc",
            version: "2.44",
        }]
    }

    fn build_dependencies(&self) -> &'static [Dependency] {
        &[Dependency {
            name: "linux-headers",
            version: "6.12.109",
        }]
    }

    fn build_script(&self) -> &'static str {
        r#"set -euxo pipefail
case "$ARCH" in
  amd64) TGT=linux-x86_64 ;;
  arm64) TGT=linux-aarch64 ;;
  riscv64) TGT=linux64-riscv64 ;;
  *) echo "未対応の architecture です: $ARCH"; exit 1 ;;
esac
if [ -n "$CROSS" ]; then
  pacman -Sy --noconfirm "${CROSS}gcc"
fi
mkdir -p "$BUILD"
cd "$BUILD"
export CC="$CROSS"gcc
export AR="${CROSS}gcc-ar"
export RANLIB="${CROSS}gcc-ranlib"
export NM="${CROSS}gcc-nm"
if [ -n "$CROSS" ]; then
  export CC="$CC -B/dep/glibc/usr/lib64 -L/dep/glibc/usr/lib64 -L/dep/glibc/lib64"
fi
export CC="$CC --sysroot=/dep/glibc -idirafter /dep/linux-headers/usr/include"
configure() {
  "$SOURCE"/Configure "$TGT" --prefix=/usr --openssldir=/etc/ssl --libdir=lib64 enable-lto "$@"
}
if [ -z "$CROSS" ]; then
  configure enable-profile-generate
  make -j"$(nproc)"
  # PGO プロファイル生成(実行できるネイティブビルドのみ)
  ./apps/openssl speed -seconds 1 \
    rsa2048 ecdsap256 \
    aes-128-cbc aes-256-cbc \
    sha256 sha512
  configure enable-profile-use
  make -j"$(nproc)"
else
  configure
  make -j"$(nproc)"
fi
make install_sw DESTDIR="$OUT"
"#
    }
}
