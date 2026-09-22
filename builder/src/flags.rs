use crate::target::Target;

/// C コンパイラフラグ。1 バリアントが複数トークンに展開されることもある。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cflag {
    /// `-O2`。
    O2,
    /// `-pipe`。
    Pipe,
    /// ターゲット向け `-march`（ppc64le は `-mcpu`）。
    March(Target),
    /// `-ftree-vectorize`。
    TreeVectorize,
    /// `-fno-semantic-interposition`。
    NoSemanticInterposition,
    /// `-fno-plt`（amd64/arm64）。
    NoPlt,
    /// `-mtls-dialect=gnu2`（amd64）。
    TlsDialectGnu2,
    /// `-fhardened`（FORTIFY=3・PIE・RELRO/NOW・SSP strong・stack-clash・
    /// cf-protection・auto-var-init の傘フラグ）。
    Hardened,
    /// `-fno-hardened`。傘フラグのみ解除する部分 opt-out（toolchain 既定の
    /// PIE/SSP と残りの Ldflag は残る）。`Hardened` とは相互排他。
    NoHardened,
    /// `-Wno-hardened`（upstream が `_FORTIFY_SOURCE` 等を自前定義した際の
    /// 冗長警告抑止。Wolfi #31194 と同じ対策）。
    NoWhardened,
    /// `-U_FORTIFY_SOURCE -D_FORTIFY_SOURCE=3`。**Hardened 適用時は
    /// `-Whardened` が発火するため併記しない**（Hardened 非適用パッケージ用）。
    FortifySource,
    /// `-fstack-protector-strong`。
    StackProtectorStrong,
    /// `-fstack-clash-protection`（-fhardened 内包。個別指定用）。
    StackClashProtection,
    /// `-fcf-protection=full`（amd64 のみ。非 x86 はハードエラー）。
    CfProtection,
    /// `-ftrivial-auto-var-init=zero`（-fhardened 内包。個別指定用）。
    TrivialAutoVarInit,
    /// `-D_GLIBCXX_ASSERTIONS`（-fhardened 内包。個別指定用）。
    GlibcxxAssertions,
    /// `-fstrict-flex-arrays=3`。
    StrictFlexArrays3,
    /// `-fzero-init-padding-bits=all`（GCC15+）。
    ZeroInitPaddingBits,
    /// `-fzero-call-used-regs=used-gpr`（amd64/arm64）。
    ZeroCallUsedRegs,
    /// `-fno-delete-null-pointer-checks`。
    NoDeleteNullPointer,
    /// `-fno-strict-overflow`（≡`-fwrapv -fwrapv-pointer`）。
    NoStrictOverflow,
    /// `-fno-strict-aliasing`。
    NoStrictAliasing,
    /// `-fexceptions`。
    Exceptions,
    /// `-Wtrampolines`。
    Wtrampolines,
    /// `-Wformat -Werror=format-security`。
    FormatSecurityError,
    /// `-mbranch-protection=standard`（arm64。v8.0 でも hint 空間命令=NOP）。
    BranchProtection,
    /// `-flto=auto`。
    Lto,
    /// `-Werror=odr`。
    WerrorOdr,
    /// `-Werror=lto-type-mismatch`。
    WerrorLtoTypeMismatch,
    /// `-Werror=strict-aliasing`（NoStrictAliasing 適用時は発火しない）。
    WerrorStrictAliasing,
}

