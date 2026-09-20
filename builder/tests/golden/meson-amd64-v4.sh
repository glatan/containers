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
