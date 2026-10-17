//! レンダリング済みビルドスクリプトの静的検証。
//! フラグ・PGO・toolchain configure が意図どおり emit されることを確認する。
use containers_builder::flags::Pgo;
use containers_builder::package::definition::{all, find};
use containers_builder::target::{Amd64Variant, Arm64Variant, Target};

const NATIVE: Target = Target::Amd64(Amd64Variant::V4);
const CROSS: Target = Target::Arm64(Arm64Variant::V8);

fn script(name: &str, target: Target) -> String {
    find(name).unwrap().rendered_script(target)
}

// -- PGO テンプレート A（autotools）----------------------------------------

#[test]
fn xz_native_expands_pgo_phases() {
    let s = script("xz", NATIVE);
    assert!(s.contains("export PGODIR=/tmp/pgo"), "{s}");
    assert!(
        s.contains("-fprofile-generate=$PGODIR -fprofile-update=atomic"),
        "{s}"
    );
    assert!(
        s.contains("-fprofile-use=$PGODIR -fprofile-correction"),
        "{s}"
    );
    assert!(s.contains(") || true"), "{s}");
    assert!(s.contains("make distclean || make clean"), "{s}");
    // configure probe は sanitized、make はフルフラグの変数上書き
    assert!(s.contains("sanitize_flags"), "{s}");
    assert!(s.contains("CFLAGS=\"${CFLAGS:-}\""), "{s}");
    // ワークロードが emit されている
    assert!(s.contains("./src/xz/xz -T4"), "{s}");
}

#[test]
fn xz_cross_falls_back_to_plain_build() {
    let s = script("xz", CROSS);
    assert!(!s.contains("-fprofile-generate"), "{s}");
    assert!(!s.contains("-fprofile-use"), "{s}");
    assert!(!s.contains("PGODIR"), "{s}");
    assert!(s.contains("autotools_configure"), "{s}");
    assert!(s.contains("--host=\"${PACKAGE_HOST:-$TRIPLE}\""), "{s}");
}

// -- Env::Lto + PGO（configure が env を焼く系）-----------------------------

#[test]
fn bash_lto_env_injects_profile_flags_before_configure() {
    let s = script("bash", NATIVE);
    // Env::Lto は sanitize しない
    assert!(!s.contains("sanitize_flags"), "{s}");
    // configure 前に -fprofile-generate を export する
    let generate = s.find("-fprofile-generate=$PGODIR").unwrap();
    let configure = s.find("autotools_configure --prefix=/usr").unwrap();
    assert!(generate < configure, "{s}");
    // bash 固有のシンボリックリンクが残る
    assert!(s.contains("usr/bin/sh"), "{s}");
}

// -- PGO テンプレート B（in-source make）-----------------------------------

#[test]
fn bzip2_replays_package_args_in_both_phases() {
    let s = script("bzip2", NATIVE);
    // パッケージ固有 CFLAGS（_FILE_OFFSET_BITS 等）は gen/use 双方で replay
    // （2 本の make 行 × 2 フェーズ）
    assert_eq!(s.matches("-D_FILE_OFFSET_BITS=64").count(), 4, "{s}");
    assert_eq!(s.matches("bzip2recover").count(), 3, "{s}");
    // install は use フェーズ後に 1 回だけ
    assert_eq!(s.matches("make install").count(), 1, "{s}");
    assert!(s.contains("-fprofile-generate"), "{s}");
    assert!(s.contains("-fprofile-use"), "{s}");
    assert!(s.contains("make clean || make distclean"), "{s}");
}

#[test]
fn zstd_replays_make_vars_in_both_phases() {
    let s = script("zstd", NATIVE);
    assert_eq!(s.matches("HAVE_ZLIB=0").count(), 2, "{s}");
    assert!(s.contains("./programs/zstd -b"), "{s}");
}

// -- Custom PGO --------------------------------------------------------------

#[test]
fn openssl_cleans_between_profile_phases() {
    let s = script("openssl", NATIVE);
    assert!(s.contains("-fprofile-generate"), "{s}");
    assert!(s.contains("make clean"), "{s}");
    assert!(s.contains("-fprofile-use"), "{s}");
    let cross = script("openssl", CROSS);
    assert!(!cross.contains("-fprofile-generate"), "{cross}");
}

#[test]
fn python_enables_optimizations_only_when_native() {
    assert!(script("python", NATIVE).contains("--enable-optimizations"));
    assert!(!script("python", CROSS).contains("--enable-optimizations"));
}

#[test]
fn python_exports_dep_ld_library_path_only_when_native() {
    let needle = "LD_LIBRARY_PATH=\"/dep/${_dep}/usr/lib";
    assert!(script("python", NATIVE).contains(needle));
    let cross = script("python", CROSS);
    assert!(!cross.contains(needle));
    assert!(
        cross.contains(
            "sed -i \"s|check_extension_modules.py|check_extension_modules.py --cross-compiling|g\" Makefile"
        ),
        "{cross}"
    );
    assert!(cross.contains("make -j\"$(nproc)\" RUNSHARED="), "{cross}");
    assert!(!script("python", NATIVE).contains("check_extension_modules.py --cross-compiling"));
}

