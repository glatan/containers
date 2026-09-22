<!-- GENERATED -->
# gzip

| 項目 | 値 |
|-|-|
| パッケージ名 | `gzip` |
| バージョン | 1.14 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/gzip.rs`](../../../builder/src/package/definition/gzip.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/gzip/gzip-1.14.tar.xz |
| sha3-512 | `6e7b2e6c5d87a64d2adde0dbd81e0262d638c60088dc5a61be5a0e64b8fa1ed9452bc2a7c3c29667ddb37fac8f8dc35ef16e10154f7a6a57ec8de6aa24ee4162` |
| blake3 | `4d318e6b8ff830c98b98314bcafb54d540ab940553ef1ad5e0743b21a36a473be1c98b3671dae64e436526b39c2359f623d66dae1a003067c3a5ed9e0c0d0617` |
| 展開ディレクトリ | `gzip-1.14` |

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
| `--prefix` | `/usr` | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | 同梱ソースを連結した複数 MB 入力を `gzip` で `-1` / `-9` の 2 レベルで圧縮・解凍 |
