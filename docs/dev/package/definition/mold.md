<!-- GENERATED -->
# mold

| 項目 | 値 |
|-|-|
| パッケージ名 | `mold` |
| バージョン | 2.42.1 |
| ライセンス | MIT |
| 分類 | パッケージ（Bootstrap） |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| PGO | amd64 で適用（`Pgo::Custom`、レシピ内の個別手順） |
| 定義 | [`builder/src/package/definition/mold.rs`](../../../../builder/src/package/definition/mold.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/rui314/mold/archive/refs/tags/v2.42.1.tar.gz |
| sha3-512 | `893446caeb9a904d3d796a8fc3f5c6f6361b7f2ea0d164ee56fcd48a8976857984e10faf51bb4610808faf43d87dd73bb4d02bd022dc6b93b42f80727e258ef4` |
| blake3 | `ab64ef69b9c842942058e678423ce9fb3901f66725295fbd4f426f14af549f62d6f818fa4161ae6a6bdec58d4f27485f65b2ebfb3374e0f3714a81066555ceca` |
| 展開ディレクトリ | `mold-2.42.1` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | — |
| [zlib](./zlib.md) | 1.3.2 | — |
| [zstd](./zstd.md) | 1.5.7 | — |

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
| CFLAGS | — | `-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |
| LDFLAGS | — | `-fuse-ld=mold`<br>`-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

### `cmake -S "${SOURCE}" -B "${BUILD}"`

- 対象: arm64, ppc64le, riscv64, s390x

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-DCMAKE_SYSTEM_NAME` | `Linux` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SYSTEM_PROCESSOR` | `aarch64` | arm64 |
| `-DCMAKE_C_COMPILER_TARGET` | `aarch64-linux-gnu` | arm64 |
| `-DCMAKE_CXX_COMPILER_TARGET` | `aarch64-linux-gnu` | arm64 |
| `-DCMAKE_BUILD_TYPE` | `Release` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_INSTALL_PREFIX` | `/usr` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_C_COMPILER` | `"${CC}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_CXX_COMPILER` | `"${CXX}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_C_FLAGS` | `"${CFLAGS:-}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_CXX_FLAGS` | `"${CXXFLAGS:-}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_EXE_LINKER_FLAGS` | `"${LDFLAGS:-}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SHARED_LINKER_FLAGS` | `"${LDFLAGS:-}"` | arm64, ppc64le, riscv64, s390x |
| `-DMOLD_USE_MIMALLOC` | `ON` | arm64, ppc64le, riscv64, s390x |
| `-DMOLD_USE_SYSTEM_MIMALLOC` | `OFF` | arm64, ppc64le, riscv64, s390x |
| `-DMOLD_USE_SYSTEM_TBB` | `OFF` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SYSTEM_PROCESSOR` | `powerpc64le` | ppc64le |
| `-DCMAKE_C_COMPILER_TARGET` | `powerpc64le-linux-gnu` | ppc64le |
| `-DCMAKE_CXX_COMPILER_TARGET` | `powerpc64le-linux-gnu` | ppc64le |
| `-DCMAKE_SYSTEM_PROCESSOR` | `riscv64` | riscv64 |
| `-DCMAKE_C_COMPILER_TARGET` | `riscv64-linux-gnu` | riscv64 |
| `-DCMAKE_CXX_COMPILER_TARGET` | `riscv64-linux-gnu` | riscv64 |
| `-DCMAKE_SYSTEM_PROCESSOR` | `s390x` | s390x |
| `-DCMAKE_C_COMPILER_TARGET` | `s390x-linux-gnu` | s390x |
| `-DCMAKE_CXX_COMPILER_TARGET` | `s390x-linux-gnu` | s390x |

### `cmake -S "${SOURCE}" -B "${BUILD}/gen"`

- 対象: amd64

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-DCMAKE_BUILD_TYPE` | `Release` | amd64 |
| `-DCMAKE_INSTALL_PREFIX` | `/usr` | amd64 |
| `-DCMAKE_C_COMPILER` | `"${CC}"` | amd64 |
| `-DCMAKE_CXX_COMPILER` | `"${CXX}"` | amd64 |
| `-DCMAKE_C_FLAGS` | `"${CFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic"` | amd64 |
| `-DCMAKE_CXX_FLAGS` | `"${CXXFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic"` | amd64 |
| `-DCMAKE_EXE_LINKER_FLAGS` | `"${LDFLAGS:-} -fprofile-generate"` | amd64 |
| `-DCMAKE_SHARED_LINKER_FLAGS` | `"${LDFLAGS:-} -fprofile-generate"` | amd64 |
| `-DMOLD_USE_MIMALLOC` | `ON` | amd64 |
| `-DMOLD_USE_SYSTEM_MIMALLOC` | `OFF` | amd64 |
| `-DMOLD_USE_SYSTEM_TBB` | `OFF` | amd64 |

### `cmake -S "${SOURCE}" -B "${BUILD}/use"`

- 対象: amd64

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-DCMAKE_BUILD_TYPE` | `Release` | amd64 |
| `-DCMAKE_INSTALL_PREFIX` | `/usr` | amd64 |
| `-DCMAKE_C_COMPILER` | `"${CC}"` | amd64 |
| `-DCMAKE_CXX_COMPILER` | `"${CXX}"` | amd64 |
| `-DCMAKE_C_FLAGS` | `"${CFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"` | amd64 |
| `-DCMAKE_CXX_FLAGS` | `"${CXXFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch"` | amd64 |
| `-DCMAKE_EXE_LINKER_FLAGS` | `"${LDFLAGS:-} -fprofile-use"` | amd64 |
| `-DCMAKE_SHARED_LINKER_FLAGS` | `"${LDFLAGS:-} -fprofile-use"` | amd64 |
| `-DMOLD_USE_MIMALLOC` | `ON` | amd64 |
| `-DMOLD_USE_SYSTEM_MIMALLOC` | `OFF` | amd64 |
| `-DMOLD_USE_SYSTEM_TBB` | `OFF` | amd64 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 5 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
CMAKE_PREFIX=
for _dep in zlib zstd; do
  if [ -d /dep/${_dep}/usr ]; then
    CMAKE_PREFIX="${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/${_dep}/usr"
  fi
done
if [ -n "${CMAKE_PREFIX:-}" ]; then
  export CMAKE_PREFIX_PATH="${CMAKE_PREFIX}${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
fi
if [ -d /dep/cmake/usr/bin ]; then
  export PATH="/dep/cmake/usr/bin:${PATH}"
fi
export PGODIR=/tmp/pgo
rm -rf "${PGODIR}"
mkdir -p "${PGODIR}"
cmake -S "${SOURCE}" -B "${BUILD}/gen" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-} -fprofile-generate=${PGODIR} -fprofile-update=atomic" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-} -fprofile-generate" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-} -fprofile-generate" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}/gen" -j$(nproc)
(
  mkdir -p /tmp/moldbin /tmp/moldwork
  ln -sf "${BUILD}/gen/mold" /tmp/moldbin/ld.mold
  for _i in 1 2 3 4 5; do
    printf 'int fn%s(void){return %s;}\n' "${_i}" "${_i}" | "${CC}" -x c - -c -o "/tmp/moldwork/t${_i}.o"
    "${CC}" -B/tmp/moldbin -fuse-ld=mold -shared -o "/tmp/moldwork/t${_i}.so" "/tmp/moldwork/t${_i}.o"
  done
  printf 'int main(void){return fn1();}\nextern int fn1(void);\n' | "${CC}" -x c - -c -o "/tmp/moldwork/main.o"
  "${CC}" -B/tmp/moldbin -fuse-ld=mold -o "/tmp/moldwork/prog" "/tmp/moldwork/main.o" "/tmp/moldwork/t1.o"
  "${CC}" -B/tmp/moldbin -fuse-ld=mold -pie -o "/tmp/moldwork/prog-pie" "/tmp/moldwork/main.o" "/tmp/moldwork/t1.o"
) || true
cmake -S "${SOURCE}" -B "${BUILD}/use" -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-} -fprofile-use=${PGODIR} -fprofile-correction -Wno-missing-profile -Wno-error=coverage-mismatch" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-} -fprofile-use" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-} -fprofile-use" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}/use" -j$(nproc)
cmake --install "${BUILD}/use" --prefix "${OUT}/usr"
```

</details>

<details>
<summary>arm64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
CMAKE_PREFIX=
for _dep in zlib zstd; do
  if [ -d /dep/${_dep}/usr ]; then
    CMAKE_PREFIX="${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/${_dep}/usr"
  fi
done
if [ -n "${CMAKE_PREFIX:-}" ]; then
  export CMAKE_PREFIX_PATH="${CMAKE_PREFIX}${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
fi
if [ -d /dep/cmake/usr/bin ]; then
  export PATH="/dep/cmake/usr/bin:${PATH}"
fi
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=aarch64 -DCMAKE_C_COMPILER_TARGET=aarch64-linux-gnu -DCMAKE_CXX_COMPILER_TARGET=aarch64-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-}" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-}" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}" -j$(nproc)
cmake --install "${BUILD}" --prefix "${OUT}/usr"
```

</details>

<details>
<summary>ppc64le（クロス・Canadian）</summary>

```bash
set -euxo pipefail
CMAKE_PREFIX=
for _dep in zlib zstd; do
  if [ -d /dep/${_dep}/usr ]; then
    CMAKE_PREFIX="${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/${_dep}/usr"
  fi
