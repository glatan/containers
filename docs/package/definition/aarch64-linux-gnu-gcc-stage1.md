<!-- GENERATED -->
# aarch64-linux-gnu-gcc-stage1

| 項目 | 値 |
|-|-|
| パッケージ名 | `aarch64-linux-gnu-gcc-stage1` |
| バージョン | 16.2.0 |
| ライセンス | GPL-3.0-or-later WITH GCC-exception-3.1 |
| 分類 | ビルド専用（Bootstrap） |
| 対象 Platform | amd64: v4 |
| シード用 pacman | make bison flex perl texinfo m4 gmp mpfr libmpc libisl zstd zlib |
| コード生成 triple | `aarch64-linux-gnu` |
| 定義 | [`builder/src/package/definition/aarch64_linux_gnu_gcc_stage1.rs`](../../../builder/src/package/definition/aarch64_linux_gnu_gcc_stage1.rs) |

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

なし。

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |
| [aarch64-linux-gnu-binutils](./aarch64-linux-gnu-binutils.md) | 2.47 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [gmp](./gmp.md) | 6.3.0 | seed:gmp |
| [mpfr](./mpfr.md) | 4.2.2 | seed:mpfr |
| [mpc](./mpc.md) | 1.4.1 | seed:libmpc |
| [isl](./isl.md) | 0.26 | seed:libisl |
| [zlib](./zlib.md) | 1.3.2 | seed:zlib |
| [zstd](./zstd.md) | 1.5.7 | seed:zstd |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | なし |
| ビルドディレクトリ | なし |

## ツールチェーンフラグ

`CFLAGS`/`LDFLAGS` は設定しません。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--build` | `${HOST_TRIPLE}` | 常時 |
| `--host` | `${HOST_TRIPLE}` | 常時 |
| `--target` | `${TRIPLE}` | 常時 |
| `--program-prefix` | `${TRIPLE}-` | 常時 |
| `--prefix` | `/usr/lib/cross/${TRIPLE}/stage1` | 常時 |
| `--with-sysroot` | `/usr/${TRIPLE}` | 常時 |
| `--with-build-sysroot` | `/usr/${TRIPLE}` | 常時 |
| `--with-native-system-header-dir` | `/include` | 常時 |
| `--with-as` | `/usr/bin/${TRIPLE}-as` | 常時 |
| `--with-ld` | `/usr/bin/${TRIPLE}-ld` | 常時 |
| `--infodir` | `/usr/share/info/${TRIPLE}` | 常時 |
| `--with-system-zlib` |  | 常時 |
| `${GCC_LIB_FLAGS}` |  | 常時 |
| `--with-arch` | `armv8-a` | 常時 |
| `--without-headers` |  | 常時 |
| `--with-newlib` |  | 常時 |
| `--disable-shared` |  | 常時 |
| `--disable-threads` |  | 常時 |
| `--disable-werror` |  | 常時 |
| `--disable-libatomic` |  | 常時 |
| `--disable-libsanitizer` |  | 常時 |
| `--disable-libssp` |  | 常時 |
| `--disable-libquadmath` |  | 常時 |
| `--disable-libgomp` |  | 常時 |
| `--disable-libstdcxx` |  | 常時 |
| `--disable-libvtv` |  | 常時 |
| `--disable-bootstrap` |  | 常時 |
| `--disable-multilib` |  | 常時 |
| `--enable-languages` | `c` | 常時 |
| `--enable-lto` |  | 常時 |
| `--enable-plugin` |  | 常時 |
| `--enable-default-pie` |  | 常時 |
| `--enable-default-ssp` |  | 常時 |
| `--enable-checking` | `release` | 常時 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で差分はありません。

<details>
<summary>常時（クロス）</summary>

```bash
set -euxo pipefail
unset CPP
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
PREFIX=/usr/lib/cross/${TRIPLE}/stage1
SYSROOT=/usr/${TRIPLE}
mkdir -p ${SYSROOT}/include ${BUILD}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
cp -a ${HDR}/. ${SYSROOT}/include/
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
if [ -d /dep/gmp/usr ]; then
  GCC_LIB_FLAGS="--with-gmp=/dep/gmp/usr --with-mpfr=/dep/mpfr/usr --with-mpc=/dep/mpc/usr --with-isl=/dep/isl/usr"
else
  GCC_LIB_FLAGS=--with-isl
fi
if [ -d /dep/zlib/usr/include ]; then
  export CFLAGS=" -I/dep/zlib/usr/include"
  export CXXFLAGS=" -I/dep/zlib/usr/include"
  export LDFLAGS=" -L/dep/zlib/usr/lib64 -L/dep/zlib/usr/lib"
fi
if [ -d /dep/zstd/usr/include ]; then
  export CFLAGS=" -I/dep/zstd/usr/include"
  export CXXFLAGS=" -I/dep/zstd/usr/include"
  export LDFLAGS=" -L/dep/zstd/usr/lib64 -L/dep/zstd/usr/lib"
fi
cd ${BUILD}
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr/lib/cross/${TRIPLE}/stage1 --with-sysroot=/usr/${TRIPLE} --with-build-sysroot=/usr/${TRIPLE} --with-native-system-header-dir=/include --with-as=/usr/bin/${TRIPLE}-as --with-ld=/usr/bin/${TRIPLE}-ld --infodir=/usr/share/info/${TRIPLE} --with-system-zlib ${GCC_LIB_FLAGS} --with-arch=armv8-a --without-headers --with-newlib --disable-shared --disable-threads --disable-werror --disable-libatomic --disable-libsanitizer --disable-libssp --disable-libquadmath --disable-libgomp --disable-libstdcxx --disable-libvtv --disable-bootstrap --disable-multilib --enable-languages=c --enable-lto --enable-plugin --enable-default-pie --enable-default-ssp --enable-checking=release
make -j"$(nproc)" all-gcc all-target-libgcc
make DESTDIR="$OUT" install-gcc install-target-libgcc
```

</details>

<!-- /GENERATED -->




## ビルド設定

C のみ、静的 libgcc。プレフィックスは `/usr/lib/cross/aarch64-linux-gnu/stage1/`。`supports` は HOST（amd64 / v4）のみです。gcc フラグ `--with-arch=armv8-a`。
