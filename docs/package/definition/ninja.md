# ninja

| 項目 | 値 |
|-|-|
| パッケージ名 | `ninja` |
| バージョン | 1.13.2 |
| ライセンス | Apache-2.0 |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/ninja.rs`](../../../builder/src/package/definition/ninja.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/ninja-build/ninja/archive/refs/tags/v1.13.2.tar.gz |
| sha3-512 | `4d48df9a0ea4a0b74ac6ec6206ef4743dfabe8acabffb69d66aeee15f9608414f7ba6a4f755728c329fe3827abdbac9d58afbffff4b95af4cf82010f30838b6d` |
| blake3 | `b32100cf57fde2a19d3c242f363c308c3d7a7ed3167774eefcbd107baada26e4fb5240af8ae22fde9d1cec079f5001098a30bc90f4fb3b5303c3bccd636fe161` |
| 展開ディレクトリ | `ninja-1.13.2` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md)（他 Platform ではその `{triple}-gcc`） | 16.2.0 | C++ ランタイム（libstdc++ / libgcc_s） |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（実行する python は HOST 成果物をマウント） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。他パッケージがビルド時に実行する ninja は HOST 成果物をマウントします。

Autotools や CMake ではなく、付属の Python スクリプトでブートストラップします。

| コマンド / フラグ | 設定値 | 条件 |
|-|-|-|
| `python3 configure.py --bootstrap` | | 常時 |
| LTO | 適用する | `configure.py` は `CXXFLAGS` / `LDFLAGS` を環境変数から拾うため、既定フラグの `-flto=auto` が注入される |
| PGO | ネイティブビルドのみ適用（`Pgo::Custom`） | `-fprofile-generate` で bootstrap した ninja が自身のオブジェクトを再ビルドしてプロファイルを採取し、`-fprofile-use` で再 bootstrap。`configure.py` の実行には host tool として python をマウント |
