use crate::package::recipe::{BuildContext, Env, Recipe, Step, Steps};
use crate::package::{Package, Source};

pub struct Ncurses;

impl Package for Ncurses {
    fn name(&self) -> &'static str {
        "ncurses"
    }
    fn version(&self) -> &'static str {
        "6.6"
    }
    fn license(&self) -> &'static str {
        "MIT-open-group"
    }
    fn source(&self) -> Source {
        Source {
            url: "https://ftp.gnu.org/gnu/ncurses/ncurses-6.6.tar.gz",
            sha3_512: "c569ffea33791685cd04890e85fc457f7be7bb62b6a478341f9e17ccc2ef248edf084ac4078ef1ea08e43b7aca045c38e49054bde51fa30cd34eebf89269672d",
            blake3: "fbec55697a01f99b9cc3f25be55e73ae7091f4c53e5d81a1ea15734c4e5b72385c811def3c51272818645313ef1007cebb8edebdc6cec4a5feab9607a7c6636b",
            subdir: "ncurses-6.6",
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
    fn recipe(&self, ctx: &BuildContext) -> Recipe {
        let mut args = vec![
            "--prefix=/usr".into(),
            "--with-shared".into(),
            "--without-cxx".into(),
            "--enable-widec".into(),
            "--enable-pc-files".into(),
            "--with-pkg-config-libdir=/usr/lib/pkgconfig".into(),
            "--without-ada".into(),
            "--without-debug".into(),
        ];
        if ctx.cross() {
            args.extend([
                format!("--with-build-cc={}-gcc", ctx.host_triple),
                format!("--with-build-cpp={}-gcc", ctx.host_triple),
                "--with-build-cflags=-O2 -std=gnu17".into(),
                "--with-build-cppflags=".into(),
                "--with-build-ldflags=".into(),
            ]);
        }
        Recipe::Steps(
            Steps::new(Env::Lto)
                .step(Step::Configure(args))
                .step(Step::make(&[]))
                .step(Step::make_install(&[])),
        )
    }
}
