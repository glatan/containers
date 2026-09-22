<!-- GENERATED -->
# libxcrypt

| 項目 | 値 |
|-|-|
| パッケージ名 | `libxcrypt` |
| バージョン | 4.5.2 |
| ライセンス | LGPL-2.1-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/libxcrypt.rs`](../../../builder/src/package/definition/libxcrypt.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/besser82/libxcrypt/releases/download/v4.5.2/libxcrypt-4.5.2.tar.xz |
| sha3-512 | `7c5eeccd7d7706048d73109937aba4624d9641f22965b154ba23ec8373c6c2bc939d182704c214f7a1a8d7600f5d0f61720d4e4a57913ab500d050d647a3ffe8` |
| blake3 | `8294e7735d240876f4c4c38a1370e1e9e9e2da4856fcfcb81168ea20ab83f33d1cc591380e2f0c00eb45ff4ac999b47c54d0d20b05b29757b1d147559227a9c9` |
| 展開ディレクトリ | `libxcrypt-4.5.2` |

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
| [perl](./perl.md) | 5.42.3 | optional |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--disable-werror` | | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `make check` |
