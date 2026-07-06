#!/bin/bash
# Download the ONNX Runtime dylib that `ort` (built in load-dynamic mode) dlopens
# at runtime. ort ships no prebuilt ONNX Runtime for x86_64-apple-darwin and, on
# arm64, statically linking it collides with sherpa's separate copy — so we load
# it dynamically and bundle the matching build here.
#
# Usage:   fetch-ort-dylib.sh <arm64|x86_64> <dest-dir>
# Prints:  absolute path to the dylib (cached — re-download is skipped).
set -euo pipefail

ARCH="$1"
DEST="$2"
ORT_VER=1.23.2   # MUST match the ONNX Runtime version ort rc.11 targets.

case "$ARCH" in
    arm64|aarch64) PKG="onnxruntime-osx-arm64-$ORT_VER" ;;
    x86_64)        PKG="onnxruntime-osx-x86_64-$ORT_VER" ;;
    *) echo "fetch-ort-dylib: unsupported arch '$ARCH'" >&2; exit 1 ;;
esac

DYLIB="$DEST/libonnxruntime.$ORT_VER.dylib"
if [ ! -f "$DYLIB" ]; then
    mkdir -p "$DEST"
    TMP="$(mktemp -d)"
    curl -fsSL "https://github.com/microsoft/onnxruntime/releases/download/v$ORT_VER/$PKG.tgz" \
        | tar xz -C "$TMP"
    cp "$TMP/$PKG/lib/libonnxruntime.$ORT_VER.dylib" "$DYLIB"
    rm -rf "$TMP"
fi
echo "$DYLIB"
