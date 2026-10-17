use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps, Workdir, argv};
use crate::package::{Package, Source};

pub struct Glib;

impl Package for Glib {
    fn name(&self) -> &'static str {
        "glib"
    }
    fn version(&self) -> &'static str {
        "2.90.1"
    }
    fn license(&self) -> &'static str {
        "LGPL-2.1-or-later"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://download.gnome.org/sources/glib/2.90/glib-2.90.1.tar.xz",
            sha3_512: "6734ba4b739abf878ecd3f179fef61afd856006890c32fdc8a617af680234ccabc71a93ef36ee3cbb26a4e86b987c594eb7845c367ed8b51947fb415d3f9b911",
            blake3: "86a15eec2594a3fa4e5e301132713f21d342e57830b03c75d4d537c67f810b52ccc2f36199026032bd47908db1036db2478e8789dc1b1336857f35547a80e5da",
            subdir: "glib-2.90.1",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(
            target,
            &[
                crate::package::dep("libffi", "3.8.0"),
                crate::package::dep("pcre2", "10.49"),
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
            crate::package::dep("meson", "1.12.1"),
            crate::package::dep("ninja", "1.13.2"),
            crate::package::dep("python", "3.15.0"),
            crate::package::dep("pkgconf", "3.0.8"),
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
        Recipe::Steps(
            steps
                .step(Step::Meson {
                    args: meson_args,
                    env: vec![],
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