/// リンカ/ドライバへ渡すフラグ。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldflag {
    /// `-Wl,-O1`。
    O1,
    /// `-Wl,--as-needed`。
    AsNeeded,
    /// `-Wl,-z,relro`。
    Relro,
    /// `-Wl,-z,now`。
    Now,
    /// `-Wl,-z,noexecstack`。
    NoExecStack,
    /// `-Wl,--no-copy-dt-needed-entries`。
    NoCopyDtNeeded,
    /// `-Wl,-z,pack-relative-relocs`（amd64/arm64/ppc64le）。
    PackRelativeRelocs,
    /// `-Wl,-z,cet-report=error`（amd64。strict CET の build-time 強制。opt-in）。
    CetReportError,
    /// `-flto=auto`。
    Lto,
    /// `-fuse-ld=mold`。
    FuseLdMold,
    /// `-fno-hardened`（ドライバオプション。LDFLAGS 経由でも opt-out が効く）。
    NoHardened,
    /// `-Wl,-z,nodlopen`（opt-in。dlopen 対象 .so には破壊的）。
    NoDlopen,
    /// `-Wl,-z,defs`（opt-in。未定義シンボル残置の検出）。
    Zdefs,
    /// `-Wl,-Bsymbolic-functions`（opt-in。interposition 必須 pkg では破壊的）。
    BsymbolicFunctions,
    /// `-Werror=odr`。
    WerrorOdr,
    /// `-Werror=lto-type-mismatch`。
    WerrorLtoTypeMismatch,
    /// `-Werror=strict-aliasing`。
    WerrorStrictAliasing,
}

impl Cflag {
    /// 全 variant の列挙（文書生成での opt-in トークン導出用）。
    /// 新しい variant を追加したらここにも列挙すること。
    pub const ALL: &[Cflag] = &[
        Cflag::O2,
        Cflag::Pipe,
        Cflag::March(crate::target::Target::BUILD_ENV),
        Cflag::TreeVectorize,
        Cflag::NoSemanticInterposition,
        Cflag::NoPlt,
        Cflag::TlsDialectGnu2,
        Cflag::Hardened,
        Cflag::NoHardened,
        Cflag::NoWhardened,
        Cflag::FortifySource,
        Cflag::StackProtectorStrong,
        Cflag::StackClashProtection,
        Cflag::CfProtection,
        Cflag::TrivialAutoVarInit,
        Cflag::GlibcxxAssertions,
        Cflag::StrictFlexArrays3,
        Cflag::ZeroInitPaddingBits,
        Cflag::ZeroCallUsedRegs,
        Cflag::NoDeleteNullPointer,
        Cflag::NoStrictOverflow,
        Cflag::NoStrictAliasing,
        Cflag::Exceptions,
        Cflag::Wtrampolines,
        Cflag::FormatSecurityError,
        Cflag::BranchProtection,
        Cflag::Lto,
        Cflag::WerrorOdr,
        Cflag::WerrorLtoTypeMismatch,
        Cflag::WerrorStrictAliasing,
    ];

    /// 文書上の分類（既定フラグ表の種別列）。
    pub fn kind(&self) -> &'static str {
        match self {
            Self::O2
            | Self::Pipe
            | Self::March(_)
            | Self::TreeVectorize
            | Self::NoSemanticInterposition
            | Self::NoPlt
            | Self::TlsDialectGnu2
            | Self::Lto
            | Self::WerrorOdr
            | Self::WerrorLtoTypeMismatch
            | Self::WerrorStrictAliasing => "Perf",
            _ => "Harden",
        }
    }

    /// 表記名（`March(target)` のように payload を識別子へ戻す）。
    pub fn name(&self) -> &'static str {
        match self {
            Self::O2 => "O2",
            Self::Pipe => "Pipe",
            Self::March(_) => "March(target)",
            Self::TreeVectorize => "TreeVectorize",
            Self::NoSemanticInterposition => "NoSemanticInterposition",
            Self::NoPlt => "NoPlt",
            Self::TlsDialectGnu2 => "TlsDialectGnu2",
            Self::Hardened => "Hardened",
            Self::NoHardened => "NoHardened",
            Self::NoWhardened => "NoWhardened",
            Self::FortifySource => "FortifySource",
            Self::StackProtectorStrong => "StackProtectorStrong",
            Self::StackClashProtection => "StackClashProtection",
            Self::CfProtection => "CfProtection",
            Self::TrivialAutoVarInit => "TrivialAutoVarInit",
            Self::GlibcxxAssertions => "GlibcxxAssertions",
            Self::StrictFlexArrays3 => "StrictFlexArrays3",
            Self::ZeroInitPaddingBits => "ZeroInitPaddingBits",
            Self::ZeroCallUsedRegs => "ZeroCallUsedRegs",
            Self::NoDeleteNullPointer => "NoDeleteNullPointer",
            Self::NoStrictOverflow => "NoStrictOverflow",
            Self::NoStrictAliasing => "NoStrictAliasing",
            Self::Exceptions => "Exceptions",
            Self::Wtrampolines => "Wtrampolines",
            Self::FormatSecurityError => "FormatSecurityError",
            Self::BranchProtection => "BranchProtection",
            Self::Lto => "Lto",
            Self::WerrorOdr => "WerrorOdr",
            Self::WerrorLtoTypeMismatch => "WerrorLtoTypeMismatch",
            Self::WerrorStrictAliasing => "WerrorStrictAliasing",
        }
    }
}

