# zlib

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib` |
| バージョン | 1.3.1 |
| ライセンス | Zlib |
| パッケージイメージ | する。[公開一覧](../packages.md) |
| ネットワーク | 許可(クロス時の pacman) |
| 定義 | [`builder/src/package/definition/zlib.rs`](../../../builder/src/package/definition/zlib.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/madler/zlib/releases/download/v1.3.1/zlib-1.3.1.tar.gz |
| sha3-512 | `d4233237ef02d5a6182e4b3704b5a7da40169076edcc0ff0546441f27ace6f8258fc5668a241263e8f8b7726d974c82654ed06ea1680f82784f79874e6350570` |
| blake3 | `207c3b0862cb4e3686f8405f76a98c38dbad9c94bcf4be4b9efca0716aba51ecbb74a7a73fb847a40884a5a12be05d3a3d1a9b534b11b16efcec551744bd3fb8` |
| 展開ディレクトリ | `zlib-1.3.1` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [glibc](./glibc.md) | 2.44 | `--sysroot=/dep/glibc`、`-B` / `-L` |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.12.109 | `-idirafter /dep/linux-headers/usr/include` |

openssl からは依存しません。

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
| arm64 | v8.1 | armv8.1-a | 対応 |
| arm64 | v8.2 | armv8.2-a | 対応 |
| arm64 | v8.3 | armv8.3-a | 対応 |
| arm64 | v8.4 | armv8.4-a | 対応 |
| arm64 | v8.5 | armv8.5-a | 対応 |
| arm64 | v8.6 | armv8.6-a | 対応 |
| arm64 | v8.7 | armv8.7-a | 対応 |
| arm64 | v8.8 | armv8.8-a | 対応 |
| arm64 | v8.9 | armv8.9-a | 対応 |
| arm64 | v9 | armv9-a | 対応 |
| arm64 | v9.1 | armv9.1-a | 対応 |
| arm64 | v9.2 | armv9.2-a | 対応 |
| arm64 | v9.3 | armv9.3-a | 対応 |
| arm64 | v9.4 | armv9.4-a | 対応 |
| arm64 | v9.5 | armv9.5-a | 対応 |
| riscv64 | rva20u64 | rva20u64 | 対応 |
| riscv64 | rva22u64 | rva22u64 | 対応 |
| riscv64 | rva23u64 | rva23u64 | 対応 |

## ビルドフラグ

| フラグ | 値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--libdir` | `/usr/lib64` | 常時 |
| `--shared` | | 常時 |
| `CFLAGS` | `-O2 -march=... -flto` | ツールの `CFLAGS` に `-flto` |
| LTO | する | `-flto` |
| PGO | ネイティブのみ | `-fprofile-generate` / `-fprofile-use`。プロファイルは `/tmp/zlib-pgo` に置き、`minigzip` と `example` で生成 |

手順は定義ファイルの `build_script` です。zlib-ng とは別系統です。`libz.so` だけを提供します。