done
if [ -n "${CMAKE_PREFIX:-}" ]; then
  export CMAKE_PREFIX_PATH="${CMAKE_PREFIX}${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
fi
if [ -d /dep/cmake/usr/bin ]; then
  export PATH="/dep/cmake/usr/bin:${PATH}"
fi
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=powerpc64le -DCMAKE_C_COMPILER_TARGET=powerpc64le-linux-gnu -DCMAKE_CXX_COMPILER_TARGET=powerpc64le-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-}" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-}" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}" -j$(nproc)
cmake --install "${BUILD}" --prefix "${OUT}/usr"
```

</details>

<details>
<summary>riscv64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
CMAKE_PREFIX=
for _dep in zlib zstd; do
  if [ -d /dep/${_dep}/usr ]; then
    CMAKE_PREFIX="${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/${_dep}/usr"
  fi
done
if [ -n "${CMAKE_PREFIX:-}" ]; then
  export CMAKE_PREFIX_PATH="${CMAKE_PREFIX}${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
fi
if [ -d /dep/cmake/usr/bin ]; then
  export PATH="/dep/cmake/usr/bin:${PATH}"
fi
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=riscv64 -DCMAKE_C_COMPILER_TARGET=riscv64-linux-gnu -DCMAKE_CXX_COMPILER_TARGET=riscv64-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-}" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-}" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}" -j$(nproc)
cmake --install "${BUILD}" --prefix "${OUT}/usr"
```

