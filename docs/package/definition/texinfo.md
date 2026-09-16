# texinfo

| 項目 | 値 |
|-|-|
| パッケージ名 | `texinfo` |
| バージョン | 7.3 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/texinfo.rs`](../../../builder/src/package/definition/texinfo.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/texinfo/texinfo-7.3.tar.xz |
| sha3-512 | `04243a6f04eec6f208fecea57c16fc4a9f5fd0487c670370295d18edc6d266bbff3fa60d8a547687d8c17de19218c28434ac5359a3d29dde9720f53d86f33938` |
| blake3 | `01eae8c848df2fbd6903aeba43fef7ac45a019b4c72d4cd853f936940408102561499b58ecfc36753ea84e31dbbc79fc8c43d950380dc33f3bf8402c7d56aa49` |
| 展開ディレクトリ | `texinfo-7.3` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [ncurses](./ncurses.md) | 6.5 | 端末制御ライブラリ |
| [perl](./perl.md) | 5.42.3 | `makeinfo` の実行環境 |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル（シード環境では builder が `gcc` `make` `perl` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `PERL=/usr/bin/perl` | | 常時 |
