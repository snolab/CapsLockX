#!/bin/bash
# Build CapsLockX, sign, and auto-restart.
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/rs"
cargo build -p capslockx-macos --release
cp target/release/capslockx "$ROOT/clx"
# Set rpath so the binary can find libonnxruntime without DYLD_LIBRARY_PATH.
install_name_tool -add_rpath "$ROOT/rs/target/release" "$ROOT/clx" 2>/dev/null || true
codesign -s - --force --identifier "com.snomiao.capslockx" "$ROOT/clx"
echo "[build] done — clx signed with rpath"

# Auto-restart: kill old instance, launch new one.
pkill -9 -x clx 2>/dev/null || true
sleep 0.3
"$ROOT/clx" &
echo "[build] clx restarted (pid $!)"
