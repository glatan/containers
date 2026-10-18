<!-- GENERATED -->
# aarch64-linux-gnu-gcc

| 項目 | 値 |
|-|-|
| パッケージ名 | `aarch64-linux-gnu-gcc` |
| バージョン | 16.2.0 |
| ライセンス | GPL-3.0-or-later WITH GCC-exception-3.1 |
| 分類 | パッケージ（Bootstrap） |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64/v4 で適用（`Pgo::Custom`、レシピ内の個別手順） |
| シード用 pacman | make bison flex perl texinfo m4 |
| コード生成 triple | `aarch64-linux-gnu` |
| cc ラッパ | 付与しない |
| 定義 | [`builder/src/package/definition/aarch64_linux_gnu_gcc.rs`](../../../../builder/src/package/definition/aarch64_linux_gnu_gcc.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gcc/gcc-16.2.0/gcc-16.2.0.tar.xz |
| sha3-512 | `1995b2fd6c2b19fe8ca47c836fdee8c019909ab8857f81a8045a7726ce724146ab63cd8f836fb460e9b009434ea97eb51b34397760b9ec4bbcac490ca32fa0f6` |
| blake3 | `c0f53af0bbfd2bf28896ab9bdfd1a534351b851c5a2f0226feb6a4323884c43531cbb207c733e8fd8ac16e2e1b03e6d8f17d043a17cf026857eedf26c31e4e0e` |
| 展開ディレクトリ | `gcc-16.2.0` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [gmp](./gmp.md) | 6.3.0 | — |
| [mpfr](./mpfr.md) | 4.2.2 | — |
| [mpc](./mpc.md) | 1.4.1 | — |
| [isl](./isl.md) | 0.28 | — |
| [zlib](./zlib.md) | 1.3.2 | — |
| [zstd](./zstd.md) | 1.5.7 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.56 | — |
| [aarch64-linux-gnu-binutils](./aarch64-linux-gnu-binutils.md) | 2.47 | — |
| [aarch64-linux-gnu-glibc](./aarch64-linux-gnu-glibc.md) | 2.44 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [bison](./bison.md) | 3.8.2 | optional |
| [flex](./flex.md) | 2.6.4 | optional |
| [perl](./perl.md) | 5.44.0 | optional |
| [texinfo](./texinfo.md) | 7.3 | optional |
| [m4](./m4.md) | 1.4.21 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | なし |

## ツールチェーンフラグ

`tool_flags` の既定セットとの差分です（[README.md](../README.md) の既定フラグ表）。

| 対象 | 追加 | 除外 |
|-|-|-|
| CFLAGS | — | `-ftree-vectorize`<br>`-fno-semantic-interposition`<br>`-fstack-protector-strong`<br>`-Wformat -Werror=format-security`<br>`-fstrict-flex-arrays=3`<br>`-fzero-init-padding-bits=all`<br>`-fno-delete-null-pointer-checks`<br>`-fno-strict-overflow`<br>`-fno-strict-aliasing`<br>`-fexceptions`<br>`-Wtrampolines`<br>`-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch`<br>`-Werror=strict-aliasing`<br>`-fno-plt`<br>`-mtls-dialect=gnu2`<br>`-fzero-call-used-regs=used-gpr` |
| LDFLAGS | — | `-fuse-ld=mold`<br>`-Wl,-O1`<br>`-Wl,--as-needed`<br>`-Wl,--no-copy-dt-needed-entries`<br>`-Wl,-z,relro`<br>`-Wl,-z,now`<br>`-Wl,-z,noexecstack`<br>`-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch`<br>`-Werror=strict-aliasing`<br>`-Wl,-z,pack-relative-relocs` |

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

### `${SOURCE}/configure`（1回目の呼出）

- 対象: amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--build` | `${HOST_TRIPLE}` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--host` | `${PACKAGE_HOST}` | arm64, ppc64le, riscv64, s390x |
| `--target` | `${TRIPLE}` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--program-prefix` | `${TRIPLE}-` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--prefix` | `/usr` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-sysroot` | `${SYSROOT}` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-build-sysroot` | `${BUILD_SYSROOT}` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-native-system-header-dir` | `/include` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-as` | `/usr/bin/${TRIPLE}-as` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-ld` | `/usr/bin/${TRIPLE}-ld` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--infodir` | `/usr/share/info/${TRIPLE}` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-system-zlib` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `${GCC_LIB_FLAGS}` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-arch` | `${MARCH}` | arm64 |
| `--disable-nls` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--disable-werror` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--disable-libssp` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--disable-libstdcxx-pch` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--disable-bootstrap` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--disable-multilib` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--disable-libsanitizer` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-plugin` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-languages` | `c,c++` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-lto` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-checking` | `release` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-__cxa_atexit` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-threads` | `posix` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-shared` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-default-pie` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-default-ssp` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-linker-hash-style` | `gnu` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-linker-build-id` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-gnu-indirect-function` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-gnu-unique-object` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-initfini-array` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-libstdcxx-backtrace` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-host-pie` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--enable-host-bind-now` |  | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-specs` | `'%{!fno-hardened:-fhardened} -Wno-hardened'` | amd64/v1 amd64/v2 amd64/v3, arm64, ppc64le, riscv64, s390x |
| `--with-arch` | `armv8-a` | amd64/v1 amd64/v2 amd64/v3, ppc64le, riscv64, s390x |
| `--host` | `${HOST_TRIPLE}` | amd64/v1 amd64/v2 amd64/v3 |

### `${SOURCE}/configure`（2回目の呼出）

- 対象: amd64/v4
- env: `CFLAGS="${CFLAGS:-} -fprofile-generate=${_profiledir} -fprofile-update=atomic"` `CXXFLAGS="${CXXFLAGS:-} -fprofile-generate=${_profiledir} -fprofile-update=atomic"` `LDFLAGS="${LDFLAGS:-} -fprofile-generate=${_profiledir}"`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--build` | `${HOST_TRIPLE}` | amd64/v4 |
| `--host` | `${HOST_TRIPLE}` | amd64/v4 |
| `--target` | `${TRIPLE}` | amd64/v4 |
| `--program-prefix` | `${TRIPLE}-` | amd64/v4 |
| `--prefix` | `/usr` | amd64/v4 |
| `--with-sysroot` | `${SYSROOT}` | amd64/v4 |
| `--with-build-sysroot` | `${BUILD_SYSROOT}` | amd64/v4 |
| `--with-native-system-header-dir` | `/include` | amd64/v4 |
| `--with-as` | `/usr/bin/${TRIPLE}-as` | amd64/v4 |
| `--with-ld` | `/usr/bin/${TRIPLE}-ld` | amd64/v4 |
| `--infodir` | `/usr/share/info/${TRIPLE}` | amd64/v4 |
| `--with-system-zlib` |  | amd64/v4 |
| `${GCC_LIB_FLAGS}` |  | amd64/v4 |
| `--with-arch` | `armv8-a` | amd64/v4 |
| `--disable-nls` |  | amd64/v4 |
| `--disable-werror` |  | amd64/v4 |
| `--disable-libssp` |  | amd64/v4 |
| `--disable-libstdcxx-pch` |  | amd64/v4 |
| `--disable-bootstrap` |  | amd64/v4 |
| `--disable-multilib` |  | amd64/v4 |
| `--disable-libsanitizer` |  | amd64/v4 |
| `--enable-plugin` |  | amd64/v4 |
| `--enable-languages` | `c,c++` | amd64/v4 |
| `--enable-lto` |  | amd64/v4 |
| `--enable-checking` | `release` | amd64/v4 |
| `--enable-__cxa_atexit` |  | amd64/v4 |
| `--enable-threads` | `posix` | amd64/v4 |
| `--enable-shared` |  | amd64/v4 |
| `--enable-default-pie` |  | amd64/v4 |
| `--enable-default-ssp` |  | amd64/v4 |
| `--with-linker-hash-style` | `gnu` | amd64/v4 |
| `--enable-linker-build-id` |  | amd64/v4 |
| `--enable-gnu-indirect-function` |  | amd64/v4 |
| `--enable-gnu-unique-object` |  | amd64/v4 |
| `--enable-initfini-array` |  | amd64/v4 |
| `--enable-libstdcxx-backtrace` |  | amd64/v4 |
| `--enable-host-pie` |  | amd64/v4 |
| `--enable-host-bind-now` |  | amd64/v4 |
| `--with-specs` | `'%{!fno-hardened:-fhardened} -Wno-hardened'` | amd64/v4 |

### `${SOURCE}/configure`（3回目の呼出）

- 対象: amd64/v4
- env: `CFLAGS="${CFLAGS:-} -fprofile-use=${_profiledir} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"` `CXXFLAGS="${CXXFLAGS:-} -fprofile-use=${_profiledir} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"` `LDFLAGS="${LDFLAGS:-} -fprofile-use"`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--build` | `${HOST_TRIPLE}` | amd64/v4 |
| `--host` | `${HOST_TRIPLE}` | amd64/v4 |
| `--target` | `${TRIPLE}` | amd64/v4 |
| `--program-prefix` | `${TRIPLE}-` | amd64/v4 |
| `--prefix` | `/usr` | amd64/v4 |
| `--with-sysroot` | `${SYSROOT}` | amd64/v4 |
| `--with-build-sysroot` | `${BUILD_SYSROOT}` | amd64/v4 |
| `--with-native-system-header-dir` | `/include` | amd64/v4 |
| `--with-as` | `/usr/bin/${TRIPLE}-as` | amd64/v4 |
| `--with-ld` | `/usr/bin/${TRIPLE}-ld` | amd64/v4 |
| `--infodir` | `/usr/share/info/${TRIPLE}` | amd64/v4 |
| `--with-system-zlib` |  | amd64/v4 |
| `${GCC_LIB_FLAGS}` |  | amd64/v4 |
| `--with-arch` | `armv8-a` | amd64/v4 |
| `--disable-nls` |  | amd64/v4 |
| `--disable-werror` |  | amd64/v4 |
| `--disable-libssp` |  | amd64/v4 |
| `--disable-libstdcxx-pch` |  | amd64/v4 |
| `--disable-bootstrap` |  | amd64/v4 |
| `--disable-multilib` |  | amd64/v4 |
| `--disable-libsanitizer` |  | amd64/v4 |
| `--enable-plugin` |  | amd64/v4 |
| `--enable-languages` | `c,c++` | amd64/v4 |
| `--enable-lto` |  | amd64/v4 |
| `--enable-checking` | `release` | amd64/v4 |
| `--enable-__cxa_atexit` |  | amd64/v4 |
| `--enable-threads` | `posix` | amd64/v4 |
| `--enable-shared` |  | amd64/v4 |
| `--enable-default-pie` |  | amd64/v4 |
| `--enable-default-ssp` |  | amd64/v4 |
| `--with-linker-hash-style` | `gnu` | amd64/v4 |
| `--enable-linker-build-id` |  | amd64/v4 |
| `--enable-gnu-indirect-function` |  | amd64/v4 |
| `--enable-gnu-unique-object` |  | amd64/v4 |
| `--enable-initfini-array` |  | amd64/v4 |
| `--enable-libstdcxx-backtrace` |  | amd64/v4 |
| `--enable-host-pie` |  | amd64/v4 |
| `--enable-host-bind-now` |  | amd64/v4 |
| `--with-specs` | `'%{!fno-hardened:-fhardened} -Wno-hardened'` | amd64/v4 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 4 系統に分かれます。

<details>
<summary>amd64/v4（クロス）</summary>

```bash
set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
if [ -d /dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE} ]; then
  mkdir -p /tmp/build-sysroot/usr
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/include
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/usr/include
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib" /tmp/build-sysroot/lib
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib64" /tmp/build-sysroot/lib64
  ln -sfn ../lib /tmp/build-sysroot/usr/lib
  ln -sfn ../lib64 /tmp/build-sysroot/usr/lib64
  _build_root=/tmp/build-sysroot
fi
_bf="-O2 -pipe"
if [ -n "${_build_root:-}" ]; then
  _bf="${_bf} --sysroot=${_build_root}"
  if [ -f ${_build_root}/lib/crti.o ]; then
    _bf="${_bf} -B${_build_root}/lib"
  else
    if [ -f ${_build_root}/lib64/crti.o ]; then
      _bf="${_bf} -B${_build_root}/lib64"
    fi
  fi
fi
export CFLAGS_FOR_BUILD="${_bf}"
export CXXFLAGS_FOR_BUILD="${_bf}"
export LDFLAGS_FOR_BUILD="${_build_root:+--sysroot=$_build_root}"
SYSROOT=/usr/${TRIPLE}
_profiledir=${BUILD}/pgo-profile-data
rm -rf ${BUILD}/instrumented ${BUILD}/final
mkdir -p ${SYSROOT} ${_profiledir} ${BUILD}/instrumented ${BUILD}/final
if [ -d /dep/${TRIPLE}-glibc/usr/${TRIPLE} ]; then
  cp -a /dep/${TRIPLE}-glibc/usr/${TRIPLE}/. ${SYSROOT}/
fi
for _so in $(find ${SYSROOT} -type f -name "*.so"); do
  if grep -q "GNU ld script" ${_so}; then
    sed -i "s|//lib/|/lib/|g; s|//lib64/|/lib64/|g" ${_so}
  fi
done
BUILD_SYSROOT=${SYSROOT}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
mkdir -p ${BUILD_SYSROOT}/include
cp -a ${HDR}/. ${BUILD_SYSROOT}/include/ || true
export CFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export CXXFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export LDFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT}"
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS=--with-isl
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zlib/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zlib/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zstd/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zstd/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
cd ${BUILD}/instrumented
CFLAGS="${CFLAGS:-} -fprofile-generate=${_profiledir} -fprofile-update=atomic" CXXFLAGS="${CXXFLAGS:-} -fprofile-generate=${_profiledir} -fprofile-update=atomic" LDFLAGS="${LDFLAGS:-} -fprofile-generate=${_profiledir}" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --with-sysroot=${SYSROOT} --with-build-sysroot=${BUILD_SYSROOT} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=armv8-a --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib --disable-libsanitizer --enable-plugin --enable-languages=c,c++ --enable-lto --enable-checking=release --enable-__cxa_atexit --enable-threads=posix --enable-shared --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array --enable-libstdcxx-backtrace --enable-host-pie --enable-host-bind-now --with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'
make -j"$(nproc)"
_xgcc=${BUILD}/instrumented/gcc/xgcc
_xgpp=${BUILD}/instrumented/gcc/xg++
_B=${BUILD}/instrumented/gcc/
_ts=${SOURCE}/gcc/testsuite
for _opt in -O0 -O1 -O2 -O3 -Os; do
  find ${_ts}/gcc.c-torture/compile -maxdepth 1 -name "*.c" -print0 | xargs -0 -P$(nproc) -I{} ${_xgcc} -B${_B} ${_opt} -S -o /dev/null -w {} 2> /dev/null || true
done
find ${_ts}/g++.dg -maxdepth 1 -name "*.C" -print0 | xargs -0 -P$(nproc) -I{} ${_xgpp} -B${_B} -O2 -S -o /dev/null -w {} 2> /dev/null || true
cd ${BUILD}/final
CFLAGS="${CFLAGS:-} -fprofile-use=${_profiledir} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" CXXFLAGS="${CXXFLAGS:-} -fprofile-use=${_profiledir} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="${LDFLAGS:-} -fprofile-use" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --with-sysroot=${SYSROOT} --with-build-sysroot=${BUILD_SYSROOT} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=armv8-a --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib --disable-libsanitizer --enable-plugin --enable-languages=c,c++ --enable-lto --enable-checking=release --enable-__cxa_atexit --enable-threads=posix --enable-shared --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array --enable-libstdcxx-backtrace --enable-host-pie --enable-host-bind-now --with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'
make -j"$(nproc)"
make DESTDIR="$OUT" install
```

</details>

<details>
<summary>arm64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
if [ -d /dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE} ]; then
  mkdir -p /tmp/build-sysroot/usr
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/include
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/usr/include
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib" /tmp/build-sysroot/lib
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib64" /tmp/build-sysroot/lib64
  ln -sfn ../lib /tmp/build-sysroot/usr/lib
  ln -sfn ../lib64 /tmp/build-sysroot/usr/lib64
  _build_root=/tmp/build-sysroot
