# Host tools パッケージ

ビルド環境向けパッケージの共通方針です。段階とコマンドは [build-flow.md](./build-flow.md) です。個別のソースと固有フラグは [definition/{name}.md](./definition/) です。実装は [`builder/src/package/definition/{name}.rs`](../../../builder/src/package/definition/) です。

この索引には次の三つが混在します。gcc が引くライブラリ（gmp、mpfr など）、PATH に載せるコマンド（make、bash など）、ビルド時に HOST 成果物をマウントして実行する処理系（python、cmake、meson、ninja、perl）です。qemu は対象が amd64 / v4 だけなのでこの表に入れず、[qemu.md](./definition/qemu.md) にあります。

## 対象 Platform

qemu 以外の Host tools の `supports` は `Target::all()` です。GNU Autotools は `autotools_configure` により `--build=$HOST_TRIPLE --host=$PACKAGE_HOST` を付けます。cmake / python / meson / ninja / mold も同じ集合向けに組みます。

cmake は mold のホスト依存として seed 上で先に組み、python は `build-env` 合成後に HOST で組み、Phase 4 で `Target::all()` 向け成果物も出します。perl も HOST で先に組み、続けて各 Platform 向け成果物を出します。HOST 以外の perl は HOST の perl を `-Dhostperl` に渡し、入れ子の host `miniperl` は組みません。meson は自前 python に実行時依存します。最初の `build-env` 合成には python / cmake / qemu を入れません。他パッケージのビルドが実行する python / cmake / meson / ninja / perl は `host_dependencies` 辺として宣言され、HOST 成果物が `dep:false` でマウントされます。

コンパイラ接頭辞はパッケージ Platform の GNU triple です。`gcc` や `cc` という名前の実行ファイルは置きません。実行時 libc は、そのパッケージ Platform の `{triple}-glibc` です。

ビルド環境イメージの合成、および `check` による動的リンク検証には、実行時依存（`Package::dependencies(target)`）を使用します。ビルド中に実行するツール（python / cmake / meson / ninja / perl / pkgconf、および `{triple}-gcc` / `{triple}-binutils` の C ツールチェーン）は `Package::host_dependencies(target)` で宣言し、`dep:false`（PATH には載るがターゲット向け `-I` / `-L` には載らない）でマウントします。依存辺の `Via` が充足方法を表します。`Via::Ordered` 辺のみがビルド順を決め、循環を断つ辺は `Via::Seed`（シード相は distro パッケージが担保、env では成果物必須）・`Via::Artifact`（常に成果物必須）・`Via::Optional`（任意）で明示します。`{triple}-binutils` の glibc / zlib / zstd への循環は `Via::Seed` 辺で断ちます。シード上でビルドする Host tools は POSIX PATH と cmake、mold です。gmp などは依存グラフ経由で入ります。

シード中のリンクは Arch の mold（`-fuse-ld=mold`）です。compose 後は自前 mold を PATH に載せます。builder が `-fuse-ld=mold` を剥がすのは、合成後かつ自前 mold 成果物が無いときに限ります。

## コンパイラフラグと前処理

コンパイラへ渡す既定のフラグは `tool_flags`（`-O2`、CPU プラットフォーム指定、バイナリ強化フラグ（`-fhardened` を含む）、`-flto=auto`、および LTO 品質保証用の `-Werror=*`）です。トークンの展開は [`builder/src/flags.rs`](../../../builder/src/flags.rs) です。

GNU Autotools（configure）を使用するツールでは、configure 実行時のみフラグから `-flto*`・LTO 系 `-Werror=*`・`-fhardened` 系を除外し、`-std=gnu17` を追加します（flex は `gnu11`）。除外は `autotools_configure` が `sanitize_flags` で `CFLAGS` / `CXXFLAGS` / `LDFLAGS` を一時上書きする方式（`Env::Autotools` のみに適用）、`-std` 追加は recipe プリリュードの `Steps::std` で行います。シード環境の GCC 16 では、この組み合わせにおいて GMP のコンパイラ検査（compiler probe）および gnulib の検出処理が失敗するためです。configure が Makefile へ焼いた sanitized 値は、`Step::Make` / `MakeInstall` が付与する `CFLAGS="$CFLAGS"` 等の make 変数上書きでフルフラグへ戻ります。configure が CFLAGS を Makefile へ焼くパッケージ（gmp 等）は `Env::Lto` で sanitize を回避し、PGO 時は `-fprofile-generate` を configure 時の CFLAGS へ注入します。

全パッケージの env プリリュード分類は次のとおりです（`BUILD_ENV` で評価した `Steps::env`）。

<!-- GENERATED:env -->

- `Env::Autotools`: mpfr / mpc / isl / libffi / pcre2 / readline / expat / sqlite / libxcrypt / coreutils / xz / bzip2 / flex / perl / pkgconf / ninja / python / cmake / make / m4 / bison / findutils / sed / grep / gawk / diffutils / patch / tar / gzip / texinfo
- `Env::Lto`: gmp / ncurses / glib / bash
- `Env::None`: ca-certificates / linux-headers / openssl / zlib / zlib-ng / zstd / mold / meson / qemu / {triple}-binutils / {triple}-gcc-stage1 / {triple}-glibc-headers / {triple}-gcc-stage2 / {triple}-glibc / {triple}-gcc
- `Recipe::Noop`: build-env

<!-- /GENERATED:env -->