impl Ldflag {
    /// 全 variant の列挙（文書生成での opt-in トークン導出用）。
    /// 新しい variant を追加したらここにも列挙すること。
    pub const ALL: &[Ldflag] = &[
        Ldflag::O1,
        Ldflag::AsNeeded,
        Ldflag::Relro,
        Ldflag::Now,
        Ldflag::NoExecStack,
        Ldflag::NoCopyDtNeeded,
        Ldflag::PackRelativeRelocs,
        Ldflag::CetReportError,
        Ldflag::Lto,
        Ldflag::FuseLdMold,
        Ldflag::NoHardened,
        Ldflag::NoDlopen,
        Ldflag::Zdefs,
        Ldflag::BsymbolicFunctions,
        Ldflag::WerrorOdr,
        Ldflag::WerrorLtoTypeMismatch,
        Ldflag::WerrorStrictAliasing,
    ];

    /// 表記名。
    pub fn name(&self) -> &'static str {
        match self {
            Self::O1 => "O1",
            Self::AsNeeded => "AsNeeded",
            Self::Relro => "Relro",
            Self::Now => "Now",
            Self::NoExecStack => "NoExecStack",
            Self::NoCopyDtNeeded => "NoCopyDtNeeded",
            Self::PackRelativeRelocs => "PackRelativeRelocs",
            Self::CetReportError => "CetReportError",
            Self::Lto => "Lto",
            Self::FuseLdMold => "FuseLdMold",
            Self::NoHardened => "NoHardened",
            Self::NoDlopen => "NoDlopen",
            Self::Zdefs => "Zdefs",
            Self::BsymbolicFunctions => "BsymbolicFunctions",
            Self::WerrorOdr => "WerrorOdr",
            Self::WerrorLtoTypeMismatch => "WerrorLtoTypeMismatch",
            Self::WerrorStrictAliasing => "WerrorStrictAliasing",
        }
    }
}

/// PGO（プロファイルガイド最適化）ポリシー。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pgo {
    /// PGO なし。
    None,
    /// 宣言 workload を使うテンプレート PGO（ネイティブのみ）。
    /// workload 本体は `Steps::pgo` の `PgoTemplate` が持つ。
    Profiled,
    /// パッケージ固有の instrumented → workload → clean → rebuild 手順。
    Custom,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flag {
    cflags: Vec<Cflag>,
    ldflags: Vec<Ldflag>,
}

/// 既定 CFLAGS。アーキ依存トークンは `target` で絞る。
pub fn tool_cflags(target: Target) -> Vec<Cflag> {
    let mut flags = vec![
        Cflag::O2,
        Cflag::Pipe,
        Cflag::March(target),
        Cflag::TreeVectorize,
        Cflag::NoSemanticInterposition,
        Cflag::Hardened,
        Cflag::NoWhardened,
        Cflag::StackProtectorStrong,
        Cflag::FormatSecurityError,
        Cflag::StrictFlexArrays3,
        Cflag::ZeroInitPaddingBits,
        Cflag::NoDeleteNullPointer,
        Cflag::NoStrictOverflow,
        Cflag::NoStrictAliasing,
        Cflag::Exceptions,
        Cflag::Wtrampolines,
        Cflag::Lto,
        Cflag::WerrorOdr,
        Cflag::WerrorLtoTypeMismatch,
        Cflag::WerrorStrictAliasing,
    ];
    match target {
        Target::Amd64(_) => {
            flags.extend([Cflag::NoPlt, Cflag::TlsDialectGnu2, Cflag::ZeroCallUsedRegs])
        }
        Target::Arm64(_) => flags.extend([
            Cflag::NoPlt,
            Cflag::BranchProtection,
            Cflag::ZeroCallUsedRegs,
        ]),
        _ => {}
    }
    flags
}

