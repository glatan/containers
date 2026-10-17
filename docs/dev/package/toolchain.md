# プレフィックス付きツールチェーン

クロス開発およびブートストラップに使用するプレフィックス付きツールチェーンのビルド方針です。全体の段階は [build-flow.md](./build-flow.md) です。パッケージごとの依存と triple 固有の configure は [definition/](./definition/) の `{パッケージ名}.md` です。1 パッケージ 1 ファイルです。

5 つの GNU triple それぞれについて、binutils / gcc-stage1 / glibc-headers / gcc-stage2 / glibc / 最終 gcc の 6 段階で組みます（linux-headers は別パッケージ）。Phase 1 の `env bootstrap` が組むのは x86_64 だけです。他 triple は HOST 上で、基底 variant の `{triple}-glibc` のあとに `{triple}-binutils` / `{triple}-gcc` を組みます。これが他 arch パッケージを HOST でクロスコンパイルする実行コンパイラです。依存としてマウントする gcc / binutils は常に HOST（amd64 / v4）の成果物です。

## 製品キューと `supports`

`build --all-targets` は `supports` が真の Platform を対象にします。運用で組む製品キューはそれより狭いことがあります。ビルド設定（configure の `--build` / `--host` / `--target`、Canadian 経路）は残します。

<!-- GENERATED:supports -->

| パッケージ | `supports` |
|-|-|
| `{triple}-binutils` | `Target::all()` |
| `{triple}-gcc-stage1` | HOST（amd64 / v4）のみ |
| `{triple}-glibc-headers` | HOST（amd64 / v4）のみ |
| `{triple}-gcc-stage2` | HOST（amd64 / v4）のみ |
| `{triple}-glibc` | その architecture の全 variant |
| `{triple}-gcc` | `Target::all()` |

<!-- /GENERATED:supports -->

製品キューは `supports` より狭いことがあります。`{triple}-gcc` / `{triple}-binutils` は HOST（当該 triple）と、パッケージ名の GNU triple が Platform の architecture と一致する variant を組みます。`{triple}-glibc` と中間ステージは `supports` と同じです。

製品キューの例は amd64 向け `x86_64-linux-gnu-gcc`、arm64 向け `aarch64-linux-gnu-gcc` です。Canadian cross（`x86_64-linux-gnu-gcc` を arm64 / v8 向けに組む、など）は製品キューに入れません。

`{triple}-gcc` では `$PACKAGE_HOST` ≠ HOST triple（Canadian cross、`canadian()` 判定）のとき PGO を掛けず、HOST 上の `{triple}-gcc` を `CC_FOR_TARGET` にしてターゲットライブラリを組みます。

## 上流ソース

ハッシュは各 `{パッケージ名}.md` の生成ブロックにも現れます（正本は各定義の `Source` で、生成ブロックは `containers-builder docs` が再生成します）。

<!-- GENERATED:upstream -->

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

<!-- /GENERATED:upstream -->

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

<!-- GENERATED:stages -->

| パッケージ | バージョン | パッケージイメージ化 |
|-|-|-|
| `{triple}-binutils` | 2.47 | 対象 |
| `{triple}-gcc-stage1` | 16.2.0 | 対象外（ビルド専用中間ステージ） |
| `{triple}-glibc-headers` | 2.44 | 対象外（ビルド専用中間ステージ） |
| `{triple}-gcc-stage2` | 16.2.0 | 対象外（ビルド専用中間ステージ） |
| `{triple}-glibc` | 2.44 | 対象 |
| `{triple}-gcc` | 16.2.0 | 対象 |

<!-- /GENERATED:stages -->

中間コンパイラ（stage1 および stage2）の配置先プレフィックスは `/usr/lib/cross/{triple}/stage{1,2}/` です。最終 gcc は `/usr/bin/{triple}-gcc` として配置し、configure 時に `--with-sysroot=/usr/{triple}` を指定します。また、`x86_64-linux-gnu-glibc` に限り、`/lib64` および `ld-linux-x86-64.so.2` の互換パスにもファイルを配置します。

プレフィックス付きの `{triple}-glibc` が実行時 libc です。最終 gcc は libc を成果物に焼き付けず、`/dep/{triple}-glibc` へのリンクとして解決します。configure には `--build=$HOST_TRIPLE --target=$TRIPLE` を指定します。`--host` には、ビルド機上で動くクロスツールでは `$HOST_TRIPLE` を渡し、成果物を別アーキテクチャで動かす Canadian Cross では `$PACKAGE_HOST` を渡します（binutils も同様です）。info ファイルは `--infodir=/usr/share/info/{triple}` で分離し、5 本の HOST gcc が互いに衝突しないようにします。

