#!/bin/bash
# Prepare macOS development dependencies, then build and launch CapsLockX.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SIGNING=0
BUILD=1
for arg in "$@"; do
    case "$arg" in
        --signing) SIGNING=1 ;;
        --no-build) BUILD=0 ;;
        -h|--help)
            echo "Usage: ./scripts/setup-mac.sh [--signing] [--no-build]"
            echo "Check Apple/Rust tools, install missing CMake, then run build.sh."
            echo "  --signing   Create the local signing identity in your login keychain."
            echo "  --no-build  Prepare dependencies without building or launching."
            exit 0 ;;
        *) echo "Unknown option: $arg (use --help)" >&2; exit 2 ;;
    esac
done

if [ "$(uname -s)" != Darwin ]; then
    echo "[setup] This script requires macOS." >&2
    exit 1
fi

# Also work from shells that have not loaded Homebrew or rustup's PATH yet.
export PATH="$PATH:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/local/bin"
if ! xcrun --find clang >/dev/null 2>&1 || ! xcrun --find swiftc >/dev/null 2>&1; then
    echo "[setup] Install Apple Command Line Tools with xcode-select --install, then rerun." >&2
    exit 1
fi
if ! cargo --version >/dev/null 2>&1 || ! rustc --version >/dev/null 2>&1; then
    echo "[setup] Install a Rust toolchain using https://rustup.rs, then rerun." >&2
    exit 1
fi

if ! command -v cmake >/dev/null 2>&1; then
    if ! command -v brew >/dev/null 2>&1; then
        echo "[setup] CMake is missing. Install Homebrew from https://brew.sh, then rerun." >&2
        exit 1
    fi
    echo "[setup] Installing CMake for the Whisper voice dependency…"
    brew install cmake
fi
cmake --version

if [ "$SIGNING" -eq 1 ]; then
    # The signing helper's temporary private key stays in the project temp dir.
    mkdir -p "$ROOT/tmp"
    TMPDIR="$ROOT/tmp/" "$ROOT/scripts/setup-dev-signing.sh"
elif ! security find-certificate -c "${CLX_SIGN_IDENTITY:-CapsLockX Dev Signing}" >/dev/null 2>&1; then
    echo "[setup] No development signing certificate; build.sh will use ad-hoc signing."
    echo "[setup] Run ./scripts/setup-mac.sh --signing to set up a stable local identity."
fi

if [ "$BUILD" -eq 1 ]; then
    # Prevent the launcher's source check from recursively rebuilding.
    CLX_REBUILDING=1 "$ROOT/build.sh"
else
    echo "[setup] Dependencies ready. Run ./build.sh to build and launch."
fi
