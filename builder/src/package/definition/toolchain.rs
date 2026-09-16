use crate::flags::{Cflag, Flag};
use crate::package::{Bootstrap, Dependency, Package, Role, Source, Stage};
use crate::target::Target;

const LINUX_HEADERS: Dependency = Dependency {
    name: "linux-headers",
    version: "6.18.33",
};

use crate::package::GCC_VERSION;
const BINUTILS_VERSION: &str = "2.47";
const GLIBC_VERSION: &str = "2.44";

const GCC_SOURCE: Source = Source {
    url: "https://ftp.gnu.org/gnu/gcc/gcc-16.2.0/gcc-16.2.0.tar.xz",
    sha3_512: "1995b2fd6c2b19fe8ca47c836fdee8c019909ab8857f81a8045a7726ce724146ab63cd8f836fb460e9b009434ea97eb51b34397760b9ec4bbcac490ca32fa0f6",
    blake3: "c0f53af0bbfd2bf28896ab9bdfd1a534351b851c5a2f0226feb6a4323884c43531cbb207c733e8fd8ac16e2e1b03e6d8f17d043a17cf026857eedf26c31e4e0e",
    subdir: "gcc-16.2.0",
};

const BINUTILS_SOURCE: Source = Source {
    url: "https://ftp.gnu.org/gnu/binutils/binutils-2.47.tar.xz",
    sha3_512: "b2a39d2a1bfff7f27c2619e15c5d9dbf0c7918a85120975f4146c84b4a7d660bd5dc3793ea6d009df89ed865aaef4362805e949438db0f8acf97fd113ca1a1f3",
    blake3: "05a142fb2225e617a3fa30224d5d275682e5cf03a62a15af804c03214cc3a5cfadec1215345da15c94ba7a2e13b9667c094e59a26c39412a47265724cbb62a8e",
    subdir: "binutils-2.47",
};

const GLIBC_SOURCE: Source = Source {
    url: "https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz",
    sha3_512: "263dfb5e331b2033368c7455abcd9a9b15ab098ac963d5518ecf4bcb9eeb41298eb9976bf78a64af67d9471a50161bced2fa69d429c69759ea57f76b942ac521",
    blake3: "6d07bc16d8d066b1c0ae713d762175f261e27df9cab8015ec11e1c980c05e79b0fd5bab6cde6a90bf07651c432ca9039629376b57a96c9f3dcb3765f3745dbf4",
    subdir: "glibc-2.44",
};

pub struct Prefixed {
    pub name: &'static str,
    pub triple: &'static str,
    pub stage: Stage,
    #[allow(dead_code)]
    pub gcc_target_flag: &'static str,
}

impl Prefixed {
    const fn new(
        name: &'static str,
        triple: &'static str,
        stage: Stage,
        gcc_target_flag: &'static str,
    ) -> Self {
        Self {
            name,
            triple,
            stage,
            gcc_target_flag,
        }
    }
}

macro_rules! triple_tools {
    ($triple:literal, $flag:literal,
        $bin:ident, $s1:ident, $hdr:ident, $s2:ident, $libc:ident, $gcc:ident
    ) => {
        pub static $bin: Prefixed = Prefixed::new(
            concat!($triple, "-binutils"),
            $triple,
            Stage::Binutils,
            $flag,
        );
        pub static $s1: Prefixed = Prefixed::new(
            concat!($triple, "-gcc-stage1"),
            $triple,
            Stage::GccStage1,
            $flag,
        );
        pub static $hdr: Prefixed = Prefixed::new(
            concat!($triple, "-glibc-headers"),
            $triple,
            Stage::GlibcHeaders,
            $flag,
        );
        pub static $s2: Prefixed = Prefixed::new(
            concat!($triple, "-gcc-stage2"),
            $triple,
            Stage::GccStage2,
            $flag,
        );
        pub static $libc: Prefixed =
            Prefixed::new(concat!($triple, "-glibc"), $triple, Stage::Glibc, $flag);
        pub static $gcc: Prefixed =
            Prefixed::new(concat!($triple, "-gcc"), $triple, Stage::Gcc, $flag);
    };
}

triple_tools!(
    "x86_64-linux-gnu",
    "--with-arch=x86-64",
    X86_64_BINUTILS,
    X86_64_GCC_STAGE1,
    X86_64_GLIBC_HEADERS,
    X86_64_GCC_STAGE2,
    X86_64_GLIBC,
    X86_64_GCC
);
triple_tools!(
    "aarch64-linux-gnu",
    "--with-arch=armv8-a",
    AARCH64_BINUTILS,
    AARCH64_GCC_STAGE1,
    AARCH64_GLIBC_HEADERS,
    AARCH64_GCC_STAGE2,
    AARCH64_GLIBC,
    AARCH64_GCC
);
triple_tools!(
    "riscv64-linux-gnu",
    "--with-arch=rv64gc --with-abi=lp64d",
    RISCV64_BINUTILS,
    RISCV64_GCC_STAGE1,
    RISCV64_GLIBC_HEADERS,
    RISCV64_GCC_STAGE2,
    RISCV64_GLIBC,
    RISCV64_GCC
);
triple_tools!(
    "powerpc64le-linux-gnu",
    "--with-cpu=power8",
    PPC64LE_BINUTILS,
    PPC64LE_GCC_STAGE1,
    PPC64LE_GLIBC_HEADERS,
    PPC64LE_GCC_STAGE2,
    PPC64LE_GLIBC,
    PPC64LE_GCC
);
triple_tools!(
    "s390x-linux-gnu",
    "--with-arch=z900",
    S390X_BINUTILS,
    S390X_GCC_STAGE1,
    S390X_GLIBC_HEADERS,
    S390X_GCC_STAGE2,
    S390X_GLIBC,
    S390X_GCC
);

