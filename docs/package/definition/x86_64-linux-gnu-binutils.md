<!-- GENERATED -->
# x86_64-linux-gnu-binutils

| 項目 | 値 |
|-|-|
| パッケージ名 | `x86_64-linux-gnu-binutils` |
| バージョン | 2.47 |
| ライセンス | GPL-3.0-or-later |
| 分類 | パッケージ（Bootstrap・seed） |
| 定義 | [`builder/src/package/definition/x86_64_linux_gnu_binutils.rs`](../../../builder/src/package/definition/x86_64_linux_gnu_binutils.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://ftp.gnu.org/gnu/binutils/binutils-2.47.tar.xz |
| sha3-512 | `b2a39d2a1bfff7f27c2619e15c5d9dbf0c7918a85120975f4146c84b4a7d660bd5dc3793ea6d009df89ed865aaef4362805e949438db0f8acf97fd113ca1a1f3` |
| blake3 | `05a142fb2225e617a3fa30224d5d275682e5cf03a62a15af804c03214cc3a5cfadec1215345da15c94ba7a2e13b9667c094e59a26c39412a47265724cbb62a8e` |
| 展開ディレクトリ | `binutils-2.47` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | seed:glibc |
| [zlib](./zlib.md) | 1.3.2 | seed:zlib |
| [zstd](./zstd.md) | 1.5.7 | seed:zstd |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [mold](./mold.md) | 2.42.1 | optional |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



## ビルド設定

configure は `--build=$HOST_TRIPLE --host=$HOST_TRIPLE --target=x86_64-linux-gnu` です。ステージ順とプレフィックスは [toolchain.md](../toolchain.md) です。ネイティブ（`$PACKAGE_HOST = $HOST_TRIPLE`）では PGO を適用します（`Pgo::Custom`。instrumented ツリーで `as` / `ld` / `objdump` を駆動してプロファイルを採取）。クロスでは適用しません。

`supports` は `Target::all()` です。製品キューは [toolchain.md](../toolchain.md)。このパッケージでは HOST と、architecture `amd64` の variant（v1, v2, v3, v4）です。
