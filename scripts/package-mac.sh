#!/bin/bash
# Build a self-contained, signed CapsLockX.app and release artifacts (.pkg + .zip).
#
# The bundle is relocatable: helper binaries and skills travel inside the bundle,
# and the tray icon is compiled into the binary. The result can be dropped in
# /Applications, installed via the .pkg, or unzipped anywhere.
#
# Usage:
#   scripts/package-mac.sh              # full build (with AI) then package
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

# ── 1. Build release binaries ────────────────────────────────────────────────
# Remap absolute build paths so the PUBLIC release binaries don't leak the
# builder's username / home layout. Rust bakes dependency source paths into
# panic metadata (e.g. /Users/<you>/.cargo/registry/...); these rules strip the
# real home and repo prefixes. Changing RUSTFLAGS forces a full recompile.
CARGO_HOME_DIR="${CARGO_HOME:-$HOME/.cargo}"
export RUSTFLAGS="${RUSTFLAGS:-} --remap-path-prefix=$CARGO_HOME_DIR=cargo --remap-path-prefix=$ROOT=. --remap-path-prefix=$HOME=home"
# --remap-path-prefix is a rustc flag; native -sys crates (quickjs, etc.)
# are built by cc-rs and need the clang equivalent or they leak the build path.
PREFIX_MAP="-ffile-prefix-map=$ROOT=. -ffile-prefix-map=$HOME=home"
export CFLAGS="${CFLAGS:-} $PREFIX_MAP"
export CXXFLAGS="${CXXFLAGS:-} $PREFIX_MAP"
if [ "$NO_BUILD" -eq 0 ]; then
    echo "[pkg] building release binaries (--features full, path-remapped)…"
    ( cd rs && cargo build -p capslockx-macos --release --features full \
        --bin capslockx --bin clx-agent --bin clx-prompt )
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
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources" "$APP/Contents/skills"

iconutil -c icns "$ICONSET" -o "$APP/Contents/Resources/CapsLockX.icns"

# Main binary is named `clx` so the daemon's `pgrep -x clx` dedup keeps working.
cp "$TARGET/capslockx" "$APP/Contents/MacOS/clx"
for b in clx-agent clx-prompt; do
    [ -x "$TARGET/$b" ] && cp "$TARGET/$b" "$APP/Contents/MacOS/$b"
done

# Skills (agent system prompt) — read-only, resolved via ../skills from the exe.
cp -R "$ROOT/skills/." "$APP/Contents/skills/"

CLX="$APP/Contents/MacOS/clx"
chmod u+w "$CLX"

# ── 4. Info.plist ────────────────────────────────────────────────────────────
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
    <key>NSAppleEventsUsageDescription</key><string>CapsLockX uses Apple Events to read the accessibility tree for its agent.</string>
</dict>
</plist>
PLIST

# ── 5. Codesign inside-out (stable identifier ⇒ permissions persist) ─────────
echo "[pkg] codesigning…"
for b in clx-agent clx-prompt; do
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

# ── 6. Package: zip + pkg ────────────────────────────────────────────────────
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