fi
_bf="-O2 -pipe"
if [ -n "${_build_root:-}" ]; then
  _bf="${_bf} --sysroot=${_build_root}"
  if [ -f ${_build_root}/lib/crti.o ]; then
    _bf="${_bf} -B${_build_root}/lib"
  else
    if [ -f ${_build_root}/lib64/crti.o ]; then
      _bf="${_bf} -B${_build_root}/lib64"
    fi
  fi
fi
export CFLAGS_FOR_BUILD="${_bf}"
export CXXFLAGS_FOR_BUILD="${_bf}"
export LDFLAGS_FOR_BUILD="${_build_root:+--sysroot=$_build_root}"
SYSROOT=/usr/${TRIPLE}
_profiledir=${BUILD}/pgo-profile-data
rm -rf ${BUILD}/instrumented ${BUILD}/final
mkdir -p ${SYSROOT} ${_profiledir} ${BUILD}/instrumented ${BUILD}/final
if [ -d /dep/${TRIPLE}-glibc/usr/${TRIPLE} ]; then
  cp -a /dep/${TRIPLE}-glibc/usr/${TRIPLE}/. ${SYSROOT}/
fi
for _so in $(find ${SYSROOT} -type f -name "*.so"); do
  if grep -q "GNU ld script" ${_so}; then
    sed -i "s|//lib/|/lib/|g; s|//lib64/|/lib64/|g" ${_so}
  fi
