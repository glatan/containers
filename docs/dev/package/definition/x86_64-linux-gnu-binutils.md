<!-- GENERATED -->
# x86_64-linux-gnu-binutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `x86_64-linux-gnu-binutils` |
| バージョン | 2.47 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap・seed） |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Custom`、レシピ内の個別手順） |
| シード用 pacman | gcc make bison flex texinfo zlib zstd mold |
| コード生成 triple | `x86_64-linux-gnu` |
| 定義 | [`builder/src/package/definition/x86_64_linux_gnu_binutils.rs`](../../../../builder/src/package/definition/x86_64_linux_gnu_binutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/binutils/binutils-2.47.tar.xz |
| sha3-512 | `b2a39d2a1bfff7f27c2619e15c5d9dbf0c7918a85120975f4146c84b4a7d660bd5dc3793ea6d009df89ed865aaef4362805e949438db0f8acf97fd113ca1a1f3` |
| blake3 | `05a142fb2225e617a3fa30224d5d275682e5cf03a62a15af804c03214cc3a5cfadec1215345da15c94ba7a2e13b9667c094e59a26c39412a47265724cbb62a8e` |
| 展開ディレクトリ | `binutils-2.47` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | seed:glibc |
| [zlib](./zlib.md) | 1.3.2 | seed:zlib |
| [zstd](./zstd.md) | 1.5.7 | seed:zstd |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.56 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [mold](./mold.md) | 3.0.0 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | なし |

`make install` には `DESTDIR="$OUT"` を自動付与します。

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

### `${SOURCE}/configure`（1回目の呼出）

- 対象: arm64, ppc64le, riscv64, s390x

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--build` | `${HOST_TRIPLE}` | arm64, ppc64le, riscv64, s390x |
| `--host` | `${PACKAGE_HOST}` | arm64, ppc64le, riscv64, s390x |
| `--target` | `${TRIPLE}` | arm64, ppc64le, riscv64, s390x |
| `--program-prefix` | `${TRIPLE}-` | arm64, ppc64le, riscv64, s390x |
| `--prefix` | `/usr` | arm64, ppc64le, riscv64, s390x |
| `--infodir` | `/usr/share/info/${TRIPLE}` | arm64, ppc64le, riscv64, s390x |
| `--disable-nls` |  | arm64, ppc64le, riscv64, s390x |
| `--disable-werror` |  | arm64, ppc64le, riscv64, s390x |
| `--enable-plugins` |  | arm64, ppc64le, riscv64, s390x |
| `--with-system-zlib` |  | arm64, ppc64le, riscv64, s390x |
| `--without-debuginfod` |  | arm64, ppc64le, riscv64, s390x |
| `--disable-gprofng` |  | arm64, ppc64le, riscv64, s390x |
| `--enable-new-dtags` |  | arm64, ppc64le, riscv64, s390x |
| `--enable-relro` |  | arm64, ppc64le, riscv64, s390x |
| `--enable-default-execstack` | `no` | arm64, ppc64le, riscv64, s390x |
| `--enable-default-hash-style` | `gnu` | arm64, ppc64le, riscv64, s390x |
| `--enable-cet` |  | arm64, ppc64le, riscv64, s390x |

### `${SOURCE}/configure`（2回目の呼出）

