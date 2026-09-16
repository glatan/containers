# gmp

| 項目 | 値 |
|-|-|
| パッケージ名 | `gmp` |
| バージョン | 6.3.0 |
| ライセンス | LGPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/gmp.rs`](../../../builder/src/package/definition/gmp.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz |
| sha3-512 | `b459a8c258273ff76a2627a64627739d233064e5a9a48de68a544c921a1fe4d617af48528de7b67778f30754d793d645a9ecf9c38a71578bd4f0ab6febf591de` |
| blake3 | `fffe4996713928ae19331c8ef39129e46d3bf5b7182820656fd4639435cd83a40d7c77aa7c571938ae3f25a58bfdfa44b6a710f2a7e002ab11edf8389cd6928b` |
| 展開ディレクトリ | `gmp-6.3.0` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` `m4` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `ABI` | `64` | 既定 |
| `ABI` | `standard` | `riscv64-*` |
| `ABI` | `mode64` | `powerpc64le-*`、`powerpc64-*` |
| `--prefix` | `/usr` | 常時 |