pub static TOOLCHAIN: &[&Prefixed] = &[
    &X86_64_BINUTILS,
    &X86_64_GCC_STAGE1,
    &X86_64_GLIBC_HEADERS,
    &X86_64_GCC_STAGE2,
    &X86_64_GLIBC,
    &X86_64_GCC,
    &AARCH64_BINUTILS,
    &AARCH64_GCC_STAGE1,
    &AARCH64_GLIBC_HEADERS,
    &AARCH64_GCC_STAGE2,
    &AARCH64_GLIBC,
    &AARCH64_GCC,
    &RISCV64_BINUTILS,
    &RISCV64_GCC_STAGE1,
    &RISCV64_GLIBC_HEADERS,
    &RISCV64_GCC_STAGE2,
    &RISCV64_GLIBC,
    &RISCV64_GCC,
    &PPC64LE_BINUTILS,
    &PPC64LE_GCC_STAGE1,
    &PPC64LE_GLIBC_HEADERS,
    &PPC64LE_GCC_STAGE2,
    &PPC64LE_GLIBC,
    &PPC64LE_GCC,
    &S390X_BINUTILS,
    &S390X_GCC_STAGE1,
    &S390X_GLIBC_HEADERS,
    &S390X_GCC_STAGE2,
    &S390X_GLIBC,
    &S390X_GCC,
];

