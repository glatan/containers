# 利用側の機能制限

公開済みパッケージイメージの利用（`containers-builder sync` と Dockerfile ビルド）ができないこと・保証しないことをまとめます。環境要件は [requirements.md](./requirements.md)、手順は [consume.md](./consume.md) を参照してください。公開側の制限は [../dev/distribution.md](../dev/distribution.md) です。

## 検査しないもの

- mode `block` ではパス競合を検査しません。blob を pull しないため検査の手段がなく、パッケージ間で同じパスが存在する場合は `COPY --from=` の後勝ちで警告なく上書きされます。競合の検出が必要な場合は mode `compose` を使用します。
- 生成ステージ名との衝突は検査しません。`pkg-*` や `containers-packages` をユーザーステージ名として定義すると `docker build` が失敗します。

## mode: block の制限

- `FROM scratch` では `RUN` が使えないため、usrmerge（`/bin`・`/sbin` symlink）を生成ブロックで補えません。`{triple}-glibc` の同梱 symlink に依存し、glibc を含まない閉包（`ca-certificates` 単独など）では `/bin`・`/sbin` の symlink を持たないイメージになります。
- 1 つの Dockerfile に配置できる生成ブロックは 1 つだけです。1 ファイル内で複数の platform を切り替える構成は記述できません。

## mode: compose の制限

- Dockerfile に書き込む pin はネスト index digest ではなく、選択 platform の子 manifest digest です。index digest で pin すると containerd/platforms の platform マッチングにより variant 付き manifest が解決不能になるためです。
- 合成イメージに SPDX SBOM は付与されません。
- `--no-push` の ARG は `oci:{ローカルパス}`（skopeo の転送形式）を指すため、`docker build` の `FROM` 参照としては利用できません。ローカル生成物の検査用途に限られます。

## platform 解決の制限

- `--platform` 修飾は containerd/platforms が正規化できる `linux/amd64/v1`–`v4` と `linux/arm64/v8`・`v9` にのみ付与します。`ppc64le/powerN`、`riscv64/rvaN`、`s390x/zN`、`arm64` の `v8.x`/`v9.x` 系には付けず、プラットフォームの適合性は digest pin によって担保されます。
- 依存先が要求 platform の manifest を持たない場合や、依存関係に循環がある場合は解決エラーとなります。別 platform へのフォールバックは行いません。

## 運用上の制限

- index の `"auto"` 解決は `{8桁の日付}-{数値rev}` 形式のタグのみを対象とします。それ以外の命名規則のタグは候補から除外されます。
- `sync` は `--check` であってもリモートの manifest digest 照合を行うためネットワーク接続が必要であり、オフラインでは動作しません。
- キャッシュ（`$XDG_CACHE_HOME/containers-builder/`）に GC はなく、digest やタグ単位で蓄積し続けます。
- GHCR で private 公開のイメージ（個別パッケージまたは合成ベースイメージ）を利用するには、ビルド環境側でのレジストリ認証が別途必要です。
