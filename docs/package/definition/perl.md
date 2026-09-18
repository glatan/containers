# perl

| 項目 | 値 |
|-|-|
| パッケージ名 | `perl` |
| バージョン | 5.42.3 |
| ライセンス | Artistic-1.0-Perl OR GPL-1.0-or-later |
| パッケージイメージ | 公開対象（[公開一覧](../../image/packages.md)） |
| ネットワーク | 不要（シード環境では builder が導入） |
| 定義 | [`builder/src/package/definition/perl.rs`](../../../builder/src/package/definition/perl.rs) |

## ソース

| 項目 | 値 |
|-|-|
| URL | https://www.cpan.org/src/5.0/perl-5.42.3.tar.xz |
| sha3-512 | `c827af1973438b9173756f5c0b44e668d0443b2bba01b1edf748947c99a5660ecdf5119e0aeada306e47c848bdc2debe7226f45c38d7158b3cf280b1804021a8` |
| blake3 | `986547a8f7de7ad55e2d9877ec29461601b12e4642cf5806194f07b805fea547570e9bcbb2f52df883220190c1a891d1eba37a6f2d82f595b833e4223da68b88` |
| 展開ディレクトリ | `perl-5.42.3` |

## 依存

### dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [x86_64-linux-gnu-glibc](./x86_64-linux-gnu-glibc.md)（他 Platform ではその `{triple}-glibc`） | 2.44 | `--sysroot` によるリンク |
| [libxcrypt](./libxcrypt.md) | 4.5.2 | `libcrypt` |

### build-dependencies

| パッケージ | バージョン | 用途 |
|-|-|-|
| [linux-headers](./linux-headers.md) | 6.18.33 | `-idirafter /dep/linux-headers/usr/include` によるカーネルヘッダ参照 |

### ビルド環境

| 名前 | 用途 |
|-|-|
| 共通の[ビルド環境イメージ](../README.md) | `build` によるコンパイル（シード環境では builder が `gcc` `make` を導入） |

## ビルド設定

対象 Platform と共通前処理は [host-tools.md](../host-tools.md) です。openssl の Configure が実行するのは HOST の perl です。

Autotools ではなく、独自の `Configure` スクリプトを使用します。

| フラグ | 設定値 | 条件 |
|-|-|-|
| `-des` | | 常時 |
| `-Dprefix` | `/usr` | 常時 |
| `-Dvendorprefix` | `/usr` | 常時 |
| `-Duse64bitall` | | 常時 |
| `-Dusethreads` | | 常時 |
| `-Duseshrplib` | | 常時 |
| `-Dcc` / `-Dld` | `$CC` | リンクは gcc ドライバと `-fuse-ld=mold` に任せる。`LD` に GNU ld は置かない |
| `-Accflags` | 既定の CFLAGS（sysroot 含む） | 常時。Configure のコンパイラ検査が `CFLAGS` を読まないため明示する |
| `-Aldflags` | 既定の LDFLAGS（sysroot と `-fuse-ld=mold` を含む） | 常時。同上 |
| `-Dusecrosscompile` | | HOST 以外。`-Dtargethost` は SSH 先ホスト名なので付けない |
| `-Dhostperl` | `/tmp/perl-host/miniperl` | HOST 以外。同じソースを HOST gcc で組んだ `miniperl`。インストール済み HOST perl はターゲットの `.so` を読み込む |
| `PERL_USE_UNSAFE_INC` | `1` | HOST 以外。HOST の perl を `miniperl` として使うとき、`.` が `@INC` に無い |
| Configure try | 実行しない | HOST 以外。対象 glibc を qemu-user で走らせると高い `-march` で SIGILL になる。C99 はコンパイル成否だけを見る。コンパイラ一貫性検査はコンパイル成功を `Ok` とみなす。型サイズ・アラインメント・printf 書式は `-Dcharbits=8` `-Dalignbytes=8` `-Divdformat='"ld"'` `-Uebcdic` などで明示する。5.42 は `-Dlseeksize` が `config.h` に落ちないので、Configure 後に `config.sh` を補って `./Configure -S` する |
| HOST perl の `libcrypt` | `/dep/libxcrypt-host` | extra_host の HOST perl とクロス時の HOST miniperl。ターゲットの `/dep/libxcrypt` を `LD_LIBRARY_PATH` に載せない |
| `-Dsysroot` | `/dep/${TRIPLE}-glibc/usr/${TRIPLE}` | HOST 以外 |
| `-Darchname` | `$TRIPLE` | HOST 以外 |
| LTO | 適用しない | `Configure` スクリプトのプローブ処理と競合するため |
| PGO | 適用しない（`Pgo::None`） | 独自 `Configure` がフラグを管理し、PGO 手順の注入経路がないため |
