#!/bin/bash
# Build CapsLockX, sign, and auto-restart.
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/rs"
# Default = full build (voice + AI). Pass --portable for the lite variant.
FEATURES_FLAG="--features full"
if [ "${1:-}" = "--portable" ]; then
    FEATURES_FLAG=""
    shift
fi
cargo build -p capslockx-macos --release --bin capslockx $FEATURES_FLAG

# Compile the Vision OCR helper into ./clx-ocr (Swift binary, no cargo involved).
# Needs Screen Recording permission at runtime to capture window pixels.
OCR_SRC="$ROOT/rs/adapters/macos/src/bin/clx-ocr.swift"
OCR_BIN="$ROOT/clx-ocr"
if [ -f "$OCR_SRC" ] && { [ ! -x "$OCR_BIN" ] || [ "$OCR_SRC" -nt "$OCR_BIN" ]; }; then
    swiftc -O -o "$OCR_BIN" "$OCR_SRC"
    codesign -s - --force --identifier "com.snomiao.capslockx.ocr" "$OCR_BIN"
    echo "[build] clx-ocr compiled + signed"
fi

# Only update the binary if the cargo output is newer than the signed clx.
# This preserves the codesign CDHash (and Accessibility permission) across rebuilds
# that don't change the binary.
CARGO_BIN="target/release/capslockx"
CLX_BIN="$ROOT/clx"

if [ ! -f "$CLX_BIN" ] || ! cmp -s "$CARGO_BIN" "$CLX_BIN" 2>/dev/null; then
    # Binary changed — need to copy, set rpath, and re-codesign.
    cp "$CARGO_BIN" "$CLX_BIN"
    # Set rpath so the binary can find libonnxruntime without DYLD_LIBRARY_PATH.
    install_name_tool -delete_rpath "$ROOT/rs/target/release" "$CLX_BIN" 2>/dev/null || true
    install_name_tool -add_rpath "$ROOT/rs/target/release" "$CLX_BIN"
    # Codesign AFTER install_name_tool (it invalidates any prior signature).
    codesign -s - --force --identifier "com.snomiao.capslockx" "$CLX_BIN"
    echo "[build] done — clx signed with rpath (NEW binary)"
else
    echo "[build] done — binary unchanged, signature preserved"
fi

# Auto-restart: kill old instance, launch via wrapper (sets DYLD_LIBRARY_PATH as fallback).
pkill -f "CapsLockX/clx" 2>/dev/null || true
sleep 0.3
"$ROOT/bin/clx" &
echo "[build] clx restarted (pid $!)"