impl Package for Prefixed {
    fn name(&self) -> &'static str {
        self.name
    }

    fn version(&self) -> &'static str {
        match self.stage {
            Stage::Binutils => BINUTILS_VERSION,
            Stage::GlibcHeaders | Stage::Glibc => GLIBC_VERSION,
            Stage::GccStage1 | Stage::GccStage2 | Stage::Gcc => GCC_VERSION,
        }
    }

    fn license(&self) -> &'static str {
        match self.stage {
            Stage::Binutils => "GPL-3.0-or-later",
            Stage::GlibcHeaders | Stage::Glibc => "LGPL-2.1-or-later",
            Stage::GccStage1 | Stage::GccStage2 | Stage::Gcc => {
                "GPL-3.0-or-later WITH GCC-exception-3.1"
            }
        }
    }

    fn build_only(&self) -> bool {
        matches!(
            self.stage,
            Stage::GccStage1 | Stage::GlibcHeaders | Stage::GccStage2
        )
    }

    fn role(&self) -> Role {
        Role::Toolchain {
            triple: self.triple,
            stage: self.stage,
        }
    }

    fn source(&self) -> Source {
        match self.stage {
            Stage::Binutils => BINUTILS_SOURCE,
            Stage::GlibcHeaders | Stage::Glibc => GLIBC_SOURCE,
            Stage::GccStage1 | Stage::GccStage2 | Stage::Gcc => GCC_SOURCE,
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        match self.stage {
            Stage::Binutils => crate::package::with_libc(
                target,
                &[
                    Dependency {
                        name: "zlib",
                        version: "1.3.2",
                    },
                    Dependency {
                        name: "zstd",
                        version: "1.5.7",
                    },
                ],
            ),
            Stage::GccStage1 | Stage::GlibcHeaders | Stage::GccStage2 | Stage::Glibc => vec![],
            Stage::Gcc => crate::package::with_libc(target, &GCC_RUNTIME_LIBS),
        }
    }

    fn build_dependencies(&self, target: Target) -> Vec<Dependency> {
        match (self.stage, self.triple) {
            (Stage::Binutils, _) => {
                let mut deps = vec![LINUX_HEADERS];
                if !(self.seed() && target == Target::BUILD_ENV) {
                    deps.push(Dependency {
                        name: "zlib",
                        version: "1.3.2",
                    });
                    deps.push(Dependency {
                        name: "zstd",
                        version: "1.5.7",
                    });
                }
                deps
            }
            (Stage::GccStage1, "x86_64-linux-gnu") => X86_STAGE1_BUILD.to_vec(),
            (Stage::GccStage1, "aarch64-linux-gnu") => AARCH64_STAGE1_BUILD.to_vec(),
            (Stage::GccStage1, "riscv64-linux-gnu") => RISCV64_STAGE1_BUILD.to_vec(),
            (Stage::GccStage1, "powerpc64le-linux-gnu") => PPC64LE_STAGE1_BUILD.to_vec(),
            (Stage::GccStage1, _) => S390X_STAGE1_BUILD.to_vec(),
            (Stage::GlibcHeaders, "x86_64-linux-gnu") => X86_HEADERS_BUILD.to_vec(),
            (Stage::GlibcHeaders, "aarch64-linux-gnu") => AARCH64_HEADERS_BUILD.to_vec(),
            (Stage::GlibcHeaders, "riscv64-linux-gnu") => RISCV64_HEADERS_BUILD.to_vec(),
            (Stage::GlibcHeaders, "powerpc64le-linux-gnu") => PPC64LE_HEADERS_BUILD.to_vec(),
            (Stage::GlibcHeaders, _) => S390X_HEADERS_BUILD.to_vec(),
            (Stage::GccStage2, "x86_64-linux-gnu") => X86_STAGE2_BUILD.to_vec(),
            (Stage::GccStage2, "aarch64-linux-gnu") => AARCH64_STAGE2_BUILD.to_vec(),
            (Stage::GccStage2, "riscv64-linux-gnu") => RISCV64_STAGE2_BUILD.to_vec(),
            (Stage::GccStage2, "powerpc64le-linux-gnu") => PPC64LE_STAGE2_BUILD.to_vec(),
            (Stage::GccStage2, _) => S390X_STAGE2_BUILD.to_vec(),
            (Stage::Glibc, "x86_64-linux-gnu") => X86_GLIBC_BUILD.to_vec(),
            (Stage::Glibc, "aarch64-linux-gnu") => AARCH64_GLIBC_BUILD.to_vec(),
            (Stage::Glibc, "riscv64-linux-gnu") => RISCV64_GLIBC_BUILD.to_vec(),
            (Stage::Glibc, "powerpc64le-linux-gnu") => PPC64LE_GLIBC_BUILD.to_vec(),
            (Stage::Glibc, _) => S390X_GLIBC_BUILD.to_vec(),
            (Stage::Gcc, "x86_64-linux-gnu") => gcc_build_deps(&X86_GCC_BUILD),
            (Stage::Gcc, "aarch64-linux-gnu") => gcc_build_deps(&AARCH64_GCC_BUILD),
            (Stage::Gcc, "riscv64-linux-gnu") => gcc_build_deps(&RISCV64_GCC_BUILD),
            (Stage::Gcc, "powerpc64le-linux-gnu") => gcc_build_deps(&PPC64LE_GCC_BUILD),
            (Stage::Gcc, _) => gcc_build_deps(&S390X_GCC_BUILD),
        }
    }

    fn flags(&self, target: Target) -> Flag {
        match self.stage {
            Stage::GlibcHeaders | Stage::GccStage1 | Stage::GccStage2 | Stage::Glibc => {
                Flag::new(vec![], vec![])
            }
            Stage::Gcc => Flag::new(vec![Cflag::O2, Cflag::Pipe, Cflag::March(target)], vec![]),
            Stage::Binutils => crate::flags::tool_flags(target),
        }
    }

    fn supports(&self, target: Target) -> bool {
        match self.stage {
            Stage::GccStage1 | Stage::GlibcHeaders | Stage::GccStage2 => {
                target == Target::BUILD_ENV
            }
            Stage::Glibc => target.gnu_triple() == self.triple,
            Stage::Binutils | Stage::Gcc => true,
        }
    }

    /// glibc / glibc-headers は依存宣言にない HOST ツールを optional mount する
    /// （glibc→python→openssl→glibc の循環を避けるため依存には書かない）。
    fn host_tools(&self, _target: Target) -> Vec<&'static str> {
        match self.stage {
            Stage::GlibcHeaders | Stage::Glibc => vec!["bison", "python", "m4"],
            _ => vec![],
        }
    }

    fn build_script(&self) -> &'static str {
        match self.stage {
            Stage::Binutils => BINUTILS_SCRIPT,
            Stage::GccStage1 => GCC_STAGE1_SCRIPT,
            Stage::GlibcHeaders => GLIBC_HEADERS_SCRIPT,
            Stage::GccStage2 => GCC_STAGE2_SCRIPT,
            Stage::Glibc => GLIBC_SCRIPT,
            Stage::Gcc => GCC_FINAL_SCRIPT,
        }
    }
}

impl Bootstrap for Prefixed {
    /// シード相の依存グラフ。binutils は glibc との循環を避けるため
    /// 実行時依存（libc/zlib/zstd）を切り離す。
    fn seed_dependencies(&self, target: Target) -> Option<(Vec<Dependency>, Vec<Dependency>)> {
        let runtime = match self.stage {
            Stage::Gcc => crate::package::with_libc(target, &GCC_RUNTIME_LIBS),
            _ => vec![],
        };
        Some((runtime, self.build_dependencies(target)))
    }

    fn pacman(&self) -> &'static [&'static str] {
        match self.stage {
            Stage::Binutils if self.seed() => &[
                "gcc", "make", "bison", "flex", "texinfo", "zlib", "zstd", "mold",
            ],
            Stage::Binutils => &["make", "bison", "flex", "texinfo"],
            Stage::GccStage1 | Stage::GccStage2 if self.seed() => &[
                "gcc", "make", "bison", "flex", "perl", "texinfo", "m4", "gmp", "mpfr", "libmpc",
                "libisl", "zstd", "zlib", "mold",
            ],
            Stage::GccStage1 | Stage::GccStage2 => &[
                "make", "bison", "flex", "perl", "texinfo", "m4", "gmp", "mpfr", "libmpc",
                "libisl", "zstd", "zlib",
            ],
            Stage::Gcc => &["make", "bison", "flex", "perl", "texinfo", "m4"],
            Stage::GlibcHeaders | Stage::Glibc => &["python", "bison", "gawk", "make", "mold"],
        }
    }

    fn seed(&self) -> bool {
        self.triple == "x86_64-linux-gnu"
            && matches!(
                self.stage,
                Stage::Binutils
                    | Stage::GccStage1
                    | Stage::GlibcHeaders
                    | Stage::GccStage2
                    | Stage::Glibc
            )
    }
}

