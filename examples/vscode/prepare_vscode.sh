#!/bin/bash
set -e

# Configuration
VERSION="1.85.1"
FILENAME="code-stable-x64-1702460457.tar.gz" # This is a specific version for stability in this demo
URL="https://update.code.visualstudio.com/1.85.1/linux-x64/stable"
PAYLOAD_DIR="payload"

echo "Preparing VS Code package..."

# Create payload directory
mkdir -p "$PAYLOAD_DIR"

# Download VS Code if not already present
if [ ! -f "$FILENAME" ]; then
    echo "Downloading VS Code $VERSION..."
    curl -L "$URL" -o "$FILENAME"
fi

# Extract to payload
echo "Extracting VS Code..."
tar -xzf "$FILENAME" -C "$PAYLOAD_DIR" --strip-components=1

# The binary is usually at payload/bin/code
# We need to make sure the entry in manifest matches the path
# manifest says "entry": "code", so it should be in payload/bin/code
# After extraction, VS Code has bin/code inside the extracted folder
# Since we stripped 1 component, the contents of the archive (VSCode-linux-x64/) are now in payload/

echo "VS Code package prepared in $PAYLOAD_DIR"
