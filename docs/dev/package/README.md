# パッケージ

パッケージ定義に共通するビルド方針です。段階ごとのコマンドは [build-flow.md](./build-flow.md)、個別設定は [definition/](./definition/)、Host tools の共通前処理は [host-tools.md](./host-tools.md)、プレフィックス付きツールチェーンは [toolchain.md](./toolchain.md) です。公開対象の分類は [user/published.md](../../user/published.md) です。

`PACKAGES` に載るパッケージは [definition/](./definition/) に `{パッケージ名}.md` を 1 ファイル置きます。索引は [definition/README.md](./definition/README.md) です。実装は [`builder/src/package/definition/`](../../../builder/src/package/definition/) です。ハイフンはモジュール名ではアンダースコア `_` です。プレフィックス付きツールチェーンも `{パッケージ名}.rs` の 1 ファイル 1 定義で、ステージ共通の実装は `binutils.rs` / `gcc.rs` / `gcc_stage1.rs` / `gcc_stage2.rs` / `glibc.rs` / `glibc_headers.rs` / `stages.rs` に置きます。

パッケージイメージには各パッケージ固有のファイルのみを含めます。実行時に必要となる他のパッケージは `dependencies`、ターゲット向け `-I` / `-L` 入力となるビルド時依存は `build-dependencies`、ビルド中に HOST 側で実行するツール（C ツールチェーンを含む）は `host-dependencies` として分離して宣言します。暗黙の依存は許容しません。`check` コマンドは、この実行時依存の宣言と、パッケージ成果物（`packages/artifacts/`）に含まれる ELF バイナリの動的リンク情報を静的に照合します。

## ビルド環境の構成

ビルド環境は特定のディストリビューションそのものではなく、HOST（amd64 / v4）向けパッケージを合成した単一の出力イメージです。その正本は `env compose` によって合成された成果物（`oci-layouts/build-env-amd64`）です。現在のビルド環境の参照先は `containers-builder env status` コマンドで確認できます。合成済みイメージが存在しないブートストラップ初期段階においては、シード環境として [`builder/src/package/env.rs`](../../../builder/src/package/env.rs) でダイジェストを固定した Arch Linux `base-devel` イメージを使用します。

各パッケージのビルドスクリプトには、環境変数として `SOURCE`、`BUILD`、`OUT`、`ARCH`、`VARIANT`、`TRIPLE`、`HOST_TRIPLE`、`PACKAGE_HOST`、`MARCH`、`CROSS`、および `CFLAGS` / `CXXFLAGS` / `LDFLAGS` が渡されます。`CFLAGS` および `LDFLAGS` は、パッケージ定義が返したフラグトークンから builder が合成します。`TRIPLE` はコード生成側の GNU triple（プレフィックス付き gcc ではパッケージ名の triple、Host tools ではパッケージ Platform の triple）です。`PACKAGE_HOST` は成果物が動く Platform の triple、`HOST_TRIPLE` はビルドマシン（常に `x86_64-linux-gnu`）です。`MARCH` は対象 variant の CPU 指定値（`-march=` / `-mcpu=` の引数）です。`CXXFLAGS` には `CFLAGS` と同じ値が入ります。

`CC`、`CXX`、`AR`、`RANLIB`、`NM` などのツールチェーン環境変数は builder が制御します。シード上で HOST 向けに組むあいだは `CC=gcc` です。自前 gcc のあと、および合成イメージ上では、対象 Platform の `CROSS` 接頭辞を付けます。amd64 では `x86_64-linux-gnu-gcc`、arm64 では `aarch64-linux-gnu-gcc` です。

フラグトークンの正本は [`builder/src/flags.rs`](../../../builder/src/flags.rs) です。builder は `CFLAGS` に指定された `-flto` を `LDFLAGS` へ暗黙に複製せず、両者で独立してフラグを管理します。基本最適化レベルには `-O2` を指定し、CPU 最適化オプションは対象 variant から一意に決定します（ppc64le は `-mcpu=`、それ以外は `-march=` を使用し、ホスト環境に依存する `native` は指定しません）。`FormatSecurityError` トークンは `-Wformat` と `-Werror=format-security` の 2 つのコンパイラフラグへ、`FortifySource` トークンは `-U_FORTIFY_SOURCE` と `-D_FORTIFY_SOURCE=3` へ展開されます。PGO の適用方針は `Package::pgo()`（`Pgo::None` / `Pgo::Profiled(ワークロード)` / `Pgo::Custom`）で宣言し、プロファイル採取および適用手順は `recipe(ctx)` が `ctx.cross()` で分岐して静的に組み込みます。`-O3`、`-g`、`-Wall` はカタログ用コンパイラフラグとしては付与しません（`-fexceptions` は既定セットに含みます）。

