use crate::flags::Flag;
use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps};
use crate::package::{Dependency, Package, Source};
use crate::target::Target;

pub struct Python;

impl Package for Python {
    fn name(&self) -> &'static str {
        "python"
    }

    fn version(&self) -> &'static str {
        "3.14.7"
    }

    fn license(&self) -> &'static str {
        "Python-2.0"
    }

    /// HOST 側で実行されるツール。依存として常に BUILD_ENV 成果物を使う。
    fn runs_on_host(&self) -> bool {
        true
    }

    fn source(&self) -> Source {
        Source {
            url: "https://www.python.org/ftp/python/3.14.7/Python-3.14.7.tar.xz",
            sha3_512: "2da7cc048103dfd9f3681848e13fd3dfa06f25cc9afd76f89f772c1e3a8f14410ba44546754ee177672d0f86fac551bbd26685335830e316ca0e917629934420",
            blake3: "4406a4956bb3265c370b011383004cc2dcc94ed2c99b5f3ed67338eeab9e5eaceedabc538098da6772b361383e84ccf0274451296970e8d7c9b4261e9a30de59",
            subdir: "Python-3.14.7",
        }
    }

    fn dependencies(&self, target: Target) -> Vec<Dependency> {
        crate::package::with_libc(
            target,
            &[
                Dependency {
                    name: "libffi",
                    version: "3.8.0",
                },
                Dependency {
                    name: "zlib",
                    version: "1.3.2",
                },
                Dependency {
                    name: "openssl",
                    version: "4.0.2",
                },
                Dependency {
                    name: "expat",
                    version: "2.8.4",
                },
                Dependency {
                    name: "sqlite",
                    version: "3.53.4",
                },
                Dependency {
                    name: "ncurses",
                    version: "6.6",
                },
                Dependency {
                    name: "readline",
                    version: "8.3",
                },
                Dependency {
                    name: "xz",
                    version: "5.8.4",
                },
                Dependency {
                    name: "bzip2",
                    version: "1.0.8",
                },
                Dependency {
                    name: "zstd",
                    version: "1.5.7",
                },
            ],
        )
    }

    fn build_dependencies(&self, _target: Target) -> Vec<Dependency> {
        crate::package::headers_dep()
    }

    fn flags(&self, target: Target) -> Flag {
        crate::flags::tool_flags(target)
    }

    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .step(Step::Shell(
                    r#"for _dep in libffi zlib openssl expat sqlite ncurses readline xz bzip2 zstd; do
  if [ -d /dep/$_dep/usr/include ]; then
    export CPPFLAGS="${CPPFLAGS:-} -I/dep/$_dep/usr/include"
    export CFLAGS="$CFLAGS -I/dep/$_dep/usr/include"
  fi
  if [ -d /dep/$_dep/usr/lib64 ]; then
    export LDFLAGS="$LDFLAGS -L/dep/$_dep/usr/lib64"
    export PKG_CONFIG_PATH="/dep/$_dep/usr/lib64/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  elif [ -d /dep/$_dep/usr/lib ]; then
    export LDFLAGS="$LDFLAGS -L/dep/$_dep/usr/lib"
    export PKG_CONFIG_PATH="/dep/$_dep/usr/lib/pkgconfig${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
  fi
done"#,
                ))
                .step(Step::configure(&[
                    "--prefix=/usr",
                    "--enable-shared",
                    "--without-ensurepip",
                ]))
                .step(Step::make(&[]))
                .step(Step::make_install(&[]))
                .step(Step::Symlink {
                    target: "python3",
                    link: "\"$OUT\"/usr/bin/python",
                    force: true,
                }),
        )
    }
}
