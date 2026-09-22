# パッケージ定義

`PACKAGES` に載るすべてのパッケージの仕様文書です。1 パッケージにつき 1 ファイルです。実装は [`builder/src/package/definition/`](../../../builder/src/package/definition/) です。プレフィックス付きツールチェーンのステージ順と共通ソースは [toolchain.md](../toolchain.md)、Host tools の共通前処理は [host-tools.md](../host-tools.md) です。公開対象の分類は [image/packages.md](../../image/packages.md) です。

各ファイルの先頭部（タイトル・メタデータ・ソース・依存表）と下記の索引表は生成ブロックであり、`containers-builder docs` が `PACKAGES` から再生成します（生成区間は `GENERATED` コメントマーカーで囲まれています）。差分の検査は `docs --check` です。手書きの補足（ビルド設定・フラグ・手順の根拠）はマーカーの外側に書きます。並びは `PACKAGES` 登録順です。

<!-- GENERATED -->
| パッケージ名 | バージョン | 分類 | 文書 |
|-|-|-|-|
| ca-certificates | 2026.07.22 | カタログ | [ca-certificates.md](./ca-certificates.md) |
| linux-headers | 6.18.33 | ビルド専用（Bootstrap・seed） | [linux-headers.md](./linux-headers.md) |
| openssl | 4.0.2 | カタログ | [openssl.md](./openssl.md) |
| zlib | 1.3.2 | カタログ | [zlib.md](./zlib.md) |
| zlib-ng | 2.3.3 | カタログ | [zlib-ng.md](./zlib-ng.md) |
| x86_64-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap・seed） | [x86_64-linux-gnu-binutils.md](./x86_64-linux-gnu-binutils.md) |
| x86_64-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap・seed） | [x86_64-linux-gnu-gcc-stage1.md](./x86_64-linux-gnu-gcc-stage1.md) |
| x86_64-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap・seed） | [x86_64-linux-gnu-glibc-headers.md](./x86_64-linux-gnu-glibc-headers.md) |
| x86_64-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap・seed） | [x86_64-linux-gnu-gcc-stage2.md](./x86_64-linux-gnu-gcc-stage2.md) |
| x86_64-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap・seed） | [x86_64-linux-gnu-glibc.md](./x86_64-linux-gnu-glibc.md) |
| x86_64-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | [x86_64-linux-gnu-gcc.md](./x86_64-linux-gnu-gcc.md) |
| aarch64-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | [aarch64-linux-gnu-binutils.md](./aarch64-linux-gnu-binutils.md) |
| aarch64-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | [aarch64-linux-gnu-gcc-stage1.md](./aarch64-linux-gnu-gcc-stage1.md) |
| aarch64-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | [aarch64-linux-gnu-glibc-headers.md](./aarch64-linux-gnu-glibc-headers.md) |
| aarch64-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | [aarch64-linux-gnu-gcc-stage2.md](./aarch64-linux-gnu-gcc-stage2.md) |
| aarch64-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | [aarch64-linux-gnu-glibc.md](./aarch64-linux-gnu-glibc.md) |
| aarch64-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | [aarch64-linux-gnu-gcc.md](./aarch64-linux-gnu-gcc.md) |
| riscv64-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | [riscv64-linux-gnu-binutils.md](./riscv64-linux-gnu-binutils.md) |
| riscv64-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | [riscv64-linux-gnu-gcc-stage1.md](./riscv64-linux-gnu-gcc-stage1.md) |
| riscv64-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | [riscv64-linux-gnu-glibc-headers.md](./riscv64-linux-gnu-glibc-headers.md) |
| riscv64-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | [riscv64-linux-gnu-gcc-stage2.md](./riscv64-linux-gnu-gcc-stage2.md) |
| riscv64-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | [riscv64-linux-gnu-glibc.md](./riscv64-linux-gnu-glibc.md) |
| riscv64-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | [riscv64-linux-gnu-gcc.md](./riscv64-linux-gnu-gcc.md) |
| powerpc64le-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | [powerpc64le-linux-gnu-binutils.md](./powerpc64le-linux-gnu-binutils.md) |
| powerpc64le-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | [powerpc64le-linux-gnu-gcc-stage1.md](./powerpc64le-linux-gnu-gcc-stage1.md) |
| powerpc64le-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | [powerpc64le-linux-gnu-glibc-headers.md](./powerpc64le-linux-gnu-glibc-headers.md) |
| powerpc64le-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | [powerpc64le-linux-gnu-gcc-stage2.md](./powerpc64le-linux-gnu-gcc-stage2.md) |
| powerpc64le-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | [powerpc64le-linux-gnu-glibc.md](./powerpc64le-linux-gnu-glibc.md) |
| powerpc64le-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | [powerpc64le-linux-gnu-gcc.md](./powerpc64le-linux-gnu-gcc.md) |
| s390x-linux-gnu-binutils | 2.47 | パッケージ（Bootstrap） | [s390x-linux-gnu-binutils.md](./s390x-linux-gnu-binutils.md) |
| s390x-linux-gnu-gcc-stage1 | 16.2.0 | ビルド専用（Bootstrap） | [s390x-linux-gnu-gcc-stage1.md](./s390x-linux-gnu-gcc-stage1.md) |
| s390x-linux-gnu-glibc-headers | 2.44 | ビルド専用（Bootstrap） | [s390x-linux-gnu-glibc-headers.md](./s390x-linux-gnu-glibc-headers.md) |
| s390x-linux-gnu-gcc-stage2 | 16.2.0 | ビルド専用（Bootstrap） | [s390x-linux-gnu-gcc-stage2.md](./s390x-linux-gnu-gcc-stage2.md) |
| s390x-linux-gnu-glibc | 2.44 | パッケージ（Bootstrap） | [s390x-linux-gnu-glibc.md](./s390x-linux-gnu-glibc.md) |
| s390x-linux-gnu-gcc | 16.2.0 | パッケージ（Bootstrap） | [s390x-linux-gnu-gcc.md](./s390x-linux-gnu-gcc.md) |
| gmp | 6.3.0 | パッケージ | [gmp.md](./gmp.md) |
| mpfr | 4.2.2 | パッケージ | [mpfr.md](./mpfr.md) |
| mpc | 1.4.1 | パッケージ | [mpc.md](./mpc.md) |
| isl | 0.26 | パッケージ | [isl.md](./isl.md) |
| zstd | 1.5.7 | パッケージ | [zstd.md](./zstd.md) |
| mold | 2.42.1 | パッケージ（Bootstrap） | [mold.md](./mold.md) |
| libffi | 3.8.0 | パッケージ | [libffi.md](./libffi.md) |
| pcre2 | 10.48 | パッケージ | [pcre2.md](./pcre2.md) |
| ncurses | 6.6 | パッケージ | [ncurses.md](./ncurses.md) |
| readline | 8.3 | パッケージ | [readline.md](./readline.md) |
| expat | 2.8.4 | パッケージ | [expat.md](./expat.md) |
| sqlite | 3.53.4 | パッケージ | [sqlite.md](./sqlite.md) |
| libxcrypt | 4.5.2 | パッケージ | [libxcrypt.md](./libxcrypt.md) |
| glib | 2.90.0 | パッケージ | [glib.md](./glib.md) |
| bash | 5.3 | パッケージ（Bootstrap） | [bash.md](./bash.md) |
| coreutils | 9.12 | パッケージ（Bootstrap） | [coreutils.md](./coreutils.md) |
| xz | 5.8.4 | パッケージ（Bootstrap） | [xz.md](./xz.md) |
| bzip2 | 1.0.8 | パッケージ | [bzip2.md](./bzip2.md) |
| flex | 2.6.4 | パッケージ | [flex.md](./flex.md) |
| perl | 5.42.3 | パッケージ | [perl.md](./perl.md) |
| pkgconf | 2.5.1 | パッケージ | [pkgconf.md](./pkgconf.md) |
| ninja | 1.13.2 | パッケージ | [ninja.md](./ninja.md) |
| meson | 1.12.0 | パッケージ | [meson.md](./meson.md) |
| python | 3.14.7 | パッケージ | [python.md](./python.md) |
| cmake | 4.4.3 | パッケージ（Bootstrap） | [cmake.md](./cmake.md) |
| make | 4.4.1 | パッケージ（Bootstrap） | [make.md](./make.md) |
| m4 | 1.4.21 | パッケージ（Bootstrap） | [m4.md](./m4.md) |
| bison | 3.8.2 | パッケージ | [bison.md](./bison.md) |
| findutils | 4.11.0 | パッケージ（Bootstrap） | [findutils.md](./findutils.md) |
| sed | 4.10 | パッケージ（Bootstrap） | [sed.md](./sed.md) |
| grep | 3.12 | パッケージ（Bootstrap） | [grep.md](./grep.md) |
| gawk | 5.4.1 | パッケージ（Bootstrap） | [gawk.md](./gawk.md) |
| diffutils | 3.12 | パッケージ（Bootstrap） | [diffutils.md](./diffutils.md) |
| patch | 2.8 | パッケージ（Bootstrap） | [patch.md](./patch.md) |
| tar | 1.35 | パッケージ（Bootstrap） | [tar.md](./tar.md) |
| gzip | 1.14 | パッケージ（Bootstrap） | [gzip.md](./gzip.md) |
| texinfo | 7.3 | パッケージ | [texinfo.md](./texinfo.md) |
| qemu | 11.1.1 | パッケージ | [qemu.md](./qemu.md) |
| build-env | 1 | build-env（合成ルート） | [build-env.md](./build-env.md) |
<!-- /GENERATED -->
