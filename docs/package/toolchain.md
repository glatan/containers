# プレフィックス付きツールチェーン

クロス開発およびブートストラップに使用するプレフィックス付きツールチェーンのビルド方針です。全体の段階は [build-flow.md](./build-flow.md) です。パッケージごとの依存と triple 固有の configure は [definition/](./definition/) の `{パッケージ名}.md` です。1 パッケージ 1 ファイルです。

5 つの GNU triple それぞれについて、binutils / gcc-stage1 / glibc-headers / gcc-stage2 / glibc / 最終 gcc の 6 段階で組みます（linux-headers は別パッケージ）。Phase 1 の `env bootstrap` が組むのは x86_64 だけです。他 triple は HOST 上で、基底 variant の `{triple}-glibc` のあとに `{triple}-binutils` / `{triple}-gcc` を組みます。これが他 arch パッケージを HOST でクロスコンパイルする実行コンパイラです。依存としてマウントする gcc / binutils は常に HOST（amd64 / v4）の成果物です。

## 製品キューと `supports`

`build --all-targets` は `supports` が真の Platform を対象にします。運用で組む製品キューはそれより狭いことがあります。ビルド設定（configure の `--build` / `--host` / `--target`、Canadian 経路）は残します。

| パッケージ | `supports` | 製品キュー |
|-|-|-|
| `{triple}-gcc` / `{triple}-binutils` | `Target::all()` | HOST（当該 triple）と、パッケージ名の GNU triple が Platform の architecture と一致する variant |
| `{triple}-glibc` | その architecture の全 variant | `supports` と同じ |
| `{triple}-gcc-stage1` / `*-glibc-headers` / `*-gcc-stage2` | HOST のみ | `supports` と同じ |

製品キューの例は amd64 向け `x86_64-linux-gnu-gcc`、arm64 向け `aarch64-linux-gnu-gcc` です。Canadian cross（`x86_64-linux-gnu-gcc` を arm64 / v8 向けに組む、など）は製品キューに入れません。

`PACKAGE_HOST` が HOST triple と異なるときは PGO を掛けず、HOST 上の `{triple}-gcc` を `CC_FOR_TARGET` にしてターゲットライブラリを組みます。

## 上流ソース

ハッシュは各 `{パッケージ名}.md` の生成ブロックにも現れます（正本は各定義の `Source` で、生成ブロックは `containers-builder docs` が再生成します）。

| 上流 | バージョン | URL | 展開ディレクトリ |
|-|-|-|-|
| binutils | 2.47 | https://ftp.gnu.org/gnu/binutils/binutils-2.47.tar.xz | `binutils-2.47` |
| gcc | 16.2.0 | https://ftp.gnu.org/gnu/gcc/gcc-16.2.0/gcc-16.2.0.tar.xz | `gcc-16.2.0` |
| glibc | 2.44 | https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz | `glibc-2.44` |

binutils sha3-512: `b2a39d2a1bfff7f27c2619e15c5d9dbf0c7918a85120975f4146c84b4a7d660bd5dc3793ea6d009df89ed865aaef4362805e949438db0f8acf97fd113ca1a1f3`  
binutils blake3: `05a142fb2225e617a3fa30224d5d275682e5cf03a62a15af804c03214cc3a5cfadec1215345da15c94ba7a2e13b9667c094e59a26c39412a47265724cbb62a8e`

gcc sha3-512: `1995b2fd6c2b19fe8ca47c836fdee8c019909ab8857f81a8045a7726ce724146ab63cd8f836fb460e9b009434ea97eb51b34397760b9ec4bbcac490ca32fa0f6`  
gcc blake3: `c0f53af0bbfd2bf28896ab9bdfd1a534351b851c5a2f0226feb6a4323884c43531cbb207c733e8fd8ac16e2e1b03e6d8f17d043a17cf026857eedf26c31e4e0e`

glibc sha3-512: `263dfb5e331b2033368c7455abcd9a9b15ab098ac963d5518ecf4bcb9eeb41298eb9976bf78a64af67d9471a50161bced2fa69d429c69759ea57f76b942ac521`  
glibc blake3: `6d07bc16d8d066b1c0ae713d762175f261e27df9cab8015ec11e1c980c05e79b0fd5bab6cde6a90bf07651c432ca9039629376b57a96c9f3dcb3765f3745dbf4`

## ビルドステージの構成

