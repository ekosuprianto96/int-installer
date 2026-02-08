#!/bin/bash
# Post-installation script for VS Code

echo "Running post-install script for VS Code..."
echo "Installation path: $INSTALL_PATH"

# Ensure the binary is executable
chmod +x "$INSTALL_PATH/bin/code"

echo "Post-install complete! VS Code is ready to use."
