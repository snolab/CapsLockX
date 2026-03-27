#!/bin/bash
# Build CapsLockX, sign, and auto-restart.
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/rs"
cargo build -p capslockx-macos --release
cp target/release/capslockx "$ROOT/clx"
# Set rpath so the binary can find libonnxruntime without DYLD_LIBRARY_PATH.
# Remove first to avoid "already exists" error on repeated builds, then add.
install_name_tool -delete_rpath "$ROOT/rs/target/release" "$ROOT/clx" 2>/dev/null || true
install_name_tool -add_rpath "$ROOT/rs/target/release" "$ROOT/clx"
# Codesign AFTER install_name_tool (it invalidates any prior signature).
codesign -s - --force --identifier "com.snomiao.capslockx" "$ROOT/clx"
echo "[build] done — clx signed with rpath"

# Auto-restart: kill old instance, launch via wrapper (sets DYLD_LIBRARY_PATH as fallback).
pkill -f "CapsLockX/clx" 2>/dev/null || true
sleep 0.3
"$ROOT/bin/clx" &
echo "[build] clx restarted (pid $!)"