- 対象: amd64
- env: `CFLAGS="${CFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic"` `CXXFLAGS="${CXXFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic"` `LDFLAGS="${LDFLAGS:-} -fprofile-generate"`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--build` | `${HOST_TRIPLE}` | amd64 |
| `--host` | `${HOST_TRIPLE}` | amd64 |
| `--target` | `${TRIPLE}` | amd64 |
| `--program-prefix` | `${TRIPLE}-` | amd64 |
| `--prefix` | `/usr` | amd64 |
| `--infodir` | `/usr/share/info/${TRIPLE}` | amd64 |
| `--disable-nls` |  | amd64 |
| `--disable-werror` |  | amd64 |
| `--enable-plugins` |  | amd64 |
| `--with-system-zlib` |  | amd64 |
| `--without-debuginfod` |  | amd64 |
| `--disable-gprofng` |  | amd64 |
| `--enable-new-dtags` |  | amd64 |
| `--enable-relro` |  | amd64 |
| `--enable-default-execstack` | `no` | amd64 |
| `--enable-default-hash-style` | `gnu` | amd64 |
| `--enable-cet` |  | amd64 |

### `${SOURCE}/configure`（3回目の呼出）

- 対象: amd64
- env: `CFLAGS="${CFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"` `CXXFLAGS="${CXXFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"` `LDFLAGS="${LDFLAGS:-} -fprofile-use"`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--build` | `${HOST_TRIPLE}` | amd64 |
| `--host` | `${HOST_TRIPLE}` | amd64 |
| `--target` | `${TRIPLE}` | amd64 |
| `--program-prefix` | `${TRIPLE}-` | amd64 |
| `--prefix` | `/usr` | amd64 |
| `--infodir` | `/usr/share/info/${TRIPLE}` | amd64 |
| `--disable-nls` |  | amd64 |
| `--disable-werror` |  | amd64 |
| `--enable-plugins` |  | amd64 |
| `--with-system-zlib` |  | amd64 |
| `--without-debuginfod` |  | amd64 |
| `--disable-gprofng` |  | amd64 |
| `--enable-new-dtags` |  | amd64 |
| `--enable-relro` |  | amd64 |
| `--enable-default-execstack` | `no` | amd64 |
| `--enable-default-hash-style` | `gnu` | amd64 |
| `--enable-cet` |  | amd64 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 2 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
export PGODIR=/tmp/pgo
rm -rf ${PGODIR}
mkdir -p ${PGODIR} ${BUILD}/gen ${BUILD}/final
cd ${BUILD}/gen
CFLAGS="${CFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" CXXFLAGS="${CXXFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" LDFLAGS="${LDFLAGS:-} -fprofile-generate" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --infodir=/usr/share/info/${TRIPLE} --disable-nls --disable-werror --enable-plugins --with-system-zlib --without-debuginfod --disable-gprofng --enable-new-dtags --enable-relro --enable-default-execstack=no --enable-default-hash-style=gnu --enable-cet
make -j"$(nproc)"
(
  _as=${BUILD}/gen/gas/as-new
  _ld=${BUILD}/gen/ld/ld-new
  _objdump=${BUILD}/gen/binutils/objdump
  mkdir -p /tmp/bu-work
  for _i in $(seq 1 64); do
    printf ".text\n.globl f%s\nf%s:\n  mov $%s, %%eax\n  ret\n" ${_i} ${_i} ${_i} > /tmp/bu-work/t.s
    ${_as} -o /tmp/bu-work/t.o /tmp/bu-work/t.s
    ${_ld} -r -o /tmp/bu-work/t${_i}.o /tmp/bu-work/t.o
  done
  ${_ld} -r -o /tmp/bu-work/all.o /tmp/bu-work/t*.o
  ${_objdump} -d /tmp/bu-work/all.o > /dev/null
  ${_objdump} -h /tmp/bu-work/all.o > /dev/null
) || true
cd ${BUILD}/final
CFLAGS="${CFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" CXXFLAGS="${CXXFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" LDFLAGS="${LDFLAGS:-} -fprofile-use" ${SOURCE}/configure --build=${HOST_TRIPLE} --host=${HOST_TRIPLE} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --infodir=/usr/share/info/${TRIPLE} --disable-nls --disable-werror --enable-plugins --with-system-zlib --without-debuginfod --disable-gprofng --enable-new-dtags --enable-relro --enable-default-execstack=no --enable-default-hash-style=gnu --enable-cet
make -j"$(nproc)"
make install DESTDIR="$OUT"
```

</details>

<details>
<summary>arm64, ppc64le, riscv64, s390x（Canadian）</summary>

```bash
set -euxo pipefail
export CC_FOR_BUILD=${HOST_TRIPLE}-gcc
export CXX_FOR_BUILD=${HOST_TRIPLE}-g++
mkdir -p ${BUILD}
cd ${BUILD}
${SOURCE}/configure --build=${HOST_TRIPLE} --host=${PACKAGE_HOST} --target=${TRIPLE} --program-prefix=${TRIPLE}- --prefix=/usr --infodir=/usr/share/info/${TRIPLE} --disable-nls --disable-werror --enable-plugins --with-system-zlib --without-debuginfod --disable-gprofng --enable-new-dtags --enable-relro --enable-default-execstack=no --enable-default-hash-style=gnu --enable-cet
make -j"$(nproc)"
make install DESTDIR="$OUT"
```

</details>

<!-- /GENERATED -->



## ビルド設定

configure は `--build=$HOST_TRIPLE --host=$HOST_TRIPLE --target=x86_64-linux-gnu` です。ステージ順とプレフィックスは [toolchain.md](../toolchain.md) です。PGO（`Pgo::Custom`）は `cet && !ctx.cross()` でゲートされます。instrumented ツリーで `as` / `ld` / `objdump` をビルドホスト上で駆動してプロファイルを採取するため、生成コードが HOST で動かせる x86_64 triple に限ります。このパッケージは `$TRIPLE` が常に `x86_64-linux-gnu`（= `$HOST_TRIPLE`）なので `cross()` は常に偽であり、全対象 Platform で適用されます。他 triple の `{triple}-binutils` では `cet` が偽のため適用されません。

`supports` は `Target::all()` です。製品キューは [toolchain.md](../toolchain.md)。このパッケージでは HOST と、architecture `amd64` の variant（v1, v2, v3, v4）です。