`{triple}-binutils` の configure は `--enable-plugins`、`--with-system-zlib`、`--without-debuginfod`、`--disable-gprofng` を指定します。gold は組みません。`libstdc++.so.6` / `libgcc_s.so.1` への実行時依存（gcc との循環）を避けるためです。リンカは mold に一本化しています。

各ステージの configure 引数は次のとおりです（各パッケージ文書の生成 `## ビルドフラグ` の統合ビュー。`{triple}` ごとの差分のみを示します）。

<!-- GENERATED:stage-args -->

### `{triple}-binutils`

| 対象 | configure 引数 |
|-|-|
| `x86_64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr` `--infodir=/usr/share/info/${TRIPLE}` `--disable-nls` `--disable-werror` `--enable-plugins` `--with-system-zlib` `--without-debuginfod` `--disable-gprofng` `--enable-new-dtags` `--enable-relro` `--enable-default-execstack=no` `--enable-default-hash-style=gnu` `--enable-cet` |
| `aarch64-linux-gnu` `riscv64-linux-gnu` `powerpc64le-linux-gnu` `s390x-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${PACKAGE_HOST}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr` `--infodir=/usr/share/info/${TRIPLE}` `--disable-nls` `--disable-werror` `--enable-plugins` `--with-system-zlib` `--without-debuginfod` `--disable-gprofng` `--enable-new-dtags` `--enable-relro` `--enable-default-execstack=no` `--enable-default-hash-style=gnu` |

### `{triple}-gcc-stage1`

| 対象 | configure 引数 |
|-|-|
| `x86_64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage1` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=x86-64` `--without-headers` `--with-newlib` `--disable-shared` `--disable-threads` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--disable-multilib` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-default-pie` `--enable-default-ssp` `--enable-checking=release` |
| `aarch64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage1` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=armv8-a` `--without-headers` `--with-newlib` `--disable-shared` `--disable-threads` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--disable-multilib` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-default-pie` `--enable-default-ssp` `--enable-checking=release` |
| `riscv64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage1` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=rv64gc` `--with-abi=lp64d` `--without-headers` `--with-newlib` `--disable-shared` `--disable-threads` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--disable-multilib` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-default-pie` `--enable-default-ssp` `--enable-checking=release` |
| `powerpc64le-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage1` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-cpu=power8` `--without-headers` `--with-newlib` `--disable-shared` `--disable-threads` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--disable-multilib` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-default-pie` `--enable-default-ssp` `--enable-checking=release` |
| `s390x-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage1` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=z900` `--without-headers` `--with-newlib` `--disable-shared` `--disable-threads` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--disable-multilib` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-default-pie` `--enable-default-ssp` `--enable-checking=release` |

### `{triple}-glibc-headers`

| 対象 | configure 引数 |
|-|-|
| 全 triple | `--prefix=/` `--host=${TRIPLE}` `--build=${HOST_TRIPLE}` `--with-headers=${SYSROOT}/include` `--disable-werror` `libc_cv_forced_unwind=yes` `libc_cv_c_cleanup=yes` |

### `{triple}-gcc-stage2`

| 対象 | configure 引数 |
|-|-|
| `x86_64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage2` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=x86-64` `--disable-nls` `--disable-multilib` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--enable-shared` `--enable-threads=posix` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-checking=release` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` |
| `aarch64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage2` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=armv8-a` `--disable-nls` `--disable-multilib` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--enable-shared` `--enable-threads=posix` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-checking=release` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` |
| `riscv64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage2` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=rv64gc` `--with-abi=lp64d` `--disable-nls` `--disable-multilib` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--enable-shared` `--enable-threads=posix` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-checking=release` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` |
| `powerpc64le-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage2` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-cpu=power8` `--disable-nls` `--disable-multilib` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--enable-shared` `--enable-threads=posix` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-checking=release` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` |
| `s390x-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr/lib/cross/${TRIPLE}/stage2` `--with-sysroot=/usr/${TRIPLE}` `--with-build-sysroot=/usr/${TRIPLE}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=z900` `--disable-nls` `--disable-multilib` `--disable-werror` `--disable-libatomic` `--disable-libsanitizer` `--disable-libssp` `--disable-libquadmath` `--disable-libgomp` `--disable-libstdcxx` `--disable-libvtv` `--disable-bootstrap` `--enable-shared` `--enable-threads=posix` `--enable-languages=c` `--enable-lto` `--enable-plugin` `--enable-checking=release` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` |

