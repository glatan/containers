# ncurses

| 項目 | 値 |
|-|-|
| パッケージ名 | `ncurses` |
| バージョン | 6.6 |
| ライセンス | MIT-open-group |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/ncurses.rs`](../../../builder/src/package/definition/ncurses.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/ncurses/ncurses-6.6.tar.gz |
| sha3-512 | `10bcddae2d0e74f35ff0ef720516efee0a3aee1b9cb8b9c21d8fa2aebca1e27a1ef24a1803fcfa16c596b6d57ba7fc4b78619bc5a00445d45d9fa9d46067ba14` |
| blake3 | `ba1e3a63ebce3ff56d40ba447761665a3c9b32fa998a49c66d2f34e628469e3a67046ae9eb7bc72b9a7c64fb9ca99cfc1c7ae9c30c80b95eb87da84594c77cbb` |
| 展開ディレクトリ | `ncurses-6.6` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot`（`/dep/${TRIPLE}-glibc/usr/${TRIPLE}` 優先）によるリンク |

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

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--with-shared` | | 常時 |
| `--without-cxx` | | 常時 |
| `--enable-widec` | | 常時 |
| `--with-pkg-config-libdir` | `/usr/lib/pkgconfig` | 常時 |

`--without-cxx` で C++ バインディング（`libncurses++w`）を組みません。`libstdc++.so.6` / `libgcc_s.so.1` への実行時依存を避けるためです。

Wide-character 版ライブラリ（`libncursesw.so`）へリンクする非 wide 互換シンボリックリンクを作成します。

```text
/usr/lib/libncurses.so -> libncursesw.so
/usr/lib/libcurses.so  -> libncursesw.so
```
