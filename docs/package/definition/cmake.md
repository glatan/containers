# cmake

| 項目 | 値 |
|-|-|
| パッケージ名 | `cmake` |
| バージョン | 4.4.3 |
| ライセンス | BSD-3-Clause |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要 |
| 定義 | [`builder/src/package/definition/cmake.rs`](../../../builder/src/package/definition/cmake.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/Kitware/CMake/releases/download/v4.4.3/cmake-4.4.3.tar.gz |
| sha3-512 | `170e8771bffc5d56d42c1702320e5c4dafbe90982cc9bcb8d3e21a16be2bf86b76ca35cef468a0863cb65790b5f6bc4a0e5b30ea8b34a39e56f31fab78ac8162` |
| blake3 | `347efe575e9f1c7daf9d5888de1c1f1070570d1c1e6beb62522c389b0a408a95be2fca9aaae2d2261a78e68b6777b5fbf08e6a3cce47d64bf12129a13d238f51` |
| 展開ディレクトリ | `cmake-4.4.3` |

## 依存

実行時はパッケージ Platform の `{triple}-glibc` と `{triple}-gcc`（libstdc++ / libgcc_s）です。ビルド専用は linux-headers です。上流の `./bootstrap` で C++ だけから組みます。`-DCMAKE_USE_OPENSSL=OFF` と `-DCMAKE_DISABLE_FIND_PACKAGE_Libidn2=ON` で、シード上の curl 周辺ライブラリを引き込みません。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。最初の `build-env` 合成には入れません。他パッケージがビルド時に実行する cmake は HOST 成果物をマウントします。
