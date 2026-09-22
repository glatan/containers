<!-- GENERATED -->
# mpfr

| 項目 | 値 |
|-|-|
| パッケージ名 | `mpfr` |
| バージョン | 4.2.2 |
| ライセンス | LGPL-3.0-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/mpfr.rs`](../../../builder/src/package/definition/mpfr.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/mpfr/mpfr-4.2.2.tar.xz |
| sha3-512 | `92d8b3f8b16cdb1130e59717e2c8aa8b249070bd34675000af73327b60043358b4238f94cf9c37b97fa8323ec823d518977746afb92573558ce1d40d6121b406` |
| blake3 | `11d59d061ef8db588650bc7dc5172594a6e5aad013994801c6f63011a62b191dd94685e03abb7e145f97b51c40f75ae11795c1ab6112fc4cf82d638bb68c0588` |
| 展開ディレクトリ | `mpfr-4.2.2` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [gmp](./gmp.md) | 6.3.0 | — |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `make check` |
