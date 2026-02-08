#!/bin/bash
set -e

# Configuration
VERSION="2.7.1"
URL="https://getcomposer.org/download/$VERSION/composer.phar"
PAYLOAD_BIN_DIR="payload/bin"

echo "Preparing PHP Composer package..."

# Create payload directory
mkdir -p "$PAYLOAD_BIN_DIR"

# Download composer.phar
if [ ! -f "$PAYLOAD_BIN_DIR/composer" ]; then
    echo "Downloading Composer $VERSION..."
    curl -L "$URL" -o "$PAYLOAD_BIN_DIR/composer"
    chmod +x "$PAYLOAD_BIN_DIR/composer"
fi

echo "Composer package prepared in $PAYLOAD_BIN_DIR/composer"
