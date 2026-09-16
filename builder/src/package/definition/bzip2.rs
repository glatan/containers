use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps, Workdir};
use crate::package::{Package, Source};

pub struct Bzip2;

impl Package for Bzip2 {
    fn name(&self) -> &'static str {
        "bzip2"
    }
    fn version(&self) -> &'static str {
        "1.0.8"
    }
    fn license(&self) -> &'static str {
        "bzip2-1.0.6"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://sourceware.org/pub/bzip2/bzip2-1.0.8.tar.gz",
            sha3_512: "8192e0e7a92c083354d91e114f4e1a0571d56cc106f58a11c92b4028a1bb1ecad2e00892bb81cc1d2d2dd7371e35beebab2e92e2c83ef9ccb8a2490a97a94f52",
            blake3: "97af3f520629c65fe41292f77e6ca798fe594d7987bfb2aebe7c6fcdc7ab5ed25e9035d582b0a344afcd3ae378c3f3f999d66594bd1dd88cfcabf143cd70b8ef",
            subdir: "bzip2-1.0.8",
        }
    }
    fn dependencies(&self, target: crate::target::Target) -> Vec<crate::package::Dependency> {
        crate::package::with_libc(target, &[])
    }
    fn build_dependencies(
        &self,
        _target: crate::target::Target,
    ) -> Vec<crate::package::Dependency> {
        crate::package::headers_dep()
    }
    fn flags(&self, target: crate::target::Target) -> crate::flags::Flag {
        crate::flags::tool_flags(target)
    }
    fn recipe(&self, _ctx: &BuildContext) -> Recipe {
        Recipe::Steps(
            Steps::new(Env::Autotools)
                .workdir(Workdir::InSource)
                .step(Step::Shell(
                    "sed -i 's/$(CC) -shared/$(CC) $(LDFLAGS) -shared/' Makefile-libbz2_so",
                ))
                .step(Step::Make {
                    jobs: false,
                    args: crate::package::recipe::args(&[
                        "-f",
                        "Makefile-libbz2_so",
                        "CC=\"$CC\"",
                        "CFLAGS=\"$CFLAGS -fpic -fPIC -Wall -Winline -D_FILE_OFFSET_BITS=64\"",
                        "LDFLAGS=\"$LDFLAGS\"",
                    ]),
                })
                .step(Step::Make {
                    jobs: false,
                    args: crate::package::recipe::args(&[
                        "CC=\"$CC\"",
                        "AR=\"${AR:-${TRIPLE}-ar}\"",
                        "RANLIB=\"${RANLIB:-${TRIPLE}-ranlib}\"",
                        "CFLAGS=\"$CFLAGS -Wall -Winline -D_FILE_OFFSET_BITS=64\"",
                        "LDFLAGS=\"$LDFLAGS\"",
                        "libbz2.a",
                        "bzip2",
                        "bzip2recover",
                    ]),
                })
                .step(Step::Make {
                    jobs: false,
                    args: crate::package::recipe::args(&["install", "PREFIX=\"$OUT/usr\""]),
                })
                .step(Step::Shell(
                    "mkdir -p \"$OUT/usr/lib\"
cp -a libbz2.so* \"$OUT/usr/lib/\"",
                )),
        )
    }
}
