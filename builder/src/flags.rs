use crate::target::Target;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cflag {
    O2,
    Pipe,
    March(Target),
    TreeVectorize,
    NoSemanticInterposition,
    FortifySource,
    StackProtectorStrong,
    FormatSecurity,
    Lto,
    WerrorOdr,
    WerrorLtoTypeMismatch,
    WerrorStrictAliasing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ldflag {
    O1,
    AsNeeded,
    Relro,
    Now,
    Lto,
    FuseLdMold,
    WerrorOdr,
    WerrorLtoTypeMismatch,
    WerrorStrictAliasing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Flag {
    cflags: Vec<Cflag>,
    ldflags: Vec<Ldflag>,
}

pub fn tool_cflags(target: Target) -> Vec<Cflag> {
    vec![
        Cflag::O2,
        Cflag::Pipe,
        Cflag::March(target),
        Cflag::TreeVectorize,
        Cflag::NoSemanticInterposition,
        Cflag::FortifySource,
        Cflag::StackProtectorStrong,
        Cflag::FormatSecurity,
        Cflag::Lto,
        Cflag::WerrorOdr,
        Cflag::WerrorLtoTypeMismatch,
        Cflag::WerrorStrictAliasing,
    ]
}

pub fn tool_ldflags() -> Vec<Ldflag> {
    vec![
        Ldflag::FuseLdMold,
        Ldflag::O1,
        Ldflag::AsNeeded,
        Ldflag::Relro,
        Ldflag::Now,
        Ldflag::Lto,
        Ldflag::WerrorOdr,
        Ldflag::WerrorLtoTypeMismatch,
        Ldflag::WerrorStrictAliasing,
    ]
}

pub fn tool_flags(target: Target) -> Flag {
    Flag::new(tool_cflags(target), tool_ldflags())
}

impl Flag {
    pub fn new(cflags: Vec<Cflag>, ldflags: Vec<Ldflag>) -> Self {
        Self { cflags, ldflags }
    }

    pub fn cflags(&self) -> &[Cflag] {
        &self.cflags
    }

    pub fn ldflags(&self) -> &[Ldflag] {
        &self.ldflags
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
            Cflag::FortifySource => vec!["-D_FORTIFY_SOURCE=2".into()],
            Cflag::StackProtectorStrong => vec!["-fstack-protector-strong".into()],
            Cflag::FormatSecurity => vec!["-Wformat".into(), "-Wformat-security".into()],
            Cflag::Lto => vec!["-flto".into()],
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
            Ldflag::Relro => "-Wl,-z,relro".into(),
            Ldflag::Now => "-Wl,-z,now".into(),
            Ldflag::Lto => "-flto".into(),
            Ldflag::FuseLdMold => "-fuse-ld=mold".into(),
            Ldflag::WerrorOdr => "-Werror=odr".into(),
            Ldflag::WerrorLtoTypeMismatch => "-Werror=lto-type-mismatch".into(),
            Ldflag::WerrorStrictAliasing => "-Werror=strict-aliasing".into(),
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::{Amd64Variant, Target};

    #[test]
    fn env_joins_requested_cflags_in_order() {
        let target = Target::Amd64(Amd64Variant::V1);
        let flag = Flag::new(vec![Cflag::O2, Cflag::March(target)], vec![]);
        assert_eq!(flag.env(), [("CFLAGS", "-O2 -march=x86-64".to_string())]);
        assert!(Flag::new(vec![], vec![]).env().is_empty());
    }

    #[test]
    fn format_security_expands_to_two_tokens() {
        let flag = Flag::new(vec![Cflag::O2, Cflag::FormatSecurity], vec![]);
        assert_eq!(
            flag.env(),
            [("CFLAGS", "-O2 -Wformat -Wformat-security".to_string())]
        );
        assert_eq!(
            Vec::<String>::from(Cflag::FormatSecurity),
            ["-Wformat", "-Wformat-security"]
        );
    }

    #[test]
    fn env_sets_cflags_and_ldflags_together_without_mixing() {
        let flag = Flag::new(vec![Cflag::Lto], vec![Ldflag::Relro]);
        assert_eq!(
            flag.env(),
            [
                ("CFLAGS", "-flto".to_string()),
                ("LDFLAGS", "-Wl,-z,relro".to_string()),
            ]
        );
    }

    #[test]
    fn lto_on_cflags_does_not_set_ldflags() {
        let flag = Flag::new(vec![Cflag::Lto], vec![]);
        assert_eq!(flag.env(), [("CFLAGS", "-flto".to_string())]);
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
}
