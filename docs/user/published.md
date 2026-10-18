# パッケージイメージ一覧

公開するパッケージイメージの分類です。`linux-headers`、`{triple}-gcc-stage1`、`{triple}-glibc-headers`、`{triple}-gcc-stage2` は公開しません。

## カタログ

対象は [targets.md](./targets.md) の platform です。実行時 libc のパッケージ名は `{triple}-glibc` です。

| パッケージ名 | バージョン |
|-|-|
| ca-certificates | 2026.07.22 |
| openssl | 4.0.3 |
| zlib | 1.3.2 |
| zlib-ng | 2.3.3 |

## プレフィックス付きツールチェーン

公開するのは最終の `{triple}-binutils`、`{triple}-glibc`、`{triple}-gcc` です。

`{triple}-glibc` の公開 platform は、その architecture の variant だけです。`aarch64-linux-gnu-glibc` は arm64 の variant だけで、amd64/v4 にはありません。

`{triple}-gcc` と `{triple}-binutils` の公開 platform は、amd64/v4 と、そのパッケージの architecture に属する variant です。`x86_64-linux-gnu-gcc` は amd64 の variant だけです。`aarch64-linux-gnu-gcc` は amd64/v4 と arm64 の variant です。`x86_64-linux-gnu-gcc` を arm64 向けには公開しません。

| パッケージ名 | バージョン |
|-|-|
| `{triple}-binutils` | 2.47 |
| `{triple}-glibc` | 2.44 |
| `{triple}-gcc` | 16.2.0 |

## Host tools

qemu 以外の対象は [targets.md](./targets.md) の platform です。

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

| パッケージ名 | バージョン |
|-|-|
| qemu | 11.1.2 |
| build-env | 1 |
