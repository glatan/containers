# パッケージ定義

`PACKAGES` に載るすべてのパッケージの仕様文書です。1 パッケージにつき 1 ファイルです。実装は [`builder/src/package/definition/`](../../../../builder/src/package/definition/) です。プレフィックス付きツールチェーンのステージ順と共通ソースは [toolchain.md](../toolchain.md)、Host tools の共通前処理は [host-tools.md](../host-tools.md) です。公開対象の分類は [user/published.md](../../../user/published.md) です。

各ファイルの先頭部（タイトル・メタデータ・ソース・依存表）と下記の索引表は生成ブロックであり、`containers-builder docs` が `PACKAGES` から再生成します（生成区間は `GENERATED` コメントマーカーで囲まれています）。差分の検査は `docs --check` です。手書きの補足（ビルド設定・フラグ・手順の根拠）はマーカーの外側に書きます。並びは `PACKAGES` 登録順です。

<!-- GENERATED -->
| パッケージ名 | バージョン | 分類 | PGO | env | 文書 |
|-|-|-|-|-|-|
| ca-certificates | 2026.07.22 | カタログ | None | None | [ca-certificates.md](./ca-certificates.md) |
| linux-headers | 6.18.33 | ビルド専用（Bootstrap・seed） | None | None | [linux-headers.md](./linux-headers.md) |
| openssl | 4.0.2 | カタログ | Custom | None | [openssl.md](./openssl.md) |
| zlib | 1.3.2 | カタログ | Custom | None | [zlib.md](./zlib.md) |
| zlib-ng | 2.3.3 | カタログ | Custom | None | [zlib-ng.md](./zlib-ng.md) |
| x86_64-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap・seed） | Custom | None | [x86_64-linux-gnu-binutils.md](./x86_64-linux-gnu-binutils.md) |
| x86_64-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap・seed） | None | None | [x86_64-linux-gnu-gcc-stage1.md](./x86_64-linux-gnu-gcc-stage1.md) |
| x86_64-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap・seed） | None | None | [x86_64-linux-gnu-glibc-headers.md](./x86_64-linux-gnu-glibc-headers.md) |
| x86_64-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap・seed） | None | None | [x86_64-linux-gnu-gcc-stage2.md](./x86_64-linux-gnu-gcc-stage2.md) |
| x86_64-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap・seed） | None | None | [x86_64-linux-gnu-glibc.md](./x86_64-linux-gnu-glibc.md) |
| x86_64-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | Custom | None | [x86_64-linux-gnu-gcc.md](./x86_64-linux-gnu-gcc.md) |
| aarch64-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | Custom | None | [aarch64-linux-gnu-binutils.md](./aarch64-linux-gnu-binutils.md) |
| aarch64-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [aarch64-linux-gnu-gcc-stage1.md](./aarch64-linux-gnu-gcc-stage1.md) |
| aarch64-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | None | None | [aarch64-linux-gnu-glibc-headers.md](./aarch64-linux-gnu-glibc-headers.md) |
| aarch64-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [aarch64-linux-gnu-gcc-stage2.md](./aarch64-linux-gnu-gcc-stage2.md) |
| aarch64-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | None | None | [aarch64-linux-gnu-glibc.md](./aarch64-linux-gnu-glibc.md) |
| aarch64-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | Custom | None | [aarch64-linux-gnu-gcc.md](./aarch64-linux-gnu-gcc.md) |
| riscv64-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | Custom | None | [riscv64-linux-gnu-binutils.md](./riscv64-linux-gnu-binutils.md) |
| riscv64-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [riscv64-linux-gnu-gcc-stage1.md](./riscv64-linux-gnu-gcc-stage1.md) |
| riscv64-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | None | None | [riscv64-linux-gnu-glibc-headers.md](./riscv64-linux-gnu-glibc-headers.md) |
| riscv64-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [riscv64-linux-gnu-gcc-stage2.md](./riscv64-linux-gnu-gcc-stage2.md) |
| riscv64-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | None | None | [riscv64-linux-gnu-glibc.md](./riscv64-linux-gnu-glibc.md) |
| riscv64-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | Custom | None | [riscv64-linux-gnu-gcc.md](./riscv64-linux-gnu-gcc.md) |
| powerpc64le-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | Custom | None | [powerpc64le-linux-gnu-binutils.md](./powerpc64le-linux-gnu-binutils.md) |
| powerpc64le-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [powerpc64le-linux-gnu-gcc-stage1.md](./powerpc64le-linux-gnu-gcc-stage1.md) |
| powerpc64le-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | None | None | [powerpc64le-linux-gnu-glibc-headers.md](./powerpc64le-linux-gnu-glibc-headers.md) |
| powerpc64le-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [powerpc64le-linux-gnu-gcc-stage2.md](./powerpc64le-linux-gnu-gcc-stage2.md) |
| powerpc64le-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | None | None | [powerpc64le-linux-gnu-glibc.md](./powerpc64le-linux-gnu-glibc.md) |
| powerpc64le-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | Custom | None | [powerpc64le-linux-gnu-gcc.md](./powerpc64le-linux-gnu-gcc.md) |
| s390x-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | Custom | None | [s390x-linux-gnu-binutils.md](./s390x-linux-gnu-binutils.md) |
| s390x-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [s390x-linux-gnu-gcc-stage1.md](./s390x-linux-gnu-gcc-stage1.md) |
| s390x-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | None | None | [s390x-linux-gnu-glibc-headers.md](./s390x-linux-gnu-glibc-headers.md) |
| s390x-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | None | None | [s390x-linux-gnu-gcc-stage2.md](./s390x-linux-gnu-gcc-stage2.md) |
| s390x-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | None | None | [s390x-linux-gnu-glibc.md](./s390x-linux-gnu-glibc.md) |
| s390x-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | Custom | None | [s390x-linux-gnu-gcc.md](./s390x-linux-gnu-gcc.md) |
| gmp | 6.3.0 | パッケージ | Profiled | Lto | [gmp.md](./gmp.md) |
| mpfr | 4.2.2 | パッケージ | Profiled | Autotools | [mpfr.md](./mpfr.md) |
| mpc | 1.4.1 | パッケージ | Profiled | Autotools | [mpc.md](./mpc.md) |
| isl | 0.26 | パッケージ | Profiled | Autotools | [isl.md](./isl.md) |
| zstd | 1.5.7 | パッケージ | Profiled | None | [zstd.md](./zstd.md) |
| mold | 2.42.1 | パッケージ（Bootstrap） | Custom | None | [mold.md](./mold.md) |
| libffi | 3.8.0 | パッケージ | Profiled | Autotools | [libffi.md](./libffi.md) |
| pcre2 | 10.48 | パッケージ | Profiled | Autotools | [pcre2.md](./pcre2.md) |
| ncurses | 6.6 | パッケージ | None | Lto | [ncurses.md](./ncurses.md) |
| readline | 8.3 | パッケージ | Profiled | Autotools | [readline.md](./readline.md) |
| expat | 2.8.4 | パッケージ | Profiled | Autotools | [expat.md](./expat.md) |
| sqlite | 3.53.4 | パッケージ | Profiled | Autotools | [sqlite.md](./sqlite.md) |
| libxcrypt | 4.5.2 | パッケージ | Profiled | Autotools | [libxcrypt.md](./libxcrypt.md) |
| glib | 2.90.0 | パッケージ | None | Lto | [glib.md](./glib.md) |
| bash | 5.3 | パッケージ（Bootstrap） | Profiled | Lto | [bash.md](./bash.md) |
| coreutils | 9.12 | パッケージ（Bootstrap） | Profiled | Autotools | [coreutils.md](./coreutils.md) |
| xz | 5.8.4 | パッケージ（Bootstrap） | Profiled | Autotools | [xz.md](./xz.md) |
| bzip2 | 1.0.8 | パッケージ | Profiled | Autotools | [bzip2.md](./bzip2.md) |
| flex | 2.6.4 | パッケージ | Profiled | Autotools | [flex.md](./flex.md) |
| perl | 5.42.3 | パッケージ | None | Autotools | [perl.md](./perl.md) |
| pkgconf | 2.5.1 | パッケージ | Profiled | Autotools | [pkgconf.md](./pkgconf.md) |
| ninja | 1.13.2 | パッケージ | Custom | Autotools | [ninja.md](./ninja.md) |
| meson | 1.12.0 | パッケージ | None | None | [meson.md](./meson.md) |
| python | 3.14.7 | パッケージ | Custom | Autotools | [python.md](./python.md) |
| cmake | 4.4.3 | パッケージ（Bootstrap） | None | Autotools | [cmake.md](./cmake.md) |
| make | 4.4.1 | パッケージ（Bootstrap） | Profiled | Autotools | [make.md](./make.md) |
| m4 | 1.4.21 | パッケージ（Bootstrap） | Profiled | Autotools | [m4.md](./m4.md) |
| bison | 3.8.2 | パッケージ | Profiled | Autotools | [bison.md](./bison.md) |
| findutils | 4.11.0 | パッケージ（Bootstrap） | Profiled | Autotools | [findutils.md](./findutils.md) |
| sed | 4.10 | パッケージ（Bootstrap） | Profiled | Autotools | [sed.md](./sed.md) |
| grep | 3.12 | パッケージ（Bootstrap） | Profiled | Autotools | [grep.md](./grep.md) |
| gawk | 5.4.1 | パッケージ（Bootstrap） | Profiled | Autotools | [gawk.md](./gawk.md) |
| diffutils | 3.12 | パッケージ（Bootstrap） | Profiled | Autotools | [diffutils.md](./diffutils.md) |
| patch | 2.8 | パッケージ（Bootstrap） | Profiled | Autotools | [patch.md](./patch.md) |
| tar | 1.35 | パッケージ（Bootstrap） | Profiled | Autotools | [tar.md](./tar.md) |
| gzip | 1.14 | パッケージ（Bootstrap） | Profiled | Autotools | [gzip.md](./gzip.md) |
| texinfo | 7.3 | パッケージ | Profiled | Autotools | [texinfo.md](./texinfo.md) |
| qemu | 11.1.1 | パッケージ | None | None | [qemu.md](./qemu.md) |
| build-env | 1 | build-env（合成ルート） | None | — | [build-env.md](./build-env.md) |
<!-- /GENERATED -->
