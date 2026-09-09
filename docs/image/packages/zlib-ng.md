# zlib-ng

| 項目 | 値 |
|-|-|
| パッケージ名 | `zlib-ng` |
| バージョン | 2.3.3 |
| ライセンス | Zlib |
| パッケージイメージ | する。[公開一覧](../packages.md) |
| ネットワーク | 許可(ネイティブの cmake、クロス時は gcc も) |
| 定義 | [`builder/src/package/definition/zlib_ng.rs`](../../../builder/src/package/definition/zlib_ng.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/zlib-ng/zlib-ng/archive/refs/tags/2.3.3.tar.gz |
| sha3-512 | `b045a4d3358f73db5566951944186293b9a99c5bc9d395866bc0a539617b9584b1499d3d0e9a8943284a7b10a829902dd37e1d6ec55823e6203b000644ba0170` |
| blake3 | `103cd4e5aa69bb2d0383a242f231bccf64c8a79c21156e7c6187db3cebc1367dc975216cfa092b7d37cdfb303e1fc9cdb2b63d6ef59ce6dc662fbe89286ada42` |
| 展開ディレクトリ | `zlib-ng-2.3.3` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [glibc](./glibc.md) | 2.44 | `--sysroot=/dep/glibc`、`-B` / `-L` |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.12.109 | `-idirafter /dep/linux-headers/usr/include` |

openssl からは依存しません。zlib の `libz.so` とはパスを分けます。

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../../package/README.md) | `package build` |
| `cmake` | ビルド。pacman で導入 |
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
| `CMAKE_INSTALL_PREFIX` | `/usr` | 常時 |
| `CMAKE_INSTALL_LIBDIR` | `lib64` | 常時 |
| `ZLIB_COMPAT` | `OFF` | 常時。`libz.so` は出さない |
| `BUILD_TESTING` | `OFF` | 常時。テストバイナリは作らない |
| `CFLAGS` | `-O2 -march=...` | ツールが付与 |
| LTO | する | `CMAKE_INTERPROCEDURAL_OPTIMIZATION` |
| PGO | しない | CMake にプロファイル生成の手順が無い |

手順は定義ファイルの `build_script` です。提供するのは `libz-ng.so` です。