依存パッケージは `/dep/{パッケージ名}` に読み取り専用でマウントされます。`dependencies` と `build-dependencies` は `-I` / `-L` の対象（`dep:true`）で、`host-dependencies` は PATH・`LD_LIBRARY_PATH`・`PKG_CONFIG_PATH` に載る `dep:false` マウントです（ターゲット向け `-I` / `-L` の入力にはなりません）。同名パッケージが両種別で必要な場合、HOST 側は `/dep/{パッケージ名}-host` へ退避されます。リンカ環境変数 `LD` には gcc を入れません。共有ライブラリのリンクは gcc ドライバと `-fuse-ld=mold` に任せます。libtool が `LD` を GNU ld と誤認しないようにするためです。

- architecture および variant 単位で CPU プラットフォームを指定します。対象プラットフォームの一覧は [targets.md](../../user/targets.md) です。
- LTO および PGO の適用方針はパッケージの種別に応じて分かれます。カタログの共有ライブラリでは各ビルド設定で個別に宣言し、Host tools の共通前処理やプレフィックス付き最終 gcc・qemu の例外設定については各設計文書に記載しています。
- 自前で合成したビルド環境イメージ上では `--network=none` を指定してネットワークを遮断します。シード環境上でのビルド時に限り、pacman による依存解決や、カタログのクロス gcc および cmake 導入のためにネットワークアクセスを許可します。
- シード環境には ppc64le および s390x 向けのクロス gcc が存在しないため、`--all-targets` 指定時はそれらのターゲットをスキップします。自前の合成イメージが存在する場合はスキップせずビルドします。

## フラグ注入経路

いつ・どこで・どのオプションが渡されるかの一覧です。

| 層 | 注入点 | 実装 | 内容 | 適用条件 |
|-|-|-|-|-|
| L1 ツールチェーン既定 | 最終 gcc の specs / configure | `definition/gcc.rs` の `--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'`、`--enable-default-pie`、`--enable-default-ssp` | CFLAGS を無視するビルド系にも `-fhardened` / PIE / SSP を既定適用 | 最終 gcc 経由の全コンパイル。`-fno-hardened` で傘フラグのみ解除（toolchain 既定 PIE/SSP は残存） |
| L2 環境変数 | コンテナ `CFLAGS` / `CXXFLAGS` / `LDFLAGS` | `container.rs` の `linker_env()` が `flags().env()` を渡す | `tool_flags(target)`（後述の既定セット）またはパッケージ固有 `flags()` | 常時。`CXXFLAGS` は `CFLAGS` と同値。mold 成果物が無い時のみ `-fuse-ld=mold` を除去 |
| L3 recipe プリリュード | configure 実行時の一時 env | `recipe.rs` の `autotools_configure` が `sanitize_flags` で `CFLAGS` / `CXXFLAGS` / `LDFLAGS` を一時上書き | `-flto*`・LTO 系 `-Werror={odr,lto-type-mismatch,strict-aliasing}`・`-f{,no-}hardened` を configure のみ除去（`-Werror=format-security` は保持） | `Env::Autotools` のみ。`Env::Lto`（configure が CFLAGS を Makefile へ焼くパッケージ）と `Env::None` では除去しない |
| L3b `-std` 付与 | `CFLAGS` への末尾追加 | `Steps::std`（プリリュードで `export CFLAGS="... -std=gnu17"`、flex は `gnu11`） | gnulib 等の K&R 宣言保護 | `Env::Autotools` のみ |
| L4 make 変数上書き | `make` / `make install` 引数 | `Step::Make` / `MakeInstall` が `CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"` を自動付与 | configure が `@CFLAGS@` で Makefile へ焼いた sanitize 済み値をフルフラグで打ち消す | `Env::Autotools`。ステップが同名変数を明示済み（bzip2 の `CFLAGS=` 等）の場合は付与しない |
| L5 PGO フェーズ | `-fprofile-generate` / `-fprofile-use` の env 切替 | `PgoTemplate` が `_PGO_*` へ退避して export | instrumented / use フェーズのフラグ（後述の PGO フェーズ表） | ネイティブビルドのみ。クロスでは PGO ブロック全体が素の手順へフォールバック |
| L6 パッケージ固有 | スクリプト内 export / 引数 | 各 `definition/*.rs` | openssl / zlib / zlib-ng / mold / toolchain / qemu の独自手順、glibc の CFLAGS 等 | パッケージごと |

