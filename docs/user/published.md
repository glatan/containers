# パッケージイメージ一覧

公開対象とするパッケージイメージの分類です。ビルド対象の全件は [../dev/package/definition/README.md](../dev/package/definition/README.md) です。ビルド専用（`linux-headers` とプレフィックス付きツールチェーンの中間ステージ）は公開しません。

個別のソース、依存、固有フラグは [definition/](../dev/package/definition/) の `{パッケージ名}.md` です。

## カタログ

[targets.md](./targets.md) の `Target::all()` 向けに提供します。実行時 libc は各 Platform の `{triple}-glibc` です。

| パッケージ名 | バージョン | ビルド設定 |
|-|-|-|
| ca-certificates | 2026.07.22 | [ca-certificates.md](../dev/package/definition/ca-certificates.md) |
| openssl | 4.0.2 | [openssl.md](../dev/package/definition/openssl.md) |
| zlib | 1.3.2 | [zlib.md](../dev/package/definition/zlib.md) |
| zlib-ng | 2.3.3 | [zlib-ng.md](../dev/package/definition/zlib-ng.md) |

## プレフィックス付きツールチェーン

公開するのは最終 `{triple}-binutils`、`{triple}-glibc`、`{triple}-gcc` です。製品キューと `supports` の関係は [toolchain.md](../dev/package/toolchain.md) です。パッケージごとのファイルは [definition/](../dev/package/definition/) です。

| パッケージ名 | バージョン |
|-|-|
| `{triple}-binutils` | 2.47 |
| `{triple}-glibc` | 2.44 |
| `{triple}-gcc` | 16.2.0 |

## Host tools

qemu 以外は `Target::all()` 向け成果物も出します。対象パッケージの列挙と共通前処理は [host-tools.md](../dev/package/host-tools.md) が正本です。各パッケージのファイルは [definition/](../dev/package/definition/) です。

## amd64 / v4 のみ

| パッケージ名 | バージョン | ビルド設定 |
|-|-|-|
| qemu | 11.1.1 | [qemu.md](../dev/package/definition/qemu.md) |
| build-env | 1 | [build-env.md](../dev/package/definition/build-env.md) |
