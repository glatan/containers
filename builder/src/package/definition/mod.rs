use crate::error::{Error, Result};
use crate::package::{Bootstrap, Package};

mod bash;
mod bison;
mod build_env;
mod bzip2;
mod ca_certificates;
mod cmake;
mod coreutils;
mod diffutils;
mod expat;
mod findutils;
mod flex;
mod gawk;
mod glib;
mod gmp;
mod grep;
mod gzip;
mod isl;
mod libffi;
mod libxcrypt;
mod linux_headers;
mod m4;
mod make;
mod meson;
mod mold;
mod mpc;
mod mpfr;
mod ncurses;
mod ninja;
mod openssl;
mod patch;
mod pcre2;
mod perl;
mod pkgconf;
mod python;
mod qemu;
mod readline;
mod sed;
mod sqlite;
mod tar;
mod texinfo;
mod toolchain;
mod xz;
mod zlib;
mod zlib_ng;
mod zstd;

pub static PACKAGES: &[&dyn Package] = &[
    &ca_certificates::CaCertificates,
    &linux_headers::LinuxHeaders,
    &openssl::Openssl,
    &zlib::Zlib,
    &zlib_ng::ZlibNg,
    &toolchain::X86_64_BINUTILS,
    &toolchain::X86_64_GCC_STAGE1,
    &toolchain::X86_64_GLIBC_HEADERS,
    &toolchain::X86_64_GCC_STAGE2,
    &toolchain::X86_64_GLIBC,
    &toolchain::X86_64_GCC,
    &toolchain::AARCH64_BINUTILS,
    &toolchain::AARCH64_GCC_STAGE1,
    &toolchain::AARCH64_GLIBC_HEADERS,
    &toolchain::AARCH64_GCC_STAGE2,
    &toolchain::AARCH64_GLIBC,
    &toolchain::AARCH64_GCC,
    &toolchain::RISCV64_BINUTILS,
    &toolchain::RISCV64_GCC_STAGE1,
    &toolchain::RISCV64_GLIBC_HEADERS,
    &toolchain::RISCV64_GCC_STAGE2,
    &toolchain::RISCV64_GLIBC,
    &toolchain::RISCV64_GCC,
    &toolchain::PPC64LE_BINUTILS,
    &toolchain::PPC64LE_GCC_STAGE1,
    &toolchain::PPC64LE_GLIBC_HEADERS,
    &toolchain::PPC64LE_GCC_STAGE2,
    &toolchain::PPC64LE_GLIBC,
    &toolchain::PPC64LE_GCC,
    &toolchain::S390X_BINUTILS,
    &toolchain::S390X_GCC_STAGE1,
    &toolchain::S390X_GLIBC_HEADERS,
    &toolchain::S390X_GCC_STAGE2,
    &toolchain::S390X_GLIBC,
    &toolchain::S390X_GCC,
    &gmp::Gmp,
    &mpfr::Mpfr,
    &mpc::Mpc,
    &isl::Isl,
    &zstd::Zstd,
    &mold::Mold,
    &libffi::Libffi,
    &pcre2::Pcre2,
    &ncurses::Ncurses,
    &readline::Readline,
    &expat::Expat,
    &sqlite::Sqlite,
    &libxcrypt::Libxcrypt,
    &glib::Glib,
    &bash::Bash,
    &coreutils::Coreutils,
    &xz::Xz,
    &bzip2::Bzip2,
    &flex::Flex,
    &perl::Perl,
    &pkgconf::Pkgconf,
    &ninja::Ninja,
    &meson::Meson,
    &python::Python,
    &cmake::Cmake,
    &make::Make,
    &m4::M4,
    &bison::Bison,
    &findutils::Findutils,
    &sed::Sed,
    &grep::Grep,
    &gawk::Gawk,
    &diffutils::Diffutils,
    &patch::Patch,
    &tar::Tar,
    &gzip::Gzip,
    &texinfo::Texinfo,
    &qemu::Qemu,
    &build_env::BuildEnv,
];

