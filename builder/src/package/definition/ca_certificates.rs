use crate::flags::Var;
use crate::package::{Dependency, Package, Source};

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

    fn build_only(&self) -> bool {
        false
    }

    fn network(&self) -> bool {
        false
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/certifi/python-certifi/archive/refs/tags/2026.07.22.tar.gz",
            sha3_512: "1a708f7616f0e6e55159740c21a6af6d7207f40a39c3d04d1faccb848e544b2275c19160835c08f7a1127cd876242945b11147b6e1dceebe7db8cacbec7abbd7",
            blake3: "6c932fab86221709499401ce0ceddc997ad9b91d2f6c95e73493f18b90ddae45070a7a97681a866cce09f61c490d66e9e4ff2b2122f88918743ec98c34c63ad7",
            subdir: "python-certifi-2026.07.22",
        }
    }

    fn dependencies(&self) -> &'static [Dependency] {
        &[]
    }

    fn build_dependencies(&self) -> &'static [Dependency] {
        &[]
    }

    fn flags(&self) -> &'static [Var] {
        &[]
    }

    fn build_script(&self) -> &'static str {
        r#"set -euxo pipefail
install -D -m 644 "$SOURCE"/certifi/cacert.pem "$OUT"/etc/ssl/certs/ca-certificates.crt
ln -s ca-certificates.crt "$OUT"/etc/ssl/certs/ca-bundle.crt
ln -s certs/ca-certificates.crt "$OUT"/etc/ssl/cert.pem
"#
    }
}
