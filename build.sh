#!/bin/bash
set -e

echo "� Building INT Installer Project..."

# Check requirements
if ! command -v cargo &> /dev/null; then
    echo "❌ Error: cargo is not installed"
    exit 1
fi

echo "📦 Building workspace in release mode..."
cargo build --release

echo ""
echo "✅ Build complete!"
echo "Binaries available at:"
echo "  - int-pack:   target/release/int-pack"
echo "  - int-engine: target/release/int-engine"
