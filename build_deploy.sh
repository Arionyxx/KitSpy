#!/bin/bash
set -euo pipefail

# Project variables
APP_NAME="rustspy"
BINARY_NAME="rustspy"
VERSION=$(grep '^version =' Cargo.toml | head -n 1 | cut -d '"' -f 2)
APP_DIR="AppDir"
DIST_DIR="dist"
LINUXDEPLOY_URL="https://github.com/linuxdeploy/linuxdeploy/releases/download/continuous/linuxdeploy-x86_64.AppImage"
LINUXDEPLOY_BIN="linuxdeploy-x86_64.AppImage"

# Ensure cargo is available
if ! command -v cargo &> /dev/null; then
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

echo "Building $APP_NAME v$VERSION..."

# Ensure dist directory exists
mkdir -p "$DIST_DIR"

# 1. Build release
echo "Running cargo build --release..."
cargo build --release

# 2. Create AppDir structure
echo "Creating AppDir structure..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/usr/bin"
mkdir -p "$APP_DIR/usr/share/applications"
mkdir -p "$APP_DIR/usr/share/icons/hicolor/256x256/apps"

# 3. Copy binary and resources
echo "Copying binary..."
cp "target/release/$BINARY_NAME" "$APP_DIR/usr/bin/"

# 4. Create .desktop file
echo "Creating .desktop file..."
cat > "$APP_DIR/usr/share/applications/$APP_NAME.desktop" <<EOF
[Desktop Entry]
Type=Application
Name=RustSpy
Exec=$BINARY_NAME
Icon=$APP_NAME
Categories=Utility;Development;
Terminal=false
EOF

# 5. Create placeholder icon
echo "Creating placeholder icon..."
# Create a simple SVG icon
cat > "$APP_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.svg" <<EOF
<svg width="256" height="256" viewBox="0 0 256 256" xmlns="http://www.w3.org/2000/svg">
  <rect width="256" height="256" fill="#e0e0e0"/>
  <circle cx="128" cy="128" r="100" fill="#f74c00"/>
  <text x="50%" y="50%" font-family="Arial" font-size="100" fill="white" text-anchor="middle" dy=".3em">RS</text>
</svg>
EOF

# Also copy icon to root of AppDir for linuxdeploy to find easily
cp "$APP_DIR/usr/share/icons/hicolor/256x256/apps/$APP_NAME.svg" "$APP_DIR/$APP_NAME.svg"

# 6. Download linuxdeploy
if [ ! -f "$LINUXDEPLOY_BIN" ]; then
    echo "Downloading linuxdeploy..."
    wget -O "$LINUXDEPLOY_BIN" "$LINUXDEPLOY_URL"
    chmod +x "$LINUXDEPLOY_BIN"
fi

# 7. Build AppImage
echo "Building AppImage..."
# Use APPIMAGE_EXTRACT_AND_RUN=1 because we might be in a container environment without FUSE
export APPIMAGE_EXTRACT_AND_RUN=1
./"$LINUXDEPLOY_BIN" --appdir "$APP_DIR" --output appimage

# Move AppImage to dist
# Find the generated AppImage (linuxdeploy names it based on desktop file)
GENERATED_APPIMAGE=$(find . -maxdepth 1 -name "RustSpy-*.AppImage" | head -n 1)
if [ -f "$GENERATED_APPIMAGE" ]; then
    mv "$GENERATED_APPIMAGE" "$DIST_DIR/"
    echo "AppImage moved to $DIST_DIR/"
else
    echo "Error: AppImage not found."
    exit 1
fi

echo "AppImage created successfully!"
ls -lh "$DIST_DIR/"*.AppImage
echo "Absolute path: $(readlink -f "$DIST_DIR/"*.AppImage)"
