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

# ort (ten-vad) runs in load-dynamic mode and dlopens ONNX Runtime at runtime.
# Make sure the matching dylib sits next to the dev binaries (target/release);
# the bin/clx wrapper points ORT_DYLIB_PATH at it. Cached — downloads once.
"$ROOT/scripts/fetch-ort-dylib.sh" "$(uname -m)" "$ROOT/rs/target/release" >/dev/null || \
    echo "[build] warning: could not fetch ONNX Runtime dylib (voice VAD may not load)" >&2

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

# ── Lightweight dev app bundle (Launch-at-Login branding) ───────────────────
# Wraps the already-signed dev `clx` binary in a minimal .app so System
# Settings → Privacy & Security → Accessibility shows it as "CapsLockX" with
# its icon, instead of a bare unbranded executable — macOS only reads
# Info.plist/icon branding for a process running from inside a real .app
# bundle. NOT the release bundle (see scripts/package-mac.sh for that): no
# path remapping, no zip/pkg, just a copy + static Info.plist, so it's cheap
# enough to refresh on every build.
DEV_APP="$ROOT/dist/CapsLockX-dev.app"
mkdir -p "$DEV_APP/Contents/MacOS" "$DEV_APP/Contents/Resources"

ICNS="$DEV_APP/Contents/Resources/CapsLockX.icns"
if [ ! -f "$ICNS" ]; then
    ICONSET="$ROOT/dist/.CapsLockX-dev.iconset"
    rm -rf "$ICONSET" && mkdir -p "$ICONSET"
    for s in 16 32 128 256 512; do
        sips -z "$s" "$s" "$ROOT/Data/XIconBlue.png" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
    done
    cp "$ICONSET/icon_32x32.png"   "$ICONSET/icon_16x16@2x.png"
    cp "$ICONSET/icon_256x256.png" "$ICONSET/icon_128x128@2x.png"
    cp "$ICONSET/icon_512x512.png" "$ICONSET/icon_256x256@2x.png"
    iconutil -c icns "$ICONSET" -o "$ICNS"
    rm -rf "$ICONSET"
fi

cat > "$DEV_APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>CapsLockX</string>
    <key>CFBundleDisplayName</key><string>CapsLockX</string>
    <key>CFBundleIdentifier</key><string>com.snomiao.capslockx</string>
    <key>CFBundleExecutable</key><string>clx</string>
    <key>CFBundleIconFile</key><string>CapsLockX.icns</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>LSMinimumSystemVersion</key><string>12.0</string>
    <key>LSUIElement</key><true/>
    <key>NSHighResolutionCapable</key><true/>
</dict>
</plist>
PLIST

# Copy the built binary in, then codesign the WHOLE .app bundle. Signing the
# bundle (not just the inner Mach-O) is required so the Info.plist and
# Resources are sealed — otherwise `codesign --verify` reports "code has no
# resources but signature indicates they must be present" / "Info.plist not
# bound", TCC sees a broken app identity, and the Accessibility grant won't
# attach to the CapsLockX entry. The stable --identifier keeps that grant
# persistent across rebuilds (same rule as the raw clx binary).
cp "$CLX_BIN" "$DEV_APP/Contents/MacOS/clx"
codesign -s - --force --identifier "com.snomiao.capslockx" "$DEV_APP" >/dev/null 2>&1
if codesign --verify --strict "$DEV_APP" 2>/dev/null; then
    echo "[build] dev app bundle refreshed + signed: $DEV_APP"
else
    echo "[build] WARNING: dev app bundle signature failed to verify" >&2
fi

# Auto-restart: kill old instance, launch via wrapper (sets DYLD_LIBRARY_PATH as fallback).
pkill -f "CapsLockX/clx" 2>/dev/null || true
sleep 0.3
"$ROOT/bin/clx" &
echo "[build] clx restarted (pid $!)"
