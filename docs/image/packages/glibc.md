# glibc

| 項目 | 値 |
|-|-|
| パッケージ名 | `glibc` |
| バージョン | 2.44 |
| ライセンス | LGPL-2.1-or-later |
| パッケージイメージ | する。[公開一覧](../packages.md) |
| ネットワーク | 許可(クロス時の pacman) |
| 定義 | [`builder/src/package/definition/glibc.rs`](../../../builder/src/package/definition/glibc.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz |
| sha3-512 | `263dfb5e331b2033368c7455abcd9a9b15ab098ac963d5518ecf4bcb9eeb41298eb9976bf78a64af67d9471a50161bced2fa69d429c69759ea57f76b942ac521` |
| blake3 | `6d07bc16d8d066b1c0ae713d762175f261e27df9cab8015ec11e1c980c05e79b0fd5bab6cde6a90bf07651c432ca9039629376b57a96c9f3dcb3765f3745dbf4` |
| 展開ディレクトリ | `glibc-2.44` |

## 依存

### dependencies

なし。

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.12.109 | `--with-headers=/dep/linux-headers/usr/include` |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../../package/README.md) | `package build` |
| `${CROSS}gcc`(例: `aarch64-linux-gnu-gcc`) | クロス時。pacman で導入 |

## ターゲット

`CFLAGS` の `-march` はツールが architecture / variant から決めます。表に無い組み合わせは [platforms.md](../platforms.md) にあり、未対応です。

| architecture | variant | `-march` | 状態 |
|-|-|-|-|
| amd64 | v1 | x86-64 | 対応 |
| amd64 | v2 | x86-64-v2 | 対応 |
| amd64 | v3 | x86-64-v3 | 対応 |
| amd64 | v4 | x86-64-v4 | 対応 |
| arm64 | v8 | armv8-a | 対応 |

## ビルドフラグ

| フラグ | 値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--disable-werror` | | 常時 |
| `--with-headers` | `/dep/linux-headers/usr/include` | 常時 |
| `--host` | `$TRIPLE` | クロス |
| `--build` | `$HOST_TRIPLE` | クロス |
| `libc_cv_forced_unwind` | `yes` | クロス |
| `libc_cv_c_cleanup` | `yes` | クロス |
| `libc_cv_ssp` | `no` | クロス |
| `CFLAGS` | `-O2 -march=...` | ツールが付与 |
| LTO | しない | ビルドシステムの制約 |
| PGO | しない | ビルドシステムの制約 |

クロス時は `usr/{triple}/{include,lib,lib64}` へのシンボリックリンクと、`libc.so` リンカスクリプトの相対パス化を行います。手順は定義ファイルの `build_script` です。
