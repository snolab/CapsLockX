#!/bin/bash
# Build a self-contained, signed CapsLockX.app and release artifacts (.pkg + .zip).
#
# The bundle is relocatable: dylibs live in Contents/Frameworks (reached via an
# @executable_path/../Frameworks rpath), helper binaries and skills travel inside
# the bundle, and the tray icon is compiled into the binary. The result can be
# dropped in /Applications, installed via the .pkg, or unzipped anywhere.
#
# Usage:
#   scripts/package-mac.sh              # full build (voice + AI) then package
#   scripts/package-mac.sh --no-build   # package whatever is already in target/release
#
# Output (dist/):
#   CapsLockX.app                       signed app bundle
#   CapsLockX-<ver>-macos-arm64.zip     ditto archive (preserves signature)
#   CapsLockX-<ver>-macos-arm64.pkg     installer (drops the app in /Applications)
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

VERSION="$(grep -m1 '"version"' package.json | sed -E 's/.*"version": *"([^"]+)".*/\1/')"
ARCH="$(uname -m)"
APP_ID="com.snomiao.capslockx"
APP_NAME="CapsLockX"
# Dedicated target dir: the path-remapped RUSTFLAGS below differ from the dev
# build, so isolating here keeps `build.sh`'s incremental cache warm.
export CARGO_TARGET_DIR="$ROOT/rs/target/dist"
TARGET="$CARGO_TARGET_DIR/release"
DIST="$ROOT/dist"
APP="$DIST/$APP_NAME.app"

NO_BUILD=0
[ "${1:-}" = "--no-build" ] && NO_BUILD=1

# ── 1. Build release binaries (voice + AI) ───────────────────────────────────
# Remap absolute build paths so the PUBLIC release binaries don't leak the
# builder's username / home layout. Rust bakes dependency source paths into
# panic metadata (e.g. /Users/<you>/.cargo/registry/...); these rules strip the
# real home and repo prefixes. Changing RUSTFLAGS forces a full recompile.
CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$CARGO_HOME_DIR=cargo --remap-path-prefix=$ROOT=. --remap-path-prefix=$HOME=home"
# --remap-path-prefix is a rustc flag; native -sys crates (whisper.cpp, etc.)
# are built by cc-rs and need the clang equivalent or they leak the build path.
PREFIX_MAP="-ffile-prefix-map=$ROOT=. -ffile-prefix-map=$HOME=home"
export CFLAGS="${CFLAGS:-} $PREFIX_MAP"
export CXXFLAGS="${CXXFLAGS:-} $PREFIX_MAP"
if [ "$NO_BUILD" -eq 0 ]; then
    echo "[pkg] building release binaries (--features full, path-remapped)…"
    ( cd rs && cargo build -p capslockx-macos --release --features full \
        --bin capslockx --bin clx-agent --bin clx-prompt --bin clx-media )
fi

# Vision OCR helper (Swift). Rebuild if source is newer.
OCR_SRC="$ROOT/rs/adapters/macos/src/bin/clx-ocr.swift"
OCR_BIN="$ROOT/clx-ocr"
if [ -f "$OCR_SRC" ] && { [ ! -x "$OCR_BIN" ] || [ "$OCR_SRC" -nt "$OCR_BIN" ]; }; then
    echo "[pkg] compiling clx-ocr…"
    swiftc -O -o "$OCR_BIN" "$OCR_SRC"
fi

# ── 2. Generate CapsLockX.icns from the brand icon ───────────────────────────
echo "[pkg] generating icon…"
ICON_SRC="$ROOT/Data/XIconBlue.png"
ICONSET="$DIST/CapsLockX.iconset"
rm -rf "$ICONSET" && mkdir -p "$ICONSET"
for s in 16 32 64 128 256 512 1024; do
    sips -z "$s" "$s" "$ICON_SRC" --out "$ICONSET/icon_${s}x${s}.png" >/dev/null
done
# Retina (@2x) variants reuse the next size up.
cp "$ICONSET/icon_32x32.png"     "$ICONSET/icon_16x16@2x.png"
cp "$ICONSET/icon_64x64.png"     "$ICONSET/icon_32x32@2x.png"
cp "$ICONSET/icon_256x256.png"   "$ICONSET/icon_128x128@2x.png"
cp "$ICONSET/icon_512x512.png"   "$ICONSET/icon_256x256@2x.png"
cp "$ICONSET/icon_1024x1024.png" "$ICONSET/icon_512x512@2x.png"
rm -f "$ICONSET/icon_64x64.png" "$ICONSET/icon_1024x1024.png"  # not valid iconset names

# ── 3. Assemble the bundle ───────────────────────────────────────────────────
echo "[pkg] assembling $APP_NAME.app…"
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Frameworks" \
         "$APP/Contents/Resources" "$APP/Contents/skills"

iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/CapsLockX.icns"

# Main binary is named `clx` so the daemon's `pgrep -x clx` dedup keeps working.
cp "$TARGET/capslockx" "$APP/Contents/MacOS/clx"
for b in clx-agent clx-prompt clx-media; do
    [ -x "$TARGET/$b" ] && cp "$TARGET/$b" "$APP/Contents/MacOS/$b"
done
[ -x "$OCR_BIN" ] && cp "$OCR_BIN" "$APP/Contents/MacOS/clx-ocr"

# Dylibs → Frameworks. clx links libsherpa-onnx-c-api + libonnxruntime via @rpath.
cp "$TARGET/libonnxruntime.1.17.1.dylib"   "$APP/Contents/Frameworks/"
cp "$TARGET/libsherpa-onnx-c-api.dylib"    "$APP/Contents/Frameworks/"
[ -f "$TARGET/libsherpa-onnx-cxx-api.dylib" ] && \
    cp "$TARGET/libsherpa-onnx-cxx-api.dylib" "$APP/Contents/Frameworks/" || true
( cd "$APP/Contents/Frameworks" && ln -sf libonnxruntime.1.17.1.dylib libonnxruntime.dylib )

# Intel only: ort (via ten-vad) has no x86_64-macOS prebuilt, so CI links ONNX
# Runtime dynamically through ORT_LIB_LOCATION. Bundle that versioned dylib so
# its @rpath/libonnxruntime.<ver>.dylib reference resolves at runtime. Copy just
# the real file (not the generic symlink) to avoid clashing with sherpa's above.
if [ -n "${ORT_LIB_LOCATION:-}" ]; then
    find "$ORT_LIB_LOCATION" -maxdepth 1 -type f -name 'libonnxruntime.*.dylib' \
        -exec cp {} "$APP/Contents/Frameworks/" \;
fi

# Skills (agent system prompt) — read-only, resolved via ../skills from the exe.
cp -R "$ROOT/skills/." "$APP/Contents/skills/"

# ── 4. Repoint rpath: repo target dir → bundled Frameworks ───────────────────
echo "[pkg] fixing rpath…"
CLX="$APP/Contents/MacOS/clx"
chmod u+w "$CLX"
install_name_tool -delete_rpath "$ROOT/rs/target/release" "$CLX" 2>/dev/null || true
install_name_tool -add_rpath "@executable_path/../Frameworks" "$CLX"

# ── 5. Info.plist ────────────────────────────────────────────────────────────
cat > "$APP/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key><string>$APP_NAME</string>
    <key>CFBundleDisplayName</key><string>$APP_NAME</string>
    <key>CFBundleIdentifier</key><string>$APP_ID</string>
    <key>CFBundleVersion</key><string>$VERSION</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundleExecutable</key><string>clx</string>
    <key>CFBundleIconFile</key><string>CapsLockX.icns</string>
    <key>CFBundlePackageType</key><string>APPL</string>
    <key>LSMinimumSystemVersion</key><string>12.0</string>
    <key>LSUIElement</key><true/>
    <key>NSHighResolutionCapable</key><true/>
    <key>NSMicrophoneUsageDescription</key><string>CapsLockX uses the microphone for push-to-talk voice input.</string>
    <key>NSAppleEventsUsageDescription</key><string>CapsLockX uses Apple Events to read the accessibility tree for its agent.</string>
</dict>
</plist>
PLIST

# ── 6. Codesign inside-out (stable identifier ⇒ permissions persist) ─────────
echo "[pkg] codesigning…"
for f in "$APP/Contents/Frameworks/"*.dylib; do
    [ -L "$f" ] && continue
    codesign -s - --force --timestamp=none "$f"
done
for b in clx-agent clx-prompt clx-media clx-ocr; do
    [ -e "$APP/Contents/MacOS/$b" ] && \
        codesign -s - --force --timestamp=none --identifier "$APP_ID.$b" "$APP/Contents/MacOS/$b"
done
# Main binary keeps the historical identifier so Accessibility/Screen-Recording
# grants survive across versions.
codesign -s - --force --timestamp=none --identifier "$APP_ID" \
    --options runtime "$CLX" 2>/dev/null || \
    codesign -s - --force --identifier "$APP_ID" "$CLX"
codesign -s - --force --identifier "$APP_ID" "$APP"
codesign --verify --deep --strict "$APP" && echo "[pkg] signature OK"

# ── 7. Package: zip + pkg ────────────────────────────────────────────────────
BASE="$APP_NAME-$VERSION-macos-$ARCH"
echo "[pkg] zipping…"
rm -f "$DIST/$BASE.zip"
ditto -c -k --keepParent "$APP" "$DIST/$BASE.zip"

echo "[pkg] building installer .pkg…"
pkgbuild --install-location /Applications \
    --component "$APP" \
    --identifier "$APP_ID" \
    --version "$VERSION" \
    "$DIST/$BASE.pkg" >/dev/null

# Cleanup intermediates.
rm -rf "$ICONSET"

echo
echo "[pkg] done — artifacts in dist/:"
ls -lh "$DIST" | grep -E "$BASE|$APP_NAME.app" | awk '{print "  "$9"  "$5}'
echo
echo "  Upload to GitHub release with:"
echo "    gh release create v$VERSION dist/$BASE.zip dist/$BASE.pkg --title \"$APP_NAME v$VERSION\""