## 既定フラグセット（`tool_flags`）

<!-- GENERATED:tool-flags -->

| トークン | 展開 | 種別 | 対象 |
|-|-|-|-|
| `O2` | `-O2` | Perf | 常時 |
| `Pipe` | `-pipe` | Perf | 常時 |
| `March(target)` | `-march=<variant>`（ppc64le は `-mcpu=`） | Perf | 常時 |
| `TreeVectorize` | `-ftree-vectorize` | Perf | 常時 |
| `NoSemanticInterposition` | `-fno-semantic-interposition` | Perf | 常時 |
| `Hardened` | `-fhardened` | Harden | 常時 |
| `NoWhardened` | `-Wno-hardened` | Harden | 常時 |
| `StackProtectorStrong` | `-fstack-protector-strong` | Harden | 常時 |
| `FormatSecurityError` | `-Wformat` `-Werror=format-security` | Harden | 常時 |
| `StrictFlexArrays3` | `-fstrict-flex-arrays=3` | Harden | 常時 |
| `ZeroInitPaddingBits` | `-fzero-init-padding-bits=all` | Harden | 常時 |
| `NoDeleteNullPointer` | `-fno-delete-null-pointer-checks` | Harden | 常時 |
| `NoStrictOverflow` | `-fno-strict-overflow` | Harden | 常時 |
| `NoStrictAliasing` | `-fno-strict-aliasing` | Harden | 常時 |
| `Exceptions` | `-fexceptions` | Harden | 常時 |
| `Wtrampolines` | `-Wtrampolines` | Harden | 常時 |
| `Lto` | `-flto=auto` | Perf | 常時 |
| `WerrorOdr` | `-Werror=odr` | Perf | 常時 |
| `WerrorLtoTypeMismatch` | `-Werror=lto-type-mismatch` | Perf | 常時 |
| `WerrorStrictAliasing` | `-Werror=strict-aliasing` | Perf | 常時 |
| `NoPlt` | `-fno-plt` | Perf | amd64, arm64 |
| `BranchProtection` | `-mbranch-protection=standard` | Harden | arm64 |
| `ZeroCallUsedRegs` | `-fzero-call-used-regs=used-gpr` | Harden | amd64, arm64 |
| `TlsDialectGnu2` | `-mtls-dialect=gnu2` | Perf | amd64 |

| トークン（Ldflag） | 展開 | 対象 |
|-|-|-|
| `FuseLdMold` | `-fuse-ld=mold` | 常時 |
| `O1` | `-Wl,-O1` | 常時 |
| `AsNeeded` | `-Wl,--as-needed` | 常時 |
| `NoCopyDtNeeded` | `-Wl,--no-copy-dt-needed-entries` | 常時 |
| `Relro` | `-Wl,-z,relro` | 常時 |
| `Now` | `-Wl,-z,now` | 常時 |
| `NoExecStack` | `-Wl,-z,noexecstack` | 常時 |
| `Lto` | `-flto=auto` | 常時 |
| `WerrorOdr` | `-Werror=odr` | 常時 |
| `WerrorLtoTypeMismatch` | `-Werror=lto-type-mismatch` | 常時 |
| `WerrorStrictAliasing` | `-Werror=strict-aliasing` | 常時 |
| `PackRelativeRelocs` | `-Wl,-z,pack-relative-relocs` | amd64, arm64, ppc64le |

既定セットに含まれない opt-in トークン:

| トークン | 変数 | 展開 |
|-|-|-|
| `NoHardened` | CFLAGS | `-fno-hardened` |
| `FortifySource` | CFLAGS | `-U_FORTIFY_SOURCE` `-D_FORTIFY_SOURCE=3` |
| `StackClashProtection` | CFLAGS | `-fstack-clash-protection` |
| `CfProtection` | CFLAGS | `-fcf-protection=full` |
| `TrivialAutoVarInit` | CFLAGS | `-ftrivial-auto-var-init=zero` |
| `GlibcxxAssertions` | CFLAGS | `-D_GLIBCXX_ASSERTIONS` |
| `CetReportError` | LDFLAGS | `-Wl,-z,cet-report=error` |
| `NoHardened` | LDFLAGS | `-fno-hardened` |
| `NoDlopen` | LDFLAGS | `-Wl,-z,nodlopen` |
| `Zdefs` | LDFLAGS | `-Wl,-z,defs` |
| `BsymbolicFunctions` | LDFLAGS | `-Wl,-Bsymbolic-functions` |