```
*-binutils
  → linux-headers
  → *-gcc-stage1     (C 言語のみ、静的 libgcc)
  → *-glibc-headers  (ヘッダ + crt + スタブ libc.so。ビルド専用)
  → *-gcc-stage2     (C 言語のみ、共有 libgcc_s。ビルド専用)
  → *-glibc          (完全な sysroot)
  → *-gcc            (C/C++、PGO 適用、`--disable-bootstrap`)
```

対象とする `{triple}` は、`x86_64-linux-gnu`、`aarch64-linux-gnu`、`riscv64-linux-gnu`、`powerpc64le-linux-gnu`、および `s390x-linux-gnu` の 5 種類です。段階ごとのコマンドは [build-flow.md](./build-flow.md) です。プレフィックス付きパッケージも他パッケージと同じ `Package` 宣言グラフに載り、ツールチェーンの循環（binutils ↔ glibc、glibc ↔ gcc 等）は `Via::Seed` / `Via::Artifact` の切断辺で明示します。

| パッケージ | バージョン | パッケージイメージ化 |
|-|-|-|
| `{triple}-binutils` | 2.47 | 対象 |
| `{triple}-gcc-stage1` | 16.2.0 | 対象外（ビルド専用中間ステージ） |
| `{triple}-glibc-headers` | 2.44 | 対象外（ビルド専用中間ステージ） |
| `{triple}-gcc-stage2` | 16.2.0 | 対象外（ビルド専用中間ステージ） |
| `{triple}-glibc` | 2.44 | 対象 |
| `{triple}-gcc` | 16.2.0 | 対象 |

中間コンパイラ（stage1 および stage2）の配置先プレフィックスは `/usr/lib/cross/{triple}/stage{1,2}/` です。最終 gcc は `/usr/bin/{triple}-gcc` として配置し、configure 時に `--with-sysroot=/usr/{triple}` を指定します。また、`x86_64-linux-gnu-glibc` に限り、`/lib64` および `ld-linux-x86-64.so.2` の互換パスにもファイルを配置します。

プレフィックス付きの `{triple}-glibc` が実行時 libc です。最終 gcc は libc を成果物に焼き付けません。リンクは `/dep/{triple}-glibc` です。configure は `--build=$HOST_TRIPLE --host=$HOST_TRIPLE --target=$TRIPLE` です（ツールはビルド機で動くクロスツールのため `--host` は `$HOST_TRIPLE` です。`$PACKAGE_HOST` を渡すと host == target になり誤りです）。info は `--infodir=/usr/share/info/{triple}` に分け、5 本の HOST gcc が衝突しないようにします。

`{triple}-binutils` の configure は `--enable-plugins`、`--with-system-zlib`、`--without-debuginfod`、`--disable-gprofng` を指定します。gold は組みません。`libstdc++.so.6` / `libgcc_s.so.1` への実行時依存（gcc との循環）を避けるためです。リンカは mold に一本化しています。

| 対象 | configure オプション | 効果 |
|-|-|-|
| binutils（全 triple） | `--enable-new-dtags` `--enable-relro` `--enable-default-execstack=no` `--enable-default-hash-style=gnu` | RUNPATH 化・RELRO 既定・`-z noexecstack` を ld 既定化・生 `ld` 呼び出しも gnu hash に |
| binutils（`x86_64-linux-gnu`） | `--enable-cet` | ld を CET 対応に |
| gcc-stage1（全 triple） | `--enable-default-pie` `--enable-default-ssp` | 生成コンパイラの既定（freestanding でも無害） |
| gcc-stage2（全 triple） | stage1 と同じ + `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` | stage2 が libgcc_s.so をリンクするため hash-style 等が必要 |
| glibc（全 triple） | `--enable-stack-protector=strong` `--enable-bind-now` `--enable-fortify-source` | libc 自身の SSP / eager binding / fortify（LEVEL はコンパイラ最大 = 3） |
| glibc（`x86_64-linux-gnu`） | `--enable-cet`（strict） | libc の CET。全 ELF 出力の `readelf -n` CET マーク audit が前提（非マーク DSO の dlopen は strict ではエラー） |

x86_64 のネイティブ binutils（`$TRIPLE = x86_64-linux-gnu` かつ `$PACKAGE_HOST = $HOST_TRIPLE`）には PGO を適用します。instrumented ツリーで `as` / `ld` / `objdump` を生成アセンブリの assemble・リンク・逆アセンブルで駆動してプロファイルを採取し、別ツリーで `-fprofile-use` の成果物を組みます。クロス binutils には適用しません。

## linux-headers の取り込み

