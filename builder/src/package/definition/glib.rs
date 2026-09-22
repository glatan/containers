use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir, argv};
use crate::package::{Package, Source};

pub struct Glib;

impl Package for Glib {
    fn name(&self) -> &'static str {
        "glib"
    }
    fn version(&self) -> &'static str {
        "2.90.0"
    }
    fn license(&self) -> &'static str {
        "LGPL-2.1-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://download.gnome.org/sources/glib/2.90/glib-2.90.0.tar.xz",
            sha3_512: "c896df2b43beb87254843d355e2fc389d9d62e01a49c58e078f3736286c0f0f16eedcfb611b19edb13ebf38ec8714ab1639b7b52b1ad02301eb1365cddaa205d",
            blake3: "0cd33e9b83b0fbc9b8396e99a59504909ced4e2f233f445e6127dacbcf17c0d00cc425186477da98e7f37cacfc8a27f8957f28b6b1cbd94ba1e98deef2570866",
            subdir: "glib-2.90.0",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[
                crate::package::dep("libffi", "3.8.0"),
                crate::package::dep("pcre2", "10.48"),
                crate::package::dep("zlib", "1.3.2"),
            ],
        )
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }

    /// meson ビルド系。meson/ninja/python/pkgconf はビルド中に
    /// 実行するため必須の HOST 依存。
    fn host_dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        deps.extend([
            crate::package::dep("meson", "1.12.0"),
            crate::package::dep("ninja", "1.13.2"),
            crate::package::dep("python", "3.14.7"),
            crate::package::dep("pkgconf", "2.5.1"),
        ]);
        deps
    }
    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let mut steps = Steps::new(Env::Lto).workdir(Workdir::Build);
        let mut meson_args = argv(&[
            "--prefix=/usr",
            "--wrap-mode=nofallback",
            "-Dlibmount=disabled",
            "-Dnls=disabled",
            "-Dtests=false",
            "-Dsysprof=disabled",
            "-Dintrospection=disabled",
        ]);
        if ctx.cross() {
            // case "$TRIPLE" の cpu 名解決を Rust 側へ移す。
            let (cpu_family, cpu) = match ctx.triple.split('-').next() {
                Some("aarch64") => ("aarch64", "aarch64"),
                Some("riscv64") => ("riscv64", "riscv64"),
                Some("x86_64") => ("x86_64", "x86_64"),
                Some("powerpc64le") => ("ppc64", "ppc64le"),
                Some("s390x") => ("s390x", "s390x"),
                other => panic!("未対応の meson cpu: {}", other.unwrap_or("")),
            };
            steps = steps.step(Step::WriteFile {
                path: Arg::lit("meson-cross.ini"),
                lines: vec![
                    Arg::lit("[binaries]"),
                    Arg::lit("c = '$CC'"),
                    Arg::lit("cpp = '${CXX:-$CC}'"),
                    Arg::lit("ar = '${AR:-${TRIPLE}-ar}'"),
                    Arg::owned(format!("strip = '{}-strip'", ctx.triple)),
                    Arg::lit("pkg-config = 'pkg-config'"),
                    Arg::lit("[host_machine]"),
                    Arg::lit("system = 'linux'"),
                    Arg::owned(format!("cpu_family = '{cpu_family}'")),
                    Arg::owned(format!("cpu = '{cpu}'")),
                    Arg::lit("endian = 'little'"),
                ],
                append: false,
                expand: true,
            });
            meson_args.push(Arg::lit("--cross-file"));
            meson_args.push(Arg::quoted(Arg::Concat(vec![
                Arg::Var("BUILD"),
                Arg::lit("/meson-cross.ini"),
            ])));
        }
        let mut setup = argv(&["meson", "setup"]);
        setup.push(Arg::quoted(Arg::Var("SOURCE")));
        setup.extend(meson_args);
        Recipe::Steps(
            steps
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(setup),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(
                        [
                            argv(&["meson", "compile", "-C", ".", "-j"]),
                            vec![Arg::nproc()],
                        ]
                        .concat(),
                    ),
                })
                .step(Step::Exec {
                    env: vec![],
                    pipe: Pipe::cmd(vec![
                        Arg::lit("meson"),
                        Arg::lit("install"),
                        Arg::lit("-C"),
                        Arg::lit("."),
                        Arg::lit("--destdir"),
                        Arg::quoted(Arg::Var("OUT")),
                    ]),
                }),
        )
    }
}
