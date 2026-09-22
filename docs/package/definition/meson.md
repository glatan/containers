<!-- GENERATED -->
# meson

| 項目 | 値 |
|-|-|
| パッケージ名 | `meson` |
| バージョン | 1.12.0 |
| ライセンス | Apache-2.0 |
| 分類 | パッケージ |
| 定義 | [`builder/src/package/definition/meson.rs`](../../../builder/src/package/definition/meson.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/mesonbuild/meson/releases/download/1.12.0/meson-1.12.0.tar.gz |
| sha3-512 | `e1832949d31d2e7010a5c3883779dbb0811937a22dac233b4d35dfc6e5e2e7fcbeb0148fc803c795c209a84407269e11df6f123e7f45e397676d1a2100e34e50` |
| blake3 | `6115839425ffe60c0f036f8f9445e9572a48d207b7c1aa758c624e3f1e66d77ab71d0cff644c4e91c1169c62726baae74625a7832d64e5f0101b6742574e1f8a` |
| 展開ディレクトリ | `meson-1.12.0` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [python](./python.md) | 3.14.7 | — |

### build-dependencies

なし。

### host-dependencies

なし。

<!-- /GENERATED -->




## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。他パッケージがビルド時に実行する meson は HOST 成果物をマウントします。

C 言語のコンパイルは行いません。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `mesonbuild` 配置 | `$SOURCE/mesonbuild` を `$OUT/usr/lib/python{ver}/site-packages/` へコピーし、`python3 -m mesonbuild.mesonmain` を呼ぶ `usr/bin/meson` を生成 | 常時（python 成果物は `--without-ensurepip` で pip を持たないため） |
| LTO | 適用しない | C 言語のコンパイルを伴わないため |
| PGO | 適用しない（`Pgo::None`） | C 言語のコンパイルを伴わないため |