const GCC_RUNTIME_LIBS: [Dependency; 6] = [
    Dependency {
        name: "gmp",
        version: "6.3.0",
    },
    Dependency {
        name: "mpfr",
        version: "4.2.2",
    },
    Dependency {
        name: "mpc",
        version: "1.4.1",
    },
    Dependency {
        name: "isl",
        version: "0.26",
    },
    Dependency {
        name: "zlib",
        version: "1.3.2",
    },
    Dependency {
        name: "zstd",
        version: "1.5.7",
    },
];

const GCC_HOST_TOOLS: [Dependency; 5] = [
    dep("bison", "3.8.2"),
    dep("flex", "2.6.4"),
    dep("perl", "5.42.3"),
    dep("texinfo", "7.3"),
    dep("m4", "1.4.21"),
];

fn gcc_build_deps(base: &[Dependency]) -> Vec<Dependency> {
    let mut dependencies = base.to_vec();
    dependencies.extend_from_slice(&GCC_HOST_TOOLS);
    dependencies
}

const X86_STAGE1_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
];
const AARCH64_STAGE1_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
];
const RISCV64_STAGE1_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
];
const PPC64LE_STAGE1_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
];
const S390X_STAGE1_BUILD: [Dependency; 2] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
];
const X86_HEADERS_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("x86_64-linux-gnu-gcc-stage1", GCC_VERSION),
];
const AARCH64_HEADERS_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("aarch64-linux-gnu-gcc-stage1", GCC_VERSION),
];
const RISCV64_HEADERS_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("riscv64-linux-gnu-gcc-stage1", GCC_VERSION),
];
const PPC64LE_HEADERS_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
    dep("powerpc64le-linux-gnu-gcc-stage1", GCC_VERSION),
];
const S390X_HEADERS_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
    dep("s390x-linux-gnu-gcc-stage1", GCC_VERSION),
];

const X86_STAGE2_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("x86_64-linux-gnu-glibc-headers", GLIBC_VERSION),
];
const AARCH64_STAGE2_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("aarch64-linux-gnu-glibc-headers", GLIBC_VERSION),
];
const RISCV64_STAGE2_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("riscv64-linux-gnu-glibc-headers", GLIBC_VERSION),
];
const PPC64LE_STAGE2_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
    dep("powerpc64le-linux-gnu-glibc-headers", GLIBC_VERSION),
];
const S390X_STAGE2_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
    dep("s390x-linux-gnu-glibc-headers", GLIBC_VERSION),
];

const X86_GLIBC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("x86_64-linux-gnu-gcc-stage2", GCC_VERSION),
];
const AARCH64_GLIBC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("aarch64-linux-gnu-gcc-stage2", GCC_VERSION),
];
const RISCV64_GLIBC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("riscv64-linux-gnu-gcc-stage2", GCC_VERSION),
];
const PPC64LE_GLIBC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
    dep("powerpc64le-linux-gnu-gcc-stage2", GCC_VERSION),
];
const S390X_GLIBC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
    dep("s390x-linux-gnu-gcc-stage2", GCC_VERSION),
];

const X86_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("x86_64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("x86_64-linux-gnu-glibc", GLIBC_VERSION),
];
const AARCH64_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("aarch64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("aarch64-linux-gnu-glibc", GLIBC_VERSION),
];
const RISCV64_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("riscv64-linux-gnu-binutils", BINUTILS_VERSION),
    dep("riscv64-linux-gnu-glibc", GLIBC_VERSION),
];
const PPC64LE_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("powerpc64le-linux-gnu-binutils", BINUTILS_VERSION),
    dep("powerpc64le-linux-gnu-glibc", GLIBC_VERSION),
];
const S390X_GCC_BUILD: [Dependency; 3] = [
    LINUX_HEADERS,
    dep("s390x-linux-gnu-binutils", BINUTILS_VERSION),
    dep("s390x-linux-gnu-glibc", GLIBC_VERSION),
];

const fn dep(name: &'static str, version: &'static str) -> Dependency {
    Dependency { name, version }
}

const BINUTILS_SCRIPT: &str = concat!(
    include_str!("sysroot_env.sh"),
    r#"set -euxo pipefail
export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
export AS="${PACKAGE_HOST}-as"
mkdir -p "$BUILD"
cd "$BUILD"
"$SOURCE"/configure --build="$HOST_TRIPLE" --host="$PACKAGE_HOST" --target="$TRIPLE" \
  --prefix=/usr --infodir=/usr/share/info/"$TRIPLE" --disable-nls --disable-werror \
  --enable-plugins --with-system-zlib --without-debuginfod --disable-gprofng
make -j"$(nproc)"
make install DESTDIR="$OUT"
"#,
);

