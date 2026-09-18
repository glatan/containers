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
| sha3-512 | `c569ffea33791685cd04890e85fc457f7be7bb62b6a478341f9e17ccc2ef248edf084ac4078ef1ea08e43b7aca045c38e49054bde51fa30cd34eebf89269672d` |
| blake3 | `fbec55697a01f99b9cc3f25be55e73ae7091f4c53e5d81a1ea15734c4e5b72385c811def3c51272818645313ef1007cebb8edebdc6cec4a5feab9607a7c6636b` |
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
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `--prefix` | `/usr` | 常時 |
| `--with-shared` | | 常時 |
| `--without-cxx` | | 常時 |
| `--enable-widec` | | 常時 |
| `--enable-pc-files` | | 常時 |
| `--with-pkg-config-libdir` | `/usr/lib/pkgconfig` | 常時 |
| `--without-ada` | | 常時 |
| `--without-debug` | | 常時 |
| `--with-build-cc` / `--with-build-cpp` | `{host_triple}-gcc` | クロスビルド時のみ（HOST のコンパイラを指定） |
| `--with-build-cflags` | `-O2 -std=gnu17` | クロスビルド時のみ |
| `--with-build-cppflags` / `--with-build-ldflags` | 空 | クロスビルド時のみ |
| PGO | 適用しない（`Pgo::None`） | `tic` は libncurses の消費者経路を駆動しないため、意味のあるプロファイルを採取できない |

`--without-cxx` で C++ バインディング（`libncurses++w`）を組みません。`libstdc++.so.6` / `libgcc_s.so.1` への実行時依存を避けるためです。

Wide-character 版ライブラリ（`libncursesw.so`）へリンクする非 wide 互換シンボリックリンクを作成します。

```text
/usr/lib/libncurses.so -> libncursesw.so
/usr/lib/libcurses.so  -> libncursesw.so
```
