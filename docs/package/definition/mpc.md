<!-- GENERATED -->
# mpc

| 項目 | 値 |
|-|-|
| パッケージ名 | `mpc` |
| バージョン | 1.4.1 |
| ライセンス | LGPL-3.0-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/mpc.rs`](../../../builder/src/package/definition/mpc.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/mpc/mpc-1.4.1.tar.xz |
| sha3-512 | `347aa36ed9745dab597e13373ac0c3ff35c8413427a89e3ee1e74a8640a51c893610f3f5d0e30c13a496025142c0539ec936b020138df945f3c8df424dd59ed9` |
| blake3 | `994e14084833795dbb36ee1abe46059c74cab1845e7ca17585bc7bc8727444f8ac5bec1247722a8a98c0ae0196b0fb813dd39586b7303dc6e612649534c7b8e3` |
| 展開ディレクトリ | `mpc-1.4.1` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [gmp](./gmp.md) | 6.3.0 | — |
| [mpfr](./mpfr.md) | 4.2.2 | — |

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



## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--with-gmp` | オーバーレイの gmp 成果物 | 常時 |
| `--with-mpfr` | オーバーレイの mpfr 成果物 | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `make check` |
