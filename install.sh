#!/bin/bash
set -e

# Configuration
INSTALL_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
MIME_DIR="$HOME/.local/share/mime"
PROJECT_ROOT=$(pwd)

echo "🚀 INT Installer - Installation Script"
echo "======================================"

# Check requirements
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo is not installed"
    echo "Please install Rust and Cargo: https://rustup.rs/"
    exit 1
fi

# Ensure directories exist
mkdir -p "$INSTALL_DIR"
mkdir -p "$APP_DIR"
mkdir -p "$MIME_DIR/packages"

export PATH="$INSTALL_DIR:$PATH"

export PATH="$INSTALL_DIR:$PATH"

echo "🎨 Building Frontend..."
cd "crates/int-engine/src-ui" || exit 1
# Check if node_modules exists, install if not
if [ ! -d "node_modules" ]; then
    echo "  📦 Installing dependencies..."
    npm install
fi
echo "  🔨 Building Vue app..."
npm run build
cd "$PROJECT_ROOT" || exit 1

echo "🔨 Building binaries (release mode)..."
cargo build --release

echo "📦 Installing to $INSTALL_DIR..."

if [ -f "target/release/int-pack" ]; then
    cp "target/release/int-pack" "$INSTALL_DIR/int-pack"
    echo "  ✓ int-pack installed"
else
    echo "  ❌ int-pack binary not found!"
    exit 1
fi

if [ -f "target/release/int-engine" ]; then
    cp "target/release/int-engine" "$INSTALL_DIR/int-engine"
    echo "  ✓ int-engine installed"
else
    echo "  ❌ int-engine binary not found!"
    exit 1
fi

# Create a convenience script "int-installer"
echo '#!/bin/bash
exec int-engine "$@"
' > "$INSTALL_DIR/int-installer"
chmod +x "$INSTALL_DIR/int-installer"
echo "  ✓ int-installer wrapper created"

echo "🖥️  Registering Desktop Entry & MIME Type..."

# Create MIME type
echo '<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-int-installer">
    <comment>INT Package</comment>
    <glob pattern="*.int"/>
    <icon name="package-x-generic"/>
  </mime-type>
</mime-info>' > "$MIME_DIR/packages/int-installer.xml"

# Update MIME database
if command -v update-mime-database &> /dev/null; then
    update-mime-database "$MIME_DIR"
    echo "  ✓ MIME database updated"
fi

# Create Desktop Entry
echo "[Desktop Entry]
Type=Application
Name=INT Installer
Comment=Installer for INT packages
Exec=$INSTALL_DIR/int-engine --gui %f
Icon=system-software-install
Terminal=false
Categories=Utility;
MimeType=application/x-int-installer;
" > "$APP_DIR/int-installer.desktop"

# Update Desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$APP_DIR"
    echo "  ✓ Desktop database updated"
fi

echo ""
echo "✅ Installation complete!"
echo "You can now double-click .int files to install them!"
echo ""
echo "CLI Usage:"
echo "  int-pack build my-app/"
echo "  int-installer my-app.int"
