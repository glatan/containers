<!-- GENERATED -->
# libffi

| 項目 | 値 |
|-|-|
| パッケージ名 | `libffi` |
| バージョン | 3.8.0 |
| ライセンス | MIT |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/libffi.rs`](../../../builder/src/package/definition/libffi.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/libffi/libffi/releases/download/v3.8.0/libffi-3.8.0.tar.gz |
| sha3-512 | `adad40a2b8be7e69762ba21e0d6d1a51da4ab55340ee2924cb9e74d292ee0fbcf9b9f52bc096c6d6ae285c39ca02fb12508add63e92c505efb0f7490483022ad` |
| blake3 | `df92f5720298710e22ffbfbb7d9a9969e90b1f95c01e02e6356a56de162b57ea6585c6fedb65ca37cf816e9d8f12dbadf706f75e266b04cdaab06ee4c270bb87` |
| 展開ディレクトリ | `libffi-3.8.0` |

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
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | ビルドした libffi へリンクする FFI スモークテストを実行（`make check` は dejagnu 依存のため代替。存在すれば併せて試行） |