done
BUILD_SYSROOT=${SYSROOT}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
mkdir -p ${BUILD_SYSROOT}/include
cp -a ${HDR}/. ${BUILD_SYSROOT}/include/ || true
export CFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export CXXFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export LDFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT}"
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS=--with-isl
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zlib/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zlib/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zstd/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zstd/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
cat > /tmp/gen-wrap.pl <<'EOF'
my ($dest, @cmd) = @ARGV;
open my $f, '>', $dest or die $!;
print $f "#!$^X\n";
print $f "use strict;\n";
print $f "my \@cmd = (\n";
print $f "  '$_',\n" for map { my $q = $_; $q =~ s/'/\\'/g; $q } @cmd;
print $f ");\n";
print $f "exec \@cmd, \@ARGV;\n";
close $f;
chmod 0755, $dest;
EOF
_wrap=/tmp/canadian-host-cc
mkdir -p ${_wrap}
_real_cc=$(command -v ${PACKAGE_HOST}-gcc)
_real_cxx=$(command -v ${PACKAGE_HOST}-g++)
_cflags=${CFLAGS:-}
_cxxflags=${CXXFLAGS:-}
_ldflags=${LDFLAGS:-}
if [ "${PACKAGE_HOST}" = "${TRIPLE}" ]; then
  _cflags=$(printf %s "${_cflags}" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|${SYSROOT}|g")
  _cxxflags=$(printf %s "${_cxxflags}" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|${SYSROOT}|g")
  _ldflags=$(printf %s "${_ldflags}" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|${SYSROOT}|g")
  export gcc_cv_objdump="/dep/${PACKAGE_HOST}-binutils/usr/${PACKAGE_HOST}/bin/objdump"
fi
perl /tmp/gen-wrap.pl ${_wrap}/${PACKAGE_HOST}-gcc ${_real_cc} ${_cflags} ${_ldflags}
perl /tmp/gen-wrap.pl ${_wrap}/${PACKAGE_HOST}-g++ ${_real_cxx} ${_cxxflags} ${_ldflags}
export PATH="${_wrap}:${PATH}"
export CC="${PACKAGE_HOST}-gcc"
export CXX="${PACKAGE_HOST}-g++"
unset CFLAGS
unset CXXFLAGS
unset LDFLAGS
unset CPPFLAGS
unset CPP
cat > /tmp/canadian-test.c <<'EOF'
int main(void) { return 0; }
EOF
${CC} -x c -o /tmp/canadian-cc-ok - < /tmp/canadian-test.c
export CC_FOR_TARGET="${TRIPLE}-gcc"
export CXX_FOR_TARGET="${TRIPLE}-g++"
export GCC_FOR_TARGET="${TRIPLE}-gcc"
export AR_FOR_TARGET="${TRIPLE}-ar"
export RANLIB_FOR_TARGET="${TRIPLE}-ranlib"
export NM_FOR_TARGET="${TRIPLE}-nm"
export AS_FOR_TARGET="${TRIPLE}-as"
export LD_FOR_TARGET="${TRIPLE}-ld"
cd ${BUILD}/final
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${PACKAGE_HOST} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --with-sysroot=${SYSROOT} --with-build-sysroot=${BUILD_SYSROOT} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=${MARCH} --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib --disable-libsanitizer --enable-plugin --enable-languages=c,c++ --enable-lto --enable-checking=release --enable-__cxa_atexit --enable-threads=posix --enable-shared --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array --enable-libstdcxx-backtrace --enable-host-pie --enable-host-bind-now --with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'
make -j"$(nproc)"
make DESTDIR="$OUT" install
```

</details>

<details>
<summary>ppc64le, riscv64, s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
if [ -d /dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE} ]; then
  mkdir -p /tmp/build-sysroot/usr
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/include
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/usr/include
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib" /tmp/build-sysroot/lib
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib64" /tmp/build-sysroot/lib64
  ln -sfn ../lib /tmp/build-sysroot/usr/lib
  ln -sfn ../lib64 /tmp/build-sysroot/usr/lib64
  _build_root=/tmp/build-sysroot
fi
_bf="-O2 -pipe"
if [ -n "${_build_root:-}" ]; then
  _bf="${_bf} --sysroot=${_build_root}"
  if [ -f ${_build_root}/lib/crti.o ]; then
    _bf="${_bf} -B${_build_root}/lib"
  else
    if [ -f ${_build_root}/lib64/crti.o ]; then
      _bf="${_bf} -B${_build_root}/lib64"
    fi
  fi
fi
export CFLAGS_FOR_BUILD="${_bf}"
export CXXFLAGS_FOR_BUILD="${_bf}"
export LDFLAGS_FOR_BUILD="${_build_root:+--sysroot=$_build_root}"
SYSROOT=/usr/${TRIPLE}
_profiledir=${BUILD}/pgo-profile-data
rm -rf ${BUILD}/instrumented ${BUILD}/final
mkdir -p ${SYSROOT} ${_profiledir} ${BUILD}/instrumented ${BUILD}/final
if [ -d /dep/${TRIPLE}-glibc/usr/${TRIPLE} ]; then
  cp -a /dep/${TRIPLE}-glibc/usr/${TRIPLE}/. ${SYSROOT}/
fi
for _so in $(find ${SYSROOT} -type f -name "*.so"); do
  if grep -q "GNU ld script" ${_so}; then
    sed -i "s|//lib/|/lib/|g; s|//lib64/|/lib64/|g" ${_so}
  fi
done
BUILD_SYSROOT=${SYSROOT}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
mkdir -p ${BUILD_SYSROOT}/include
cp -a ${HDR}/. ${BUILD_SYSROOT}/include/ || true
export CFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export CXXFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export LDFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT}"
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS=--with-isl
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zlib/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zlib/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zstd/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zstd/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
cat > /tmp/gen-wrap.pl <<'EOF'
my ($dest, @cmd) = @ARGV;
open my $f, '>', $dest or die $!;
print $f "#!$^X\n";
print $f "use strict;\n";
print $f "my \@cmd = (\n";
print $f "  '$_',\n" for map { my $q = $_; $q =~ s/'/\\'/g; $q } @cmd;
print $f ");\n";
print $f "exec \@cmd, \@ARGV;\n";
close $f;
chmod 0755, $dest;
EOF
_wrap=/tmp/canadian-host-cc
mkdir -p ${_wrap}
_real_cc=$(command -v ${PACKAGE_HOST}-gcc)
_real_cxx=$(command -v ${PACKAGE_HOST}-g++)
_cflags=${CFLAGS:-}
_cxxflags=${CXXFLAGS:-}
_ldflags=${LDFLAGS:-}
if [ "${PACKAGE_HOST}" = "${TRIPLE}" ]; then
  _cflags=$(printf %s "${_cflags}" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|${SYSROOT}|g")
  _cxxflags=$(printf %s "${_cxxflags}" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|${SYSROOT}|g")
  _ldflags=$(printf %s "${_ldflags}" | sed "s|/dep/${PACKAGE_HOST}-glibc/usr/${PACKAGE_HOST}|${SYSROOT}|g")
  export gcc_cv_objdump="/dep/${PACKAGE_HOST}-binutils/usr/${PACKAGE_HOST}/bin/objdump"
fi
perl /tmp/gen-wrap.pl ${_wrap}/${PACKAGE_HOST}-gcc ${_real_cc} ${_cflags} ${_ldflags}
perl /tmp/gen-wrap.pl ${_wrap}/${PACKAGE_HOST}-g++ ${_real_cxx} ${_cxxflags} ${_ldflags}
export PATH="${_wrap}:${PATH}"
export CC="${PACKAGE_HOST}-gcc"
export CXX="${PACKAGE_HOST}-g++"
unset CFLAGS
unset CXXFLAGS
unset LDFLAGS
unset CPPFLAGS
unset CPP
cat > /tmp/canadian-test.c <<'EOF'
int main(void) { return 0; }
EOF
${CC} -x c -o /tmp/canadian-cc-ok - < /tmp/canadian-test.c
export CC_FOR_TARGET="${TRIPLE}-gcc"
export CXX_FOR_TARGET="${TRIPLE}-g++"
export GCC_FOR_TARGET="${TRIPLE}-gcc"
export AR_FOR_TARGET="${TRIPLE}-ar"
export RANLIB_FOR_TARGET="${TRIPLE}-ranlib"
export NM_FOR_TARGET="${TRIPLE}-nm"
export AS_FOR_TARGET="${TRIPLE}-as"
export LD_FOR_TARGET="${TRIPLE}-ld"
cd ${BUILD}/final
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${PACKAGE_HOST} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --with-sysroot=${SYSROOT} --with-build-sysroot=${BUILD_SYSROOT} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=armv8-a --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib --disable-libsanitizer --enable-plugin --enable-languages=c,c++ --enable-lto --enable-checking=release --enable-__cxa_atexit --enable-threads=posix --enable-shared --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array --enable-libstdcxx-backtrace --enable-host-pie --enable-host-bind-now --with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'
make -j"$(nproc)"
make DESTDIR="$OUT" install
```

</details>

<details>
<summary>amd64/v1 amd64/v2 amd64/v3（クロス）</summary>

```bash
set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
if [ -d /dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE} ]; then
  mkdir -p /tmp/build-sysroot/usr
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/include
  ln -sfn "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/include" /tmp/build-sysroot/usr/include
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib" /tmp/build-sysroot/lib
  cp -a "/dep/${HOST_TRIPLE}-glibc/usr/${HOST_TRIPLE}/lib64" /tmp/build-sysroot/lib64
  ln -sfn ../lib /tmp/build-sysroot/usr/lib
  ln -sfn ../lib64 /tmp/build-sysroot/usr/lib64
  _build_root=/tmp/build-sysroot
fi
_bf="-O2 -pipe"
if [ -n "${_build_root:-}" ]; then
  _bf="${_bf} --sysroot=${_build_root}"
  if [ -f ${_build_root}/lib/crti.o ]; then
    _bf="${_bf} -B${_build_root}/lib"
  else
    if [ -f ${_build_root}/lib64/crti.o ]; then
      _bf="${_bf} -B${_build_root}/lib64"
    fi
  fi
fi
export CFLAGS_FOR_BUILD="${_bf}"
export CXXFLAGS_FOR_BUILD="${_bf}"
export LDFLAGS_FOR_BUILD="${_build_root:+--sysroot=$_build_root}"
SYSROOT=/usr/${TRIPLE}
_profiledir=${BUILD}/pgo-profile-data
rm -rf ${BUILD}/instrumented ${BUILD}/final
mkdir -p ${SYSROOT} ${_profiledir} ${BUILD}/instrumented ${BUILD}/final
if [ -d /dep/${TRIPLE}-glibc/usr/${TRIPLE} ]; then
  cp -a /dep/${TRIPLE}-glibc/usr/${TRIPLE}/. ${SYSROOT}/
fi
for _so in $(find ${SYSROOT} -type f -name "*.so"); do
  if grep -q "GNU ld script" ${_so}; then
    sed -i "s|//lib/|/lib/|g; s|//lib64/|/lib64/|g" ${_so}
  fi
done
BUILD_SYSROOT=${SYSROOT}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
mkdir -p ${BUILD_SYSROOT}/include
cp -a ${HDR}/. ${BUILD_SYSROOT}/include/ || true
export CFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export CXXFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT} -ftrivial-auto-var-init=uninitialized"
export LDFLAGS_FOR_TARGET="--sysroot=${BUILD_SYSROOT}"
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS=--with-isl
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zlib/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zlib/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS="${CFLAGS:-} -I/dep/zstd/usr/include"
  export CXXFLAGS="${CXXFLAGS:-} -I/dep/zstd/usr/include"
  export LDFLAGS="${LDFLAGS:-} -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
cd ${BUILD}/final
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --with-sysroot=${SYSROOT} --with-build-sysroot=${BUILD_SYSROOT} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=armv8-a --disable-nls --disable-werror --disable-libssp --disable-libstdcxx-pch --disable-bootstrap --disable-multilib --disable-libsanitizer --enable-plugin --enable-languages=c,c++ --enable-lto --enable-checking=release --enable-__cxa_atexit --enable-threads=posix --enable-shared --enable-default-pie --enable-default-ssp --with-linker-hash-style=gnu --enable-linker-build-id --enable-gnu-indirect-function --enable-gnu-unique-object --enable-initfini-array --enable-libstdcxx-backtrace --enable-host-pie --enable-host-bind-now --with-specs='%{!fno-hardened:-fhardened} -Wno-hardened'
make -j"$(nproc)"
make DESTDIR="$OUT" install
```

</details>

<!-- /GENERATED -->



## ビルド設定

C/C++、`--disable-bootstrap`。gcc フラグ `--with-arch=$MARCH`（`$MARCH` は host arch と codegen arch が一致する場合のみ。異種クロスでは `armv8-a` へフォールバック。stage1/2 は常に `--with-arch=armv8-a`）。PGO と sanitizer は [toolchain.md](../toolchain.md) です。configure は `--build=$HOST_TRIPLE --host=$HOST_TRIPLE --target=aarch64-linux-gnu` です。

`supports` は `Target::all()` です。製品キューは [toolchain.md](../toolchain.md)。このパッケージでは HOST と、architecture `arm64` の variant（v8 から v9.5）です。
