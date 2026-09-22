<!-- GENERATED -->
# zlib-ng

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib-ng` |
| バージョン | 2.3.3 |
| ライセンス | Zlib |
| 分類 | カタログ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Custom`、レシピ内の個別手順） |
| シード用 pacman | cmake |
| 定義 | [`builder/src/package/definition/zlib_ng.rs`](../../../builder/src/package/definition/zlib_ng.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/zlib-ng/zlib-ng/archive/refs/tags/2.3.3.tar.gz |
| sha3-512 | `b045a4d3358f73db5566951944186293b9a99c5bc9d395866bc0a539617b9584b1499d3d0e9a8943284a7b10a829902dd37e1d6ec55823e6203b000644ba0170` |
| blake3 | `103cd4e5aa69bb2d0383a242f231bccf64c8a79c21156e7c6187db3cebc1367dc975216cfa092b7d37cdfb303e1fc9cdb2b63d6ef59ce6dc662fbe89286ada42` |
| 展開ディレクトリ | `zlib-ng-2.3.3` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [cmake](./cmake.md) | 4.4.3 | — |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | なし |

## ツールチェーンフラグ

`tool_flags` の既定セットと同一です（[README.md](../README.md) の既定フラグ表）。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

### `cmake -S "${SOURCE}" -B "${BUILD}"`

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-DCMAKE_SYSTEM_NAME` | `Linux` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SYSTEM_PROCESSOR` | `aarch64` | arm64 |
| `-DCMAKE_C_COMPILER_TARGET` | `aarch64-linux-gnu` | arm64 |
| `-DCMAKE_BUILD_TYPE` | `Release` | 常時 |
| `-DCMAKE_INSTALL_PREFIX` | `/usr` | 常時 |
| `-DCMAKE_INSTALL_LIBDIR` | `lib64` | 常時 |
| `-DZLIB_COMPAT` | `OFF` | 常時 |
| `-DWITH_GTEST` | `OFF` | 常時 |
| `-DBUILD_TESTING` | `OFF` | 常時 |
| `-DBUILD_SHARED_LIBS` | `ON` | 常時 |
| `-DCMAKE_C_COMPILER` | `"${CC}"` | 常時 |
| `-DCMAKE_AR` | `"${AR}"` | 常時 |
| `-DCMAKE_NM` | `"${NM}"` | 常時 |
| `-DCMAKE_RANLIB` | `"${RANLIB}"` | 常時 |
| `-DCMAKE_C_FLAGS` | `"${CFLAGS}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_EXE_LINKER_FLAGS` | `"${LDFLAGS}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SHARED_LINKER_FLAGS` | `"${LDFLAGS}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SYSTEM_PROCESSOR` | `powerpc64le` | ppc64le |
| `-DCMAKE_C_COMPILER_TARGET` | `powerpc64le-linux-gnu` | ppc64le |
| `-DCMAKE_SYSTEM_PROCESSOR` | `riscv64` | riscv64 |
| `-DCMAKE_C_COMPILER_TARGET` | `riscv64-linux-gnu` | riscv64 |
| `-DCMAKE_C_FLAGS` | `"${CFLAGS} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"` | amd64 |
| `-DCMAKE_EXE_LINKER_FLAGS` | `"${LDFLAGS} -fprofile-use"` | amd64 |
| `-DCMAKE_SHARED_LINKER_FLAGS` | `"${LDFLAGS} -fprofile-use"` | amd64 |
| `-DCMAKE_SYSTEM_PROCESSOR` | `s390x` | s390x |
| `-DCMAKE_C_COMPILER_TARGET` | `s390x-linux-gnu` | s390x |

### `cmake -S "${SOURCE}" -B "${BUILD}/gen"`

- 対象: amd64

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-DCMAKE_BUILD_TYPE` | `Release` | amd64 |
| `-DCMAKE_INSTALL_PREFIX` | `/usr` | amd64 |
| `-DCMAKE_INSTALL_LIBDIR` | `lib64` | amd64 |
| `-DZLIB_COMPAT` | `OFF` | amd64 |
| `-DWITH_GTEST` | `OFF` | amd64 |
| `-DBUILD_TESTING` | `ON` | amd64 |
| `-DBUILD_SHARED_LIBS` | `ON` | amd64 |
| `-DCMAKE_C_COMPILER` | `"${CC}"` | amd64 |
| `-DCMAKE_AR` | `"${AR}"` | amd64 |
| `-DCMAKE_NM` | `"${NM}"` | amd64 |
| `-DCMAKE_RANLIB` | `"${RANLIB}"` | amd64 |
| `-DCMAKE_C_FLAGS` | `"${CFLAGS} -fprofile-generate=${PGODIR} -fprofile-update=atomic"` | amd64 |
| `-DCMAKE_EXE_LINKER_FLAGS` | `"${LDFLAGS} -fprofile-generate"` | amd64 |
| `-DCMAKE_SHARED_LINKER_FLAGS` | `"${LDFLAGS} -fprofile-generate"` | amd64 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 5 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
export PGODIR=/tmp/pgo
rm -rf "${PGODIR}"
mkdir -p "${PGODIR}"
cmake -S "${SOURCE}" -B "${BUILD}/gen" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=ON -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS} -fprofile-generate=${PGODIR} -fprofile-update=atomic" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS} -fprofile-generate" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS} -fprofile-generate"
cmake --build "${BUILD}/gen" -j$(nproc)
PGO_LLP_SAVE=${LD_LIBRARY_PATH}
export LD_LIBRARY_PATH="${BUILD}/gen${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}"
(
  for _i in 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16; do
    cat "${SOURCE}"/*.c "${SOURCE}"/*.h
  done > /tmp/pgo-input
  for _level in 1 6 9; do
    "${BUILD}/gen/minideflate" -c "-${_level}" < /tmp/pgo-input | "${BUILD}/gen/minideflate" -d -c > /dev/null
  done
) || true
export LD_LIBRARY_PATH=${PGO_LLP_SAVE}
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=OFF -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS} -fprofile-use" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS} -fprofile-use"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<details>
<summary>arm64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=aarch64 -DCMAKE_C_COMPILER_TARGET=aarch64-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=OFF -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<details>
<summary>ppc64le（クロス・Canadian）</summary>

```bash
set -euxo pipefail
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=powerpc64le -DCMAKE_C_COMPILER_TARGET=powerpc64le-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=OFF -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<details>
<summary>riscv64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=riscv64 -DCMAKE_C_COMPILER_TARGET=riscv64-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=OFF -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<details>
<summary>s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=s390x -DCMAKE_C_COMPILER_TARGET=s390x-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_INSTALL_LIBDIR=lib64 -DZLIB_COMPAT=OFF -DWITH_GTEST=OFF -DBUILD_TESTING=OFF -DBUILD_SHARED_LIBS=ON -DCMAKE_C_COMPILER="${CC}" -DCMAKE_AR="${AR}" -DCMAKE_NM="${NM}" -DCMAKE_RANLIB="${RANLIB}" -DCMAKE_C_FLAGS="${CFLAGS}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<!-- /GENERATED -->



openssl からは依存されません。標準 zlib の `libz.so` とは配置パスを分離して共存可能にします。

## PGO ワークロード

ネイティブビルドのみ適用します（`Pgo::Custom`）。別ツリー `$BUILD/gen` で `-fprofile-generate` の instrumented ビルドを組み、同梱ソースを 16 回連結した複数 MB 入力を `minideflate` で `-1` / `-6` / `-9` の 3 レベルで圧縮・解凍してプロファイルを採取します。`$BUILD` で `-fprofile-use` の成果物を組みます。`ZLIB_COMPAT=OFF` は標準 zlib との競合を防ぐため `libz.so` を出力しない設定です。
