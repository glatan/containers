<!-- GENERATED -->
# bash

| 項目 | 値 |
|-|-|
| パッケージ名 | `bash` |
| バージョン | 5.3 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/bash.rs`](../../../builder/src/package/definition/bash.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/bash/bash-5.3.tar.gz |
| sha3-512 | `8c253baa0accc25a7e83c9d916f0af272b53a368b463afa5035675be7d86e8f63f2e727a1f711615ebed085fe1492fab2ef81490504a7fe9ed09428ffc6fd4c0` |
| blake3 | `5bf3ac6c18f3dc3fe3e049a1e10ea5ae5c5b39dac2356cd796730f49b273fd0f90dedfa16e5db8348625dbcb4225cb157d6c3bfbc8948e1ae025d3d61b2fed3f` |
| 展開ディレクトリ | `bash-5.3` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [ncurses](./ncurses.md) | 6.6 | — |

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

`$OUT/usr/bin/sh` および `$OUT/bin/{bash,sh}` から bash へのシンボリックリンクを作成します。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--without-bash-malloc` | | 常時 |
| LTO | 適用する | `Env::Lto` で configure し、configure が Makefile へ焼いたフルフラグをそのまま使う |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `bash -c` で算術ループ・文字列展開・パラメータ展開・glob を実行 |
