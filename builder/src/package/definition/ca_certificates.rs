use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps, Workdir};
use crate::package::{Dependency, Package, Role, Source};
use crate::target::Target;

pub struct CaCertificates;

impl Package for CaCertificates {
    fn name(&self) -> &'static str {
        "ca-certificates"
    }

    fn version(&self) -> &'static str {
        "2026.07.22"
    }

    fn license(&self) -> &'static str {
        "MPL-2.0"
    }

    fn role(&self) -> Role {
        Role::Catalog
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/certifi/python-certifi/archive/refs/tags/2026.07.22.tar.gz",
            sha3_512: "1a708f7616f0e6e55159740c21a6af6d7207f40a39c3d04d1faccb848e544b2275c19160835c08f7a1127cd876242945b11147b6e1dceebe7db8cacbec7abbd7",
            blake3: "6c932fab86221709499401ce0ceddc997ad9b91d2f6c95e73493f18b90ddae45070a7a97681a866cce09f61c490d66e9e4ff2b2122f88918743ec98c34c63ad7",
            subdir: "python-certifi-2026.07.22",
        }
    }

    fn dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        vec![]
    }

    fn flags(&self, _target: Target) -> Flag {
        Flag::new(vec![], vec![])
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::None)
                .sysroot(false)
                .workdir(Workdir::None)
                .step(Step::Install {
                    src: "\"$SOURCE\"/certifi/cacert.pem",
                    dst: "\"$OUT\"/etc/ssl/certs/ca-certificates.crt",
                    mode: "644",
                })
                .step(Step::Symlink {
                    target: "ca-certificates.crt",
                    link: "\"$OUT\"/etc/ssl/certs/ca-bundle.crt",
                    force: false,
                })
                .step(Step::Symlink {
                    target: "certs/ca-certificates.crt",
                    link: "\"$OUT\"/etc/ssl/cert.pem",
                    force: false,
                }),
        )
    }
}
