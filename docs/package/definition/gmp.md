<!-- GENERATED -->
# gmp

| 項目 | 値 |
|-|-|
| パッケージ名 | `gmp` |
| バージョン | 6.3.0 |
| ライセンス | LGPL-3.0-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/gmp.rs`](../../../builder/src/package/definition/gmp.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gmp/gmp-6.3.0.tar.xz |
| sha3-512 | `b459a8c258273ff76a2627a64627739d233064e5a9a48de68a544c921a1fe4d617af48528de7b67778f30754d793d645a9ecf9c38a71578bd4f0ab6febf591de` |
| blake3 | `fffe4996713928ae19331c8ef39129e46d3bf5b7182820656fd4639435cd83a40d7c77aa7c571938ae3f25a58bfdfa44b6a710f2a7e002ab11edf8389cd6928b` |
| 展開ディレクトリ | `gmp-6.3.0` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |

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
| `ABI` | `64` | 既定 |
| `ABI` | `standard` | `riscv64-*` |
| `ABI` | `mode64` | `powerpc64le-*`、`powerpc64-*` |
| `--prefix` | `/usr` | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `make check`。`Env::Lto` のため `-fprofile-generate` / `-fprofile-use` は configure 時の `CFLAGS` へ注入 |
