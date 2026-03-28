#!/bin/bash
# Auto-recompile and restart CLX core on code changes.
# Usage: ./dev-clx.sh
cd "$(dirname "$0")/rs"
export DYLD_LIBRARY_PATH="$(pwd)/target/release:${DYLD_LIBRARY_PATH}"

cargo watch \
  -w core/src \
  -w adapters/macos/src \
  -s '
    pkill -f "CapsLockX/clx" 2>/dev/null
    cargo build -p capslockx-macos --release 2>&1 | tail -5 \
    && cp target/release/capslockx ../clx \
    && codesign -s - --force --identifier "com.snomiao.capslockx" ../clx 2>/dev/null \
    && echo "[dev] restarting CLX..." \
    && clx &
  '
