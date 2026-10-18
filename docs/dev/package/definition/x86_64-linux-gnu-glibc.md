<!-- GENERATED -->
# x86_64-linux-gnu-glibc

| 項目 | 値 |
|-|-|
| パッケージ名 | `x86_64-linux-gnu-glibc` |
| バージョン | 2.44 |
| ライセンス | LGPL-2.1-or-later |
| 分類 | パッケージ（Bootstrap・seed） |
| 対象 Platform | amd64: v1 v2 v3 v4 |
| シード用 pacman | python bison gawk make mold |
| コード生成 triple | `x86_64-linux-gnu` |
| ターゲット成果物 | 依存マウント時に depender のターゲット成果物を選択 |
| cc ラッパ | 付与しない |
| 定義 | [`builder/src/package/definition/x86_64_linux_gnu_glibc.rs`](../../../../builder/src/package/definition/x86_64_linux_gnu_glibc.rs) |

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
| [linux-headers](./linux-headers.md) | 6.18.56 | — |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | — |
| [x86_64-linux-gnu-gcc-stage2](./x86_64-linux-gnu-gcc-stage2.md) | 16.2.0 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [python](./python.md) | 3.15.0 | seed:python |
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

- env: `BUILD_CC="${HOST_TRIPLE}-gcc"` `CC="${STAGE2}/bin/${TRIPLE}-gcc"` `AR="${TRIPLE}-ar"` `RANLIB="${TRIPLE}-ranlib"` `NM="${TRIPLE}-nm"` `AS="${TRIPLE}-as"` `LD="${TRIPLE}-ld"` `OBJCOPY="${TRIPLE}-objcopy"` `OBJDUMP="${TRIPLE}-objdump"` `READELF="${TRIPLE}-readelf"`
| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/` | 常時 |
| `--host` | `${TRIPLE}` | 常時 |
| `--build` | `${HOST_TRIPLE}` | 常時 |
| `--libdir` | `/lib` | 常時 |
| `--with-headers` | `${SYSROOT}/include` | 常時 |
| `--disable-werror` |  | 常時 |
| `--disable-nscd` |  | 常時 |
| `--disable-profile` |  | 常時 |
| `--enable-stack-protector` | `strong` | 常時 |
| `--enable-bind-now` |  | 常時 |
| `--enable-fortify-source` |  | 常時 |
| `--enable-cet` |  | 常時 |
| `libc_cv_forced_unwind` | `yes` | 常時 |
| `libc_cv_c_cleanup` | `yes` | 常時 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で差分はありません。

<details>
<summary>常時</summary>

```bash
set -euxo pipefail
export PATH="/dep/${TRIPLE}-binutils/usr/bin:${PATH}"
if [ -d /dep/bison/usr/share/bison ]; then
  export BISON_PKGDATADIR=/dep/bison/usr/share/bison
fi
STAGE2=/dep/${TRIPLE}-gcc-stage2/usr/lib/cross/${TRIPLE}/stage2
SYSROOT=${OUT}/usr/${TRIPLE}
mkdir -p ${SYSROOT}/include ${BUILD}
HDR=/dep/linux-headers/usr/${TRIPLE}/include
[ -d ${HDR} ] || HDR=/dep/linux-headers/usr/include
cp -a ${HDR}/. ${SYSROOT}/include/
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-as /usr/bin/${TRIPLE}-as
ln -sfn /dep/${TRIPLE}-binutils/usr/bin/${TRIPLE}-ld /usr/bin/${TRIPLE}-ld
cd ${BUILD}
unset CFLAGS
unset CXXFLAGS
unset LDFLAGS
export CFLAGS="-O2 -pipe -march=${MARCH}"
export CXXFLAGS="${CFLAGS:-}"
unset LD
export CPP="${STAGE2}/bin/${TRIPLE}-gcc -E"
BUILD_CC="${HOST_TRIPLE}-gcc" CC="${STAGE2}/bin/${TRIPLE}-gcc" AR="${TRIPLE}-ar" RANLIB="${TRIPLE}-ranlib" NM="${TRIPLE}-nm" AS="${TRIPLE}-as" LD="${TRIPLE}-ld" OBJCOPY="${TRIPLE}-objcopy" OBJDUMP="${TRIPLE}-objdump" READELF="${TRIPLE}-readelf" ${SOURCE}/configure --prefix=/ --host=${TRIPLE} --build=${HOST_TRIPLE} --libdir=/lib --with-headers=${SYSROOT}/include --disable-werror --disable-nscd --disable-profile --enable-stack-protector=strong --enable-bind-now --enable-fortify-source --enable-cet libc_cv_forced_unwind=yes libc_cv_c_cleanup=yes
echo "build-programs=no" >> configparms
make -j"$(nproc)"
make DESTDIR="$SYSROOT" install
rm -rf ${SYSROOT}/share ${SYSROOT}/etc ${SYSROOT}/var ${SYSROOT}/bin ${SYSROOT}/sbin
for _so in $(find ${SYSROOT} -type f -name "*.so"); do
  if grep -q "GNU ld script" ${_so}; then
    sed -i "s|//lib/|/lib/|g; s|//lib64/|/lib64/|g" ${_so}
  fi
done
ln -sfn lib ${SYSROOT}/lib64
ln -s usr/bin ${OUT}/bin
ln -s usr/sbin ${OUT}/sbin
mkdir -p ${OUT}/lib64 ${OUT}/usr/lib64
ln -sfn /usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2 ${OUT}/lib64/ld-linux-x86-64.so.2 || true
if [ -f ${SYSROOT}/lib/ld-linux-x86-64.so.2 ]; then
  ln -sfn ../usr/x86_64-linux-gnu/lib/ld-linux-x86-64.so.2 ${OUT}/lib64/ld-linux-x86-64.so.2
fi
for so in ${SYSROOT}/lib/libc.so.6 ${SYSROOT}/lib/libm.so.6 ${SYSROOT}/lib/libdl.so.2 ${SYSROOT}/lib/libpthread.so.0 ${SYSROOT}/lib/librt.so.1 ${SYSROOT}/lib/libresolv.so.2; do
  if [ -f ${so} ]; then
    ln -sfn "$(realpath --relative-to=${OUT}/usr/lib64 ${so})" ${OUT}/usr/lib64/$(basename ${so})
  fi
done
```

</details>

<!-- /GENERATED -->

python / bison / m4 のホスト依存は、glibc→python→openssl→glibc の循環を断つため `Via::Seed` です（シード相は distro パッケージが担保し、合成 env 上では成果物が必須）。




## ビルド設定

実行時 libc です。`supports` は architecture `amd64` の variant（v1, v2, v3, v4）です。
