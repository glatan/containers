<!-- GENERATED -->
# linux-headers

| 項目 | 値 |
|-|-|
| パッケージ名 | `linux-headers` |
| バージョン | 6.18.33 |
| ライセンス | GPL-2.0-only WITH Linux-syscall-note |
| 分類 | ビルド専用（Bootstrap・seed） |
| 定義 | [`builder/src/package/definition/linux_headers.rs`](../../../builder/src/package/definition/linux_headers.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.18.33.tar.xz |
| sha3-512 | `ad87ec123afe83db49e304a4f32d1c529d525f2e55d8c78de208cff0de2b1306d1eaa35d0b49a3738ffa170977e252e1e85fa58654710f11de9f4267947258ca` |
| blake3 | `00d95e1098670a56ee94558692f1e63e0ac3d2113c9e46568e2da9423a37ad8855a530aba3e6f67bc43beacb5edd7a54834cb0f946112630143b7b7bd930f707` |
| 展開ディレクトリ | `linux-6.18.33` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

なし。

### build-dependencies

なし。

### host-dependencies

なし。

<!-- /GENERATED -->




## ヘッダファイルの配置構成

単一の成果物内に、5 つの GNU triple に対応するカーネルユーザー空間ヘッダをまとめて配置します。`$ARCH` による条件分岐は行わず、コンパイラフラグの要求や LTO・PGO の適用も行いません。

| GNU triple | カーネル `ARCH` | 出力先パス |
|-|-|-|
| `x86_64-linux-gnu` | `x86_64` | `$OUT/usr/x86_64-linux-gnu/include` |
| `aarch64-linux-gnu` | `arm64` | `$OUT/usr/aarch64-linux-gnu/include` |
| `riscv64-linux-gnu` | `riscv` | `$OUT/usr/riscv64-linux-gnu/include` |
| `powerpc64le-linux-gnu` | `powerpc` | `$OUT/usr/powerpc64le-linux-gnu/include` |
| `s390x-linux-gnu` | `s390` | `$OUT/usr/s390x-linux-gnu/include` |

`$OUT/usr/include` には、上表のうち `$HOST_TRIPLE` と一致するディレクトリツリーを複製して配置します。`{triple}-glibc` と openssl / zlib は `usr/${TRIPLE}/include` を優先し、無い場合にこの共通パスへフォールバックします。プレフィックス付きツールチェーンも各 `$OUT/usr/{triple}/include` を使います。`supports` は全 Platform で真ですが、成果物は HOST の 1 系統を使い回します。
