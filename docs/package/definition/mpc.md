# mpc

| 項目 | 値 |
|-|-|
| パッケージ名 | `mpc` |
| バージョン | 1.4.1 |
| ライセンス | LGPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/mpc.rs`](../../../builder/src/package/definition/mpc.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/mpc/mpc-1.4.1.tar.xz |
| sha3-512 | `347aa36ed9745dab597e13373ac0c3ff35c8413427a89e3ee1e74a8640a51c893610f3f5d0e30c13a496025142c0539ec936b020138df945f3c8df424dd59ed9` |
| blake3 | `994e14084833795dbb36ee1abe46059c74cab1845e7ca17585bc7bc8727444f8ac5bec1247722a8a98c0ae0196b0fb813dd39586b7303dc6e612649534c7b8e3` |
| 展開ディレクトリ | `mpc-1.4.1` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [gmp](./gmp.md) | 6.3.0 | 多倍長演算サポート |
| [mpfr](./mpfr.md) | 4.2.2 | 多倍長浮動小数点演算サポート |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--with-gmp` | オーバーレイの gmp 成果物 | 常時 |
| `--with-mpfr` | オーバーレイの mpfr 成果物 | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `make check` |
