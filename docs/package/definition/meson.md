# meson

| 項目 | 値 |
|-|-|
| パッケージ名 | `meson` |
| バージョン | 1.12.0 |
| ライセンス | Apache-2.0 |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/meson.rs`](../../../builder/src/package/definition/meson.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/mesonbuild/meson/releases/download/1.12.0/meson-1.12.0.tar.gz |
| sha3-512 | `e1832949d31d2e7010a5c3883779dbb0811937a22dac233b4d35dfc6e5e2e7fcbeb0148fc803c795c209a84407269e11df6f123e7f45e397676d1a2100e34e50` |
| blake3 | `6115839425ffe60c0f036f8f9445e9572a48d207b7c1aa758c624e3f1e66d77ab71d0cff644c4e91c1169c62726baae74625a7832d64e5f0101b6742574e1f8a` |
| 展開ディレクトリ | `meson-1.12.0` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [python](./python.md) | 3.14.7 | meson の実行インタプリタ（実行時にも必要） |

### build-dependencies

なし。

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるインストール（実行する python は HOST 成果物をマウント） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。他パッケージがビルド時に実行する meson は HOST 成果物をマウントします。

C 言語のコンパイルは行いません。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `mesonbuild` 配置 | `$SOURCE/mesonbuild` を `$OUT/usr/lib/python{ver}/site-packages/` へコピーし、`python3 -m mesonbuild.mesonmain` を呼ぶ `usr/bin/meson` を生成 | 常時（python 成果物は `--without-ensurepip` で pip を持たないため） |
| LTO | 適用しない | C 言語のコンパイルを伴わないため |
| PGO | 適用しない（`Pgo::None`） | C 言語のコンパイルを伴わないため |
