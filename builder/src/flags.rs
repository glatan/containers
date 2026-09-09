use crate::error::Result;
use crate::target::Target;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Var {
    Cflags,
    Ldflags,
    Rustflags,
}

pub struct Flags {
    cflags: String,
    ldflags: String,
    rustflags: String,
}

impl Flags {
    pub fn resolve(target: Target) -> Result<Self> {
        let march = target.march()?;
        Ok(Self {
            cflags: format!("-O2 -march={march}"),
            ldflags: String::new(),
            rustflags: String::new(),
        })
    }

    pub fn env(&self, requested: &[Var]) -> Vec<(&'static str, &str)> {
        requested
            .iter()
            .map(|var| match var {
                Var::Cflags => ("CFLAGS", self.cflags.as_str()),
                Var::Ldflags => ("LDFLAGS", self.ldflags.as_str()),
                Var::Rustflags => ("RUSTFLAGS", self.rustflags.as_str()),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::{Amd64Variant, S390xVariant, Target};

    #[test]
    fn resolve_sets_cflags_from_march() {
        let flags = Flags::resolve(Target::Amd64(Amd64Variant::V1)).unwrap();
        assert_eq!(flags.env(&[Var::Cflags]), [("CFLAGS", "-O2 -march=x86-64")]);
        assert!(flags.env(&[]).is_empty());
        assert!(Flags::resolve(Target::S390x(S390xVariant::Z17)).is_err());
    }
}
