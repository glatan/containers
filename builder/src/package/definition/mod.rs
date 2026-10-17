use crate::error::{Error, Result};
use crate::package::Package;

mod aarch64_linux_gnu_binutils;
mod aarch64_linux_gnu_gcc;
mod aarch64_linux_gnu_gcc_stage1;
mod aarch64_linux_gnu_gcc_stage2;
mod aarch64_linux_gnu_glibc;
mod aarch64_linux_gnu_glibc_headers;
mod bash;
mod binutils;
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
mod gcc;
mod gcc_stage1;
mod gcc_stage2;
mod glib;
mod glibc;
mod glibc_headers;
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
mod powerpc64le_linux_gnu_binutils;
mod powerpc64le_linux_gnu_gcc;
mod powerpc64le_linux_gnu_gcc_stage1;
mod powerpc64le_linux_gnu_gcc_stage2;
mod powerpc64le_linux_gnu_glibc;
mod powerpc64le_linux_gnu_glibc_headers;
mod python;
mod qemu;
mod readline;
mod riscv64_linux_gnu_binutils;
mod riscv64_linux_gnu_gcc;
mod riscv64_linux_gnu_gcc_stage1;
mod riscv64_linux_gnu_gcc_stage2;
mod riscv64_linux_gnu_glibc;
mod riscv64_linux_gnu_glibc_headers;
mod rust;
mod s390x_linux_gnu_binutils;
mod s390x_linux_gnu_gcc;
mod s390x_linux_gnu_gcc_stage1;
mod s390x_linux_gnu_gcc_stage2;
mod s390x_linux_gnu_glibc;
mod s390x_linux_gnu_glibc_headers;
mod sed;
mod sqlite;
mod stages;
mod tar;
mod texinfo;
mod x86_64_linux_gnu_binutils;
mod x86_64_linux_gnu_gcc;
mod x86_64_linux_gnu_gcc_stage1;
mod x86_64_linux_gnu_gcc_stage2;
mod x86_64_linux_gnu_glibc;
mod x86_64_linux_gnu_glibc_headers;
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
    &x86_64_linux_gnu_binutils::X8664LinuxGnuBinutils,
    &x86_64_linux_gnu_gcc_stage1::X8664LinuxGnuGccStage1,
    &x86_64_linux_gnu_glibc_headers::X8664LinuxGnuGlibcHeaders,
    &x86_64_linux_gnu_gcc_stage2::X8664LinuxGnuGccStage2,
    &x86_64_linux_gnu_glibc::X8664LinuxGnuGlibc,
    &x86_64_linux_gnu_gcc::X8664LinuxGnuGcc,
    &aarch64_linux_gnu_binutils::Aarch64LinuxGnuBinutils,
    &aarch64_linux_gnu_gcc_stage1::Aarch64LinuxGnuGccStage1,
    &aarch64_linux_gnu_glibc_headers::Aarch64LinuxGnuGlibcHeaders,
    &aarch64_linux_gnu_gcc_stage2::Aarch64LinuxGnuGccStage2,
    &aarch64_linux_gnu_glibc::Aarch64LinuxGnuGlibc,
    &aarch64_linux_gnu_gcc::Aarch64LinuxGnuGcc,
    &riscv64_linux_gnu_binutils::Riscv64LinuxGnuBinutils,
    &riscv64_linux_gnu_gcc_stage1::Riscv64LinuxGnuGccStage1,
    &riscv64_linux_gnu_glibc_headers::Riscv64LinuxGnuGlibcHeaders,
    &riscv64_linux_gnu_gcc_stage2::Riscv64LinuxGnuGccStage2,
    &riscv64_linux_gnu_glibc::Riscv64LinuxGnuGlibc,
    &riscv64_linux_gnu_gcc::Riscv64LinuxGnuGcc,
    &powerpc64le_linux_gnu_binutils::Powerpc64leLinuxGnuBinutils,
    &powerpc64le_linux_gnu_gcc_stage1::Powerpc64leLinuxGnuGccStage1,
    &powerpc64le_linux_gnu_glibc_headers::Powerpc64leLinuxGnuGlibcHeaders,
    &powerpc64le_linux_gnu_gcc_stage2::Powerpc64leLinuxGnuGccStage2,
    &powerpc64le_linux_gnu_glibc::Powerpc64leLinuxGnuGlibc,
    &powerpc64le_linux_gnu_gcc::Powerpc64leLinuxGnuGcc,
    &s390x_linux_gnu_binutils::S390xLinuxGnuBinutils,
    &s390x_linux_gnu_gcc_stage1::S390xLinuxGnuGccStage1,
    &s390x_linux_gnu_glibc_headers::S390xLinuxGnuGlibcHeaders,
    &s390x_linux_gnu_gcc_stage2::S390xLinuxGnuGccStage2,
    &s390x_linux_gnu_glibc::S390xLinuxGnuGlibc,
    &s390x_linux_gnu_gcc::S390xLinuxGnuGcc,
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
    &rust::Rust,
    &build_env::BuildEnv,
];

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
    use crate::target::Target;

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
            let mut host = std::collections::BTreeSet::new();
            for dependency in package.host_dependencies(target) {
                assert!(
                    host.insert(dependency.name),
                    "{} のホスト依存が重複しています: {}",
                    package.name(),
                    dependency.name
                );
                let declared = find(dependency.name).unwrap();
                assert_eq!(declared.version(), dependency.version);
            }
            if let Some(aux) = package.aux_source() {
                assert_eq!(aux.sha3_512.len(), 128);
                assert_eq!(aux.blake3.len(), 128);
                assert!(!aux.subdir.is_empty());
            }
            assert!(!package.source().subdir.is_empty());
            assert!(!package.rendered_script(target).is_empty());
            if package.catalog() || package.compose_root() {
                assert!(
                    package.as_bootstrap().is_none(),
                    "カタログ / compose ルートは bootstrap パッケージではありません: {}",
                    package.name()
                );
            }
            if package.as_bootstrap().is_some() {
                assert!(
                    !package.rendered_script(target).contains("pacman"),
                    "{} のスクリプトに pacman があります",
                    package.name()
                );
            }
        }
        let target = crate::target::Target::BUILD_ENV;
        assert!(find("mpc").unwrap().depends_on(target, "gmp"));
        assert!(find("make").unwrap().as_bootstrap().is_some());
        assert!(
            find("x86_64-linux-gnu-gcc")
                .unwrap()
                .as_bootstrap()
                .is_some()
        );
        assert!(
            find("aarch64-linux-gnu-gcc")
                .unwrap()
                .as_bootstrap()
                .is_some()
        );
        assert!(find("openssl").unwrap().as_bootstrap().is_none());
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
        // case "$ARCH" の TGT 解決は ctx.target で静的に行われる。
        let openssl = find("openssl").unwrap();
        for (t, tgt) in [
            (crate::target::Target::BUILD_ENV, "linux-x86_64"),
            (
                crate::target::Target::Arm64(crate::target::Arm64Variant::V8),
                "linux-aarch64",
            ),
        ] {
            let script = openssl.rendered_script(t);
            assert!(script.contains(tgt), "{script}");
        }
        assert!(
            find("linux-headers")
                .unwrap()
                .rendered_script(target)
                .contains("ARCH=\"powerpc\"")
        );
        assert!(
            find("linux-headers")
                .unwrap()
                .rendered_script(target)
                .contains("ARCH=\"s390\"")
        );
        assert!(
            find("zlib-ng")
                .unwrap()
                .rendered_script(target)
                .contains("-DBUILD_TESTING=")
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
                .contains("-DCMAKE_SHARED_LINKER_FLAGS=")
        );
        // CMAKE_CROSS は ctx.cross() で静的解決: ネイティブでは付かず、
        // クロスでは triple 特化の値が直接入る。
        assert!(
            !find("zlib-ng")
                .unwrap()
                .rendered_script(target)
                .contains("CMAKE_SYSTEM_NAME=Linux")
        );
        let cross = find("zlib-ng")
            .unwrap()
            .rendered_script(crate::target::Target::Arm64(
                crate::target::Arm64Variant::V8,
            ));
        assert!(cross.contains("-DCMAKE_SYSTEM_NAME=Linux"), "{cross}");
        assert!(
            cross.contains("-DCMAKE_C_COMPILER_TARGET=aarch64-linux-gnu"),
            "{cross}"
        );
        assert!(
            cross.contains("-DCMAKE_SYSTEM_PROCESSOR=aarch64"),
            "{cross}"
        );
        // sysroot 検出は wrap_script 側に移動（Steps{sysroot:true}）。
        assert!(!cross.contains("/dep/"), "{cross}");
    }

    #[test]
    fn packages_request_individual_cflags_and_ldflags() {
        use crate::target::{Amd64Variant, Target};
        let target = Target::Amd64(Amd64Variant::V1);
        let glibc = find("x86_64-linux-gnu-glibc").unwrap().flags(target);
        assert!(glibc.cflags().is_empty());
        assert!(glibc.ldflags().is_empty());
        // openssl/zlib/zlib-ng は既定ポリシー（tool_flags）をそのまま使う
        let expected = crate::flags::tool_flags(target);
        for name in ["openssl", "zlib", "zlib-ng"] {
            let flags = find(name).unwrap().flags(target);
            assert_eq!(flags.cflags(), expected.cflags(), "{name}");
            assert_eq!(flags.ldflags(), expected.ldflags(), "{name}");
        }
        // mold だけは -fuse-ld=mold を外す（自身をリンクできない）
        let mold = find("mold").unwrap().flags(target);
        assert!(!mold.ldflags().contains(&crate::flags::Ldflag::FuseLdMold));
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
        // glibc の ld.so リンクに mold は使えない
        //（_PROCEDURE_LINKAGE_TABLE_ が UND のまま残り UND チェックに落ちる）。
        assert!(
            !find("x86_64-linux-gnu-glibc")
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
        assert!(
            gcc_script.contains("rm -rf ${BUILD}/instrumented"),
            "{gcc_script}"
        );
        assert!(gcc_script.contains(r#"s|//lib/|/lib/|g"#), "{gcc_script}");
        // Canadian wrap はネイティブでは emit されず、arm ターゲット向け
        // x86_64 gcc のみ出る（perl 生成器 + sed 書き換え + wrap 検査）。
        let gcc_arm = find("x86_64-linux-gnu-gcc").unwrap().rendered_script(arm);
        assert!(gcc_arm.contains("/tmp/canadian-host-cc"), "{gcc_arm}");
        assert!(gcc_arm.contains("/tmp/gen-wrap.pl"), "{gcc_arm}");
        assert!(gcc_arm.contains(r#"|${SYSROOT}|g"#), "{gcc_arm}");
        assert!(
            !gcc_script.contains("/tmp/canadian-host-cc"),
            "{gcc_script}"
        );
        assert!(find("python").is_ok());
        assert!(find("cmake").is_ok());
        assert!(find("mold").is_ok());
        assert!(find("python").unwrap().as_bootstrap().is_none());
        assert!(find("cmake").unwrap().as_bootstrap().is_some());
        assert!(find("mold").unwrap().as_bootstrap().is_some());
        assert!(find("qemu").unwrap().as_bootstrap().is_none());
        // env bootstrap の対象は build-env の依存閉包から導出される。
        // HOST 以外の toolchain が混ざっていないことを確認する。
        let bootstrap_order = crate::package::build::packages_in_graph_order(
            find("build-env").unwrap(),
            &|name| find(name),
            host,
        )
        .unwrap();
        for (package, _) in bootstrap_order {
            assert!(
                !package.name().starts_with("aarch64-")
                    && !package.name().starts_with("riscv64-")
                    && !package.name().starts_with("powerpc64le-")
                    && !package.name().starts_with("s390x-"),
                "bootstrap 閉包に HOST 以外の toolchain があります: {}",
                package.name()
            );
        }
        assert!(find("x86_64-linux-gnu-gcc").unwrap().supports(host));
        assert!(find("x86_64-linux-gnu-gcc").unwrap().supports(arm));
        let x86_binutils = find("x86_64-linux-gnu-binutils").unwrap();
        assert!(x86_binutils.supports(arm));
        // glibc/zlib/zstd との循環は via(Seed) 辺で断つ（全ターゲット共通）。
        assert!(
            x86_binutils
                .dependencies(arm)
                .iter()
                .all(|d| matches!(d.via, crate::package::Via::Seed(_)))
        );
        assert!(x86_binutils.depends_on(arm, "zlib"));
        assert!(x86_binutils.depends_on(host, "zlib"));
        assert!(find("aarch64-linux-gnu-gcc").unwrap().supports(host));
        assert!(find("aarch64-linux-gnu-gcc").unwrap().supports(arm));
        // *_FOR_TARGET は Canadian（PACKAGE_HOST != HOST_TRIPLE）のみ emit。
        assert!(
            find("x86_64-linux-gnu-gcc")
                .unwrap()
                .rendered_script(arm)
                .contains("CC_FOR_TARGET")
        );
        assert!(
            find("glib")
                .unwrap()
                .host_dependencies(host)
                .iter()
                .any(|d| d.name == "meson" && d.ordered())
        );
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
        assert!(
            find("openssl")
                .unwrap()
                .host_dependencies(host)
                .iter()
                .any(|d| d.name == "perl" && d.ordered())
        );
        assert!(!find("x86_64-linux-gnu-gcc-stage1").unwrap().supports(arm));
        assert!(find("x86_64-linux-gnu-glibc").unwrap().supports(host));
        assert!(!find("x86_64-linux-gnu-glibc").unwrap().supports(arm));
        assert!(find("aarch64-linux-gnu-glibc").unwrap().supports(arm));
        assert!(find("build-env").unwrap().supports(host));
        assert!(!find("build-env").unwrap().supports(arm));
        // linux-headers は全 arch のヘッダを内包する単一成果物のため
        // BUILD_ENV のみでビルドする。
        assert!(find("linux-headers").unwrap().supports(host));
        assert!(!find("linux-headers").unwrap().supports(arm));
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

    #[test]
    fn toolchain_definitions_cover_all_thirty() {
        let toolchain = PACKAGES
            .iter()
            .filter(|package| package.gnu_triple().is_some())
            .count();
        assert_eq!(toolchain, 30);
        // toolchain 30 + ホストツール 15 + linux-headers
        let bootstrap = PACKAGES
            .iter()
            .filter(|package| package.as_bootstrap().is_some())
            .count();
        assert_eq!(bootstrap, 47);
    }

    #[test]
    fn host_dependencies_cover_cc_edges() {
        // host_dependencies() を上書きする実装は cc_deps の戻り値に
        // 追記する契約。C ツールチェーンを使わないパッケージだけが
        // 空を返してよい。
        const NO_CC: &[&str] = &["meson", "ca-certificates", "build-env"];
        for package in PACKAGES {
            for target in Target::all() {
                if !package.supports(target) {
                    continue;
                }
                let cc = crate::package::cc_deps(*package, target);
                let host = package.host_dependencies(target);
                if NO_CC.contains(&package.name()) {
                    // C ツールチェーンを使わないが、python 等の非 cc な
                    // HOST 依存は持ちうる。cc 辺を含まなければよい。
                    assert!(
                        host.iter().all(|d| !cc.iter().any(|e| e.name == d.name)),
                        "{} は C ツールチェーンを使わないはず",
                        package.name()
                    );
                    continue;
                }
                for edge in &cc {
                    assert!(
                        host.iter().any(|d| d.name == edge.name),
                        "{} ({:?}) の host_dependencies に cc 辺 {} がありません",
                        package.name(),
                        target,
                        edge.name
                    );
                }
            }
        }
    }

    #[test]
    fn env_fragments_render_in_shell_recipes() {
        let host = crate::target::Target::BUILD_ENV;
        let arm = crate::target::Target::Arm64(crate::target::Arm64Variant::V8);
        // perl も Steps 化済み。-idirafter/sysroot 付与は wrap_script 側。
        let perl = find("perl").unwrap().rendered_script(arm);
        assert!(perl.starts_with("set -euxo pipefail"), "{perl}");
        assert!(perl.contains("sanitize_flags"), "{perl}");
        assert!(perl.contains("gl_cv_func_malloc_0_nonnull"), "{perl}");
        assert!(!perl.contains("-idirafter"), "{perl}");
        // glib は Steps(Env::Lto) 化済み。autotools_configure プリリュードは
        // 残るが、sysroot 付与は wrap_script 側へ移った。
        let glib = find("glib").unwrap().rendered_script(host);
        assert!(glib.starts_with("set -euxo pipefail"), "{glib}");
        assert!(glib.contains("autotools_configure"), "{glib}");
        assert!(!glib.contains("--sysroot=$SYSROOT"), "{glib}");

        // toolchain は全ステージ Steps 化済み。sysroot 付与は wrap_script
        // 側へ移り、スクリプトは set -euxo pipefail で始まる。
        for name in ["x86_64-linux-gnu-gcc", "x86_64-linux-gnu-binutils"] {
            let script = find(name).unwrap().rendered_script(host);
            assert!(script.starts_with("set -euxo pipefail"), "{name}");
        }
        let binutils = find("x86_64-linux-gnu-binutils")
            .unwrap()
            .rendered_script(host);
        assert!(binutils.contains("--enable-cet"), "{binutils}");
    }
}
