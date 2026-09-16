# bash

| 項目 | 値 |
|-|-|
| パッケージ名 | `bash` |
| バージョン | 5.3 |
| ライセンス | GPL-3.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/bash.rs`](../../../builder/src/package/definition/bash.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/bash/bash-5.3.tar.gz |
| sha3-512 | `8c253baa0accc25a7e83c9d916f0af272b53a368b463afa5035675be7d86e8f63f2e727a1f711615ebed085fe1492fab2ef81490504a7fe9ed09428ffc6fd4c0` |
| blake3 | `5bf3ac6c18f3dc3fe3e049a1e10ea5ae5c5b39dac2356cd796730f49b273fd0f90dedfa16e5db8348625dbcb4225cb157d6c3bfbc8948e1ae025d3d61b2fed3f` |
| 展開ディレクトリ | `bash-5.3` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |
| [ncurses](./ncurses.md) | 6.6 | 端末制御 |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `package build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

`$OUT/usr/bin/sh` および `$OUT/bin/{bash,sh}` から bash へのシンボリックリンクを作成します。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--without-bash-malloc` | | 常時 |
| LTO | 適用する | 共通前処理を適用しないため、既定フラグの `-flto` を維持 |
