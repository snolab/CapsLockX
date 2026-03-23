#!/bin/bash
# Build CapsLockX, sign, and auto-restart.
set -e
ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$ROOT/rs"
cargo build -p capslockx-macos --release
cp target/release/capslockx "$ROOT/clx"
codesign -s - --force --identifier "com.snomiao.capslockx" "$ROOT/clx"
echo "[build] done — clx signed"

# Auto-restart: kill old instance, launch new one.
pkill -f 'CapsLockX/clx' 2>/dev/null || true
sleep 0.3
DYLD_LIBRARY_PATH="$ROOT/rs/target/release:${DYLD_LIBRARY_PATH}" "$ROOT/clx" &
echo "[build] clx restarted (pid $!)"