### `{triple}-glibc`

| 対象 | configure 引数 |
|-|-|
| `x86_64-linux-gnu` | `--prefix=/` `--host=${TRIPLE}` `--build=${HOST_TRIPLE}` `--libdir=/lib` `--with-headers=${SYSROOT}/include` `--disable-werror` `--disable-nscd` `--disable-profile` `--enable-stack-protector=strong` `--enable-bind-now` `--enable-fortify-source` `--enable-cet` `libc_cv_forced_unwind=yes` `libc_cv_c_cleanup=yes` |
| `aarch64-linux-gnu` `riscv64-linux-gnu` `powerpc64le-linux-gnu` `s390x-linux-gnu` | `--prefix=/` `--host=${TRIPLE}` `--build=${HOST_TRIPLE}` `--libdir=/lib` `--with-headers=${SYSROOT}/include` `--disable-werror` `--disable-nscd` `--disable-profile` `--enable-stack-protector=strong` `--enable-bind-now` `--enable-fortify-source` `libc_cv_forced_unwind=yes` `libc_cv_c_cleanup=yes` |

<!-- /GENERATED:stage-args -->

オプションの効果:

- binutils の `--enable-new-dtags` `--enable-relro` `--enable-default-execstack=no` `--enable-default-hash-style=gnu`: RUNPATH 化・RELRO 既定・`-z noexecstack` を ld 既定化・生 `ld` 呼び出しも gnu hash にします。`--enable-cet`（`x86_64-linux-gnu` のみ）は ld を CET 対応にします。
- gcc-stage1 / stage2 の `--enable-default-pie` `--enable-default-ssp`: 生成コンパイラの既定です（freestanding でも無害）。stage2 は libgcc_s.so をリンクするため `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` を追加します。
- glibc の `--enable-stack-protector=strong` `--enable-bind-now` `--enable-fortify-source`: libc 自身の SSP / eager binding / fortify です（LEVEL はコンパイラ最大 = 3）。`--enable-cet`（`x86_64-linux-gnu` のみ、strict）は全 ELF 出力の `readelf -n` CET マーク audit が前提です（非マーク DSO の dlopen は strict ではエラー）。

binutils の PGO は `cet && !ctx.cross()` でゲートされます（`cet` は `$TRIPLE` = `x86_64-linux-gnu` のときのみ真）。instrumented な `as` / `ld` / `objdump` をビルドホスト上で実行してプロファイルを採取するため、生成コードが HOST で動かせる triple に限ります。`x86_64-linux-gnu-binutils` は `$TRIPLE` が常に `$HOST_TRIPLE` と一致するため全対象 Platform で適用され、他 triple の `{triple}-binutils` では `cet` が偽のため適用されません。instrumented ツリーで `as` / `ld` / `objdump` を生成アセンブリの assemble・リンク・逆アセンブルで駆動してプロファイルを採取し、別ツリーで `-fprofile-use` の成果物を組みます。クロス binutils には適用しません。

## linux-headers の取り込み

stage1、glibc-headers、stage2、glibc、および最終 gcc の各ステージでは、sysroot の `include/` ディレクトリへ [linux-headers](./definition/linux-headers.md) のカーネルユーザー空間ヘッダをコピーします。まず優先して `/dep/linux-headers/usr/${TRIPLE}/include` を参照し、該当ディレクトリが存在しない場合に限り共通の `/dep/linux-headers/usr/include` へフォールバックします。

`openssl` や `zlib` の `-idirafter` は、`/dep/linux-headers/usr/${TRIPLE}/include` を優先し、無い場合に `/dep/linux-headers/usr/include` へフォールバックします。

## 最終 gcc の構成

最終 gcc の configure では、instrumented（プロファイル採取用）および final（最終成果物）の双方において、`--disable-bootstrap`、`--disable-multilib`、`--enable-languages=c,c++`、`--enable-lto`、および `--enable-plugin` を指定します。なお、stage1 および stage2 ではすべての triple において `--disable-libsanitizer` を指定します。

<!-- GENERATED:gcc-args -->

### `{triple}-gcc`

