<!-- GENERATED -->
# pcre2

| 項目 | 値 |
|-|-|
| パッケージ名 | `pcre2` |
| バージョン | 10.48 |
| ライセンス | BSD-3-Clause |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/pcre2.rs`](../../../builder/src/package/definition/pcre2.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/PCRE2Project/pcre2/releases/download/pcre2-10.48/pcre2-10.48.tar.bz2 |
| sha3-512 | `445ca5ab94350b9c4307405350758e0c148c92160329b0443159274ce12bae0f437a1d788fb566d7731081553a2a31cc9915055e38e4f97ae17b834d7a1c17e6` |
| blake3 | `5e84ce242f146b8a97ce10e3cf31ac4c2c2b23d67814fa1cf266636d0545f3a50f08634ee1a724793b20cdae76647113f56f13a814328e8c3de48a837168ca45` |
| 展開ディレクトリ | `pcre2-10.48` |

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
| `--enable-pcre2-16` | | 常時 |
| `--enable-pcre2-32` | | 常時 |
| `--enable-jit` | | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `pcre2test` で `testdata/testinput{1,2,4,5}` を実行 |
