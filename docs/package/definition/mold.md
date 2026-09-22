<!-- GENERATED -->
# mold

| 項目 | 値 |
|-|-|
| パッケージ名 | `mold` |
| バージョン | 2.42.1 |
| ライセンス | MIT |
| 分類 | パッケージ（Bootstrap） |
| 定義 | [`builder/src/package/definition/mold.rs`](../../../builder/src/package/definition/mold.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/rui314/mold/archive/refs/tags/v2.42.1.tar.gz |
| sha3-512 | `893446caeb9a904d3d796a8fc3f5c6f6361b7f2ea0d164ee56fcd48a8976857984e10faf51bb4610808faf43d87dd73bb4d02bd022dc6b93b42f80727e258ef4` |
| blake3 | `ab64ef69b9c842942058e678423ce9fb3901f66725295fbd4f426f14af549f62d6f818fa4161ae6a6bdec58d4f27485f65b2ebfb3374e0f3714a81066555ceca` |
| 展開ディレクトリ | `mold-2.42.1` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md) | 2.44 | — |
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | — |
| [zlib](./zlib.md) | 1.3.2 | — |
| [zstd](./zstd.md) | 1.5.7 | — |

### build-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | — |

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [x86_64-linux-gnu-gcc](./x86_64-linux-gnu-gcc.md) | 16.2.0 | seed |
| [x86_64-linux-gnu-binutils](./x86_64-linux-gnu-binutils.md) | 2.47 | seed |
| [cmake](./cmake.md) | 4.4.3 | — |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
<!-- /GENERATED -->



実行時はパッケージ Platform の `{triple}-glibc`、`{triple}-gcc`（libstdc++ / libgcc_s / libatomic）、zlib、zstd です。ビルド専用は linux-headers と cmake です。自分自身には `-fuse-ld=mold` を付けません。シード中のリンクは Arch の mold です。compose 後の標準リンカは自前 mold です。対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。他パッケージがビルド時に実行する cmake は HOST 成果物をマウントします。

## ビルド設定

| フラグ | 設定値 | 条件 |
|-|-|-|
| `CMAKE_BUILD_TYPE` | `Release` | 常時 |
| `CMAKE_INSTALL_PREFIX` | `/usr` | 常時 |
| `-fuse-ld=mold` | 付与しない | 常時（自身を mold でリンクしない。`Ldflag::FuseLdMold` を flags から除外） |
| PGO | ネイティブビルドのみ適用（`Pgo::Custom`） | `-fprofile-generate` の instrumented ビルドを組み、その mold でオブジェクトのリンクを実行してプロファイルを採取。`-fprofile-use` の成果物を別ツリーで組む |
