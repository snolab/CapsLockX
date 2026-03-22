#!/bin/bash
# Build CapsLockX and sign the binary so macOS permissions persist across rebuilds.
set -e
cd "$(dirname "$0")/rs"
cargo build -p capslockx-macos --release
cp target/release/capslockx ../clx
codesign -s - --force ../clx
echo "[build] done — ./clx is signed and ready"
