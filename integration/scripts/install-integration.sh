#!/bin/bash
# Install INT Installer Linux integration files
#
# This script installs:
# - MIME type definition
# - Desktop file handler
# - PolicyKit policy

set -e

# Determine if user or system install
SYSTEM_INSTALL=true
if [ "$1" = "--user" ]; then
    SYSTEM_INSTALL=false
fi

echo "Installing INT Installer integration..."

if [ "$SYSTEM_INSTALL" = true ]; then
    # System install (requires sudo)
    if [ "$EUID" -ne 0 ]; then
        echo "System install requires root privileges. Please run with sudo."
        exit 1
    fi
    
    # Install MIME type
    echo "Installing MIME type..."
    cp integration/mime/int-installer.xml /usr/share/mime/packages/
    update-mime-database /usr/share/mime
    
    # Install desktop file
    echo "Installing desktop file..."
    cp integration/desktop/int-engine.desktop /usr/share/applications/
    update-desktop-database /usr/share/applications
    
    # Install PolicyKit policy
    echo "Installing PolicyKit policy..."
    cp integration/polkit/com.intinstaller.install.policy /usr/share/polkit-1/actions/
    
    echo "System integration installed successfully!"
else
    # User install
    MIME_DIR="$HOME/.local/share/mime/packages"
    DESKTOP_DIR="$HOME/.local/share/applications"
    
    # Install MIME type
    echo "Installing MIME type..."
    mkdir -p "$MIME_DIR"
    cp integration/mime/int-installer.xml "$MIME_DIR/"
    update-mime-database "$HOME/.local/share/mime" 2>/dev/null || true
    
    # Install desktop file
    echo "Installing desktop file..."
    mkdir -p "$DESKTOP_DIR"
    cp integration/desktop/int-engine.desktop "$DESKTOP_DIR/"
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    
    echo "User integration installed successfully!"
    echo "Note: PolicyKit policies can only be installed system-wide."
fi

echo "Done!"