/// 既定 LDFLAGS。アーキ依存トークンは `target` で絞る。
pub fn tool_ldflags(target: Target) -> Vec<Ldflag> {
    let mut flags = vec![
        Ldflag::FuseLdMold,
        Ldflag::O1,
        Ldflag::AsNeeded,
        Ldflag::NoCopyDtNeeded,
        Ldflag::Relro,
        Ldflag::Now,
        Ldflag::NoExecStack,
        Ldflag::Lto,
        Ldflag::WerrorOdr,
        Ldflag::WerrorLtoTypeMismatch,
        Ldflag::WerrorStrictAliasing,
    ];
    if matches!(
        target,
        Target::Amd64(_) | Target::Arm64(_) | Target::Ppc64le(_)
    ) {
        flags.push(Ldflag::PackRelativeRelocs);
    }
    flags
}

pub fn tool_flags(target: Target) -> Flag {
    Flag::new(tool_cflags(target), tool_ldflags(target))
}

impl Flag {
    pub const fn new(cflags: Vec<Cflag>, ldflags: Vec<Ldflag>) -> Self {
        Self { cflags, ldflags }
    }

    pub fn cflags(&self) -> &[Cflag] {
        &self.cflags
    }

    pub fn ldflags(&self) -> &[Ldflag] {
        &self.ldflags
    }

    /// `sanitize_flags` 相当: configure probe が壊れないよう
    /// `-flto*`・LTO 系 `-Werror=*`・`-f{,no-}hardened` を落とす
    /// （`-fuse-ld=mold` と `-Werror=format-security` は保持）。
    pub fn sanitized(&self) -> Self {
        const fn keep_c(flag: &Cflag) -> bool {
            !matches!(
                flag,
                Cflag::Lto
                    | Cflag::WerrorOdr
                    | Cflag::WerrorLtoTypeMismatch
                    | Cflag::WerrorStrictAliasing
                    | Cflag::Hardened
                    | Cflag::NoHardened
            )
        }
        const fn keep_l(flag: &Ldflag) -> bool {
            !matches!(
                flag,
                Ldflag::Lto
                    | Ldflag::WerrorOdr
                    | Ldflag::WerrorLtoTypeMismatch
                    | Ldflag::WerrorStrictAliasing
                    | Ldflag::NoHardened
            )
        }
        Self::new(
            self.cflags.iter().copied().filter(keep_c).collect(),
            self.ldflags.iter().copied().filter(keep_l).collect(),
        )
    }

    pub fn env(&self) -> Vec<(&'static str, String)> {
        let mut env = Vec::new();
        let c = self
            .cflags
            .iter()
            .copied()
            .flat_map(Vec::<String>::from)
            .collect::<Vec<_>>()
            .join(" ");
        if !c.is_empty() {
            env.push(("CFLAGS", c));
        }
        let ld = self
            .ldflags
            .iter()
            .copied()
            .flat_map(Vec::<String>::from)
            .collect::<Vec<_>>()
            .join(" ");
        if !ld.is_empty() {
            env.push(("LDFLAGS", ld));
        }
        env
    }
}