stage1、glibc-headers、stage2、glibc、および最終 gcc の各ステージでは、sysroot の `include/` ディレクトリへ [linux-headers](./definition/linux-headers.md) のカーネルユーザー空間ヘッダをコピーします。まず優先して `/dep/linux-headers/usr/${TRIPLE}/include` を参照し、該当ディレクトリが存在しない場合に限り共通の `/dep/linux-headers/usr/include` へフォールバックします。

`openssl` や `zlib` の `-idirafter` は、`/dep/linux-headers/usr/${TRIPLE}/include` を優先し、無い場合に `/dep/linux-headers/usr/include` へフォールバックします。

## 最終 gcc の構成

最終 gcc の configure では、instrumented（プロファイル採取用）および final（最終成果物）の双方において、`--disable-bootstrap`、`--disable-multilib`、`--enable-languages=c,c++`、`--enable-lto`、および `--enable-plugin` を指定します。なお、stage1 および stage2 ではすべての triple において `--disable-libsanitizer` を指定します。

| 対象 | configure オプション | 効果 |
|-|-|-|
| 最終 gcc（全 triple） | `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` `--enable-libstdcxx-backtrace` `--enable-host-pie` `--enable-host-bind-now` | 既定 PIE/SSP + リンカー挙動 + cc1/xgcc 等コンパイラ自身のバイナリも PIE/BIND_NOW 化 |
| 最終 gcc（amd64） | `--enable-cet` | CET（`-fcf-protection` と対） |
| 最終 gcc（s390x） | `--enable-decimal-float` | libgcc の DFP ランタイム |
| 最終 gcc（全 triple） | `--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'` | CFLAGS を無視するビルド系へのベースライン適用。`-fno-hardened` で傘フラグのみ解除可能（`NoHardened` トークン）。target ランタイム lib（libgcc/libstdc++ 等）も `-fhardened` でビルドされる（intended） |

`GCC_ARCH_FLAGS`（`--with-arch` / `--with-cpu`）は最終 gcc のみ per-variant にします。libgcc / libstdc++ / libatomic 等のランタイムを variant の ISA で組み、フラグを落としたパッケージの既定も底上げします。stage1 / stage2 / glibc-headers は BUILD_ENV（amd64 / v4）でしか組まないため triple ベースライン（`x86-64` / `armv8-a` / `rv64gc` / `power8` / `z900`）のままです。

`$MARCH` は `PACKAGE_HOST`（成果物が動く Platform）の ISA なので、host arch = codegen arch の場合だけ使用します。amd64 で組む aarch64 gcc 等の異種クロスでは triple ベースラインへフォールバックします。

| triple | 最終 gcc の `GCC_ARCH_FLAGS`（同 arch 時 / 異種クロス時） |
|-|-|
| `x86_64-linux-gnu` | `--with-arch=$MARCH` / `--with-arch=x86-64`、いずれも `--enable-cet` |
| `aarch64-linux-gnu` | `--with-arch=$MARCH` / `--with-arch=armv8-a` |
| `riscv64-linux-gnu` | `--with-arch=$MARCH` / `--with-arch=rv64gc`、いずれも `--with-abi=lp64d` |
| `powerpc64le-linux-gnu` | `--with-cpu=$MARCH` / `--with-cpu=power8` |
| `s390x-linux-gnu` | `--with-arch=$MARCH` / `--with-arch=z900`、いずれも `--enable-decimal-float` |

ppc64le の `--with-long-double-format=ieee` は ABI 変更を伴うため現状採用しません（ターゲット既定の IBM extended のまま）。

最終 gcc においては、`$TRIPLE` が `x86_64-linux-gnu` である場合に限り libsanitizer を有効化し、それ以外の非ホスト向け triple には `--disable-libsanitizer` を付与します。これは riscv64 の PGO ステージにおいて `sanitizer_platform_limits_posix` のビルドが失敗したためです。保守的な措置として他の非ホスト向け triple にも同様に `--disable-libsanitizer` を付与していますが、aarch64、powerpc64le、および s390x において同一の失敗が必ず発生することを確認したわけではありません。

PGO の適用手順は、instrumented ビルド時に `-fprofile-generate=$_profiledir -fprofile-update=atomic`（binutils は `$PGODIR`）を指定し、final ビルド時に `-fprofile-use=$_profiledir -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch` を指定します（LDFLAGS には素の `-fprofile-generate` / `-fprofile-use`）。final ビルドの `CFLAGS` には `-flto=auto` を付与しません。`-flto=auto` を付与した場合、`libgcc_s.so` のシンボルバージョン情報が破損するためです。なお、configure レベルでの `--enable-lto` 自体は有効のまま維持します。
