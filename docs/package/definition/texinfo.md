<!-- GENERATED -->
# texinfo

| 項目 | 値 |
|-|-|
| パッケージ名 | `texinfo` |
| バージョン | 7.3 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/texinfo.rs`](../../../builder/src/package/definition/texinfo.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/texinfo/texinfo-7.3.tar.xz |
| sha3-512 | `04243a6f04eec6f208fecea57c16fc4a9f5fd0487c670370295d18edc6d266bbff3fa60d8a547687d8c17de19218c28434ac5359a3d29dde9720f53d86f33938` |
| blake3 | `01eae8c848df2fbd6903aeba43fef7ac45a019b4c72d4cd853f936940408102561499b58ecfc36753ea84e31dbbc79fc8c43d950380dc33f3bf8402c7d56aa49` |
| 展開ディレクトリ | `texinfo-7.3` |

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
| [perl](./perl.md) | 5.42.3 | optional |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--disable-perl-xs` | | クロスビルド時のみ（perl XS をターゲット向けにビルドしない） |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | ビルドツリーから解決した `texi2any` で同梱 `.texi` を `--plaintext` / `--html` 変換 |