impl From<Cflag> for Vec<String> {
    fn from(flag: Cflag) -> Self {
        match flag {
            Cflag::O2 => vec!["-O2".into()],
            Cflag::Pipe => vec!["-pipe".into()],
            Cflag::March(target) => vec![match target {
                Target::Ppc64le(_) => format!("-mcpu={}", target.march()),
                _ => format!("-march={}", target.march()),
            }],
            Cflag::TreeVectorize => vec!["-ftree-vectorize".into()],
            Cflag::NoSemanticInterposition => vec!["-fno-semantic-interposition".into()],
            Cflag::NoPlt => vec!["-fno-plt".into()],
            Cflag::TlsDialectGnu2 => vec!["-mtls-dialect=gnu2".into()],
            Cflag::Hardened => vec!["-fhardened".into()],
            Cflag::NoHardened => vec!["-fno-hardened".into()],
            Cflag::NoWhardened => vec!["-Wno-hardened".into()],
            Cflag::FortifySource => {
                vec!["-U_FORTIFY_SOURCE".into(), "-D_FORTIFY_SOURCE=3".into()]
            }
            Cflag::StackProtectorStrong => vec!["-fstack-protector-strong".into()],
            Cflag::StackClashProtection => vec!["-fstack-clash-protection".into()],
            Cflag::CfProtection => vec!["-fcf-protection=full".into()],
            Cflag::TrivialAutoVarInit => vec!["-ftrivial-auto-var-init=zero".into()],
            Cflag::GlibcxxAssertions => vec!["-D_GLIBCXX_ASSERTIONS".into()],
            Cflag::StrictFlexArrays3 => vec!["-fstrict-flex-arrays=3".into()],
            Cflag::ZeroInitPaddingBits => vec!["-fzero-init-padding-bits=all".into()],
            Cflag::ZeroCallUsedRegs => vec!["-fzero-call-used-regs=used-gpr".into()],
            Cflag::NoDeleteNullPointer => vec!["-fno-delete-null-pointer-checks".into()],
            Cflag::NoStrictOverflow => vec!["-fno-strict-overflow".into()],
            Cflag::NoStrictAliasing => vec!["-fno-strict-aliasing".into()],
            Cflag::Exceptions => vec!["-fexceptions".into()],
            Cflag::Wtrampolines => vec!["-Wtrampolines".into()],
            Cflag::FormatSecurityError => {
                vec!["-Wformat".into(), "-Werror=format-security".into()]
            }
            Cflag::BranchProtection => vec!["-mbranch-protection=standard".into()],
            Cflag::Lto => vec!["-flto=auto".into()],
            Cflag::WerrorOdr => vec!["-Werror=odr".into()],
            Cflag::WerrorLtoTypeMismatch => vec!["-Werror=lto-type-mismatch".into()],
            Cflag::WerrorStrictAliasing => vec!["-Werror=strict-aliasing".into()],
        }
    }
}

