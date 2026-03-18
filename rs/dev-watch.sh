#!/bin/bash
# HMR-like dev experience: auto-rebuild + restart on file change.
# Usage: ./rs/dev-watch.sh
export PATH="$HOME/.cargo/bin:/opt/homebrew/bin:$PATH"
cd "$(dirname "$0")"

cargo watch \
  -w adapters/macos/src -w core/src \
  -s 'cargo build -p capslockx-macos --release 2>&1 && (
    pkill -f "target/release/capslockx" 2>/dev/null
    sleep 0.3
    ./target/release/capslockx 2>/tmp/clx-debug.log &
    echo "[HMR] ✓ restarted capslockx"
  ) || echo "[HMR] ✗ build failed"'
