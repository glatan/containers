# pkgconf

| 項目 | 値 |
|-|-|
| パッケージ名 | `pkgconf` |
| バージョン | 2.5.1 |
| ライセンス | ISC |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/pkgconf.rs`](../../../builder/src/package/definition/pkgconf.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://distfiles.ariadne.space/pkgconf/pkgconf-2.5.1.tar.xz |
| sha3-512 | `9c27796bf156adec40eaf2a10be4f30c53ef8f99f810984116dfc77a2f8f0868a9c203337bbd6661ad08da58b6f192ce33a360476c181479062d804e25490974` |
| blake3 | `c524d73c938e7382d3c12c9cfba16ed32ee092ab21b3e9659a095b5628ea2e8854d574e1591345eb70d3eeb5e08cd99af007751a488adc9b16ad9d6dd24c7081` |
| 展開ディレクトリ | `pkgconf-2.5.1` |

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
| `--with-pkg-config-dir` | `/usr/lib/pkgconfig:/usr/share/pkgconfig` | 常時 |
| `--with-system-libdir` | `/usr/lib` | 常時 |
| `--with-system-includedir` | `/usr/include` | 常時 |
| PGO | ネイティブビルドのみ適用（`Pgo::Profiled`） | `/dep`・`/usr` の `.pc` ファイルを `pkgconf --cflags --libs` / `--modversion` で解決 |

`pkg-config` 互換のシンボリックリンクを作成します。

```text
/usr/bin/pkg-config -> pkgconf
```