const GCC_STAGE1_SCRIPT: &str = r#"set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:$PATH"
case "$TRIPLE" in
  x86_64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=x86-64' ;;
  aarch64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=armv8-a' ;;
  riscv64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=rv64gc --with-abi=lp64d' ;;
  powerpc64le-linux-gnu) GCC_ARCH_FLAGS='--with-cpu=power8' ;;
  s390x-linux-gnu) GCC_ARCH_FLAGS='--with-arch=z900' ;;
  *) echo "未対応の triple: $TRIPLE"; exit 1 ;;
esac
PREFIX=/usr/lib/cross/"$TRIPLE"/stage1
SYSROOT=/usr/"$TRIPLE"
mkdir -p "$SYSROOT/include" "$BUILD"
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d "$HDR" ] || HDR=/dep/linux-headers/usr/include
cp -a "$HDR"/. "$SYSROOT/include/"
ln -sfn /dep/"$TRIPLE"-binutils/usr/bin/"$TRIPLE"-as /usr/bin/"$TRIPLE"-as
ln -sfn /dep/"$TRIPLE"-binutils/usr/bin/"$TRIPLE"-ld /usr/bin/"$TRIPLE"-ld
cd "$BUILD"
"$SOURCE"/configure --build="$HOST_TRIPLE" --host="$PACKAGE_HOST" --target="$TRIPLE" --prefix="$PREFIX" \
  --with-sysroot="$SYSROOT" --with-build-sysroot="$SYSROOT" \
  --with-native-system-header-dir=/include \
  --with-as=/usr/bin/"$TRIPLE"-as \
  --with-ld=/usr/bin/"$TRIPLE"-ld \
  --infodir=/usr/share/info/"$TRIPLE" \
  --with-system-zlib --with-isl \
  $GCC_ARCH_FLAGS \
  --without-headers --with-newlib --disable-shared --disable-threads \
  --disable-werror --disable-libatomic --disable-libsanitizer --disable-libssp \
  --disable-libquadmath --disable-libgomp --disable-libstdcxx --disable-libvtv \
  --disable-bootstrap --disable-multilib --enable-languages=c --enable-lto --enable-plugin \
  --enable-checking=release
make -j"$(nproc)" all-gcc all-target-libgcc
make DESTDIR="$OUT" install-gcc install-target-libgcc
"#;

const GLIBC_HEADERS_SCRIPT: &str = r#"set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:$PATH"
STAGE1=/dep/"$TRIPLE"-gcc-stage1/usr/lib/cross/"$TRIPLE"/stage1
SYSROOT="$OUT/usr/$TRIPLE"
mkdir -p "$SYSROOT/include" "$SYSROOT/lib" "$BUILD"
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d "$HDR" ] || HDR=/dep/linux-headers/usr/include
cp -a "$HDR"/. "$SYSROOT/include/"
cd "$BUILD"
unset CFLAGS CXXFLAGS LDFLAGS
case "$TRIPLE" in
  riscv64-linux-gnu) export CFLAGS="-O2 -pipe -march=rv64gc -mabi=lp64d" ;;
esac
export CPP="$STAGE1/bin/${TRIPLE}-gcc -E"
BUILD_CC="${HOST_TRIPLE}-gcc" CC="$STAGE1/bin/${TRIPLE}-gcc" \
AR="${TRIPLE}-ar" RANLIB="${TRIPLE}-ranlib" NM="${TRIPLE}-nm" \
OBJCOPY="${TRIPLE}-objcopy" OBJDUMP="${TRIPLE}-objdump" READELF="${TRIPLE}-readelf" \
"$SOURCE"/configure --prefix=/ --host="$TRIPLE" --build="$HOST_TRIPLE" \
  --with-headers="$SYSROOT/include" --disable-werror \
  libc_cv_forced_unwind=yes libc_cv_c_cleanup=yes
make csu/subdir_lib
make install-bootstrap-headers=yes install-headers DESTDIR="$SYSROOT"
install -m644 csu/crt1.o csu/crti.o csu/crtn.o "$SYSROOT/lib/"
"$STAGE1/bin/${TRIPLE}-gcc" -nostdlib -nostartfiles -shared -x c /dev/null -o "$SYSROOT/lib/libc.so"
mkdir -p "$SYSROOT/include/gnu"
touch "$SYSROOT/include/gnu/stubs.h"
ln -sfn lib "$SYSROOT/lib64"
"#;

const GCC_STAGE2_SCRIPT: &str = r#"set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:$PATH"
case "$TRIPLE" in
  x86_64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=x86-64' ;;
  aarch64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=armv8-a' ;;
  riscv64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=rv64gc --with-abi=lp64d' ;;
  powerpc64le-linux-gnu) GCC_ARCH_FLAGS='--with-cpu=power8' ;;
  s390x-linux-gnu) GCC_ARCH_FLAGS='--with-arch=z900' ;;
  *) echo "未対応の triple: $TRIPLE"; exit 1 ;;
esac
PREFIX=/usr/lib/cross/"$TRIPLE"/stage2
SYSROOT=/usr/"$TRIPLE"
mkdir -p "$SYSROOT" "$BUILD"
if [ -d /dep/${TRIPLE}-glibc-headers/usr/"$TRIPLE" ]; then
  cp -a /dep/${TRIPLE}-glibc-headers/usr/"$TRIPLE"/. "$SYSROOT"/
