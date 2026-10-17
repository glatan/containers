<!-- GENERATED -->
# mold

| 項目 | 値 |
|-|-|
| パッケージ名 | `mold` |
| バージョン | 3.0.0 |
| ライセンス | MIT |
| 分類 | パッケージ（Bootstrap） |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| 定義 | [`builder/src/package/definition/mold.rs`](../../../../builder/src/package/definition/mold.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/rui314/mold/archive/refs/tags/v3.0.0.tar.gz |
| sha3-512 | `cd0ccfd5957fbc1329e1a8d0b829326a8a13d5b005e07a4d70a506e1bd775c0eb4b3cd46b3ecfbd78b7418c2e26981c7839fbd96991806d641ff9802229afd41` |
| blake3 | `6a14f3ebf8e69e73d001b33add174f6b838af6e5557779c952ee74ebeb173a5026a8df100b7663614ffd3651807902e64b33075d052951adcdcbd92eea7f95a3` |
| 展開ディレクトリ | `mold-3.0.0` |

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
| [rust](./rust.md) | 1.99.0 | — |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | あり（`--sysroot`/`-idirafter` は `wrap_script` が注入） |
| ビルドディレクトリ | `$BUILD`（`$SOURCE` をコピーして in-source） |

## ツールチェーンフラグ

`tool_flags` の既定セットとの差分です（[README.md](../README.md) の既定フラグ表）。

| 対象 | 追加 | 除外 |
|-|-|-|
| CFLAGS | — | `-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |
| LDFLAGS | — | `-fuse-ld=mold`<br>`-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で 5 系統に分かれます。

<details>
<summary>amd64</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
mkdir -p "$BUILD/.cargo"
mkdir -p /build/ccw
cat > /build/ccw/mold-link <<'EOF'
#!/bin/sh
exec "$CC" --sysroot=/tmp/sysroot -L/tmp/sysroot/lib -L/tmp/sysroot/lib64 "$@"
EOF
chmod +x /build/ccw/mold-link
cat > $BUILD/.cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/rui314/mimalloc_rust?rev=3979460494f1cd1e7f936cb8e10f41e927c9f698"]
git = "https://github.com/rui314/mimalloc_rust"
rev = "3979460494f1cd1e7f936cb8e10f41e927c9f698"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/src/vendor"
EOF
CARGO_HOME="${BUILD}/cargo-home" CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/build/ccw/mold-link CC_x86_64_unknown_linux_gnu=${CC} CXX_x86_64_unknown_linux_gnu=${CXX} AR_x86_64_unknown_linux_gnu=${AR} CFLAGS_x86_64_unknown_linux_gnu=${CFLAGS} cargo build --release --locked --offline
DESTDIR="${OUT}" PREFIX=/usr MOLD_LIBDIR=/usr/lib sh "$BUILD/install-mold.sh"
```

</details>

<details>
<summary>arm64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
mkdir -p "$BUILD/.cargo"
mkdir -p /build/ccw
cat > /build/ccw/mold-link <<'EOF'
#!/bin/sh
exec "$CC" --sysroot=/tmp/sysroot -L/tmp/sysroot/lib -L/tmp/sysroot/lib64 "$@"
EOF
chmod +x /build/ccw/mold-link
cat > $BUILD/.cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/rui314/mimalloc_rust?rev=3979460494f1cd1e7f936cb8e10f41e927c9f698"]
git = "https://github.com/rui314/mimalloc_rust"
rev = "3979460494f1cd1e7f936cb8e10f41e927c9f698"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/src/vendor"
EOF
CARGO_HOME="${BUILD}/cargo-home" CARGO_TARGET_AARCH64_UNKNOWN_LINUX_GNU_LINKER=/build/ccw/mold-link CC_aarch64_unknown_linux_gnu=${CC} CXX_aarch64_unknown_linux_gnu=${CXX} AR_aarch64_unknown_linux_gnu=${AR} CFLAGS_aarch64_unknown_linux_gnu=${CFLAGS} CC_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-gcc" CXX_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-g++" AR_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-ar" HOST_CC="${HOST_TRIPLE}-gcc" HOST_CXX="${HOST_TRIPLE}-g++" CFLAGS_x86_64_unknown_linux_gnu= CXXFLAGS_x86_64_unknown_linux_gnu= LDFLAGS_x86_64_unknown_linux_gnu= cargo build --release --locked --offline --target aarch64-unknown-linux-gnu
DESTDIR="${OUT}" PREFIX=/usr MOLD_LIBDIR=/usr/lib CARGO_TARGET_DIR="$BUILD/target/aarch64-unknown-linux-gnu" sh "$BUILD/install-mold.sh"
```

</details>

<details>
<summary>ppc64le（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
mkdir -p "$BUILD/.cargo"
mkdir -p /build/ccw
cat > /build/ccw/mold-link <<'EOF'
#!/bin/sh
exec "$CC" --sysroot=/tmp/sysroot -L/tmp/sysroot/lib -L/tmp/sysroot/lib64 "$@"
EOF
chmod +x /build/ccw/mold-link
cat > $BUILD/.cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/rui314/mimalloc_rust?rev=3979460494f1cd1e7f936cb8e10f41e927c9f698"]
git = "https://github.com/rui314/mimalloc_rust"
rev = "3979460494f1cd1e7f936cb8e10f41e927c9f698"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/src/vendor"
EOF
CARGO_HOME="${BUILD}/cargo-home" CARGO_TARGET_POWERPC64LE_UNKNOWN_LINUX_GNU_LINKER=/build/ccw/mold-link CC_powerpc64le_unknown_linux_gnu=${CC} CXX_powerpc64le_unknown_linux_gnu=${CXX} AR_powerpc64le_unknown_linux_gnu=${AR} CFLAGS_powerpc64le_unknown_linux_gnu=${CFLAGS} CC_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-gcc" CXX_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-g++" AR_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-ar" HOST_CC="${HOST_TRIPLE}-gcc" HOST_CXX="${HOST_TRIPLE}-g++" CFLAGS_x86_64_unknown_linux_gnu= CXXFLAGS_x86_64_unknown_linux_gnu= LDFLAGS_x86_64_unknown_linux_gnu= cargo build --release --locked --offline --target powerpc64le-unknown-linux-gnu
DESTDIR="${OUT}" PREFIX=/usr MOLD_LIBDIR=/usr/lib CARGO_TARGET_DIR="$BUILD/target/powerpc64le-unknown-linux-gnu" sh "$BUILD/install-mold.sh"
```

</details>

<details>
<summary>riscv64（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
mkdir -p "$BUILD/.cargo"
mkdir -p /build/ccw
cat > /build/ccw/mold-link <<'EOF'
#!/bin/sh
exec "$CC" --sysroot=/tmp/sysroot -L/tmp/sysroot/lib -L/tmp/sysroot/lib64 "$@"
EOF
chmod +x /build/ccw/mold-link
cat > $BUILD/.cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/rui314/mimalloc_rust?rev=3979460494f1cd1e7f936cb8e10f41e927c9f698"]
git = "https://github.com/rui314/mimalloc_rust"
rev = "3979460494f1cd1e7f936cb8e10f41e927c9f698"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/src/vendor"
EOF
CARGO_HOME="${BUILD}/cargo-home" CARGO_TARGET_RISCV64GC_UNKNOWN_LINUX_GNU_LINKER=/build/ccw/mold-link CC_riscv64gc_unknown_linux_gnu=${CC} CXX_riscv64gc_unknown_linux_gnu=${CXX} AR_riscv64gc_unknown_linux_gnu=${AR} CFLAGS_riscv64gc_unknown_linux_gnu=${CFLAGS} CC_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-gcc" CXX_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-g++" AR_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-ar" HOST_CC="${HOST_TRIPLE}-gcc" HOST_CXX="${HOST_TRIPLE}-g++" CFLAGS_x86_64_unknown_linux_gnu= CXXFLAGS_x86_64_unknown_linux_gnu= LDFLAGS_x86_64_unknown_linux_gnu= cargo build --release --locked --offline --target riscv64gc-unknown-linux-gnu
DESTDIR="${OUT}" PREFIX=/usr MOLD_LIBDIR=/usr/lib CARGO_TARGET_DIR="$BUILD/target/riscv64gc-unknown-linux-gnu" sh "$BUILD/install-mold.sh"
```

</details>

<details>
<summary>s390x（クロス・Canadian）</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
mkdir -p "$BUILD/.cargo"
mkdir -p /build/ccw
cat > /build/ccw/mold-link <<'EOF'
#!/bin/sh
exec "$CC" --sysroot=/tmp/sysroot -L/tmp/sysroot/lib -L/tmp/sysroot/lib64 "$@"
EOF
chmod +x /build/ccw/mold-link
cat > $BUILD/.cargo/config.toml <<'EOF'
[source.crates-io]
replace-with = "vendored-sources"

[source."git+https://github.com/rui314/mimalloc_rust?rev=3979460494f1cd1e7f936cb8e10f41e927c9f698"]
git = "https://github.com/rui314/mimalloc_rust"
rev = "3979460494f1cd1e7f936cb8e10f41e927c9f698"
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "/src/vendor"
EOF
CARGO_HOME="${BUILD}/cargo-home" CARGO_TARGET_S390X_UNKNOWN_LINUX_GNU_LINKER=/build/ccw/mold-link CC_s390x_unknown_linux_gnu=${CC} CXX_s390x_unknown_linux_gnu=${CXX} AR_s390x_unknown_linux_gnu=${AR} CFLAGS_s390x_unknown_linux_gnu=${CFLAGS} CC_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-gcc" CXX_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-g++" AR_x86_64_unknown_linux_gnu="${HOST_TRIPLE}-ar" HOST_CC="${HOST_TRIPLE}-gcc" HOST_CXX="${HOST_TRIPLE}-g++" CFLAGS_x86_64_unknown_linux_gnu= CXXFLAGS_x86_64_unknown_linux_gnu= LDFLAGS_x86_64_unknown_linux_gnu= cargo build --release --locked --offline --target s390x-unknown-linux-gnu
DESTDIR="${OUT}" PREFIX=/usr MOLD_LIBDIR=/usr/lib CARGO_TARGET_DIR="$BUILD/target/s390x-unknown-linux-gnu" sh "$BUILD/install-mold.sh"
```

</details>

<!-- /GENERATED -->



実行時はパッケージ Platform の `{triple}-glibc`、`{triple}-gcc`（libstdc++ / libgcc_s / libatomic）、zlib、zstd です。ビルド専用は linux-headers と cmake です。自分自身には `-fuse-ld=mold` を付けません。シード中のリンクは Arch の mold です。compose 後の標準リンカは自前 mold です。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。他パッケージがビルド時に実行する cmake は HOST 成果物をマウントします。


## 補足

自身を mold でリンクしないため `Ldflag::FuseLdMold` をフラグから除外します（上記ツールチェーンフラグ表参照）。

PGO はネイティブビルドのみ適用します（`Pgo` ポリシーはメタデータ表を参照）。`-fprofile-generate` の instrumented ビルドを組み、その mold でオブジェクトのリンクを実行してプロファイルを採取します。`-fprofile-use` の成果物を別ツリーで組みます。
