<!-- GENERATED -->
# cmake

| 項目 | 値 |
|-|-|
| パッケージ名 | `cmake` |
| バージョン | 4.4.4 |
| ライセンス | BSD-3-Clause |
| 分類 | パッケージ（Bootstrap） |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| host 実行 | 依存として常に BUILD_ENV 側で実行 |
| 定義 | [`builder/src/package/definition/cmake.rs`](../../../../builder/src/package/definition/cmake.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/Kitware/CMake/releases/download/v4.4.4/cmake-4.4.4.tar.gz |
| sha3-512 | `b7174122a3702699030a6c2712c6b3b4730c5e8ec949837dcf1a6bf7d83a96075490ea6fcffb4faacc8216b6706f53f19cd9a84609482b677080f737f5513cfe` |
| blake3 | `7552b5d0fe236d45d321d7a17d44d1a2d89a68ad11759fa63be905b74f0a33991634803482fb0074c11aabae172c88971326a21e20d2902f2e3dc85a9cf581c4` |
| 展開ディレクトリ | `cmake-4.4.4` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.56 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [mold](./mold.md) | 3.0.0 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::Autotools`（configure のみ `-flto*`・LTO 系 `-Werror=*`・`-f{,no-}hardened` を `sanitize_flags` で除去。`make`/`make install` に `CFLAGS`/`CXXFLAGS`/`LDFLAGS` の変数上書きを自動付与） |
| `-std` 付与 | `-std=gnu17`（CFLAGS 末尾） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（out-of-tree） |

`make install` には `DESTDIR="$OUT"` を自動付与します。

## ツールチェーンフラグ

`tool_flags` の既定セットとの差分です（[README.md](../README.md) の既定フラグ表）。

| 対象 | 追加 | 除外 |
|-|-|-|
| CFLAGS | — | `-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |
| LDFLAGS | — | `-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

### `cmake -S "${SOURCE}" -B "${BUILD}"`

- 対象: arm64, ppc64le, riscv64, s390x

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-DCMAKE_SYSTEM_NAME` | `Linux` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SYSTEM_PROCESSOR` | `aarch64` | arm64 |
| `-DCMAKE_SYSROOT` | `/tmp/sysroot` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_BUILD_TYPE` | `Release` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_INSTALL_PREFIX` | `/usr` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_USE_OPENSSL` | `OFF` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_DISABLE_FIND_PACKAGE_Libidn2` | `ON` | arm64, ppc64le, riscv64, s390x |
| `-DBUILD_TESTING` | `OFF` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_C_COMPILER` | `"${CC}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_CXX_COMPILER` | `"${CXX}"` | arm64, ppc64le, riscv64, s390x |
| `-DCMAKE_SYSTEM_PROCESSOR` | `powerpc64le` | ppc64le |
| `-DCMAKE_SYSTEM_PROCESSOR` | `riscv64` | riscv64 |
| `-DCMAKE_SYSTEM_PROCESSOR` | `s390x` | s390x |

### `"${SOURCE}"/bootstrap`

- 対象: amd64

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | amd64 |
| `--parallel` | `$(nproc)` | amd64 |
| `--` |  | amd64 |
| `-DCMAKE_SYSROOT` | `/tmp/sysroot` | amd64 |
| `-DCMAKE_USE_OPENSSL` | `OFF` | amd64 |
| `-DCMAKE_DISABLE_FIND_PACKAGE_Libidn2` | `ON` | amd64 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 5 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
export FORCE_UNSAFE_CONFIGURE=1
export CFLAGS="${CFLAGS:-} -std=gnu17"
sanitize_flags() {
  printf '%s' "$1" | sed -E 's/-flto(=[^[:space:]]*)?//g; s/-Werror=(odr|lto-type-mismatch|strict-aliasing)//g; s/-f(no-)?hardened//g'
}
autotools_configure() {
  if ! CFLAGS="$(sanitize_flags "${CFLAGS:-}")" \
       CXXFLAGS="$(sanitize_flags "${CXXFLAGS:-}")" \
       LDFLAGS="$(sanitize_flags "${LDFLAGS:-}")" \
       "$SOURCE"/configure --build="$HOST_TRIPLE" --host="${PACKAGE_HOST:-$TRIPLE}" "$@"; then
    if [ -f config.log ]; then
      echo "===== config.log (last 80 lines) ====="
      tail -n 80 config.log
    fi
    return 1
  fi
}
mkdir -p "$BUILD"
cd "$BUILD"
"${SOURCE}"/bootstrap --prefix=/usr --parallel=$(nproc) -- -DCMAKE_SYSROOT=/tmp/sysroot -DCMAKE_USE_OPENSSL=OFF -DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON
make -j"$(nproc)" CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}"
make install CFLAGS="${CFLAGS:-}" CXXFLAGS="${CXXFLAGS:-}" LDFLAGS="${LDFLAGS:-}" DESTDIR="$OUT"
```

</details>

<details>
<summary>arm64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=aarch64 -DCMAKE_SYSROOT=/tmp/sysroot -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_USE_OPENSSL=OFF -DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON -DBUILD_TESTING=OFF -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<details>
<summary>ppc64le（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=powerpc64le -DCMAKE_SYSROOT=/tmp/sysroot -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_USE_OPENSSL=OFF -DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON -DBUILD_TESTING=OFF -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<details>
<summary>riscv64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=riscv64 -DCMAKE_SYSROOT=/tmp/sysroot -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_USE_OPENSSL=OFF -DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON -DBUILD_TESTING=OFF -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<details>
<summary>s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cd "$BUILD"
cmake -S "${SOURCE}" -B "${BUILD}" -DCMAKE_SYSTEM_NAME=Linux -DCMAKE_SYSTEM_PROCESSOR=s390x -DCMAKE_SYSROOT=/tmp/sysroot -DCMAKE_BUILD_TYPE=Release -DCMAKE_INSTALL_PREFIX=/usr -DCMAKE_USE_OPENSSL=OFF -DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON -DBUILD_TESTING=OFF -DCMAKE_C_COMPILER="${CC}" -DCMAKE_CXX_COMPILER="${CXX}"
cmake --build "${BUILD}" -j$(nproc)
DESTDIR="${OUT}" cmake --install "${BUILD}"
```

</details>

<!-- /GENERATED -->



実行時はパッケージ Platform の `{triple}-glibc` と `{triple}-gcc`（libstdc++ / libgcc_s）です。ビルド専用は linux-headers です。上流の `./bootstrap` で C++ だけから組みます。`-DCMAKE_USE_OPENSSL=OFF` と `-DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON` で、シード上の curl 周辺ライブラリを引き込みません。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。最初の `build-env` 合成には入れません。他パッケージがビルド時に実行する cmake は HOST 成果物をマウントします。
