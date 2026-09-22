<!-- GENERATED -->
# riscv64-linux-gnu-glibc

| 項目 | 値 |
|-|-|
| パッケージ名 | `riscv64-linux-gnu-glibc` |
| バージョン | 2.44 |
| ライセンス | LGPL-2.1-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/riscv64_linux_gnu_glibc.rs`](../../../builder/src/package/definition/riscv64_linux_gnu_glibc.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/glibc/glibc-2.44.tar.xz |
| sha3-512 | `263dfb5e331b2033368c7455abcd9a9b15ab098ac963d5518ecf4bcb9eeb41298eb9976bf78a64af67d9471a50161bced2fa69d429c69759ea57f76b942ac521` |
| blake3 | `6d07bc16d8d066b1c0ae713d762175f261e27df9cab8015ec11e1c980c05e79b0fd5bab6cde6a90bf07651c432ca9039629376b57a96c9f3dcb3765f3745dbf4` |
| 展開ディレクトリ | `glibc-2.44` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

なし。

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |
| [riscv64-linux-gnu-binutils](./riscv64-linux-gnu-binutils.md) | 2.47 | — |
| [riscv64-linux-gnu-gcc-stage2](./riscv64-linux-gnu-gcc-stage2.md) | 16.2.0 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [python](./python.md) | 3.14.7 | seed:python |
| [bison](./bison.md) | 3.8.2 | seed:bison |
| [m4](./m4.md) | 1.4.21 | seed:m4 |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->

python / bison / m4 のホスト依存は、glibc→python→openssl→glibc の循環を断つため `Via::Seed` です（シード相は distro パッケージが担保し、合成 env 上では成果物が必須）。




## ビルド設定

実行時 libc です。`supports` は architecture `riscv64` の variant（rva20u64, rva22u64, rva23u64）です。
