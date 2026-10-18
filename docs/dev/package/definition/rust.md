<!-- GENERATED -->
# rust

| 項目 | 値 |
|-|-|
| パッケージ名 | `rust` |
| バージョン | 1.99.0 |
| ライセンス | MIT OR Apache-2.0 |
| 分類 | パッケージ（Bootstrap・seed） |
| 対象 Platform | amd64: v4 |
| host 実行 | 依存として常に BUILD_ENV 側で実行 |
| cc ラッパ | 付与しない |
| 定義 | [`builder/src/package/definition/rust.rs`](../../../../builder/src/package/definition/rust.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://static.rust-lang.org/dist/rustc-1.99.0-src.tar.xz |
| sha3-512 | `9e135098ddcae1040361c6f87dd95aed9d83f7e51d37a7671e67b30bdc4deed7b9c3bd79fc7b646ab5d8edbed6e5abe2a1cf8df499eb1796944abeede0e4dbfd` |
| blake3 | `3ca49872e960518052d1e709cc6860224356ce79f184d8f2b1ef900591ec2695b7d227f169e38204c6e31e42c93ded1f57a283537cf3c25e4b6540bfbbcd6d19` |
| 展開ディレクトリ | `rustc-1.99.0-src` |

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
| [rust](./rust.md) | 1.99.0 | seed:rust |
| [python](./python.md) | 3.15.0 | seed:python |
| [cmake](./cmake.md) | 4.4.4 | seed:cmake |
| [ninja](./ninja.md) | 1.13.2 | seed:ninja |
| [perl](./perl.md) | 5.44.0 | seed:perl |
| [zlib](./zlib.md) | 1.3.2 | seed:zlib |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | seed |
| [aarch64-linux-gnu-gcc](./aarch64-linux-gnu-gcc.md) | 16.2.0 | seed:aarch64-linux-gnu-gcc |
| [aarch64-linux-gnu-binutils](./aarch64-linux-gnu-binutils.md) | 2.47 | seed:aarch64-linux-gnu-binutils |
| [aarch64-linux-gnu-glibc](./aarch64-linux-gnu-glibc.md) | 2.44 | seed:aarch64-linux-gnu-glibc |
| [riscv64-linux-gnu-gcc](./riscv64-linux-gnu-gcc.md) | 16.2.0 | seed:riscv64-linux-gnu-gcc |
| [riscv64-linux-gnu-binutils](./riscv64-linux-gnu-binutils.md) | 2.47 | seed:riscv64-linux-gnu-binutils |
| [riscv64-linux-gnu-glibc](./riscv64-linux-gnu-glibc.md) | 2.44 | seed:riscv64-linux-gnu-glibc |
| [powerpc64le-linux-gnu-gcc](./powerpc64le-linux-gnu-gcc.md) | 16.2.0 | seed |
| [powerpc64le-linux-gnu-binutils](./powerpc64le-linux-gnu-binutils.md) | 2.47 | seed |
| [powerpc64le-linux-gnu-glibc](./powerpc64le-linux-gnu-glibc.md) | 2.44 | seed |
| [s390x-linux-gnu-gcc](./s390x-linux-gnu-gcc.md) | 16.2.0 | seed |
| [s390x-linux-gnu-binutils](./s390x-linux-gnu-binutils.md) | 2.47 | seed |
| [s390x-linux-gnu-glibc](./s390x-linux-gnu-glibc.md) | 2.44 | seed |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | なし |
| ビルドディレクトリ | `$BUILD`（`$SOURCE` をコピーして in-source） |

## ツールチェーンフラグ

`tool_flags` の既定セットとの差分です（[README.md](../README.md) の既定フラグ表）。

| 対象 | 追加 | 除外 |
|-|-|-|
| CFLAGS | — | `-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |
| LDFLAGS | — | `-fuse-ld=mold`<br>`-flto=auto`<br>`-Werror=odr`<br>`-Werror=lto-type-mismatch` |

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で差分はありません。

<details>
<summary>常時</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
# stage0 と cross toolchain を検出し config.toml を生成
sed -i "/if constexpr (std::is_pointer_v/,/static_assert(ConstFn != nullptr)/d" src/llvm-project/llvm/include/llvm/ADT/STLForwardCompat.h
RUSTC=$(command -v rustc)
STAGE0_CARGO=$(command -v cargo)
RUST_TARGETS=\"x86_64-unknown-linux-gnu\"
if [ -d /dep/x86_64-linux-gnu-glibc/usr/x86_64-linux-gnu/lib ] && [ -d /dep/x86_64-linux-gnu-binutils/usr/bin ] && command -v x86_64-linux-gnu-gcc; then
  mkdir -p "/tmp/sysroot-x86_64-linux-gnu/usr" "$BUILD/ccw"
  ln -sfn "/dep/x86_64-linux-gnu-glibc/usr/x86_64-linux-gnu/include" /tmp/sysroot-x86_64-linux-gnu/include
  ln -sfn "/dep/x86_64-linux-gnu-glibc/usr/x86_64-linux-gnu/include" /tmp/sysroot-x86_64-linux-gnu/usr/include
  cp -a "/dep/x86_64-linux-gnu-glibc/usr/x86_64-linux-gnu/lib" /tmp/sysroot-x86_64-linux-gnu/lib
  if [ -e "/dep/x86_64-linux-gnu-glibc/usr/x86_64-linux-gnu/lib64" ]; then
    cp -a "/dep/x86_64-linux-gnu-glibc/usr/x86_64-linux-gnu/lib64" /tmp/sysroot-x86_64-linux-gnu/lib64
  fi
  ln -sfn ../lib /tmp/sysroot-x86_64-linux-gnu/usr/lib
  ln -sfn ../lib /tmp/sysroot-x86_64-linux-gnu/usr/lib64
  cat > $BUILD/ccw/x86_64-linux-gnu-gcc <<'EOF'
#!/bin/sh
exec x86_64-linux-gnu-gcc -B/dep/x86_64-linux-gnu-binutils/usr/x86_64-linux-gnu/bin --sysroot=/tmp/sysroot-x86_64-linux-gnu -idirafter /dep/linux-headers/usr/x86_64-linux-gnu/include "$@"
EOF
  cat > $BUILD/ccw/x86_64-linux-gnu-g++ <<'EOF'
#!/bin/sh
exec x86_64-linux-gnu-g++ -B/dep/x86_64-linux-gnu-binutils/usr/x86_64-linux-gnu/bin --sysroot=/tmp/sysroot-x86_64-linux-gnu -idirafter /dep/linux-headers/usr/x86_64-linux-gnu/include "$@"
EOF
  chmod +x "$BUILD/ccw/x86_64-linux-gnu-gcc" "$BUILD/ccw/x86_64-linux-gnu-g++"
  export CFLAGS="${CFLAGS:-} --sysroot=/tmp/sysroot-x86_64-linux-gnu -idirafter /dep/linux-headers/usr/x86_64-linux-gnu/include"
  export CXXFLAGS="${CXXFLAGS:-} --sysroot=/tmp/sysroot-x86_64-linux-gnu -idirafter /dep/linux-headers/usr/x86_64-linux-gnu/include"
  export LDFLAGS="-L/tmp/sysroot-x86_64-linux-gnu/lib -L/tmp/sysroot-x86_64-linux-gnu/usr/lib -L/tmp/sysroot-x86_64-linux-gnu/lib64 -L/tmp/sysroot-x86_64-linux-gnu/usr/lib64 ${LDFLAGS:-} --sysroot=/tmp/sysroot-x86_64-linux-gnu"
  cat >> $BUILD/targets.conf <<EOF

[target.x86_64-unknown-linux-gnu]
cc = "$BUILD/ccw/x86_64-linux-gnu-gcc"
cxx = "$BUILD/ccw/x86_64-linux-gnu-g++"
ar = "/dep/x86_64-linux-gnu-binutils/usr/bin/x86_64-linux-gnu-ar"
ranlib = "/dep/x86_64-linux-gnu-binutils/usr/bin/x86_64-linux-gnu-ranlib"
linker = "$BUILD/ccw/x86_64-linux-gnu-gcc"
EOF
else
  cat >> $BUILD/targets.conf <<'EOF'

[target.x86_64-unknown-linux-gnu]
cc = "gcc"
cxx = "g++"
ar = "/usr/bin/ar"
ranlib = "/usr/bin/ranlib"
linker = "gcc"
EOF
fi
if [ -d /dep/aarch64-linux-gnu-glibc/usr/aarch64-linux-gnu/lib ] && [ -d /dep/aarch64-linux-gnu-binutils/usr/bin ] && command -v aarch64-linux-gnu-gcc; then
  mkdir -p "/tmp/sysroot-aarch64-linux-gnu/usr" "$BUILD/ccw"
  ln -sfn "/dep/aarch64-linux-gnu-glibc/usr/aarch64-linux-gnu/include" /tmp/sysroot-aarch64-linux-gnu/include
  ln -sfn "/dep/aarch64-linux-gnu-glibc/usr/aarch64-linux-gnu/include" /tmp/sysroot-aarch64-linux-gnu/usr/include
  cp -a "/dep/aarch64-linux-gnu-glibc/usr/aarch64-linux-gnu/lib" /tmp/sysroot-aarch64-linux-gnu/lib
  if [ -e "/dep/aarch64-linux-gnu-glibc/usr/aarch64-linux-gnu/lib64" ]; then
    cp -a "/dep/aarch64-linux-gnu-glibc/usr/aarch64-linux-gnu/lib64" /tmp/sysroot-aarch64-linux-gnu/lib64
  fi
  ln -sfn ../lib /tmp/sysroot-aarch64-linux-gnu/usr/lib
  ln -sfn ../lib /tmp/sysroot-aarch64-linux-gnu/usr/lib64
  cat > $BUILD/ccw/aarch64-linux-gnu-gcc <<'EOF'
#!/bin/sh
exec aarch64-linux-gnu-gcc -B/dep/aarch64-linux-gnu-binutils/usr/aarch64-linux-gnu/bin --sysroot=/tmp/sysroot-aarch64-linux-gnu -idirafter /dep/linux-headers/usr/aarch64-linux-gnu/include "$@"
EOF
  cat > $BUILD/ccw/aarch64-linux-gnu-g++ <<'EOF'
#!/bin/sh
exec aarch64-linux-gnu-g++ -B/dep/aarch64-linux-gnu-binutils/usr/aarch64-linux-gnu/bin --sysroot=/tmp/sysroot-aarch64-linux-gnu -idirafter /dep/linux-headers/usr/aarch64-linux-gnu/include "$@"
EOF
  chmod +x "$BUILD/ccw/aarch64-linux-gnu-gcc" "$BUILD/ccw/aarch64-linux-gnu-g++"
  RUST_TARGETS=${RUST_TARGETS},\"aarch64-unknown-linux-gnu\"
  cat >> $BUILD/targets.conf <<EOF

[target.aarch64-unknown-linux-gnu]
cc = "$BUILD/ccw/aarch64-linux-gnu-gcc"
cxx = "$BUILD/ccw/aarch64-linux-gnu-g++"
ar = "/dep/aarch64-linux-gnu-binutils/usr/bin/aarch64-linux-gnu-ar"
ranlib = "/dep/aarch64-linux-gnu-binutils/usr/bin/aarch64-linux-gnu-ranlib"
linker = "$BUILD/ccw/aarch64-linux-gnu-gcc"
EOF
else
  if command -v aarch64-linux-gnu-gcc; then
    RUST_TARGETS=${RUST_TARGETS},\"aarch64-unknown-linux-gnu\"
    cat >> $BUILD/targets.conf <<'EOF'

[target.aarch64-unknown-linux-gnu]
cc = "aarch64-linux-gnu-gcc"
cxx = "aarch64-linux-gnu-g++"
ar = "/usr/bin/aarch64-linux-gnu-ar"
ranlib = "/usr/bin/aarch64-linux-gnu-ranlib"
linker = "aarch64-linux-gnu-gcc"
EOF
  fi
fi
if [ -d /dep/riscv64-linux-gnu-glibc/usr/riscv64-linux-gnu/lib ] && [ -d /dep/riscv64-linux-gnu-binutils/usr/bin ] && command -v riscv64-linux-gnu-gcc; then
  mkdir -p "/tmp/sysroot-riscv64-linux-gnu/usr" "$BUILD/ccw"
  ln -sfn "/dep/riscv64-linux-gnu-glibc/usr/riscv64-linux-gnu/include" /tmp/sysroot-riscv64-linux-gnu/include
  ln -sfn "/dep/riscv64-linux-gnu-glibc/usr/riscv64-linux-gnu/include" /tmp/sysroot-riscv64-linux-gnu/usr/include
  cp -a "/dep/riscv64-linux-gnu-glibc/usr/riscv64-linux-gnu/lib" /tmp/sysroot-riscv64-linux-gnu/lib
  if [ -e "/dep/riscv64-linux-gnu-glibc/usr/riscv64-linux-gnu/lib64" ]; then
    cp -a "/dep/riscv64-linux-gnu-glibc/usr/riscv64-linux-gnu/lib64" /tmp/sysroot-riscv64-linux-gnu/lib64
  fi
  ln -sfn ../lib /tmp/sysroot-riscv64-linux-gnu/usr/lib
  ln -sfn ../lib /tmp/sysroot-riscv64-linux-gnu/usr/lib64
  cat > $BUILD/ccw/riscv64-linux-gnu-gcc <<'EOF'
#!/bin/sh
exec riscv64-linux-gnu-gcc -B/dep/riscv64-linux-gnu-binutils/usr/riscv64-linux-gnu/bin --sysroot=/tmp/sysroot-riscv64-linux-gnu -idirafter /dep/linux-headers/usr/riscv64-linux-gnu/include "$@"
EOF
  cat > $BUILD/ccw/riscv64-linux-gnu-g++ <<'EOF'
#!/bin/sh
exec riscv64-linux-gnu-g++ -B/dep/riscv64-linux-gnu-binutils/usr/riscv64-linux-gnu/bin --sysroot=/tmp/sysroot-riscv64-linux-gnu -idirafter /dep/linux-headers/usr/riscv64-linux-gnu/include "$@"
EOF
  chmod +x "$BUILD/ccw/riscv64-linux-gnu-gcc" "$BUILD/ccw/riscv64-linux-gnu-g++"
  RUST_TARGETS=${RUST_TARGETS},\"riscv64gc-unknown-linux-gnu\"
  cat >> $BUILD/targets.conf <<EOF

[target.riscv64gc-unknown-linux-gnu]
cc = "$BUILD/ccw/riscv64-linux-gnu-gcc"
cxx = "$BUILD/ccw/riscv64-linux-gnu-g++"
ar = "/dep/riscv64-linux-gnu-binutils/usr/bin/riscv64-linux-gnu-ar"
ranlib = "/dep/riscv64-linux-gnu-binutils/usr/bin/riscv64-linux-gnu-ranlib"
linker = "$BUILD/ccw/riscv64-linux-gnu-gcc"
EOF
else
  if command -v riscv64-linux-gnu-gcc; then
    RUST_TARGETS=${RUST_TARGETS},\"riscv64gc-unknown-linux-gnu\"
    cat >> $BUILD/targets.conf <<'EOF'

[target.riscv64gc-unknown-linux-gnu]
cc = "riscv64-linux-gnu-gcc"
cxx = "riscv64-linux-gnu-g++"
ar = "/usr/bin/riscv64-linux-gnu-ar"
ranlib = "/usr/bin/riscv64-linux-gnu-ranlib"
linker = "riscv64-linux-gnu-gcc"
EOF
  fi
fi
if [ -d /dep/powerpc64le-linux-gnu-glibc/usr/powerpc64le-linux-gnu/lib ] && [ -d /dep/powerpc64le-linux-gnu-binutils/usr/bin ] && command -v powerpc64le-linux-gnu-gcc; then
  mkdir -p "/tmp/sysroot-powerpc64le-linux-gnu/usr" "$BUILD/ccw"
  ln -sfn "/dep/powerpc64le-linux-gnu-glibc/usr/powerpc64le-linux-gnu/include" /tmp/sysroot-powerpc64le-linux-gnu/include
  ln -sfn "/dep/powerpc64le-linux-gnu-glibc/usr/powerpc64le-linux-gnu/include" /tmp/sysroot-powerpc64le-linux-gnu/usr/include
  cp -a "/dep/powerpc64le-linux-gnu-glibc/usr/powerpc64le-linux-gnu/lib" /tmp/sysroot-powerpc64le-linux-gnu/lib
  if [ -e "/dep/powerpc64le-linux-gnu-glibc/usr/powerpc64le-linux-gnu/lib64" ]; then
    cp -a "/dep/powerpc64le-linux-gnu-glibc/usr/powerpc64le-linux-gnu/lib64" /tmp/sysroot-powerpc64le-linux-gnu/lib64
  fi
  ln -sfn ../lib /tmp/sysroot-powerpc64le-linux-gnu/usr/lib
  ln -sfn ../lib /tmp/sysroot-powerpc64le-linux-gnu/usr/lib64
  cat > $BUILD/ccw/powerpc64le-linux-gnu-gcc <<'EOF'
#!/bin/sh
exec powerpc64le-linux-gnu-gcc -B/dep/powerpc64le-linux-gnu-binutils/usr/powerpc64le-linux-gnu/bin --sysroot=/tmp/sysroot-powerpc64le-linux-gnu -idirafter /dep/linux-headers/usr/powerpc64le-linux-gnu/include "$@"
EOF
  cat > $BUILD/ccw/powerpc64le-linux-gnu-g++ <<'EOF'
#!/bin/sh
exec powerpc64le-linux-gnu-g++ -B/dep/powerpc64le-linux-gnu-binutils/usr/powerpc64le-linux-gnu/bin --sysroot=/tmp/sysroot-powerpc64le-linux-gnu -idirafter /dep/linux-headers/usr/powerpc64le-linux-gnu/include "$@"
EOF
  chmod +x "$BUILD/ccw/powerpc64le-linux-gnu-gcc" "$BUILD/ccw/powerpc64le-linux-gnu-g++"
  RUST_TARGETS=${RUST_TARGETS},\"powerpc64le-unknown-linux-gnu\"
  cat >> $BUILD/targets.conf <<EOF

[target.powerpc64le-unknown-linux-gnu]
cc = "$BUILD/ccw/powerpc64le-linux-gnu-gcc"
cxx = "$BUILD/ccw/powerpc64le-linux-gnu-g++"
ar = "/dep/powerpc64le-linux-gnu-binutils/usr/bin/powerpc64le-linux-gnu-ar"
ranlib = "/dep/powerpc64le-linux-gnu-binutils/usr/bin/powerpc64le-linux-gnu-ranlib"
linker = "$BUILD/ccw/powerpc64le-linux-gnu-gcc"
EOF
else
  if command -v powerpc64le-linux-gnu-gcc; then
    RUST_TARGETS=${RUST_TARGETS},\"powerpc64le-unknown-linux-gnu\"
    cat >> $BUILD/targets.conf <<'EOF'

[target.powerpc64le-unknown-linux-gnu]
cc = "powerpc64le-linux-gnu-gcc"
cxx = "powerpc64le-linux-gnu-g++"
ar = "/usr/bin/powerpc64le-linux-gnu-ar"
ranlib = "/usr/bin/powerpc64le-linux-gnu-ranlib"
linker = "powerpc64le-linux-gnu-gcc"
EOF
  fi
fi
if [ -d /dep/s390x-linux-gnu-glibc/usr/s390x-linux-gnu/lib ] && [ -d /dep/s390x-linux-gnu-binutils/usr/bin ] && command -v s390x-linux-gnu-gcc; then
  mkdir -p "/tmp/sysroot-s390x-linux-gnu/usr" "$BUILD/ccw"
  ln -sfn "/dep/s390x-linux-gnu-glibc/usr/s390x-linux-gnu/include" /tmp/sysroot-s390x-linux-gnu/include
  ln -sfn "/dep/s390x-linux-gnu-glibc/usr/s390x-linux-gnu/include" /tmp/sysroot-s390x-linux-gnu/usr/include
  cp -a "/dep/s390x-linux-gnu-glibc/usr/s390x-linux-gnu/lib" /tmp/sysroot-s390x-linux-gnu/lib
  if [ -e "/dep/s390x-linux-gnu-glibc/usr/s390x-linux-gnu/lib64" ]; then
    cp -a "/dep/s390x-linux-gnu-glibc/usr/s390x-linux-gnu/lib64" /tmp/sysroot-s390x-linux-gnu/lib64
  fi
  ln -sfn ../lib /tmp/sysroot-s390x-linux-gnu/usr/lib
  ln -sfn ../lib /tmp/sysroot-s390x-linux-gnu/usr/lib64
  cat > $BUILD/ccw/s390x-linux-gnu-gcc <<'EOF'
#!/bin/sh
exec s390x-linux-gnu-gcc -B/dep/s390x-linux-gnu-binutils/usr/s390x-linux-gnu/bin --sysroot=/tmp/sysroot-s390x-linux-gnu -idirafter /dep/linux-headers/usr/s390x-linux-gnu/include "$@"
EOF
  cat > $BUILD/ccw/s390x-linux-gnu-g++ <<'EOF'
#!/bin/sh
exec s390x-linux-gnu-g++ -B/dep/s390x-linux-gnu-binutils/usr/s390x-linux-gnu/bin --sysroot=/tmp/sysroot-s390x-linux-gnu -idirafter /dep/linux-headers/usr/s390x-linux-gnu/include "$@"
EOF
  chmod +x "$BUILD/ccw/s390x-linux-gnu-gcc" "$BUILD/ccw/s390x-linux-gnu-g++"
  RUST_TARGETS=${RUST_TARGETS},\"s390x-unknown-linux-gnu\"
  cat >> $BUILD/targets.conf <<EOF

[target.s390x-unknown-linux-gnu]
cc = "$BUILD/ccw/s390x-linux-gnu-gcc"
cxx = "$BUILD/ccw/s390x-linux-gnu-g++"
ar = "/dep/s390x-linux-gnu-binutils/usr/bin/s390x-linux-gnu-ar"
ranlib = "/dep/s390x-linux-gnu-binutils/usr/bin/s390x-linux-gnu-ranlib"
linker = "$BUILD/ccw/s390x-linux-gnu-gcc"
EOF
else
  if command -v s390x-linux-gnu-gcc; then
    RUST_TARGETS=${RUST_TARGETS},\"s390x-unknown-linux-gnu\"
    cat >> $BUILD/targets.conf <<'EOF'

[target.s390x-unknown-linux-gnu]
cc = "s390x-linux-gnu-gcc"
cxx = "s390x-linux-gnu-g++"
ar = "/usr/bin/s390x-linux-gnu-ar"
ranlib = "/usr/bin/s390x-linux-gnu-ranlib"
linker = "s390x-linux-gnu-gcc"
EOF
  fi
fi
cat > $BUILD/config.toml <<EOF

[build]
build = "x86_64-unknown-linux-gnu"
host = ["x86_64-unknown-linux-gnu"]
target = [${RUST_TARGETS}]
rustc = "${RUSTC}"
cargo = "${STAGE0_CARGO}"
vendor = true
cargo-native-static = true
docs = false
submodules = false
extended = true
tools = ["cargo"]

[install]
prefix = "/usr"
sysconfdir = "etc"

[llvm]
ninja = true
targets = "X86;AArch64;RISCV;PowerPC;SystemZ"
download-ci-llvm = false
EOF
cat "$BUILD/targets.conf" >> "$BUILD/config.toml"
python3 x.py build --stage 2 -j$(nproc)
DESTDIR="${OUT}" python3 x.py install -j$(nproc)
if [ -d /dep/x86_64-linux-gnu-gcc/usr/lib64 ]; then
  cp -a /dep/x86_64-linux-gnu-gcc/usr/lib64/libstdc++.so.6* /dep/x86_64-linux-gnu-gcc/usr/lib64/libgcc_s.so.1* "$OUT/usr/lib/"
else
  cp -a /usr/lib/libstdc++.so.6* /usr/lib/libgcc_s.so.1* "$OUT/usr/lib/"
fi
if [ -d /dep/zlib/usr/lib64 ]; then
  cp -a /dep/zlib/usr/lib64/libz.so.1* "$OUT/usr/lib/"
else
  cp -a /usr/lib/libz.so.1* "$OUT/usr/lib/"
fi
```

</details>

<!-- /GENERATED -->