perl は `Env::Autotools` ですが `Step::Configure` ではなく `Step::ConfigureScript` で `./Configure` を直接起動するため、レシピ冒頭で `sanitize_flags` を `CFLAGS` / `CXXFLAGS` / `LDFLAGS` に明示 export しています。`Env::Lto` では configure がフルフラグを Makefile へ焼くため、make 時の変数上書きなしに LTO とバイナリ強化が維持されます。meson は Python パッケージの配置処理のみを行うため、C コンパイラの LTO は使用しません。

Host tools の多くにはネイティブ実行可能な PGO を適用します。`Pgo::Profiled` は各パッケージのワークロード（`make check` またはビルドしたツール自身でのデータ処理）、`Pgo::Custom` は openssl / ninja / zlib / zlib-ng / mold / python の個別手順です。適用対象とワークロードは [README.md](./README.md) の PGO 対象パッケージ表を参照してください。ncurses は `tic` が libncurses の消費者経路を駆動しないため `Pgo::None` です。

ホスト用 glibc がビルド済みである場合は、`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` を優先して `--sysroot` に渡し、linux-headers の `-idirafter` をコンパイラ引数に追加します。qemu のリンク方針は [qemu.md](./definition/qemu.md) です。

## パッケージ固有設定の一覧

`--prefix=/usr` だけのパッケージは、上の共通前処理に加えそのフラグだけです。個別の configure 引数・ビルドフラグ・手順は各定義文書の生成表を参照してください。

<!-- GENERATED:packages -->

| パッケージ | バージョン | ライセンス | PGO | env |
|-|-|-|-|-|
| [gmp](./definition/gmp.md) | 6.3.0 | LGPL-3.0-or-later | Profiled | Lto |
| [mpfr](./definition/mpfr.md) | 4.2.2 | LGPL-3.0-or-later | Profiled | Autotools |
| [mpc](./definition/mpc.md) | 1.4.1 | LGPL-3.0-or-later | Profiled | Autotools |
| [isl](./definition/isl.md) | 0.26 | MIT | Profiled | Autotools |
| [zstd](./definition/zstd.md) | 1.5.7 | BSD-3-Clause OR GPL-2.0-only | Profiled | None |
| [mold](./definition/mold.md) | 2.42.1 | MIT | Custom | None |
| [libffi](./definition/libffi.md) | 3.8.0 | MIT | Profiled | Autotools |
| [pcre2](./definition/pcre2.md) | 10.48 | BSD-3-Clause | Profiled | Autotools |
| [ncurses](./definition/ncurses.md) | 6.6 | MIT-open-group | None | Lto |
| [readline](./definition/readline.md) | 8.3 | GPL-3.0-or-later | Profiled | Autotools |
| [expat](./definition/expat.md) | 2.8.4 | MIT | Profiled | Autotools |
| [sqlite](./definition/sqlite.md) | 3.53.4 | blessing | Profiled | Autotools |
| [libxcrypt](./definition/libxcrypt.md) | 4.5.2 | LGPL-2.1-or-later | Profiled | Autotools |
| [glib](./definition/glib.md) | 2.90.0 | LGPL-2.1-or-later | None | Lto |
| [bash](./definition/bash.md) | 5.3 | GPL-3.0-or-later | Profiled | Lto |
| [coreutils](./definition/coreutils.md) | 9.12 | GPL-3.0-or-later | Profiled | Autotools |
| [xz](./definition/xz.md) | 5.8.4 | 0BSD | Profiled | Autotools |
| [bzip2](./definition/bzip2.md) | 1.0.8 | bzip2-1.0.6 | Profiled | Autotools |
| [flex](./definition/flex.md) | 2.6.4 | BSD-2-Clause | Profiled | Autotools |
| [perl](./definition/perl.md) | 5.42.3 | Artistic-1.0-Perl OR GPL-1.0-or-later | None | Autotools |
| [pkgconf](./definition/pkgconf.md) | 2.5.1 | ISC | Profiled | Autotools |
| [ninja](./definition/ninja.md) | 1.13.2 | Apache-2.0 | Custom | Autotools |
| [meson](./definition/meson.md) | 1.12.0 | Apache-2.0 | None | None |
| [python](./definition/python.md) | 3.14.7 | Python-2.0 | Custom | Autotools |
| [cmake](./definition/cmake.md) | 4.4.3 | BSD-3-Clause | None | Autotools |
| [make](./definition/make.md) | 4.4.1 | GPL-3.0-or-later | Profiled | Autotools |
| [m4](./definition/m4.md) | 1.4.21 | GPL-3.0-or-later | Profiled | Autotools |
| [bison](./definition/bison.md) | 3.8.2 | GPL-3.0-or-later | Profiled | Autotools |
| [findutils](./definition/findutils.md) | 4.11.0 | GPL-3.0-or-later | Profiled | Autotools |
| [sed](./definition/sed.md) | 4.10 | GPL-3.0-or-later | Profiled | Autotools |
| [grep](./definition/grep.md) | 3.12 | GPL-3.0-or-later | Profiled | Autotools |
| [gawk](./definition/gawk.md) | 5.4.1 | GPL-3.0-or-later | Profiled | Autotools |
| [diffutils](./definition/diffutils.md) | 3.12 | GPL-3.0-or-later | Profiled | Autotools |
| [patch](./definition/patch.md) | 2.8 | GPL-3.0-or-later | Profiled | Autotools |
| [tar](./definition/tar.md) | 1.35 | GPL-3.0-or-later | Profiled | Autotools |
| [gzip](./definition/gzip.md) | 1.14 | GPL-3.0-or-later | Profiled | Autotools |
| [texinfo](./definition/texinfo.md) | 7.3 | GPL-3.0-or-later | Profiled | Autotools |

<!-- /GENERATED:packages -->
