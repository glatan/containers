<!-- GENERATED -->
# linux-headers

| 項目 | 値 |
|-|-|
| パッケージ名 | `linux-headers` |
| バージョン | 6.18.56 |
| ライセンス | GPL-2.0-only WITH Linux-syscall-note |
| 分類 | ビルド専用（Bootstrap・seed） |
| 対象 Platform | amd64: v4 |
| host 実行 | 依存として常に BUILD_ENV 側で実行 |
| cc ラッパ | 付与しない |
| 定義 | [`builder/src/package/definition/linux_headers.rs`](../../../../builder/src/package/definition/linux_headers.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://cdn.kernel.org/pub/linux/kernel/v6.x/linux-6.18.56.tar.xz |
| sha3-512 | `1cd94e02690a2e548c49d1f9f10d874b174f8e9f6ad69dab98dea41f58133b2867775dbf1dea44761abb3a673065b27ff8a8fdd5aaf0d695fc30ded5b3cc8a30` |
| blake3 | `e7d2895ae0c1a3151f5a8795643113c736f94996fe86956e5174246b9f018a8f2e82256f215a5c851bb364720235b32fe85e7c1bccd3837d1f346825477841cd` |
| 展開ディレクトリ | `linux-6.18.56` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

なし。

### build-dependencies

なし。

### host-dependencies

なし。

## ビルド設定

| 項目 | 値 |
|-|-|
| env プリリュード | `Env::None`（前処理なし） |
| sysroot | なし |
| ビルドディレクトリ | なし |

## ツールチェーンフラグ

`CFLAGS`/`LDFLAGS` は設定しません。

`-march`/`-mcpu` の値は variant 依存です（上記は amd64/v4 で評価）。

## ビルド手順

`recipe()` が生成するスクリプトです（`builder` が runner へ渡す IR のレンダリング）。対象 Platform 間で差分はありません。

<details>
<summary>常時</summary>

```bash
set -euxo pipefail
mkdir -p "$BUILD/x86_64" "$OUT/usr/x86_64-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/x86_64" ARCH="x86_64" HOSTCC="$CC" headers
cp -a "$BUILD/x86_64/usr/include/." "$OUT/usr/x86_64-linux-gnu/include/"
mkdir -p "$BUILD/arm64" "$OUT/usr/aarch64-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/arm64" ARCH="arm64" HOSTCC="$CC" headers
cp -a "$BUILD/arm64/usr/include/." "$OUT/usr/aarch64-linux-gnu/include/"
mkdir -p "$BUILD/riscv" "$OUT/usr/riscv64-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/riscv" ARCH="riscv" HOSTCC="$CC" headers
cp -a "$BUILD/riscv/usr/include/." "$OUT/usr/riscv64-linux-gnu/include/"
mkdir -p "$BUILD/powerpc" "$OUT/usr/powerpc64le-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/powerpc" ARCH="powerpc" HOSTCC="$CC" headers
cp -a "$BUILD/powerpc/usr/include/." "$OUT/usr/powerpc64le-linux-gnu/include/"
mkdir -p "$BUILD/s390" "$OUT/usr/s390x-linux-gnu/include"
make -C "$SOURCE" O="$BUILD/s390" ARCH="s390" HOSTCC="$CC" headers
cp -a "$BUILD/s390/usr/include/." "$OUT/usr/s390x-linux-gnu/include/"
mkdir -p "$OUT/usr/include"
cp -a "$OUT/usr/${HOST_TRIPLE}/include/." "$OUT/usr/include/"
```

</details>

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

`$OUT/usr/include` には、上表のうち `$HOST_TRIPLE` と一致するディレクトリツリーを複製して配置します。`{triple}-glibc` と openssl / zlib は `usr/${TRIPLE}/include` を優先し、無い場合にこの共通パスへフォールバックします。プレフィックス付きツールチェーンも各 `$OUT/usr/{triple}/include` を使います。`supports` は BUILD_ENV（amd64 / v4）のみで、`runs_on_host` により全消費側はこの BUILD_ENV スコープの 1 系統成果物を使い回します（単一成果物が全アーキテクチャの UAPI ヘッダを内包するため）。