pub static BOOTSTRAP: &[&dyn Bootstrap] = &[
    &toolchain::X86_64_BINUTILS,
    &toolchain::X86_64_GCC_STAGE1,
    &toolchain::X86_64_GLIBC_HEADERS,
    &toolchain::X86_64_GCC_STAGE2,
    &toolchain::X86_64_GLIBC,
    &toolchain::X86_64_GCC,
    &make::Make,
    &bash::Bash,
    &coreutils::Coreutils,
    &sed::Sed,
    &grep::Grep,
    &gawk::Gawk,
    &findutils::Findutils,
    &tar::Tar,
    &gzip::Gzip,
    &xz::Xz,
    &diffutils::Diffutils,
    &patch::Patch,
    &m4::M4,
    &cmake::Cmake,
    &mold::Mold,
];

pub fn find_bootstrap(name: &str) -> Option<&'static dyn Bootstrap> {
    toolchain::TOOLCHAIN
        .iter()
        .copied()
        .find(|package| package.name() == name)
        .map(|package| package as &dyn Bootstrap)
        .or_else(|| {
            BOOTSTRAP
                .iter()
                .copied()
                .find(|package| package.name() == name)
        })
}

pub fn all() -> &'static [&'static dyn Package] {
    PACKAGES
}

pub fn images() -> impl Iterator<Item = &'static dyn Package> {
    PACKAGES
        .iter()
        .copied()
        .filter(|package| !package.build_only())
}

