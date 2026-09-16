# qemu

| 項目 | 値 |
|-|-|
| パッケージ名 | `qemu` |
| バージョン | 11.1.1 |
| ライセンス | GPL-2.0-only |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/qemu.rs`](../../../builder/src/package/definition/qemu.rs) |

ホスト環境向けとして linux-user エミュレータのみを成果物に含め、システムエミュレータ（system emulator）は含めません。

インストール後、`qemu-s390x -cpu help` を実行してサポート CPU 一覧に `z900`、`z990`、および `z9EC` が正しく表示されることを確認します。

## ソース

| 項目 | 値 |
|-|-|
| URL | https://download.qemu.org/qemu-11.1.1.tar.xz |
| sha3-512 | `8b53993040fe8ce2e6c69e2d616a952fcd3da00ddcd114b64dea2d5b1cc0a4f6df03bc899ba12437b6166559be8f2379b97a737608659f6b2c6ab56605bd20a2` |
| blake3 | `9cda9512c997d0ec176add30ff4f4531d4882681eb12e98ba5c7d11cb4cf610ca079689e5def5ae36c1c1a173dee7eb001ce8fffeba77f418048e4b4773dff67` |
| 展開ディレクトリ | `qemu-11.1.1` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | 動的リンク（`-B` / `-L` により参照） |
| [glib](./glib.md) | 2.90.0 | `glib-2.0` ライブラリの利用 |
| [zlib](./zlib.md) | 1.3.2 | pkg-config による参照 |
| [zstd](./zstd.md) | 1.5.7 | 圧縮ライブラリの利用 |

シード（Arch Linux）環境の `glib-2.0.pc` への依存を防ぐため、`PKG_CONFIG_PATH` を空にして `PKG_CONFIG_LIBDIR` を `/dep/{glib,pcre2,zlib,libffi}/usr/lib{,64}/pkgconfig` のみに限定します。`/dep` には直接依存パッケージのみがマウントされるため、`glib-2.0.pc` の `Requires.private` が参照する `pcre2` および `libffi` も明示的に依存関係へ加えてマウントします。glib の成果物が存在する場合は、`PKG_CONFIG_SYSROOT_DIR=/dep/glib` も追加で指定します。

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |
| [pcre2](./pcre2.md) | 10.48 | glib の pkg-config 解決用 |
| [libffi](./libffi.md) | 3.8.0 | glib の pkg-config 解決用 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（実行する meson / ninja / pkgconf / python は HOST 成果物をマウント） |

## 対象プラットフォーム

本パッケージは amd64 / v4 のみを対象とします。[platforms.md](../../image/platforms.md) に定義されている他のプラットフォームは対象外です。

configure の `--target-list` オプションには、ホストの CPU プラットフォーム指定とは別に、エミュレート対象とする以下の linux-user ターゲットを指定します。

| 生成バイナリ | configure の target 指定 |
|-|-|
| `qemu-x86_64` | `x86_64-linux-user` |
| `qemu-aarch64` | `aarch64-linux-user` |
| `qemu-riscv64` | `riscv64-linux-user` |
| `qemu-ppc64le` | `ppc64le-linux-user` |
| `qemu-s390x` | `s390x-linux-user` |

## ビルドフラグ

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--target-list` | 上表の 5 ターゲット | 常時 |
| `--enable-tcg` | | 常時（linux-user 用のアクセラレータ） |
| `--disable-system` | | 常時（システムエミュレータを無効化） |
| `--disable-tools` `--disable-docs` `--disable-guest-agent` | | 常時 |
| `--disable-slirp` `--disable-vnc` `--disable-sdl` `--disable-gtk` `--disable-curses` `--disable-brlapi` `--disable-spice` | | 常時 |
| `--disable-libnfs` `--disable-libusb` `--disable-vde` `--disable-nettle` `--disable-gnutls` `--disable-gcrypt` `--disable-auth-pam` | | 常時 |
| カタログ用の Cflag / Ldflag | 付与しない | meson にカタログ用 LTO 品質保証フラグを渡さない |
| CFLAGS / CXXFLAGS / LDFLAGS | `-flto=auto`（`--sysroot` は除去） | 下記参照 |
| LDFLAGS のライブラリ探索 | `-B` / `-L`（`/dep/x86_64-linux-gnu-glibc`） | ホスト用 glibc へリンク |
| LTO | 適用する | スクリプト内で `-flto=auto` を指定 |
| PGO | 適用しない | linux-user ビルド用のプロファイル手順を置いていないため |

コンパイラ引数に `--sysroot` を付与した場合、meson がポインタ長（`sizeof(void *)`）をリンクテスト付きで計測する工程で失敗します。ホスト用 glibc 成果物の直下には `/usr/include` が存在せず、コンパイラが標準ヘッダを解決できないためです。この検査が失敗するとポインタサイズが `-1` と判定され、64-bit linux-user 向けの TCG（Tiny Code Generator）バックエンドが正しく選択されなくなります。
