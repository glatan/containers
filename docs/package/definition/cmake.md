<!-- GENERATED -->
# cmake

| 項目 | 値 |
|-|-|
| パッケージ名 | `cmake` |
| バージョン | 4.4.3 |
| ライセンス | BSD-3-Clause |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/cmake.rs`](../../../builder/src/package/definition/cmake.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3.tar.gz |
| sha3-512 | `170e8771bffc5d56d42c1702320e5c4dafbe90982cc9bcb8d3e21a16be2bf86b76ca35cef468a0863cb65790b5f6bc4a0e5b30ea8b34a39e56f31fab78ac8162` |
| blake3 | `347efe575e9f1c7daf9d5888de1c1f1070570d1c1e6beb62522c389b0a408a95be2fca9aaae2d2261a78e68b6777b5fbf08e6a3cce47d64bf12129a13d238f51` |
| 展開ディレクトリ | `cmake-4.4.3` |

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
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



実行時はパッケージ Platform の `{triple}-glibc` と `{triple}-gcc`（libstdc++ / libgcc_s）です。ビルド専用は linux-headers です。上流の `./bootstrap` で C++ だけから組みます。`-DCMAKE_USE_OPENSSL=OFF` と `-DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON` で、シード上の curl 周辺ライブラリを引き込みません。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。最初の `build-env` 合成には入れません。他パッケージがビルド時に実行する cmake は HOST 成果物をマウントします。
