# Host tools パッケージ

ビルド環境向けパッケージの共通方針です。段階とコマンドは [build-flow.md](./build-flow.md) です。個別のソースと固有フラグは [definition/{name}.md](./definition/) です。実装は [`builder/src/package/definition/{name}.rs`](../../builder/src/package/definition/) です。

この索引には次の三つが混在します。gcc が引くライブラリ（gmp、mpfr など）、PATH に載せるコマンド（make、bash など）、ビルド時に HOST 成果物をマウントして実行する処理系（python、cmake、meson、ninja、perl）です。qemu は対象が amd64 / v4 だけなのでこの表に入れず、[qemu.md](./definition/qemu.md) にあります。

## 対象 Platform

qemu 以外の Host tools の `supports` は `Target::all()` です。GNU Autotools は `autotools_configure` により `--build=$HOST_TRIPLE --host=$PACKAGE_HOST` を付けます。cmake / python / meson / ninja / mold も同じ集合向けに組みます。

cmake は mold の build-dep として seed 上で先に組み、python は `build-env` 合成後に HOST で組み、Phase 4 で `Target::all()` 向け成果物も出します。perl も HOST で先に組み、続けて各 Platform 向け成果物を出します。HOST 以外の perl は HOST の perl を `-Dhostperl` に渡し、入れ子の host `miniperl` は組みません。meson は自前 python に実行時依存します。最初の `build-env` 合成には python / cmake / qemu を入れません。他パッケージのビルドが実行する python / cmake / meson / ninja / perl は HOST 成果物をマウントします。

コンパイラ接頭辞はパッケージ Platform の GNU triple です。`gcc` や `cc` という名前の実行ファイルは置きません。実行時 libc は、そのパッケージ Platform の `{triple}-glibc` です。

ビルド環境イメージの合成、および `check` による動的リンク検証には、実行時依存（`Package::dependencies(target)`）を使用します。`Role::Toolchain` のシード閉包だけが `Bootstrap::seed_dependencies(target)` を使います。`BOOTSTRAP` 配列に載る Host tools は POSIX PATH と cmake、mold です。gmp などは配列に無く、グラフ経由で入ります。

シード中のリンクは Arch の mold（`-fuse-ld=mold`）です。compose 後は自前 mold を PATH に載せます。builder が `-fuse-ld=mold` を剥がすのは、合成後かつ自前 mold 成果物が無いときに限ります。

## コンパイラフラグと前処理

コンパイラへ渡す既定のフラグは `tool_flags`（`-O2`、CPU プラットフォーム指定、バイナリ強化フラグ、`-flto`、および LTO 品質保証用の `-Werror=*`）です。トークンの展開は [`builder/src/flags.rs`](../../builder/src/flags.rs) です。

GNU Autotools（configure）を使用するツールの多くでは、フラグから `-flto` および `-Werror=*` を除外し、`-std=gnu17` を追加します（flex は `gnu11`）。除外は host 側の `Flag::sanitized()`（`Recipe::Steps` の `Env::Autotools` のみに適用）、`-std` 追加は recipe プリリュードの `Steps::std` で行います。シード環境の GCC 16 では、この組み合わせにおいて GMP のコンパイラ検査（compiler probe）および gnulib の検出処理が失敗するためです。保守的な措置として同様の前処理を configure 系の Host tools 全体に一括して適用していますが、GMP 以外のすべてのツールで同一の失敗が必ず発生することを確認したわけではありません。

この前処理を適用しないパッケージは、ncurses、bash、および glib です。これらには `tool_flags` に含まれる LTO フラグ（`-flto`）がそのまま維持されます。meson は Python パッケージのインストール処理のみを行うため、C コンパイラの LTO は使用しません。Host tools パッケージには PGO の適用手順を設けていません。

ホスト用 glibc がビルド済みである場合は、`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` を優先して `--sysroot` に渡し、linux-headers の `-idirafter` をコンパイラ引数に追加します。qemu のリンク方針は [qemu.md](./definition/qemu.md) です。

## パッケージ固有設定の一覧

`--prefix=/usr` だけのパッケージは、上の共通前処理に加えそのフラグだけです。詳細は各ファイルのフラグ表です。

