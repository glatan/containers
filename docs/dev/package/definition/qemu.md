<!-- GENERATED -->
# qemu

| 項目 | 値 |
|-|-|
| パッケージ名 | `qemu` |
| バージョン | 11.1.1 |
| ライセンス | GPL-2.0-only |
| 分類 | パッケージ |
| 対象 Platform | amd64: v4 |
| host 実行 | 依存として常に BUILD_ENV 側で実行 |
| 定義 | [`builder/src/package/definition/qemu.rs`](../../../../builder/src/package/definition/qemu.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://download.qemu.org/qemu-11.1.1.tar.xz |
| sha3-512 | `8b53993040fe8ce2e6c69e2d616a952fcd3da00ddcd114b64dea2d5b1cc0a4f6df03bc899ba12437b6166559be8f2379b97a737608659f6b2c6ab56605bd20a2` |
| blake3 | `9cda9512c997d0ec176add30ff4f4531d4882681eb12e98ba5c7d11cb4cf610ca079689e5def5ae36c1c1a173dee7eb001ce8fffeba77f418048e4b4773dff67` |
| 展開ディレクトリ | `qemu-11.1.1` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [glib](./glib.md) | 2.90.0 | — |
| [zlib](./zlib.md) | 1.3.2 | — |
| [zstd](./zstd.md) | 1.5.7 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |
| [pcre2](./pcre2.md) | 10.48 | — |
| [libffi](./libffi.md) | 3.8.0 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [python](./python.md) | 3.14.7 | — |
| [meson](./meson.md) | 1.12.0 | — |
| [ninja](./ninja.md) | 1.13.2 | — |
| [pkgconf](./pkgconf.md) | 2.5.1 | — |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | なし |
| ビルドディレクトリ | `$BUILD`（`$SOURCE` をコピーして in-source） |

`make install` には `DESTDIR="$OUT"` を自動付与します。

## ツールチェーンフラグ

`CFLAGS`/`LDFLAGS` は設定しません。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルドフラグ

対象 Platform 全件で `recipe()` を評価した configure 系呼出の引数です。条件列は引数が出現するターゲット集合です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--target-list` | `x86_64-linux-user,aarch64-linux-user,riscv64-linux-user,ppc64le-linux-user,s390x-linux-user` | 常時 |
| `--enable-tcg` |  | 常時 |
| `--disable-system` |  | 常時 |
| `--disable-tools` |  | 常時 |
| `--disable-docs` |  | 常時 |
| `--disable-guest-agent` |  | 常時 |
| `--disable-slirp` |  | 常時 |
| `--disable-vnc` |  | 常時 |
| `--disable-sdl` |  | 常時 |
| `--disable-gtk` |  | 常時 |
| `--disable-curses` |  | 常時 |
| `--disable-brlapi` |  | 常時 |
| `--disable-spice` |  | 常時 |
| `--disable-libnfs` |  | 常時 |
| `--disable-libusb` |  | 常時 |
| `--disable-vde` |  | 常時 |
| `--disable-nettle` |  | 常時 |
| `--disable-gnutls` |  | 常時 |
| `--disable-gcrypt` |  | 常時 |
| `--disable-auth-pam` |  | 常時 |

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で差分はありません。

<details>
<summary>常時</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD"
cp -a "$SOURCE"/. "$BUILD"/
cd "$BUILD"
export CFLAGS="$(printf '%s' "${CFLAGS:-}" | sed -E 's/--sysroot=[^ ]+//g') -flto=auto"
export CXXFLAGS="$(printf '%s' "${CXXFLAGS:-}" | sed -E 's/--sysroot=[^ ]+//g') -flto=auto"
export LDFLAGS="$(printf '%s' "${LDFLAGS:-}" | sed -E 's/--sysroot=[^ ]+//g') -flto=auto"
if [ -d /dep/${TRIPLE}-glibc ]; then
  _glibc=/dep/${TRIPLE}-glibc
  export LDFLAGS="${LDFLAGS} -B${_glibc}/usr/lib64 -L${_glibc}/usr/lib64 -L${_glibc}/lib64"
fi
if [ -d /dep/linux-headers/usr/include ]; then
  export CFLAGS="${CFLAGS} -idirafter /dep/linux-headers/usr/include"
  export CXXFLAGS="${CXXFLAGS} -idirafter /dep/linux-headers/usr/include"
fi
PKG_CONFIG_LIBDIR=
for _dep in glib pcre2 zlib libffi; do
  for _pc in /dep/"${_dep}"/usr/lib/pkgconfig /dep/"${_dep}"/usr/lib64/pkgconfig; do
    if [ -d "${_pc}" ]; then
      PKG_CONFIG_LIBDIR="${PKG_CONFIG_LIBDIR:+$PKG_CONFIG_LIBDIR:}${_pc}"
    fi
  done
done
if [ -n "${PKG_CONFIG_LIBDIR:-}" ]; then
  export PKG_CONFIG_LIBDIR
  export PKG_CONFIG_PATH=
fi
if [ -d /dep/glib ]; then
  export PKG_CONFIG_SYSROOT_DIR=/dep/glib
fi
sed -i '/^\[tooling\]/,/^\[/ { /^"qemu" =/d; /^"setuptools"/d; /^"wheel"/d; /^"pip"/d }' pythondeps.toml
./configure --prefix=/usr --target-list=x86_64-linux-user,aarch64-linux-user,riscv64-linux-user,ppc64le-linux-user,s390x-linux-user --enable-tcg --disable-system --disable-tools --disable-docs --disable-guest-agent --disable-slirp --disable-vnc --disable-sdl --disable-gtk --disable-curses --disable-brlapi --disable-spice --disable-libnfs --disable-libusb --disable-vde --disable-nettle --disable-gnutls --disable-gcrypt --disable-auth-pam
make -j"$(nproc)"
make install DESTDIR="$OUT"
if command -v qemu-s390x >/dev/null; then
  qemu-s390x -cpu help | grep -E 'z900|z990|z9EC' || true
fi
```

</details>

<!-- /GENERATED -->



シード（Arch Linux）環境の `glib-2.0.pc` への依存を防ぐため、`PKG_CONFIG_PATH` を空にして `PKG_CONFIG_LIBDIR` を `/dep/{glib,pcre2,zlib,libffi}/usr/lib{,64}/pkgconfig` のみに限定します。`/dep` には直接依存パッケージのみがマウントされるため、`glib-2.0.pc` の `Requires.private` が参照する `pcre2` および `libffi` も明示的に依存関係へ加えてマウントします。glib の成果物が存在する場合は、`PKG_CONFIG_SYSROOT_DIR=/dep/glib` も追加で指定します。

## 対象プラットフォーム

本パッケージは amd64 / v4 のみを対象とします。[targets.md](../../../user/targets.md) に定義されている他のプラットフォームは対象外です。

configure の `--target-list` オプションには、ホストの CPU プラットフォーム指定とは別に、エミュレート対象とする以下の linux-user ターゲットを指定します。

| 生成バイナリ | configure の target 指定 |
|-|-|
| `qemu-x86_64` | `x86_64-linux-user` |
| `qemu-aarch64` | `aarch64-linux-user` |
| `qemu-riscv64` | `riscv64-linux-user` |
| `qemu-ppc64le` | `ppc64le-linux-user` |
| `qemu-s390x` | `s390x-linux-user` |

## `--sysroot` 非適用の理由

コンパイラ引数に `--sysroot` を付与した場合、meson がポインタ長（`sizeof(void *)`）をリンクテスト付きで計測する工程で失敗します。ホスト用 glibc 成果物の直下には `/usr/include` が存在せず、コンパイラが標準ヘッダを解決できないためです。この検査が失敗するとポインタサイズが `-1` と判定され、64-bit linux-user 向けの TCG（Tiny Code Generator）バックエンドが正しく選択されなくなります。
