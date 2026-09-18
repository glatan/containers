use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Recipe};
use crate::package::{Dependency, Package, Role, Source};
use crate::target::Target;

pub struct Openssl;

impl Package for Openssl {
    fn name(&self) -> &'static str {
        "openssl"
    }

    fn version(&self) -> &'static str {
        "4.0.2"
    }

    fn license(&self) -> &'static str {
        "Apache-2.0"
    }

    fn role(&self) -> Role {
        Role::Catalog
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/openssl/openssl/releases/download/openssl-4.0.2/openssl-4.0.2.tar.gz",
            sha3_512: "a7d86c8609e98dea12cab18f22724439a1fda43ef9a45a6440a201a6d9e616fe29dcf83dbffbc5c303cc5f43a1c92f5d222adb29f03a30b1e0b578af0a47dc4b",
            blake3: "ef5af5011e1ee875521ed63331cf35d16e82167042b37f361a630b10f5bf2f999fad90fcc972bc02a0d83e2972af48040140d4e78279311b46a7c1557c206d7a",
            subdir: "openssl-4.0.2",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        let mut dependencies = crate::package::headers_dep();
        dependencies.push(Dependency {
            name: "perl",
            version: "5.42.3",
        });
        dependencies
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn pgo(&self) -> crate::flags::Pgo {
        crate::flags::Pgo::Custom
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let mut script = String::from(HEAD);
        // PGO: ネイティブビルドでのみプロファイルを実行できる
        if ctx.cross() {
            script.push_str("make -j\"$(nproc)\"\n");
        } else {
            script.push_str(PGO);
        }
        script.push_str("make install_sw DESTDIR=\"$OUT\"\n");
        Recipe::Shell(script.into())
    }
}

const HEAD: &str = concat!(
    "set -euxo pipefail\n",
    include_str!("sysroot_env.sh"),
    r#"
case "$ARCH" in
  amd64) TGT=linux-x86_64 ;;
  arm64) TGT=linux-aarch64 ;;
  riscv64) TGT=linux64-riscv64 ;;
  ppc64le) TGT=linux-ppc64le ;;
  s390x) TGT=linux64-s390x ;;
  *) echo "未対応の architecture です: $ARCH"; exit 1 ;;
esac
if [ -n "${SYSROOT:-}" ]; then
  export CC="$CC --sysroot=$SYSROOT"
fi
mkdir -p "$BUILD"
cd "$BUILD"
configure() {
  "$SOURCE"/Configure "$TGT" --prefix=/usr --openssldir=/etc/ssl --libdir=lib64
}
configure
"#,
);

const PGO: &str = r#"export PGODIR=/tmp/pgo
rm -rf "$PGODIR"
mkdir -p "$PGODIR"
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-generate=$PGODIR -fprofile-update=atomic" LDFLAGS="$LDFLAGS -fprofile-generate"
export LD_LIBRARY_PATH="$PWD${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
# ビルドツリー内の provider（default.so 等）を読めるようにする。
# これが無いと EVP 系アルゴリズムが一切実行できずプロファイルが空になる。
export OPENSSL_MODULES="$PWD/providers"
export OPENSSL_CONF=/dev/null
(
  ./apps/openssl speed -seconds 1 rsa2048 ecdsap256 ed25519
  ./apps/openssl speed -seconds 1 -evp aes-128-gcm -evp aes-256-gcm -evp chacha20-poly1305 -evp sha256 -evp sha512
  ./apps/openssl speed -seconds 1 aes-128-cbc aes-256-cbc sha256 sha512
) || true
make clean
make -j"$(nproc)" CFLAGS="$CFLAGS -fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="$LDFLAGS -fprofile-use"
"#;