fi
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d "$HDR" ] || HDR=/dep/linux-headers/usr/include
mkdir -p "$SYSROOT/include"
cp -a "$HDR"/. "$SYSROOT/include/"
ln -sfn /dep/"$TRIPLE"-binutils/usr/bin/"$TRIPLE"-as /usr/bin/"$TRIPLE"-as
ln -sfn /dep/"$TRIPLE"-binutils/usr/bin/"$TRIPLE"-ld /usr/bin/"$TRIPLE"-ld
cd "$BUILD"
"$SOURCE"/configure --build="$HOST_TRIPLE" --host="$PACKAGE_HOST" --target="$TRIPLE" --prefix="$PREFIX" \
  --with-sysroot="$SYSROOT" --with-build-sysroot="$SYSROOT" \
  --with-native-system-header-dir=/include \
  --with-as=/usr/bin/"$TRIPLE"-as \
  --with-ld=/usr/bin/"$TRIPLE"-ld \
  --infodir=/usr/share/info/"$TRIPLE" \
  --with-system-zlib --with-isl \
  $GCC_ARCH_FLAGS \
  --disable-nls --disable-multilib --disable-werror \
  --disable-libatomic --disable-libsanitizer --disable-libssp --disable-libquadmath \
  --disable-libgomp --disable-libstdcxx --disable-libvtv --disable-bootstrap --disable-multilib \
  --enable-shared --enable-threads=posix --enable-languages=c \
  --enable-lto --enable-plugin --enable-checking=release
make -j"$(nproc)" all-gcc all-target-libgcc
make DESTDIR="$OUT" install-gcc install-target-libgcc
"#;

const GLIBC_SCRIPT: &str = r#"set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:$PATH"
if [ -d /dep/bison/usr/share/bison ]; then
  export BISON_PKGDATADIR=/dep/bison/usr/share/bison
fi
STAGE2=/dep/"$TRIPLE"-gcc-stage2/usr/lib/cross/"$TRIPLE"/stage2
SYSROOT="$OUT/usr/$TRIPLE"
mkdir -p "$SYSROOT/include" "$BUILD"
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d "$HDR" ] || HDR=/dep/linux-headers/usr/include
cp -a "$HDR"/. "$SYSROOT/include/"
cd "$BUILD"
# ビルド環境向け CFLAGS は HOST 用なので、ターゲット libc にはパッケージ Platform の ISA を付ける。
unset CFLAGS CXXFLAGS LDFLAGS
case "$ARCH" in
  ppc64le) export CFLAGS="-O2 -pipe -mcpu=$VARIANT" ;;
  *) export CFLAGS="-O2 -pipe -march=$MARCH" ;;
esac
export CXXFLAGS="${CFLAGS:-}"
export LDFLAGS="-fuse-ld=mold"
unset LD
export CPP="$STAGE2/bin/${TRIPLE}-gcc -E"
BUILD_CC="${HOST_TRIPLE}-gcc" CC="$STAGE2/bin/${TRIPLE}-gcc" \
AR="${TRIPLE}-ar" RANLIB="${TRIPLE}-ranlib" NM="${TRIPLE}-nm" \
AS="${TRIPLE}-as" LD="${TRIPLE}-ld" \
OBJCOPY="${TRIPLE}-objcopy" OBJDUMP="${TRIPLE}-objdump" READELF="${TRIPLE}-readelf" \
"$SOURCE"/configure --prefix=/ --host="$TRIPLE" --build="$HOST_TRIPLE" --libdir=/lib \
  --with-headers="$SYSROOT/include" --disable-werror --disable-nscd --disable-profile \
  libc_cv_forced_unwind=yes libc_cv_c_cleanup=yes
echo "build-programs=no" >> configparms
make -j"$(nproc)"
make DESTDIR="$SYSROOT" install
rm -rf "$SYSROOT"/{share,etc,var,bin,sbin}
# --prefix=/ --libdir=/lib は GROUP ( //lib/libc.so.6 ) を書く。ld は // を sysroot しない。
find "$SYSROOT" -type f -name '*.so' | while read -r _so; do
  if grep -q 'GNU ld script' "$_so" 2>/dev/null; then
    sed -i 's|//lib/|/lib/|g; s|//lib64/|/lib64/|g' "$_so"
  fi
done
ln -sfn lib "$SYSROOT/lib64"
if [ "$TRIPLE" = "x86_64-linux-gnu" ]; then
  mkdir -p "$OUT/lib64" "$OUT/usr/lib64"
  ln -sfn /usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2 "$OUT/lib64/ld-linux-x86-64.so.2" || true
  if [ -e "$SYSROOT/lib/ld-linux-x86-64.so.2" ]; then
    ln -sfn ../usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2 "$OUT/lib64/ld-linux-x86-64.so.2"
  fi
  for so in "$SYSROOT"/lib/libc.so.6 "$SYSROOT"/lib/libm.so.6 "$SYSROOT"/lib/libdl.so.2 \
            "$SYSROOT"/lib/libpthread.so.0 "$SYSROOT"/lib/librt.so.1 "$SYSROOT"/lib/libresolv.so.2; do
    if [ -e "$so" ]; then
      ln -sfn "$(realpath --relative-to="$OUT/usr/lib64" "$so")" "$OUT/usr/lib64/$(basename "$so")"
    fi
  done
