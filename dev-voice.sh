#!/bin/bash
# Auto-recompile and restart voice-standalone on code changes.
# Usage: ./dev-voice.sh
cd "$(dirname "$0")/rs"
export DYLD_LIBRARY_PATH="$(pwd)/target/release:${DYLD_LIBRARY_PATH}"

cargo watch \
  -w core/src \
  -w adapters/macos/src \
  -s 'killall voice-standalone 2>/dev/null; cargo build -p capslockx-macos --bin voice-standalone --release 2>&1 | tail -5 && echo "[dev] restarting voice-standalone..." && target/release/voice-standalone &'
