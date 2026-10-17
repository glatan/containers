<!-- GENERATED -->
# meson

| 項目 | 値 |
|-|-|
| パッケージ名 | `meson` |
| バージョン | 1.12.1 |
| ライセンス | Apache-2.0 |
| 分類 | パッケージ |
| 対象 Platform | amd64: v1 v2 v3 v4; arm64: v8 v8.1 v8.2 v8.3 v8.4 v8.5 v8.6 v8.7 v8.8 v8.9 v9 v9.1 v9.2 v9.3 v9.4 v9.5; ppc64le: power8 power9 power10 power11; riscv64: rva20u64 rva22u64 rva23u64; s390x: z900 z990 z9-ec z10 z196 zEC12 z13 z14 z15 z16 z17 |
| host 実行 | 依存として常に BUILD_ENV 側で実行 |
| 定義 | [`builder/src/package/definition/meson.rs`](../../../../builder/src/package/definition/meson.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://github.com/mesonbuild/meson/releases/download/1.12.1/meson-1.12.1.tar.gz |
| sha3-512 | `82f6c97fdbf6877c326cb1c82cc9b3f1c3e9d0f2de16a11e9cc4e4c72d47bce89ca53c4d13802f579f47b50359a2b3171762e8c2d7561ce11789086baf2d9221` |
| blake3 | `f6d4541d0a5129992e93ce9e55a5ec7608781381451e84e9a4c2afd6400f13e077412def1775d18c6ba3b84a9fbe28edaef8d3b6621df794c7a3edbd339564d9` |
| 展開ディレクトリ | `meson-1.12.1` |

## 依存

`amd64/v4`（`Target::BUILD_ENV`）で評価した宣言です。他 Platform では `{triple}-glibc` などがその Platform の triple 名に変わり、ターゲット条件付きの辺（クロス時の自己参照など）はここには現れません。via 列は充足方法です（`seed` = シード相は distro パッケージが担保・合成 env では成果物必須、`artifact` = 常に成果物必須、`optional` = あれば使う）。

### dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [python](./python.md) | 3.15.0 | — |

### build-dependencies

なし。

### host-dependencies

| パッケージ | バージョン | via |
|-|-|-|
| [python](./python.md) | 3.15.0 | — |

host-dependencies は `dep:false` でマウントされ、PATH には載りますがターゲット向け `-I` / `-L` には載りません。
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
<summary>常時（ネイティブ・クロス混在・Canadian 混在）</summary>

```bash
set -euxo pipefail
_pyver=$(python3 -c 'import sys; print(f"python{sys.version_info.major}.{sys.version_info.minor}")')
_site="${OUT}/usr/lib/${_pyver}/site-packages"
mkdir -p "${_site}" "${OUT}/usr/bin"
cp -a "${SOURCE}"/mesonbuild "${_site}"/
cat > "${OUT}/usr/bin/meson" <<'EOF'
#!/bin/sh
exec python3 -m mesonbuild.mesonmain "$@"
EOF
chmod +x "${OUT}/usr/bin/meson"
```

</details>

<!-- /GENERATED -->





## 補足

他パッケージがビルド時に実行する meson は HOST 成果物をマウントします。C 言語のコンパイルは行いません。`$SOURCE/mesonbuild` を `$OUT/usr/lib/python{ver}/site-packages/` へコピーし、`python3 -m mesonbuild.mesonmain` を呼ぶ `usr/bin/meson` を生成します（python 成果物は `--without-ensurepip` で pip を持たないため）。

PGO は適用しません。C 言語のコンパイルを伴わないため適用しません（`Pgo::None`）。