fi
"#;

const GCC_FINAL_SCRIPT: &str = concat!(
    include_str!("sysroot_env.sh"),
    r#"set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:$PATH"
export CC_FOR_BUILD="${HOST_TRIPLE}-gcc"
export CXX_FOR_BUILD="${HOST_TRIPLE}-g++"
# wrap の CFLAGS は PACKAGE_HOST（成果物が動く Platform）向け。Canadian では
# build-libcpp が HOST の *-g++ を CFLAGS ごと使うので、ビルドマシン用を分ける。
_build_root=
if [ -d /dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE} ]; then
  _build_root=/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}
fi
_bf="-O2 -pipe"
if [ -n "$_build_root" ]; then
  _bf="$_bf --sysroot=$_build_root"
  if [ -e "$_build_root/lib/crti.o" ]; then
    _bf="$_bf -B$_build_root/lib"
  elif [ -e "$_build_root/lib64/crti.o" ]; then
    _bf="$_bf -B$_build_root/lib64"
  fi
fi
export CFLAGS_FOR_BUILD="$_bf"
export CXXFLAGS_FOR_BUILD="$_bf"
export LDFLAGS_FOR_BUILD="${_build_root:+--sysroot=$_build_root}"
case "$TRIPLE" in
  x86_64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=x86-64' ;;
  aarch64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=armv8-a' ;;
  riscv64-linux-gnu) GCC_ARCH_FLAGS='--with-arch=rv64gc --with-abi=lp64d' ;;
  powerpc64le-linux-gnu) GCC_ARCH_FLAGS='--with-cpu=power8' ;;
  s390x-linux-gnu) GCC_ARCH_FLAGS='--with-arch=z900' ;;
  *) echo "未対応の triple: $TRIPLE"; exit 1 ;;
esac
SYSROOT=/usr/"$TRIPLE"
_profiledir="$BUILD/pgo-profile-data"
# 前回失敗の config.cache / 壊れた Makefile を残さない。
rm -rf "$BUILD/instrumented" "$BUILD/final"
mkdir -p "$SYSROOT" "$_profiledir" "$BUILD/instrumented" "$BUILD/final"
if [ -d /dep/${TRIPLE}-glibc/usr/${TRIPLE} ]; then
  cp -a /dep/${TRIPLE}-glibc/usr/${TRIPLE}/. "$SYSROOT"/
fi
# 既存 glibc 成果物も //lib の ld script を持つ。xgcc の --sysroot が効く形へ直す。
find "$SYSROOT" -type f -name '*.so' | while read -r _so; do
  if grep -q 'GNU ld script' "$_so" 2>/dev/null; then
    sed -i 's|//lib/|/lib/|g; s|//lib64/|/lib64/|g' "$_so"
  fi
done
BUILD_SYSROOT="$SYSROOT"
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d "$HDR" ] || HDR=/dep/linux-headers/usr/include
mkdir -p "$BUILD_SYSROOT/include"
cp -a "$HDR"/. "$BUILD_SYSROOT/include/" || true
# wrap の CFLAGS は HOST 用。CFLAGS_FOR_TARGET がそれを継承すると /dep の未修正 libc.so を -B で拾う。
export CFLAGS_FOR_TARGET="--sysroot=$BUILD_SYSROOT"
export CXXFLAGS_FOR_TARGET="--sysroot=$BUILD_SYSROOT"
export LDFLAGS_FOR_TARGET="--sysroot=$BUILD_SYSROOT"
ln -sfn /dep/"$TRIPLE"-binutils/usr/bin/"$TRIPLE"-as /usr/bin/"$TRIPLE"-as
ln -sfn /dep/"$TRIPLE"-binutils/usr/bin/"$TRIPLE"-ld /usr/bin/"$TRIPLE"-ld
SANITIZER_FLAGS=
if [ "$TRIPLE" != x86_64-linux-gnu ]; then
  SANITIZER_FLAGS=--disable-libsanitizer