| パッケージ | バージョン | ライセンス | 個別設定の概要 |
|-|-|-|-|
| [gmp](./definition/gmp.md) | 6.3.0 | LGPL-3.0-or-later | `ABI` は既定 `64`。`riscv64-*` は `standard`、`powerpc64le-*` / `powerpc64-*` は `mode64` |
| [mpfr](./definition/mpfr.md) | 4.2.2 | LGPL-3.0-or-later | `--prefix=/usr` |
| [mpc](./definition/mpc.md) | 1.4.1 | LGPL-3.0-or-later | `--prefix=/usr` |
| [isl](./definition/isl.md) | 0.26 | MIT | `--prefix=/usr` |
| [zstd](./definition/zstd.md) | 1.5.7 | BSD-3-Clause OR GPL-2.0-only | `make PREFIX=/usr HAVE_ZLIB=0 HAVE_LZMA=0 HAVE_LZ4=0` |
| [libffi](./definition/libffi.md) | 3.8.0 | MIT | `--prefix=/usr` |
| [pcre2](./definition/pcre2.md) | 10.48 | BSD-3-Clause | `--prefix=/usr` |
| [ncurses](./definition/ncurses.md) | 6.6 | MIT-open-group | widec および pc-files。C++ バインディングなし。LTO を維持 |
| [readline](./definition/readline.md) | 8.3 | GPL-3.0-or-later | `--with-curses`、`SHLIB_LIBS=-lncursesw` |
| [expat](./definition/expat.md) | 2.8.4 | MIT | `--prefix=/usr` |
| [sqlite](./definition/sqlite.md) | 3.53.4 | blessing | `--prefix=/usr` |
| [libxcrypt](./definition/libxcrypt.md) | 4.5.2 | LGPL-2.1-or-later | `--disable-werror` |
| [glib](./definition/glib.md) | 2.90.0 | LGPL-2.1-or-later | meson。`-Dsysprof=disabled`。LTO を維持 |
| [bash](./definition/bash.md) | 5.3 | GPL-3.0-or-later | `--without-bash-malloc`。`sh` を bash へリンク。LTO を維持 |
| [coreutils](./definition/coreutils.md) | 9.12 | GPL-3.0-or-later | `--prefix=/usr` |
| [findutils](./definition/findutils.md) | 4.11.0 | GPL-3.0-or-later | `--prefix=/usr` |
| [sed](./definition/sed.md) | 4.10 | GPL-3.0-or-later | `--prefix=/usr` |
| [grep](./definition/grep.md) | 3.12 | GPL-3.0-or-later | `--prefix=/usr` |
| [gawk](./definition/gawk.md) | 5.4.1 | GPL-3.0-or-later | `--prefix=/usr` |
| [diffutils](./definition/diffutils.md) | 3.12 | GPL-3.0-or-later | `--prefix=/usr` |
| [patch](./definition/patch.md) | 2.8 | GPL-3.0-or-later | `--prefix=/usr` |
| [tar](./definition/tar.md) | 1.35 | GPL-3.0-or-later | `--disable-acl` |
| [gzip](./definition/gzip.md) | 1.14 | GPL-3.0-or-later | `--prefix=/usr` |
| [xz](./definition/xz.md) | 5.8.4 | 0BSD | `--prefix=/usr` |
| [bzip2](./definition/bzip2.md) | 1.0.8 | bzip2-1.0.6 | 共有ライブラリ用 Makefile |
| [make](./definition/make.md) | 4.4.1 | GPL-3.0-or-later | `--prefix=/usr` |
| [m4](./definition/m4.md) | 1.4.21 | GPL-3.0-or-later | `--prefix=/usr` |
| [bison](./definition/bison.md) | 3.8.2 | GPL-3.0-or-later | `--prefix=/usr` |
| [flex](./definition/flex.md) | 2.6.4 | BSD-2-Clause | `--prefix=/usr` |
| [perl](./definition/perl.md) | 5.42.3 | Artistic-1.0-Perl OR GPL-1.0-or-later | `Configure -des -Dusethreads` |
| [texinfo](./definition/texinfo.md) | 7.3 | GPL-3.0-or-later | `--prefix=/usr` |
| [pkgconf](./definition/pkgconf.md) | 2.5.1 | ISC | `--prefix=/usr` |
| [ninja](./definition/ninja.md) | 1.13.2 | Apache-2.0 | bootstrap 処理により `ninja` バイナリを生成 |
| [meson](./definition/meson.md) | 1.12.0 | Apache-2.0 | `pip install --no-deps` |
| [python](./definition/python.md) | 3.14.7 | Python-2.0 | `--enable-shared`。openssl を実行時依存に含む |
| [cmake](./definition/cmake.md) | 4.4.3 | BSD-3-Clause | `./bootstrap` で C++ のみから組む |
| [mold](./definition/mold.md) | 2.42.1 | MIT | cmake。自分には `-fuse-ld=mold` を付けない |
