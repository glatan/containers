<!-- GENERATED -->
# s390x-linux-gnu-glibc-headers

| 項目 | 値 |
|-|-|
| パッケージ名 | `s390x-linux-gnu-glibc-headers` |
| バージョン | 2.44 |
| ライセンス | LGPL-2.1-or-later |
| 分類 | ビルド専用（Bootstrap） |
| 対象 Platform | amd64: v4 |
| シード用 pacman | python bison gawk make mold |
| コード生成 triple | `s390x-linux-gnu` |
| ターゲット成果物 | 依存マウント時に depender のターゲット成果物を選択 |
| 定義 | [`builder/src/package/definition/s390x_linux_gnu_glibc_headers.rs`](../../../builder/src/package/definition/s390x_linux_gnu_glibc_headers.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz |
| sha3-512 | `263dfb5e331b2033368c7455abcd9a9b15ab098ac963d5518ecf4bcb9eeb41298eb9976bf78a64af67d9471a50161bced2fa69d429c69759ea57f76b942ac521` |
| blake3 | `6d07bc16d8d066b1c0ae713d762175f261e27df9cab8015ec11e1c980c05e79b0fd5bab6cde6a90bf07651c432ca9039629376b57a96c9f3dcb3765f3745dbf4` |
| 展開ディレクトリ | `glibc-2.44` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

なし。

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |
| [s390x-linux-gnu-binutils](./s390x-linux-gnu-binutils.md) | 2.47 | — |
| [s390x-linux-gnu-gcc-stage1](./s390x-linux-gnu-gcc-stage1.md) | 16.2.0 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [python](./python.md) | 3.14.7 | seed:python |
| [bison](./bison.md) | 3.8.2 | seed:bison |
| [m4](./m4.md) | 1.4.21 | seed:m4 |

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

- env: `BUILD_CC="${HOST_TRIPLE}-gcc"` `CC="${STAGE1}/bin/${TRIPLE}-gcc"` `AR="${TRIPLE}-ar"` `RANLIB="${TRIPLE}-ranlib"` `NM="${TRIPLE}-nm"` `OBJCOPY="${TRIPLE}-objcopy"` `OBJDUMP="${TRIPLE}-objdump"` `READELF="${TRIPLE}-readelf"`
| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/` | 常時 |
| `--host` | `${TRIPLE}` | 常時 |
| `--build` | `${HOST_TRIPLE}` | 常時 |
| `--with-headers` | `${SYSROOT}/include` | 常時 |
| `--disable-werror` |  | 常時 |
| `libc_cv_forced_unwind` | `yes` | 常時 |
| `libc_cv_c_cleanup` | `yes` | 常時 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で差分はありません。

<details>
<summary>常時（クロス）</summary>

```bash
set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
STAGE1=/dep/${TRIPLE}-gcc-stage1/usr/lib/cross/${TRIPLE}/stage1
SYSROOT=${OUT}/usr/${TRIPLE}
mkdir -p ${SYSROOT}/include ${SYSROOT}/lib ${BUILD}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
cp -a ${HDR}/. ${SYSROOT}/include/
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
cd ${BUILD}
unset CFLAGS
unset CXXFLAGS
unset LDFLAGS
export CPP="${STAGE1}/bin/${TRIPLE}-gcc -E"
BUILD_CC="${HOST_TRIPLE}-gcc" CC="${STAGE1}/bin/${TRIPLE}-gcc" AR="${TRIPLE}-ar" RANLIB="${TRIPLE}-ranlib" NM="${TRIPLE}-nm" OBJCOPY="${TRIPLE}-objcopy" OBJDUMP="${TRIPLE}-objdump" READELF="${TRIPLE}-readelf" ${SOURCE}/configure --prefix=/ --host=${TRIPLE} --build=${HOST_TRIPLE} --with-headers=${SYSROOT}/include --disable-werror libc_cv_forced_unwind=yes libc_cv_c_cleanup=yes
make csu/subdir_lib
make install-bootstrap-headers=yes install-headers DESTDIR="$SYSROOT"
install -m644 csu/crt1.o csu/crti.o csu/crtn.o ${SYSROOT}/lib/
${STAGE1}/bin/${TRIPLE}-gcc -nostdlib -nostartfiles -shared -x c /dev/null -o ${SYSROOT}/lib/libc.so
mkdir -p ${SYSROOT}/include/gnu
touch ${SYSROOT}/include/gnu/stubs.h
ln -sf lib $SYSROOT/lib64
```

</details>

<!-- /GENERATED -->




## ビルド設定

ヘッダ、crt、スタブ libc.so。`supports` は HOST（amd64 / v4）のみです。
