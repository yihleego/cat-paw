#!/bin/bash
set -e

# Configuration variables
APP_NAME="CatPaw"
CARGO_BINARY_NAME="catpaw"
BUNDLE_ID="io.leego.catpaw"
ICON_SOURCE="build/macos/icon_1024x1024.png"
PLIST_SOURCE="build/macos/src/CatPaw.app/Contents/Info.plist"

echo "🦀 Compiling Release version..."
cargo build --release

# Prepare directory structure
BUNDLE_DIR="target/release/bundle/${APP_NAME}.app"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

echo "📂 Creating App Bundle structure: ${BUNDLE_DIR}"
rm -rf "$BUNDLE_DIR"
mkdir -p "$MACOS_DIR"
mkdir -p "$RESOURCES_DIR"

# 1. Copy and rename binary file
echo "🚀 Copying executable..."
cp "target/release/${CARGO_BINARY_NAME}" "${MACOS_DIR}/${APP_NAME}"

# 2. Copy Info.plist
echo "📄 Copying metadata Info.plist..."
cp "$PLIST_SOURCE" "${CONTENTS_DIR}/Info.plist"

# 3. Copy Assets directory (Bevy resources)
if [ -d "assets" ]; then
    echo "📦 Copying assets..."
    cp -r "assets" "${RESOURCES_DIR}/"
fi

# 4. Generate icon (.icns)
if [ -f "$ICON_SOURCE" ]; then
    echo "🎨 Generating icon from image..."
    ICONSET_DIR="target/release/bundle/AppIcon.iconset"
    mkdir -p "$ICONSET_DIR"
    
    sips -z 16 16     "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_16x16.png" > /dev/null 2>&1
    sips -z 32 32     "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_16x16@2x.png" > /dev/null 2>&1
    sips -z 32 32     "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_32x32.png" > /dev/null 2>&1
    sips -z 64 64     "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_32x32@2x.png" > /dev/null 2>&1
    sips -z 128 128   "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_128x128.png" > /dev/null 2>&1
    sips -z 256 256   "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_128x128@2x.png" > /dev/null 2>&1
    sips -z 256 256   "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_256x256.png" > /dev/null 2>&1
    sips -z 512 512   "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_256x256@2x.png" > /dev/null 2>&1
    sips -z 512 512   "$ICON_SOURCE" --out "${ICONSET_DIR}/icon_512x512.png" > /dev/null 2>&1
    cp "$ICON_SOURCE" "${ICONSET_DIR}/icon_512x512@2x.png"

    iconutil -c icns "$ICONSET_DIR" -o "${RESOURCES_DIR}/AppIcon.icns"
    rm -rf "$ICONSET_DIR"
else
    echo "⚠️ Icon source file not found: $ICON_SOURCE"
fi

# 5. Code signing (Ad-hoc)
# Note: This is a necessary step for local execution on M1+ chips
echo "🔏 Performing Ad-hoc code signing..."
codesign --force --deep --sign - "${BUNDLE_DIR}"

echo "✅ Build successful!"
echo "📍 Application located at: ${BUNDLE_DIR}"
echo "✨ Use 'open ${BUNDLE_DIR}' command to launch."