impl From<Ldflag> for Vec<String> {
    fn from(flag: Ldflag) -> Self {
        vec![match flag {
            Ldflag::O1 => "-Wl,-O1".into(),
            Ldflag::AsNeeded => "-Wl,--as-needed".into(),
            Ldflag::NoCopyDtNeeded => "-Wl,--no-copy-dt-needed-entries".into(),
            Ldflag::Relro => "-Wl,-z,relro".into(),
            Ldflag::Now => "-Wl,-z,now".into(),
            Ldflag::NoExecStack => "-Wl,-z,noexecstack".into(),
            Ldflag::PackRelativeRelocs => "-Wl,-z,pack-relative-relocs".into(),
            Ldflag::CetReportError => "-Wl,-z,cet-report=error".into(),
            Ldflag::Lto => "-flto=auto".into(),
            Ldflag::FuseLdMold => "-fuse-ld=mold".into(),
            Ldflag::NoHardened => "-fno-hardened".into(),
            Ldflag::NoDlopen => "-Wl,-z,nodlopen".into(),
            Ldflag::Zdefs => "-Wl,-z,defs".into(),
            Ldflag::BsymbolicFunctions => "-Wl,-Bsymbolic-functions".into(),
            Ldflag::WerrorOdr => "-Werror=odr".into(),
            Ldflag::WerrorLtoTypeMismatch => "-Werror=lto-type-mismatch".into(),
            Ldflag::WerrorStrictAliasing => "-Werror=strict-aliasing".into(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::{Amd64Variant, Arm64Variant, Ppc64leVariant, Target};

    #[test]
    fn env_joins_requested_cflags_in_order() {
        let target = Target::Amd64(Amd64Variant::V1);
        let flag = Flag::new(vec![Cflag::O2, Cflag::March(target)], vec![]);
        assert_eq!(flag.env(), [("CFLAGS", "-O2 -march=x86-64".to_string())]);
        assert!(Flag::new(vec![], vec![]).env().is_empty());
    }

    #[test]
    fn format_security_error_expands_to_two_tokens() {
        let flag = Flag::new(vec![Cflag::O2, Cflag::FormatSecurityError], vec![]);
        assert_eq!(
            flag.env(),
            [("CFLAGS", "-O2 -Wformat -Werror=format-security".to_string())]
        );
        assert_eq!(
            Vec::<String>::from(Cflag::FormatSecurityError),
            ["-Wformat", "-Werror=format-security"]
        );
    }

    #[test]
    fn env_sets_cflags_and_ldflags_together_without_mixing() {
        let flag = Flag::new(vec![Cflag::Lto], vec![Ldflag::Relro]);
        assert_eq!(
            flag.env(),
            [
                ("CFLAGS", "-flto=auto".to_string()),
                ("LDFLAGS", "-Wl,-z,relro".to_string()),
            ]
        );
    }

    #[test]
    fn lto_on_cflags_does_not_set_ldflags() {
        let flag = Flag::new(vec![Cflag::Lto], vec![]);
        assert_eq!(flag.env(), [("CFLAGS", "-flto=auto".to_string())]);
    }

    #[test]
    fn relro_does_not_set_cflags() {
        let flag = Flag::new(vec![], vec![Ldflag::Relro]);
        assert_eq!(flag.env(), [("LDFLAGS", "-Wl,-z,relro".to_string())]);
        assert_eq!(Vec::<String>::from(Ldflag::Relro), ["-Wl,-z,relro"]);
    }

    #[test]
    fn fuse_ld_mold_is_a_driver_flag() {
        assert_eq!(Vec::<String>::from(Ldflag::FuseLdMold), ["-fuse-ld=mold"]);
        let target = Target::Amd64(Amd64Variant::V4);
        assert!(tool_flags(target).ldflags().contains(&Ldflag::FuseLdMold));
        assert_eq!(
            tool_flags(target).ldflags().first(),
            Some(&Ldflag::FuseLdMold)
        );
    }

    #[test]
    fn march_from_uses_target_payload() {
        let target = Target::Amd64(Amd64Variant::V1);
        assert_eq!(Vec::<String>::from(Cflag::March(target)), ["-march=x86-64"]);
        assert_eq!(
            Vec::<String>::from(Cflag::March(Target::Ppc64le(
                crate::target::Ppc64leVariant::Power8
            ))),
            ["-mcpu=power8"]
        );
        assert_eq!(
            Vec::<String>::from(Cflag::March(Target::S390x(
                crate::target::S390xVariant::Z13
            ))),
            ["-march=z13"]
        );
    }

    #[test]
    fn tool_flags_cover_new_policy_tokens() {
        let target = Target::Amd64(Amd64Variant::V4);
        let flag = tool_flags(target);
        for expected in [
            Cflag::Hardened,
            Cflag::NoWhardened,
            Cflag::StackProtectorStrong,
            Cflag::FormatSecurityError,
            Cflag::StrictFlexArrays3,
            Cflag::ZeroInitPaddingBits,
            Cflag::NoDeleteNullPointer,
            Cflag::NoStrictOverflow,
            Cflag::NoStrictAliasing,
            Cflag::Exceptions,
            Cflag::Wtrampolines,
            Cflag::Lto,
        ] {
            assert!(
                flag.cflags().contains(&expected),
                "{expected:?} がありません"
            );
        }
        for expected in [
            Ldflag::NoCopyDtNeeded,
            Ldflag::NoExecStack,
            Ldflag::PackRelativeRelocs,
            Ldflag::Lto,
        ] {
            assert!(
                flag.ldflags().contains(&expected),
                "{expected:?} がありません"
            );
        }
        // FortifySource は -fhardened と二重定義になるため既定セットに入れない
        assert!(!flag.cflags().contains(&Cflag::FortifySource));
        assert_eq!(
            Vec::<String>::from(Cflag::FortifySource),
            ["-U_FORTIFY_SOURCE", "-D_FORTIFY_SOURCE=3"]
        );
    }

    #[test]
    fn arch_gated_tokens_follow_target() {
        let amd64 = tool_cflags(Target::Amd64(Amd64Variant::V4));
        assert!(amd64.contains(&Cflag::NoPlt));
        assert!(amd64.contains(&Cflag::TlsDialectGnu2));
        assert!(amd64.contains(&Cflag::ZeroCallUsedRegs));
        assert!(!amd64.contains(&Cflag::BranchProtection));
        let arm64 = tool_cflags(Target::Arm64(Arm64Variant::V8));
        assert!(arm64.contains(&Cflag::NoPlt));
        assert!(arm64.contains(&Cflag::BranchProtection));
        assert!(arm64.contains(&Cflag::ZeroCallUsedRegs));
        assert!(!arm64.contains(&Cflag::TlsDialectGnu2));
        let ppc = tool_cflags(Target::Ppc64le(Ppc64leVariant::Power9));
        assert!(!ppc.contains(&Cflag::NoPlt));
        assert!(!ppc.contains(&Cflag::ZeroCallUsedRegs));
        assert!(!ppc.contains(&Cflag::BranchProtection));
        assert!(
            tool_ldflags(Target::Ppc64le(Ppc64leVariant::Power9))
                .contains(&Ldflag::PackRelativeRelocs)
        );
        assert!(
            !tool_ldflags(Target::S390x(crate::target::S390xVariant::Z13))
                .contains(&Ldflag::PackRelativeRelocs)
        );
        assert!(
            !tool_ldflags(Target::Riscv64(crate::target::Riscv64Variant::Rva23u64))
                .contains(&Ldflag::PackRelativeRelocs)
        );
    }

    #[test]
    fn sanitized_strips_lto_werror_and_hardened_umbrella() {
        let flag = tool_flags(Target::Amd64(Amd64Variant::V4));
        let sanitized = flag.sanitized();
        for removed in [
            Cflag::Lto,
            Cflag::WerrorOdr,
            Cflag::WerrorLtoTypeMismatch,
            Cflag::WerrorStrictAliasing,
            Cflag::Hardened,
        ] {
            assert!(
                !sanitized.cflags().contains(&removed),
                "{removed:?} が残っています"
            );
        }
        // configure probe でも有効なフラグは残る
        assert!(
            sanitized
                .cflags()
                .contains(&Cflag::March(Target::Amd64(Amd64Variant::V4)))
        );
        assert!(sanitized.cflags().contains(&Cflag::FormatSecurityError));
        assert!(sanitized.cflags().contains(&Cflag::StackProtectorStrong));
        assert!(sanitized.ldflags().contains(&Ldflag::FuseLdMold));
        assert!(sanitized.ldflags().contains(&Ldflag::Relro));
        // NoHardened は opt-out なので CFLAGS/LDFLAGS 両方から剥がす
        let optout = Flag::new(vec![Cflag::NoHardened], vec![Ldflag::NoHardened]);
        assert!(optout.sanitized().cflags().is_empty());
        assert!(optout.sanitized().ldflags().is_empty());
    }

    #[test]
    fn no_hardened_renders_in_cflags_and_ldflags() {
        let flag = Flag::new(vec![Cflag::NoHardened], vec![Ldflag::NoHardened]);
        assert_eq!(
            flag.env(),
            [
                ("CFLAGS", "-fno-hardened".to_string()),
                ("LDFLAGS", "-fno-hardened".to_string()),
            ]
        );
    }

    #[test]
    fn cet_report_error_is_linker_flag() {
        assert_eq!(
            Vec::<String>::from(Ldflag::CetReportError),
            ["-Wl,-z,cet-report=error"]
        );
        for opt_in in [Ldflag::NoDlopen, Ldflag::Zdefs, Ldflag::BsymbolicFunctions] {
            assert!(
                !tool_ldflags(Target::Amd64(Amd64Variant::V4)).contains(&opt_in),
                "{opt_in:?} は opt-in のはずです"
            );
        }
    }
}