| 対象 | configure 引数 |
|-|-|
| `x86_64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${HOST_TRIPLE}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr` `--with-sysroot=${SYSROOT}` `--with-build-sysroot=${BUILD_SYSROOT}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=${MARCH}` `--enable-cet` `--disable-nls` `--disable-werror` `--disable-libssp` `--disable-libstdcxx-pch` `--disable-bootstrap` `--disable-multilib` `--enable-plugin` `--enable-languages=c,c++` `--enable-lto` `--enable-checking=release` `--enable-__cxa_atexit` `--enable-threads=posix` `--enable-shared` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` `--enable-libstdcxx-backtrace` `--enable-host-pie` `--enable-host-bind-now` `--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'` |
| `aarch64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${PACKAGE_HOST}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr` `--with-sysroot=${SYSROOT}` `--with-build-sysroot=${BUILD_SYSROOT}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=${MARCH}` `--disable-nls` `--disable-werror` `--disable-libssp` `--disable-libstdcxx-pch` `--disable-bootstrap` `--disable-multilib` `--disable-libsanitizer` `--enable-plugin` `--enable-languages=c,c++` `--enable-lto` `--enable-checking=release` `--enable-__cxa_atexit` `--enable-threads=posix` `--enable-shared` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` `--enable-libstdcxx-backtrace` `--enable-host-pie` `--enable-host-bind-now` `--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'` |
| `riscv64-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${PACKAGE_HOST}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr` `--with-sysroot=${SYSROOT}` `--with-build-sysroot=${BUILD_SYSROOT}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=${MARCH}` `--with-abi=lp64d` `--disable-nls` `--disable-werror` `--disable-libssp` `--disable-libstdcxx-pch` `--disable-bootstrap` `--disable-multilib` `--disable-libsanitizer` `--enable-plugin` `--enable-languages=c,c++` `--enable-lto` `--enable-checking=release` `--enable-__cxa_atexit` `--enable-threads=posix` `--enable-shared` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` `--enable-libstdcxx-backtrace` `--enable-host-pie` `--enable-host-bind-now` `--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'` |
| `powerpc64le-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${PACKAGE_HOST}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr` `--with-sysroot=${SYSROOT}` `--with-build-sysroot=${BUILD_SYSROOT}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-cpu=${MARCH}` `--disable-nls` `--disable-werror` `--disable-libssp` `--disable-libstdcxx-pch` `--disable-bootstrap` `--disable-multilib` `--disable-libsanitizer` `--enable-plugin` `--enable-languages=c,c++` `--enable-lto` `--enable-checking=release` `--enable-__cxa_atexit` `--enable-threads=posix` `--enable-shared` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` `--enable-libstdcxx-backtrace` `--enable-host-pie` `--enable-host-bind-now` `--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'` |
| `s390x-linux-gnu` | `--build=${HOST_TRIPLE}` `--host=${PACKAGE_HOST}` `--target=${TRIPLE}` `--program-prefix=${TRIPLE}-` `--prefix=/usr` `--with-sysroot=${SYSROOT}` `--with-build-sysroot=${BUILD_SYSROOT}` `--with-native-system-header-dir=/include` `--with-as=/usr/bin/${TRIPLE}-as` `--with-ld=/usr/bin/${TRIPLE}-ld` `--infodir=/usr/share/info/${TRIPLE}` `--with-system-zlib` `${GCC_LIB_FLAGS}` `--with-arch=${MARCH}` `--enable-decimal-float` `--disable-nls` `--disable-werror` `--disable-libssp` `--disable-libstdcxx-pch` `--disable-bootstrap` `--disable-multilib` `--disable-libsanitizer` `--enable-plugin` `--enable-languages=c,c++` `--enable-lto` `--enable-checking=release` `--enable-__cxa_atexit` `--enable-threads=posix` `--enable-shared` `--enable-default-pie` `--enable-default-ssp` `--with-linker-hash-style=gnu` `--enable-linker-build-id` `--enable-gnu-indirect-function` `--enable-gnu-unique-object` `--enable-initfini-array` `--enable-libstdcxx-backtrace` `--enable-host-pie` `--enable-host-bind-now` `--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'` |

<!-- /GENERATED:gcc-args -->

オプションの効果: `--enable-default-pie` / `--enable-default-ssp` とリンカー挙動の既定に加え、`--enable-host-pie` / `--enable-host-bind-now` で cc1/xgcc 等コンパイラ自身のバイナリも PIE/BIND_NOW 化します。`--enable-cet`（amd64）は `-fcf-protection` と対です。`--enable-decimal-float`（s390x）は libgcc の DFP ランタイムです。`--with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'` は CFLAGS を無視するビルド系へのベースライン適用で、`-fno-hardened` で傘フラグのみ解除可能です（`NoHardened` トークン）。target ランタイム lib（libgcc/libstdc++ 等）も `-fhardened` でビルドされます（intended）。

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
