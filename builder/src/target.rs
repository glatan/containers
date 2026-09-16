macro_rules! variants {
    ($name:ident { $($variant:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum $name {
            $($variant,)*
        }

        impl $name {
            const ALL: &'static [Self] = &[$(Self::$variant),*];
        }
    };
}

macro_rules! define_target {
    ($($arch:ident($variant:ty)),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Target {
            $($arch($variant),)+
        }

        impl Target {
            pub fn all() -> impl Iterator<Item = Self> {
                core::iter::empty::<Self>()
                    $(.chain(<$variant>::ALL.iter().copied().map(Self::$arch)))+
            }
        }
    };
}

variants!(Amd64Variant { V1, V2, V3, V4 });

variants!(Arm64Variant {
    V8,
    V8_1,
    V8_2,
    V8_3,
    V8_4,
    V8_5,
    V8_6,
    V8_7,
    V8_8,
    V8_9,
    V9,
    V9_1,
    V9_2,
    V9_3,
    V9_4,
    V9_5,
});

variants!(Ppc64leVariant {
    Power8,
    Power9,
    Power10,
    Power11,
});

variants!(Riscv64Variant {
    Rva20u64,
    Rva22u64,
    Rva23u64,
});

variants!(S390xVariant {
    Z900,
    Z990,
    Z9Ec,
    Z10,
    Z196,
    ZEc12,
    Z13,
    Z14,
    Z15,
    Z16,
    Z17,
});

define_target! {
    Arm64(Arm64Variant),
    Ppc64le(Ppc64leVariant),
    Riscv64(Riscv64Variant),
    Amd64(Amd64Variant),
    S390x(S390xVariant),
}

impl Target {
    pub const BUILD_ENV: Self = Self::Amd64(Amd64Variant::V4);
    pub const HOST_GNU_TRIPLE: &'static str = "x86_64-linux-gnu";

    pub fn host_gnu_cross() -> &'static str {
        "x86_64-linux-gnu-"
    }

    pub fn gnu_triple(self) -> &'static str {
        match self {
            Self::Amd64(_) => "x86_64-linux-gnu",
            Self::Arm64(_) => "aarch64-linux-gnu",
            Self::Riscv64(_) => "riscv64-linux-gnu",
            Self::Ppc64le(_) => "powerpc64le-linux-gnu",
            Self::S390x(_) => "s390x-linux-gnu",
        }
    }

    pub fn glibc_package(self) -> &'static str {
        match self {
            Self::Amd64(_) => "x86_64-linux-gnu-glibc",
            Self::Arm64(_) => "aarch64-linux-gnu-glibc",
            Self::Riscv64(_) => "riscv64-linux-gnu-glibc",
            Self::Ppc64le(_) => "powerpc64le-linux-gnu-glibc",
            Self::S390x(_) => "s390x-linux-gnu-glibc",
        }
    }

    pub fn gcc_package(self) -> &'static str {
        match self {
            Self::Amd64(_) => "x86_64-linux-gnu-gcc",
            Self::Arm64(_) => "aarch64-linux-gnu-gcc",
            Self::Riscv64(_) => "riscv64-linux-gnu-gcc",
            Self::Ppc64le(_) => "powerpc64le-linux-gnu-gcc",
            Self::S390x(_) => "s390x-linux-gnu-gcc",
        }
    }

    pub fn baseline(self) -> Self {
        match self {
            Self::Amd64(_) => Self::Amd64(Amd64Variant::V1),
            Self::Arm64(_) => Self::Arm64(Arm64Variant::V8),
            Self::Riscv64(_) => Self::Riscv64(Riscv64Variant::Rva20u64),
            Self::Ppc64le(_) => Self::Ppc64le(Ppc64leVariant::Power8),
            Self::S390x(_) => Self::S390x(S390xVariant::Z900),
        }
    }

    /// 実装順の切り出し（HOST → 各 arch の基底 → 残り）。`Package::supports` とは独立。
    pub fn is_phase4(self) -> bool {
        matches!(
            self,
            Self::Amd64(Amd64Variant::V1)
                | Self::Amd64(Amd64Variant::V4)
                | Self::Arm64(Arm64Variant::V8)
                | Self::Riscv64(Riscv64Variant::Rva20u64)
        )
    }

    pub fn architecture(self) -> &'static str {
        match self {
            Self::Amd64(_) => "amd64",
            Self::Arm64(_) => "arm64",
            Self::Ppc64le(_) => "ppc64le",
            Self::Riscv64(_) => "riscv64",
            Self::S390x(_) => "s390x",
        }
    }

    pub fn variant(self) -> &'static str {
        match self {
            Self::Amd64(Amd64Variant::V1) => "v1",
            Self::Amd64(Amd64Variant::V2) => "v2",
            Self::Amd64(Amd64Variant::V3) => "v3",
            Self::Amd64(Amd64Variant::V4) => "v4",
            Self::Arm64(Arm64Variant::V8) => "v8",
            Self::Arm64(Arm64Variant::V8_1) => "v8.1",
            Self::Arm64(Arm64Variant::V8_2) => "v8.2",
            Self::Arm64(Arm64Variant::V8_3) => "v8.3",
            Self::Arm64(Arm64Variant::V8_4) => "v8.4",
            Self::Arm64(Arm64Variant::V8_5) => "v8.5",
            Self::Arm64(Arm64Variant::V8_6) => "v8.6",
            Self::Arm64(Arm64Variant::V8_7) => "v8.7",
            Self::Arm64(Arm64Variant::V8_8) => "v8.8",
            Self::Arm64(Arm64Variant::V8_9) => "v8.9",
            Self::Arm64(Arm64Variant::V9) => "v9",
            Self::Arm64(Arm64Variant::V9_1) => "v9.1",
            Self::Arm64(Arm64Variant::V9_2) => "v9.2",
            Self::Arm64(Arm64Variant::V9_3) => "v9.3",
            Self::Arm64(Arm64Variant::V9_4) => "v9.4",
            Self::Arm64(Arm64Variant::V9_5) => "v9.5",
            Self::Ppc64le(Ppc64leVariant::Power8) => "power8",
            Self::Ppc64le(Ppc64leVariant::Power9) => "power9",
            Self::Ppc64le(Ppc64leVariant::Power10) => "power10",
            Self::Ppc64le(Ppc64leVariant::Power11) => "power11",
            Self::Riscv64(Riscv64Variant::Rva20u64) => "rva20u64",
            Self::Riscv64(Riscv64Variant::Rva22u64) => "rva22u64",
            Self::Riscv64(Riscv64Variant::Rva23u64) => "rva23u64",
            Self::S390x(S390xVariant::Z900) => "z900",
            Self::S390x(S390xVariant::Z990) => "z990",
            Self::S390x(S390xVariant::Z9Ec) => "z9-ec",
            Self::S390x(S390xVariant::Z10) => "z10",
            Self::S390x(S390xVariant::Z196) => "z196",
            Self::S390x(S390xVariant::ZEc12) => "zEC12",
            Self::S390x(S390xVariant::Z13) => "z13",
            Self::S390x(S390xVariant::Z14) => "z14",
            Self::S390x(S390xVariant::Z15) => "z15",
            Self::S390x(S390xVariant::Z16) => "z16",
            Self::S390x(S390xVariant::Z17) => "z17",
        }
    }

    pub fn lookup(architecture: &str, variant: &str) -> Option<Self> {
        Self::all()
            .find(|target| target.architecture() == architecture && target.variant() == variant)
    }

    pub fn tuple(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::Amd64(_) => ("x86_64", "", "linux-gnu"),
            Self::Arm64(_) => ("aarch64", "", "linux-gnu"),
            Self::Ppc64le(_) => ("powerpc64le", "", "linux-gnu"),
            Self::Riscv64(_) => ("riscv64", "", "linux-gnu"),
            Self::S390x(_) => ("s390x", "", "linux-gnu"),
        }
    }

    pub fn format_tuple(self) -> String {
        let (cpu, vendor, os) = self.tuple();
        if vendor.is_empty() {
            format!("{cpu}-{os}")
        } else {
            format!("{cpu}-{vendor}-{os}")
        }
    }

    pub fn cross_prefix(self) -> String {
        format!("{}-", self.format_tuple())
    }

    pub fn march(self) -> &'static str {
        match self {
            Self::Amd64(Amd64Variant::V1) => "x86-64",
            Self::Amd64(Amd64Variant::V2) => "x86-64-v2",
            Self::Amd64(Amd64Variant::V3) => "x86-64-v3",
            Self::Amd64(Amd64Variant::V4) => "x86-64-v4",
            Self::Arm64(Arm64Variant::V8) => "armv8-a",
            Self::Arm64(Arm64Variant::V8_1) => "armv8.1-a",
            Self::Arm64(Arm64Variant::V8_2) => "armv8.2-a",
            Self::Arm64(Arm64Variant::V8_3) => "armv8.3-a",
            Self::Arm64(Arm64Variant::V8_4) => "armv8.4-a",
            Self::Arm64(Arm64Variant::V8_5) => "armv8.5-a",
            Self::Arm64(Arm64Variant::V8_6) => "armv8.6-a",
            Self::Arm64(Arm64Variant::V8_7) => "armv8.7-a",
            Self::Arm64(Arm64Variant::V8_8) => "armv8.8-a",
            Self::Arm64(Arm64Variant::V8_9) => "armv8.9-a",
            Self::Arm64(Arm64Variant::V9) => "armv9-a",
            Self::Arm64(Arm64Variant::V9_1) => "armv9.1-a",
            Self::Arm64(Arm64Variant::V9_2) => "armv9.2-a",
            Self::Arm64(Arm64Variant::V9_3) => "armv9.3-a",
            Self::Arm64(Arm64Variant::V9_4) => "armv9.4-a",
            Self::Arm64(Arm64Variant::V9_5) => "armv9.5-a",
            Self::Riscv64(Riscv64Variant::Rva20u64) => "rva20u64",
            Self::Riscv64(Riscv64Variant::Rva22u64) => "rva22u64",
            Self::Riscv64(Riscv64Variant::Rva23u64) => "rva23u64",
            Self::Ppc64le(_) | Self::S390x(_) => self.variant(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn march_covers_all_targets() {
        for target in Target::all() {
            let _ = target.march();
        }
        let arm64_v81 = Target::Arm64(Arm64Variant::V8_1);
        assert_eq!(arm64_v81.march(), "armv8.1-a");
        assert_eq!(arm64_v81.tuple(), ("aarch64", "", "linux-gnu"));
        assert_eq!(arm64_v81.format_tuple(), "aarch64-linux-gnu");
        assert_eq!(arm64_v81.cross_prefix(), "aarch64-linux-gnu-");
        let riscv = Target::Riscv64(Riscv64Variant::Rva20u64);
        assert_eq!(riscv.march(), "rva20u64");
        assert_eq!(riscv.tuple(), ("riscv64", "", "linux-gnu"));
        assert_eq!(riscv.cross_prefix(), "riscv64-linux-gnu-");
        let amd64 = Target::Amd64(Amd64Variant::V1);
        assert_eq!(amd64.tuple(), ("x86_64", "", "linux-gnu"));
        assert_eq!(amd64.format_tuple(), "x86_64-linux-gnu");
        assert_eq!(amd64.cross_prefix(), "x86_64-linux-gnu-");
        assert!(Target::BUILD_ENV.is_phase4());
        assert!(Target::Amd64(Amd64Variant::V1).is_phase4());
        assert!(Target::Arm64(Arm64Variant::V8).is_phase4());
        assert!(Target::Riscv64(Riscv64Variant::Rva20u64).is_phase4());
        assert!(!Target::Arm64(Arm64Variant::V8_1).is_phase4());
        assert!(!Target::S390x(S390xVariant::Z900).is_phase4());
        assert_eq!(Target::Ppc64le(Ppc64leVariant::Power8).march(), "power8");
        assert_eq!(Target::S390x(S390xVariant::Z13).march(), "z13");
        assert_eq!(
            Target::Ppc64le(Ppc64leVariant::Power8).tuple(),
            ("powerpc64le", "", "linux-gnu")
        );
    }

    #[test]
    fn lookup_round_trips_targets() {
        for target in Target::all() {
            assert_eq!(
                Target::lookup(target.architecture(), target.variant()),
                Some(target)
            );
        }
        assert_eq!(Target::lookup("amd64", "v9"), None);
    }

    #[test]
    fn native_is_equal_triples_and_cross_prefix_is_never_empty() {
        assert_eq!(
            Target::Amd64(Amd64Variant::V1).format_tuple(),
            Target::Amd64(Amd64Variant::V4).format_tuple()
        );
        assert_eq!(
            Target::Amd64(Amd64Variant::V1).cross_prefix(),
            format!("{}-", Target::Amd64(Amd64Variant::V1).format_tuple())
        );
        assert_ne!(
            Target::Amd64(Amd64Variant::V1).format_tuple(),
            Target::Arm64(Arm64Variant::V8).format_tuple()
        );
        for target in Target::all() {
            assert!(
                !target.cross_prefix().is_empty(),
                "{}/{}",
                target.architecture(),
                target.variant()
            );
        }
    }
}