fi
PLUGIN_FLAGS=--enable-plugin
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS="--with-isl"
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zlib/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zlib/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zstd/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zstd/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
# Make は環境の CXXFLAGS を build-libcpp の HOST g++ にも渡す。Canadian では
# PACKAGE_HOST 向けフラグを CC/CXX ラッパへ移し、環境からは外す。
# #!/bin/sh は ncurses 警告が stdout に混ざり LD キャッシュを壊すので perl にする。
write_canadian_wrap() {
  local dest=$1 real=$2
  shift 2
  local _perl
  _perl=$(command -v perl)
  {
    printf '#!%s\n' "$_perl"
    echo 'use strict;'
    echo 'my @cmd = ('
    local _w _q
    for _w in "$real" "$@"; do
      _q=${_w//\'/\\\'}
      printf "  '%s',\n" "$_q"
    done
    echo ');'
    echo 'exec @cmd, @ARGV;'
  } > "$dest"
  chmod +x "$dest"
}
if [ "$PACKAGE_HOST" != "$HOST_TRIPLE" ]; then
  _wrap=/tmp/canadian-host-cc
  mkdir -p "$_wrap"
  _real_cc=$(command -v "${PACKAGE_HOST}-gcc")
  _real_cxx=$(command -v "${PACKAGE_HOST}-g++")
  # wrap の -B/dep/...-glibc は未修正の GROUP ( //lib/libc.so.6 ) を拾う。
  # host==target の libgcc_s は HOST gcc でリンクするので、書き換えた SYSROOT を使う。
  _cflags=${CFLAGS:-}
  _cxxflags=${CXXFLAGS:-}
  _ldflags=${LDFLAGS:-}
  if [ "$PACKAGE_HOST" = "$TRIPLE" ]; then
    _cflags=$(printf '%s' "$_cflags" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|$SYSROOT|g")
    _cxxflags=$(printf '%s' "$_cxxflags" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|$SYSROOT|g")
    _ldflags=$(printf '%s' "$_ldflags" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|$SYSROOT|g")
  fi
  # shellcheck disable=SC2086
  write_canadian_wrap "$_wrap/${PACKAGE_HOST}-gcc" "$_real_cc" ${_cflags} ${_ldflags}
  write_canadian_wrap "$_wrap/${PACKAGE_HOST}-g++" "$_real_cxx" ${_cxxflags} ${_ldflags}
  export PATH="$_wrap:$PATH"
  export CC="${PACKAGE_HOST}-gcc"
  export CXX="${PACKAGE_HOST}-g++"
  unset CFLAGS CXXFLAGS LDFLAGS CPPFLAGS
  "$CC" -x c -o /tmp/canadian-cc-ok - <<'EOF'
int main(void) { return 0; }
EOF
fi
configure_final_gcc() {
  "$SOURCE"/configure --build="$HOST_TRIPLE" --host="$PACKAGE_HOST" --target="$TRIPLE" --prefix=/usr \
    --with-sysroot="$SYSROOT" --with-build-sysroot="$BUILD_SYSROOT" \
    --with-native-system-header-dir=/include \
    --with-as=/usr/bin/"$TRIPLE"-as \
    --with-ld=/usr/bin/"$TRIPLE"-ld \
    --infodir=/usr/share/info/"$TRIPLE" \
    --with-system-zlib $GCC_LIB_FLAGS \
    $GCC_ARCH_FLAGS \
    --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib \
    $SANITIZER_FLAGS $PLUGIN_FLAGS \
    --enable-languages=c,c++ --enable-lto --enable-checking=release \
    --enable-__cxa_atexit --enable-threads=posix --enable-shared
}
if [ "$PACKAGE_HOST" != "$HOST_TRIPLE" ] || [ "$ARCH/$VARIANT" != "amd64/v4" ]; then
  if [ "$PACKAGE_HOST" != "$HOST_TRIPLE" ]; then
    export CC_FOR_TARGET="${TRIPLE}-gcc"
    export CXX_FOR_TARGET="${TRIPLE}-g++"
    export GCC_FOR_TARGET="${TRIPLE}-gcc"
    export AR_FOR_TARGET="${TRIPLE}-ar"
    export RANLIB_FOR_TARGET="${TRIPLE}-ranlib"
    export NM_FOR_TARGET="${TRIPLE}-nm"
    export AS_FOR_TARGET="${TRIPLE}-as"
    export LD_FOR_TARGET="${TRIPLE}-ld"
  fi
  cd "$BUILD/final"
  configure_final_gcc
  make -j"$(nproc)"
  make DESTDIR="$OUT" install
  exit 0
fi
cd "$BUILD/instrumented"
CFLAGS="${CFLAGS:-} -fprofile-generate=$_profiledir" \
CXXFLAGS="${CXXFLAGS:-} -fprofile-generate=$_profiledir" \
LDFLAGS="${LDFLAGS:-} -fprofile-generate=$_profiledir" \
configure_final_gcc
make -j"$(nproc)"
_xgcc="$BUILD/instrumented/gcc/xgcc"
_xgpp="$BUILD/instrumented/gcc/xg++"
_B="$BUILD/instrumented/gcc/"
_ts="$SOURCE/gcc/testsuite"
for _opt in -O0 -O1 -O2 -O3 -Os; do
  find "$_ts/gcc.c-torture/compile" -maxdepth 1 -name '*.c' -print0 | \
    xargs -0 -P"$(nproc)" -I{} "$_xgcc" -B"$_B" $_opt -S -o /dev/null -w {} 2>/dev/null || true
done
find "$_ts/g++.dg" -maxdepth 1 -name '*.C' -print0 | \
  xargs -0 -P"$(nproc)" -I{} "$_xgpp" -B"$_B" -O2 -S -o /dev/null -w {} 2>/dev/null || true
cd "$BUILD/final"
CFLAGS="${CFLAGS:-} -fprofile-use=$_profiledir -fprofile-correction" \
CXXFLAGS="${CXXFLAGS:-} -fprofile-use=$_profiledir -fprofile-correction" \
LDFLAGS="${LDFLAGS:-}" \
configure_final_gcc
make -j"$(nproc)"
make DESTDIR="$OUT" install
"#,
);