<!-- /GENERATED:tool-flags -->

`FortifySource`（`-U_FORTIFY_SOURCE -D_FORTIFY_SOURCE=3`）は `-fhardened` と二重定義警告になるため既定セットには含めず、`NoHardened` を選ぶパッケージの明示トークンとして残します。

## PGO フェーズ別フラグ遷移

`PGODIR=/tmp/pgo`（絶対パス）に `.gcda` を集めます。`PgoTemplate::Autotools`（configure あり）は ①configure → ②instrumented build → ③workload → ④`make distclean || make clean` → ⑤再 configure → ⑥use build → ⑦install。`PgoTemplate::Make`（configure 無しの in-source make 系）は ②→③→`make clean || make distclean`→⑥→⑦です。

| フェーズ | 実行内容 | CFLAGS / CXXFLAGS | LDFLAGS | 備考 |
|-|-|-|-|
| ① configure | `autotools_configure` | sanitized（L3）+ ベース値（`Env::Lto` では `-fprofile-generate` も注入） | 同左 | `Env::Autotools` は configure が焼く値を L4 で打ち消すため make 時に注入 |
| ② instrumented build | `make -j`（pkg 固有 args を replay） | ベース + `-fprofile-generate=$PGODIR -fprofile-update=atomic` | ベース + `-fprofile-generate` | ネイティブのみ |
| ③ workload | パッケージ宣言のシェル片を `( ... ) || true` で実行 | — | — | 実行前に `$PWD`・`$PWD/lib`・`$PWD/programs`・`*/.libs` を `LD_LIBRARY_PATH` へ auto-export |
| ⑤ use build | `make -j`（同 args replay） | ベース + `-fprofile-use=$PGODIR -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch` | ベース + `-fprofile-use` | ネイティブのみ |

## PGO 対象パッケージ

<!-- GENERATED:pgo -->

| `Pgo` | パッケージ |
|-|-|
| `Profiled（A（autotools））` | gmp / mpfr / mpc / isl / libffi / pcre2 / readline / expat / sqlite / libxcrypt / bash / coreutils / xz / flex / pkgconf / make / m4 / bison / findutils / sed / grep / gawk / diffutils / patch / tar / gzip / texinfo |
| `Profiled（B（in-source make））` | zstd / bzip2 |
| `Custom` | openssl / zlib / zlib-ng / mold / ninja / python / {triple}-binutils / {triple}-gcc |
| `None` | ca-certificates / linux-headers / ncurses / glib / perl / meson / cmake / qemu / build-env / {triple}-gcc-stage1 / {triple}-glibc-headers / {triple}-gcc-stage2 / {triple}-glibc |

<!-- /GENERATED:pgo -->

種別ごとの例外は、glibc と最終 gcc は [toolchain.md](./toolchain.md)、Host tools は [host-tools.md](./host-tools.md)、カタログと qemu は各 `{パッケージ名}.md` です。

## ブートストラップと定義の管理

ブートストラップ処理は [`Bootstrap`](../../../builder/src/package/mod.rs) トレイトとして分離して定義し、判定は `Package::as_bootstrap()` で行います（登録一覧は持ちません）。`Bootstrap` を実装したパッケージは、合成済みビルド環境イメージが存在しない初期段階において、シード環境（Arch Linux）と pacman を用いてビルドします。それ以外のパッケージは、合成済みビルド環境が存在する場合はそれをベースとし、未合成の場合はシード環境へフォールバックしてビルドします。

ビルド手順は `recipe(&self, ctx)` メソッドが返す型付きの [`Recipe`](../../../builder/src/package/recipe.rs)（`Step::Configure` / `Make` / `MakeInstall` / `Install` / `Symlink` 等）として記述します。定型の autotools パッケージは `Recipe::autotools(&[...])` で configure 引数のみを列挙します。`ctx`（`BuildContext`）は `target`・`$TRIPLE`・`$HOST_TRIPLE`・クロス判定を保持するため、クロス限定の configure 引数は Rust 側の条件分岐で静的に解決します。全ての手順を型付き `Step` として記述します（生のシェル文字列への逃げ道はありません）。configure、make、`DESTDIR` を用いたインストール手順のみを記述し、pacman コマンドなどのホスト環境を変更する処理は含めません。

上流ソースコードのアーカイブ検証には SHA3-512 および BLAKE3（出力 512 bit）の双方を使用します。生成される OCI Image Layout 内の blob ダイジェストおよび DiffID については、OCI 仕様に従い SHA-256 のまま算出します。
