# パッケージイメージ一覧

ビルド専用である `linux-headers`、`{triple}-gcc-stage1`、`{triple}-glibc-headers`、`{triple}-gcc-stage2` を除くパッケージイメージを公開します。

## カタログ

[targets.md](./targets.md) に記載した全 platform 向けに公開します。openssl、zlib、zlib-ng は実行時 libc として各 platform の `{triple}-glibc` に依存します。一方、ca-certificates は証明書ファイルのみを配置するため、実行時依存を持ちません。

| パッケージ名 | バージョン |
|-|-|
| ca-certificates | 2026.07.22 |
| openssl | 4.0.3 |
| zlib | 1.3.2 |
| zlib-ng | 2.3.3 |

## プレフィックス付きツールチェーン

ツールチェーンからは、最終生成物である `{triple}-binutils`、`{triple}-glibc`、`{triple}-gcc` を公開します。

`{triple}-glibc` の公開 platform は、その architecture に属する variant のみです。たとえば `aarch64-linux-gnu-glibc` は arm64 の各 variant 向けに公開し、amd64/v4 向けには提供しません。

`{triple}-gcc` および `{triple}-binutils` の公開 platform は、ビルド環境である amd64/v4 と、各ターゲット architecture に属する variant です。たとえば `aarch64-linux-gnu-gcc` はクロスコンパイル用の amd64/v4 とターゲット用の arm64 各 variant 向けに公開します。一方、`x86_64-linux-gnu-gcc` はターゲットが amd64 であるため、amd64 の各 variant 向けのみを公開します。

| パッケージ名 | バージョン |
|-|-|
| `{triple}-binutils` | 2.47 |
| `{triple}-glibc` | 2.44 |
| `{triple}-gcc` | 16.2.0 |

## Host tools

以下のパッケージは、[targets.md](./targets.md) に記載した全 platform 向けに公開します。

| パッケージ名 | バージョン |
|-|-|
| gmp | 6.3.0 |
| mpfr | 4.2.2 |
| mpc | 1.4.1 |
| isl | 0.28 |
| zstd | 1.5.7 |
| mold | 3.0.0 |
| libffi | 3.8.0 |
| pcre2 | 10.49 |
| ncurses | 6.6 |
| readline | 8.3 |
| expat | 2.9.0 |
| sqlite | 3.54.0 |
| libxcrypt | 4.5.2 |
| glib | 2.90.1 |
| bash | 5.3 |
| coreutils | 9.12 |
| xz | 5.8.4 |
| bzip2 | 1.0.8 |
| flex | 2.6.4 |
| perl | 5.44.0 |
| pkgconf | 3.0.8 |
| ninja | 1.13.2 |
| meson | 1.12.1 |
| python | 3.15.0 |
| cmake | 4.4.4 |
| make | 4.4.1 |
| m4 | 1.4.21 |
| bison | 3.8.2 |
| findutils | 4.11.0 |
| sed | 4.10 |
| grep | 3.12 |
| gawk | 5.4.1 |
| diffutils | 3.12 |
| patch | 2.8 |
| tar | 1.35 |
| gzip | 1.15 |
| texinfo | 7.3 |

## amd64 / v4 のみ

以下のパッケージは、ホスト環境である amd64 / v4 向けにのみ公開します。クロスビルド環境のホスト上で実行されるツール群、およびホスト用ビルド環境イメージです。

| パッケージ名 | バージョン |
|-|-|
| qemu | 11.1.2 |
| rust | 1.99.0 |
| build-env | 1 |
