use crate::flags::Flag;
use crate::package::recipe::{Arg, BuildContext, Env, Pipe, Recipe, Step, Steps};
use crate::package::{Bootstrap, Dependency, Package, Source, Via, dep};
use crate::target::Target;

pub struct Cmake;

impl Package for Cmake {
    fn name(&self) -> &'static str {
        "cmake"
    }

    fn version(&self) -> &'static str {
        "4.4.4"
    }

    fn license(&self) -> &'static str {
        "BSD-3-Clause"
    }

    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://github.com/Kitware/CMake/releases/download/v4.4.4/cmake-4.4.4.tar.gz",
            sha3_512: "b7174122a3702699030a6c2712c6b3b4730c5e8ec949837dcf1a6bf7d83a96075490ea6fcffb4faacc8216b6706f53f19cd9a84609482b677080f737f5513cfe",
            blake3: "7552b5d0fe236d45d321d7a17d44d1a2d89a68ad11759fa63be905b74f0a33991634803482fb0074c11aabae172c88971326a21e20d2902f2e3dc85a9cf581c4",
            subdir: "cmake-4.4.4",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(target, &[crate::package::gcc_dep(target)])
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    fn flags(&self, target: Target) -> Flag {
        // 2 つの理由で LTO を外す:
        // 1. bootstrap の flex 生成 lexer（cmExprLexer/cmGccDepfileLexer）が
        //    `YY_EXTRA_TYPE` の型名だけ異なる実際の ODR 違反を持ち、
        //    LTO では正しい動作が保証できない。
        // 2. LTO オブジェクトのリンクではドライバ内部で `-r` が使われ、
        //    spec 由来の `-fhardened` が "linker hardening options not
        //    enabled" を警告（`-Wno-hardened` では抑止不可）する。
        //    cmake の cm_cxx_features.cmake は try_compile 出力の
        //    "warning" を機能なしと誤判定するため bootstrap が死ぬ。
        use crate::flags::{Cflag, Ldflag};
        let drop_c = |f: &&Cflag| {
            !matches!(
                **f,
                Cflag::Lto | Cflag::WerrorOdr | Cflag::WerrorLtoTypeMismatch
            )
        };
        let drop_l = |f: &&Ldflag| {
            !matches!(
                **f,
                Ldflag::Lto | Ldflag::WerrorOdr | Ldflag::WerrorLtoTypeMismatch
            )
        };
        Flag::new(
            crate::flags::tool_cflags(target)
                .iter()
                .filter(drop_c)
                .copied()
                .collect(),
            crate::flags::tool_ldflags(target)
                .iter()
                .filter(drop_l)
                .copied()
                .collect(),
        )
    }

    /// クロスビルドでは bootstrap の実行テストがホストで動かないため
    /// BUILD_ENV 側の cmake で configure する（自己参照。シード相は
    /// distro cmake が担保し、env では成果物必須）。
    fn host_dependencies(&self, target: Target) -> Vec<Dependency> {
        let mut deps = crate::package::cc_deps(self, target);
        if target.gnu_triple() != Target::HOST_GNU_TRIPLE {
            deps.push(dep("cmake", "4.4.4").via(Via::Seed("cmake")));
        }
        deps
    }

    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        if ctx.cross() {
            let arch = ctx.triple.split('-').next().unwrap_or(&ctx.triple);
            return Recipe::Steps(
                Steps::new(Env::None)
                    .step(Step::Cmake {
                        source_dir: Arg::quoted(Arg::Var("SOURCE")),
                        build_dir: Arg::quoted(Arg::Var("BUILD")),
                        env: vec![],
                        args: vec![
                            Arg::lit("-DCMAKE_SYSTEM_NAME=Linux"),
                            Arg::owned(format!("-DCMAKE_SYSTEM_PROCESSOR={arch}")),
                            // find_* は sysroot 配下を検索させる
                            Arg::lit("-DCMAKE_SYSROOT=/tmp/sysroot"),
                            Arg::lit("-DCMAKE_BUILD_TYPE=Release"),
                            Arg::lit("-DCMAKE_INSTALL_PREFIX=/usr"),
                            Arg::lit("-DCMAKE_USE_OPENSSL=OFF"),
                            Arg::lit("-DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON"),
                            Arg::lit("-DBUILD_TESTING=OFF"),
                            Arg::Concat(vec![
                                Arg::lit("-DCMAKE_C_COMPILER="),
                                Arg::quoted(Arg::Var("CC")),
                            ]),
                            Arg::Concat(vec![
                                Arg::lit("-DCMAKE_CXX_COMPILER="),
                                Arg::quoted(Arg::Var("CXX")),
                            ]),
                        ],
                    })
                    .step(Step::Exec {
                        env: vec![],
                        pipe: Pipe::cmd(vec![
                            Arg::lit("cmake"),
                            Arg::lit("--build"),
                            Arg::quoted(Arg::Var("BUILD")),
                            Arg::nproc_jobs("-j"),
                        ]),
                    })
                    .step(Step::Exec {
                        env: vec![("DESTDIR", Arg::quoted(Arg::Var("OUT")))],
                        pipe: Pipe::cmd(vec![
                            Arg::lit("cmake"),
                            Arg::lit("--install"),
                            Arg::quoted(Arg::Var("BUILD")),
                        ]),
                    }),
            );
        }
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .step(Step::ConfigureScript {
                    env: vec![],
                    program: vec![Arg::Concat(vec![
                        Arg::quoted(Arg::Var("SOURCE")),
                        Arg::lit("/bootstrap"),
                    ])],
                    args: vec![
                        Arg::lit("--prefix=/usr"),
                        Arg::nproc_jobs("--parallel="),
                        Arg::lit("--"),
                        // イメージの /usr/include は空で、glibc ヘッダは
                        // sysroot 側にある（cmlibarchive の FIND_PATH
                        // iconv.h 等が見えるようにする）
                        Arg::lit("-DCMAKE_SYSROOT=/tmp/sysroot"),
                        Arg::lit("-DCMAKE_USE_OPENSSL=OFF"),
                        Arg::lit("-DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON"),
                    ],
                })
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }
    /// `Bootstrap` を実装（bootstrap パッケージ）。
    fn as_bootstrap(&self) -> Option<&dyn Bootstrap> {
        Some(self)
    }
}

impl Bootstrap for Cmake {}
