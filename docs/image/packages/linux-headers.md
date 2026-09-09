# linux-headers

| 項目 | 値 |
|-|-|
| パッケージ名 | `linux-headers` |
| バージョン | 6.12.109 |
| ライセンス | GPL-2.0-only WITH Linux-syscall-note |
| パッケージイメージ | しない。ビルド専用 |
| ネットワーク | 許可(pacman) |
| 定義 | [`builder/src/package/linux_headers.rs`](../../../builder/src/package/linux_headers.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.12.109.tar.xz |
| sha3-512 | `e8a40f7302e8c93710001876941c49744e8120fbb5094c7fa1b08cfc360e2fa61c73b3d1b144b0fb4933ae5286d5b833377535567547a6167dac56a081c474f2` |
| blake3 | `9abb3b5ba08f97c241bb995571e927fc13e6b2c946081f23c9a28e231aef576bf9d0d2e770b963129b59efc9ac4ce950f2909a44f718f43e3fa033b3adafa069` |
| 展開ディレクトリ | `linux-6.12.109` |

## 依存

### dependencies

なし。

### build-dependencies

なし。

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../../package/README.md) | `package build` |
| `rsync` | `headers_install`。pacman で導入 |

## ターゲット

ヘッダ自体は variant で分岐しませんが、ビルドはツールの `CFLAGS` 解決を通るため、`-march` 未対応の組み合わせはスキップされます。表に無い組み合わせは [platforms.md](../platforms.md) にあり、未対応です。

| architecture | variant | カーネル `ARCH` | `-march` | 状態 |
|-|-|-|-|-|
| amd64 | v1 | x86_64 | x86-64 | 対応 |
| amd64 | v2 | x86_64 | x86-64-v2 | 対応 |
| amd64 | v3 | x86_64 | x86-64-v3 | 対応 |
| amd64 | v4 | x86_64 | x86-64-v4 | 対応 |
| arm64 | v8 | arm64 | armv8-a | 対応 |

## ビルドフラグ

| フラグ | 値 | 条件 |
|-|-|-|
| `ARCH` | 上表のカーネル `ARCH` | 常時 |
| `INSTALL_HDR_PATH` | `$OUT/usr` | 常時 |
| `O` | `$BUILD` | 常時 |
| LTO | しない | headers のみ |
| PGO | しない | headers のみ |

手順は定義ファイルの `build_script` です。
