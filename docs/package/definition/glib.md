<!-- GENERATED -->
# glib

| 項目 | 値 |
|-|-|
| パッケージ名 | `glib` |
| バージョン | 2.90.0 |
| ライセンス | LGPL-2.1-or-later |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/glib.rs`](../../../builder/src/package/definition/glib.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://download.gnome.org/sources/glib/2.90/glib-2.90.0.tar.xz |
| sha3-512 | `c896df2b43beb87254843d355e2fc389d9d62e01a49c58e078f3736286c0f0f16eedcfb611b19edb13ebf38ec8714ab1639b7b52b1ad02301eb1365cddaa205d` |
| blake3 | `0cd33e9b83b0fbc9b8396e99a59504909ced4e2f233f445e6127dacbcf17c0d00cc425186477da98e7f37cacfc8a27f8957f28b6b1cbd94ba1e98deef2570866` |
| 展開ディレクトリ | `glib-2.90.0` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [libffi](./libffi.md) | 3.8.0 | — |
| [pcre2](./pcre2.md) | 10.48 | — |
| [zlib](./zlib.md) | 1.3.2 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [meson](./meson.md) | 1.12.0 | — |
| [ninja](./ninja.md) | 1.13.2 | — |
| [python](./python.md) | 3.14.7 | — |
| [pkgconf](./pkgconf.md) | 2.5.1 | — |
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。クロス時は meson の `--cross-file` を付けます。

meson 実行前に、ホスト用 glibc の `--sysroot` オプションおよび linux-headers の `-idirafter` オプションを付与します。

| フラグ | 設定値 | 条件 |
|-|-|-|
| meson `--prefix` | `/usr` | 常時 |
| `-Dlibmount` | `disabled` | 常時 |
| `-Dnls` | `disabled` | 常時 |
| `-Dtests` | `false` | 常時 |
| `-Dsysprof` | `disabled` | 常時（`glib-2.0.pc` が外部の `sysprof-capture-4` を要求しないようにする） |
| `-Dintrospection` | `disabled` | 常時 |
| `--wrap-mode` | `nofallback` | 常時（サブプロジェクトの自動取得を禁止） |
| LTO | 適用する | 共通前処理を適用しないため、既定フラグの `-flto` を維持 |
| PGO | 適用しない（`Pgo::None`） | meson ビルドのため PGO テンプレートの対象外。現状は個別手順を設けていない |