</details>

<details>
<summary>s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
CMAKE_PREFIX=
for _dep in zlib zstd; do
  if [ -d /dep/${_dep}/usr ]; then
    CMAKE_PREFIX="${CMAKE_PREFIX:+$CMAKE_PREFIX:}/dep/${_dep}/usr"
  fi
done
if [ -n "${CMAKE_PREFIX:-}" ]; then
  export CMAKE_PREFIX_PATH="${CMAKE_PREFIX}${CMAKE_PREFIX_PATH:+:$CMAKE_PREFIX_PATH}"
fi
if [ -d /dep/cmake/usr/bin ]; then
  export PATH="/dep/cmake/usr/bin:${PATH}"
fi
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=s390x -DCMAKE_C_COMPILER_TARGET=s390x-linux-gnu -DCMAKE_CXX_COMPILER_TARGET=s390x-linux-gnu -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}" -DCMAKE_C_FLAGS="${CFLAGS:-}" -DCMAKE_CXX_FLAGS="${CXXFLAGS:-}" -DCMAKE_EXE_LINKER_FLAGS="${LDFLAGS:-}" -DCMAKE_SHARED_LINKER_FLAGS="${LDFLAGS:-}" -DMOLD_USE_MIMALLOC=ON -DMOLD_USE_SYSTEM_MIMALLOC=OFF -DMOLD_USE_SYSTEM_TBB=OFF
cmake --build "${BUILD}" -j$(nproc)
cmake --install "${BUILD}" --prefix "${OUT}/usr"
```

</details>

<!-- /GENERATED -->



実行時はパッケージ Platform の `{triple}-glibc`、`{triple}-gcc`（libstdc++ / libgcc_s / libatomic）、zlib、zstd です。ビルド専用は linux-headers と cmake です。自分自身には `-fuse-ld=mold` を付けません。シード中のリンクは Arch の mold です。compose 後の標準リンカは自前 mold です。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。他パッケージがビルド時に実行する cmake は HOST 成果物をマウントします。


## 補足

自身を mold でリンクしないため `Ldflag::FuseLdMold` をフラグから除外します（上記ツールチェーンフラグ表参照）。

PGO はネイティブビルドのみ適用します（`Pgo` ポリシーはメタデータ表を参照）。`-fprofile-generate` の instrumented ビルドを組み、その mold でオブジェクトのリンクを実行してプロファイルを採取します。`-fprofile-use` の成果物を別ツリーで組みます。