#[test]
fn ninja_uses_custom_pgo_bootstrap() {
    let s = script("ninja", NATIVE);
    assert!(s.contains("--bootstrap"), "{s}");
    assert!(s.contains("-fprofile-generate"), "{s}");
    assert!(s.contains("-fprofile-use"), "{s}");
}

// -- ツールチェーン -----------------------------------------------------------

#[test]
fn final_gcc_passes_hardened_specs_and_per_variant_arch() {
    let s = script("x86_64-linux-gnu-gcc", NATIVE);
    // specs は 1 引数としてシングルクォートされている
    assert!(
        s.contains("--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'"),
        "{s}"
    );
    // 最終 gcc のみ per-variant。host arch = codegen arch では $MARCH、
    // 異種では triple ベースラインが静的に選択される。
    assert!(s.contains("--with-arch=${MARCH} --enable-cet"), "{s}");
    assert!(s.contains("--enable-cet"), "{s}");
    assert!(s.contains("--enable-default-pie"), "{s}");
    assert!(s.contains("--enable-default-ssp"), "{s}");
    assert!(s.contains("--enable-host-pie"), "{s}");
    assert!(s.contains("--enable-host-bind-now"), "{s}");
}

#[test]
fn intermediate_gcc_keeps_baseline_arch() {
    for stage in ["x86_64-linux-gnu-gcc-stage1", "x86_64-linux-gnu-gcc-stage2"] {
        let s = script(stage, NATIVE);
        // 中間ステージは BUILD_ENV でしか組まないため triple ベースライン。
        // per-variant の $MARCH を使ってはいけない
        assert!(s.contains("--with-arch=x86-64"), "{stage}: {s}");
        assert!(!s.contains("--with-arch=$MARCH"), "{stage}: {s}");
        assert!(!s.contains("--with-specs"), "{stage}: {s}");
    }
}

#[test]
fn s390x_gcc_enables_decimal_float() {
    let s = script("s390x-linux-gnu-gcc", NATIVE);
    assert!(s.contains("--enable-decimal-float"), "{s}");
}

#[test]
fn glibc_enables_stack_protector_bind_now_fortify_and_cet() {
    let s = script("x86_64-linux-gnu-glibc", NATIVE);
    assert!(s.contains("--enable-stack-protector=strong"), "{s}");
    assert!(s.contains("--enable-bind-now"), "{s}");
    assert!(s.contains("--enable-fortify-source"), "{s}");
    // CET はレンダ時に静的解決され x86_64 のみ有効
    assert!(s.contains("--enable-cet"), "{s}");
    assert!(!s.contains("CET_FLAGS"), "{s}");
    assert!(!script("aarch64-linux-gnu-glibc", NATIVE).contains("--enable-cet"));
}

#[test]
fn native_binutils_runs_pgo_cross_does_not() {
    // PGO と --enable-cet はレンダ時に静的解決される:
    // x86_64 ネイティブのみ PGO あり、非 x86_64/クロスは素の configure+make。
    let native = script("x86_64-linux-gnu-binutils", NATIVE);
    assert!(native.contains("-fprofile-generate"), "{native}");
    assert!(native.contains("--enable-cet"), "{native}");
    let cross = script("aarch64-linux-gnu-binutils", NATIVE);
    assert!(!cross.contains("-fprofile-generate"), "{cross}");
    assert!(!cross.contains("--enable-cet"), "{cross}");
    for s in [&native, &cross] {
        assert!(s.contains("--enable-relro"), "{s}");
        assert!(s.contains("--enable-new-dtags"), "{s}");
        assert!(s.contains("--enable-default-execstack=no"), "{s}");
        assert!(s.contains("--enable-default-hash-style=gnu"), "{s}");
    }
}

// -- pgo() 宣言とレンダリングの整合 ------------------------------------------

#[test]
fn pgo_declaration_matches_rendered_script() {
    for package in all() {
        let name = package.name();
        let s = package.rendered_script(NATIVE);
        match package.pgo() {
            Pgo::None => assert!(
                !s.contains("-fprofile-"),
                "{name}: Pgo::None なのに profile フラグを含む"
            ),
            // Profiled はテンプレート経由なので -fprofile-generate が必須。
            // Custom はパッケージ固有実装（python の --enable-optimizations 等
            // -fprofile-* を出さないものもある）なので検査しない。
            Pgo::Profiled => assert!(
                s.contains("-fprofile-generate"),
                "{name}: Pgo::Profiled 宣言があるのにネイティブ script が -fprofile-generate を含まない"
            ),
            Pgo::Custom => {}
        }
    }
}
