set -euxo pipefail
install -D -m 644 "$SOURCE"/certifi/cacert.pem "$OUT"/etc/ssl/certs/ca-certificates.crt
ln -s ca-certificates.crt "$OUT"/etc/ssl/certs/ca-bundle.crt
ln -s certs/ca-certificates.crt "$OUT"/etc/ssl/cert.pem