pub fn find(name: &str) -> Result<&'static dyn Package> {
    PACKAGES
        .iter()
        .copied()
        .find(|package| package.name() == name)
        .ok_or_else(|| Error::message(format!("パッケージが見つかりません: {name}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_names_are_unique() {
        let mut names = PACKAGES
            .iter()
            .map(|package| package.name())
            .collect::<Vec<_>>();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), PACKAGES.len());
    }

    #[test]
    fn finds_known_packages() {
        assert_eq!(find("x86_64-linux-gnu-glibc").unwrap().version(), "2.44");
        assert!(find("glibc").is_err());
        assert!(find("missing").is_err());
        assert!(find("linux-headers").unwrap().build_only());
        assert!(images().any(|package| package.name() == "x86_64-linux-gnu-glibc"));
        assert!(images().any(|package| package.name() == "build-env"));
        assert!(!images().any(|package| package.name() == "linux-headers"));
    }

    #[test]
    fn openssl_declares_runtime_and_build_deps() {
        use crate::target::Target;
        let target = Target::BUILD_ENV;
        let openssl = find("openssl").unwrap();
        assert!(openssl.depends_on(target, "x86_64-linux-gnu-glibc"));
        assert!(!openssl.depends_on(target, "linux-headers"));
        assert!(openssl.build_depends_on(target, "linux-headers"));
        assert!(
            find("x86_64-linux-gnu-glibc")
                .unwrap()
                .build_depends_on(target, "linux-headers")
        );
        assert!(
            !find("x86_64-linux-gnu-glibc")
                .unwrap()
                .depends_on(target, "linux-headers")
        );
    }

    #[test]
    fn each_package_exposes_static_metadata() {
        for package in PACKAGES {
            assert!(!package.name().is_empty());
            assert!(!package.version().is_empty());
            for (field, value) in [("name", package.name()), ("version", package.version())] {
                assert!(
                    value.bytes().all(|byte| byte.is_ascii_lowercase()
                        || byte.is_ascii_digit()
                        || matches!(byte, b'.' | b'_' | b'+' | b'-')),
                    "{} の {field} がパスに使えない文字を含みます: {value}",
                    package.name()
                );
            }
            assert!(!package.license().is_empty());
            assert!(!package.source().url.is_empty());
            assert_eq!(package.source().sha3_512.len(), 128);
            assert_eq!(package.source().blake3.len(), 128);
            assert!(
                package
                    .source()
                    .sha3_512
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            );
            assert!(
                package
                    .source()
                    .blake3
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            );
            let mut runtime = std::collections::BTreeSet::new();
            let mut build = std::collections::BTreeSet::new();
            let target = crate::target::Target::BUILD_ENV;
            for dependency in package.dependencies(target) {
                assert!(
                    runtime.insert(dependency.name),
                    "{} の実行時依存が重複しています: {}",
                    package.name(),
                    dependency.name
                );
                let declared = find(dependency.name).unwrap();
                assert_eq!(declared.version(), dependency.version);
                assert!(
                    !declared.build_only(),
                    "dependencies にビルド専用パッケージは置けません: {}",
                    dependency.name
                );
            }
            for dependency in package.build_dependencies(target) {
                assert!(
                    build.insert(dependency.name),
                    "{} のビルド依存が重複しています: {}",
                    package.name(),
                    dependency.name
                );
                let declared = find(dependency.name).unwrap();
                assert_eq!(declared.version(), dependency.version);
            }
            for dependency in package
                .dependencies(target)
                .iter()
                .chain(package.build_dependencies(target).iter())
            {
                let declared = find(dependency.name).unwrap();
                assert_eq!(declared.version(), dependency.version);
            }
            assert!(!package.source().subdir.is_empty());
            assert!(!package.rendered_script(target).is_empty());
            if matches!(
                package.name(),
                "openssl" | "zlib" | "zlib-ng" | "ca-certificates" | "linux-headers" | "build-env"
            ) {
                assert!(
                    find_bootstrap(package.name()).is_none(),
                    "カタログ / compose ルートは BOOTSTRAP に載せません: {}",
                    package.name()
                );
            }
            if find_bootstrap(package.name()).is_some() {
                assert!(
                    !package.rendered_script(target).contains("pacman"),
                    "{} のスクリプトに pacman があります",
                    package.name()
                );
            }
        }
        let target = crate::target::Target::BUILD_ENV;
        assert!(find("mpc").unwrap().depends_on(target, "gmp"));
        assert!(find_bootstrap("make").is_some());
        assert!(find_bootstrap("x86_64-linux-gnu-gcc").is_some());
        assert!(find_bootstrap("aarch64-linux-gnu-gcc").is_some());
        assert!(find_bootstrap("openssl").is_none());
        assert!(find("linux-headers").unwrap().build_only());
        assert!(!find("x86_64-linux-gnu-glibc").unwrap().build_only());
        assert!(!find("zlib").unwrap().build_only());
        assert!(!find("zlib-ng").unwrap().build_only());
        assert!(!find("ca-certificates").unwrap().build_only());
        assert!(!find("openssl").unwrap().depends_on(target, "zlib"));
        assert!(!find("openssl").unwrap().depends_on(target, "zlib-ng"));
        assert!(
            !find("openssl")
                .unwrap()
                .depends_on(target, "ca-certificates")
        );
        assert!(
            find("linux-headers")
                .unwrap()
                .rendered_script(target)
                .contains("riscv64")
        );
        assert!(
            find("x86_64-linux-gnu-glibc")
                .unwrap()
                .rendered_script(target)
                .contains("realpath --relative-to")
        );
        assert!(
            find("openssl")
                .unwrap()
                .rendered_script(target)
                .contains("linux64-riscv64")
        );
        assert!(
            find("openssl")
                .unwrap()
                .rendered_script(target)
                .contains("linux-ppc64le")
        );
        assert!(
            find("openssl")
                .unwrap()
                .rendered_script(target)
                .contains("linux64-s390x")
        );
        assert!(
            find("linux-headers")
                .unwrap()
                .rendered_script(target)
                .contains("install_uapi powerpc")
        );
        assert!(
            find("linux-headers")
                .unwrap()
                .rendered_script(target)
                .contains("install_uapi s390")
        );
        assert!(
            find("zlib-ng")
                .unwrap()
                .rendered_script(target)
                .contains("BUILD_TESTING=OFF")
        );
        assert!(
            !find("zlib-ng")
                .unwrap()
                .rendered_script(target)
                .contains("CMAKE_INTERPROCEDURAL_OPTIMIZATION")
        );
        assert!(
            find("zlib-ng")
                .unwrap()
                .rendered_script(target)
                .contains(r#"CMAKE_SHARED_LINKER_FLAGS="$LDFLAGS""#)
        );
        assert!(
            find("zlib-ng")
                .unwrap()
                .rendered_script(target)
                .contains("CMAKE_SYSTEM_NAME=Linux")
        );
        assert!(
            find("zlib-ng")
                .unwrap()
                .rendered_script(target)
                .contains("/dep/${TRIPLE}-glibc/usr/${TRIPLE}")
        );
    }

    #[test]
    fn packages_request_individual_cflags_and_ldflags() {
        use crate::flags::{Cflag, Ldflag};
        use crate::target::{Amd64Variant, Target};
        let target = Target::Amd64(Amd64Variant::V1);
        let glibc = find("x86_64-linux-gnu-glibc").unwrap().flags(target);
        assert!(glibc.cflags().is_empty());
        assert!(glibc.ldflags().is_empty());
        let openssl_cflags = [
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
        ];
        let zlib_cflags = [
            Cflag::O2,
            Cflag::Pipe,
            Cflag::March(target),
            Cflag::TreeVectorize,
            Cflag::NoSemanticInterposition,
            Cflag::FortifySource,
            Cflag::StackProtectorStrong,
            Cflag::Lto,
            Cflag::WerrorOdr,
            Cflag::WerrorLtoTypeMismatch,
            Cflag::WerrorStrictAliasing,
        ];
        let dso_ldflags = [
            Ldflag::FuseLdMold,
            Ldflag::O1,
            Ldflag::AsNeeded,
            Ldflag::Relro,
            Ldflag::Now,
            Ldflag::Lto,
            Ldflag::WerrorOdr,
            Ldflag::WerrorLtoTypeMismatch,
            Ldflag::WerrorStrictAliasing,
        ];
        let openssl = find("openssl").unwrap().flags(target);
        assert_eq!(openssl.cflags(), &openssl_cflags);
        assert_eq!(openssl.ldflags(), &dso_ldflags);
        let zlib = find("zlib").unwrap().flags(target);
        assert_eq!(zlib.cflags(), &zlib_cflags);
        assert_eq!(zlib.ldflags(), &dso_ldflags);
        let zlib_ng = find("zlib-ng").unwrap().flags(target);
        assert_eq!(zlib_ng.cflags(), &zlib_cflags);
        assert_eq!(zlib_ng.ldflags(), &dso_ldflags);
        let headers = find("linux-headers").unwrap().flags(target);
        assert!(headers.cflags().is_empty());
        assert!(headers.ldflags().is_empty());
        let certs = find("ca-certificates").unwrap().flags(target);
        assert!(certs.cflags().is_empty());
        assert!(certs.ldflags().is_empty());
        assert!(
            !find("zlib")
                .unwrap()
                .rendered_script(target)
                .contains(" -flto")
        );
    }

    #[test]
    fn c_packages_detect_native_by_triple_not_empty_cross() {
        // zlib の CHOST/PGO 分岐は ctx.cross() で静的に解決される
        let native = find("zlib")
            .unwrap()
            .rendered_script(crate::target::Target::BUILD_ENV);
        assert!(native.contains("-fprofile-generate"), "{native}");
        assert!(!native.contains("CHOST="), "{native}");
        let cross = find("zlib")
            .unwrap()
            .rendered_script(crate::target::Target::Arm64(
                crate::target::Arm64Variant::V8,
            ));
        assert!(cross.contains("CHOST="), "{cross}");
        assert!(!cross.contains("-fprofile-generate"), "{cross}");
        for name in [
            "x86_64-linux-gnu-glibc",
            "openssl",
            "zlib",
            "zlib-ng",
            "linux-headers",
        ] {
            assert!(
                !find(name)
                    .unwrap()
                    .rendered_script(crate::target::Target::BUILD_ENV)
                    .contains("pacman"),
                "{name} のスクリプトに pacman があります"
            );
        }
        // openssl の PGO 分岐は ctx.cross() で静的に解決される
        let openssl_native = find("openssl")
            .unwrap()
            .rendered_script(crate::target::Target::BUILD_ENV);
        assert!(
            openssl_native.contains("-fprofile-generate"),
            "{openssl_native}"
        );
        let openssl_cross = find("openssl")
            .unwrap()
            .rendered_script(crate::target::Target::Arm64(
                crate::target::Arm64Variant::V8,
            ));
        assert!(
            !openssl_cross.contains("-fprofile-generate"),
            "{openssl_cross}"
        );
        assert!(
            !find("openssl")
                .unwrap()
                .rendered_script(crate::target::Target::BUILD_ENV)
                .contains("enable-lto")
        );
        for name in ["openssl", "zlib", "zlib-ng", "mold"] {
            let script = find(name)
                .unwrap()
                .rendered_script(crate::target::Target::BUILD_ENV);
            assert!(
                script.contains("/dep/${TRIPLE}-glibc/usr/${TRIPLE}"),
                "{name} が triple sysroot を見ていません"
            );
            assert!(
                !script.contains("-B/dep/${TRIPLE}-glibc/usr/lib64"),
                "{name} が glibc の lib64 を決め打ちしています"
            );
        }
        // Steps レシピの sysroot は wrap_script がマウント情報から静的に解決する
        let ctx = crate::package::recipe::BuildContext::of(None, crate::target::Target::BUILD_ENV);
        match find("bash").unwrap().recipe(&ctx) {
            crate::package::recipe::Recipe::Steps(steps) => assert!(steps.sysroot),
            other => panic!("bash が Steps ではありません: {other:?}"),
        }
    }

    #[test]
    fn libc_and_bootstrap_follow_host_first_policy() {
        use crate::target::{Arm64Variant, Target};
        let host = Target::BUILD_ENV;
        let arm = Target::Arm64(Arm64Variant::V8);
        assert_eq!(
            find("openssl")
                .unwrap()
                .dependencies(arm)
                .iter()
                .map(|dependency| dependency.name)
                .collect::<Vec<_>>(),
            ["aarch64-linux-gnu-glibc"]
        );
        assert!(
            find("x86_64-linux-gnu-glibc")
                .unwrap()
                .rendered_script(host)
                .contains("-fuse-ld=mold")
        );
        assert!(
            find("x86_64-linux-gnu-glibc")
                .unwrap()
                .rendered_script(host)
                .contains(r#"s|//lib/|/lib/|g"#)
        );
        assert!(
            find("x86_64-linux-gnu-glibc")
                .unwrap()
                .rendered_script(host)
                .contains("BISON_PKGDATADIR")
        );
        let gcc_script = find("x86_64-linux-gnu-gcc").unwrap().rendered_script(host);
        assert!(gcc_script.contains("CFLAGS_FOR_TARGET="), "{gcc_script}");
        assert!(gcc_script.contains("CFLAGS_FOR_BUILD="), "{gcc_script}");
        assert!(gcc_script.contains("/tmp/canadian-host-cc"), "{gcc_script}");
        assert!(gcc_script.contains("command -v perl"), "{gcc_script}");
        assert!(gcc_script.contains(r#"|$SYSROOT|g"#), "{gcc_script}");
        assert!(
            gcc_script.contains("rm -rf \"$BUILD/instrumented\""),
            "{gcc_script}"
        );
        assert!(gcc_script.contains(r#"s|//lib/|/lib/|g"#), "{gcc_script}");
        assert!(find("python").is_ok());
        assert!(find("cmake").is_ok());
        assert!(find("mold").is_ok());
        assert!(find_bootstrap("python").is_none());
        assert!(find_bootstrap("cmake").is_some());
        assert!(find_bootstrap("mold").is_some());
        assert!(find_bootstrap("qemu").is_none());
        for package in BOOTSTRAP {
            assert!(
                !package.name().starts_with("aarch64-")
                    && !package.name().starts_with("riscv64-")
                    && !package.name().starts_with("powerpc64le-")
                    && !package.name().starts_with("s390x-"),
                "BOOTSTRAP に HOST 以外の toolchain があります: {}",
                package.name()
            );
        }
        assert!(find("x86_64-linux-gnu-gcc").unwrap().supports(host));
        assert!(find("x86_64-linux-gnu-gcc").unwrap().supports(arm));
        assert!(find("x86_64-linux-gnu-binutils").unwrap().supports(arm));
        assert!(
            find("x86_64-linux-gnu-binutils")
                .unwrap()
                .build_depends_on(arm, "zlib")
        );
        assert!(
            !find("x86_64-linux-gnu-binutils")
                .unwrap()
                .build_depends_on(host, "zlib")
        );
        assert!(find("aarch64-linux-gnu-gcc").unwrap().supports(host));
        assert!(find("aarch64-linux-gnu-gcc").unwrap().supports(arm));
        assert!(
            find("x86_64-linux-gnu-gcc")
                .unwrap()
                .rendered_script(host)
                .contains("CC_FOR_TARGET")
        );
        assert!(find("glib").unwrap().build_depends_on(host, "meson"));
        assert!(find("make").unwrap().supports(arm));
        assert!(
            find("make")
                .unwrap()
                .supports(Target::Arm64(Arm64Variant::V8_1))
        );
        assert!(
            find("openssl")
                .unwrap()
                .supports(Target::S390x(crate::target::S390xVariant::Z900))
        );
        assert!(find("python").unwrap().supports(host));
        assert!(find("python").unwrap().supports(arm));
        assert!(find("cmake").unwrap().supports(arm));
        assert!(find("meson").unwrap().supports(arm));
        assert!(find("ninja").unwrap().supports(arm));
        assert!(!find("qemu").unwrap().supports(arm));
        assert!(
            find("make")
                .unwrap()
                .rendered_script(host)
                .contains("autotools_configure")
        );
        assert!(
            find("diffutils")
                .unwrap()
                .rendered_script(arm)
                .contains("gl_cv_func_strcasecmp_works=yes")
        );
        assert!(
            find("bash")
                .unwrap()
                .rendered_script(host)
                .contains("autotools_configure")
        );
        assert!(find("openssl").unwrap().build_depends_on(host, "perl"));
        assert!(!find("x86_64-linux-gnu-gcc-stage1").unwrap().supports(arm));
        assert!(find("x86_64-linux-gnu-glibc").unwrap().supports(host));
        assert!(!find("x86_64-linux-gnu-glibc").unwrap().supports(arm));
        assert!(find("aarch64-linux-gnu-glibc").unwrap().supports(arm));
        assert!(find("build-env").unwrap().supports(host));
        assert!(!find("build-env").unwrap().supports(arm));
        assert!(find("linux-headers").unwrap().supports(host));
        assert!(find("linux-headers").unwrap().supports(arm));
        assert!(
            find("x86_64-linux-gnu-gcc")
                .unwrap()
                .supports(Target::S390x(crate::target::S390xVariant::Z900))
        );
        assert!(
            find("aarch64-linux-gnu-glibc")
                .unwrap()
                .supports(Target::Arm64(Arm64Variant::V8_1))
        );
    }
}
