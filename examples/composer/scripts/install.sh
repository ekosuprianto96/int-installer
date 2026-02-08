#!/bin/bash
# Post-installation script for Composer

echo "Running post-install script for Composer..."
echo "Installation path: $INSTALL_PATH"

# Ensure the binary is executable (it should already be from payload, but just in case)
chmod +x "$INSTALL_PATH/bin/composer"

echo "Post-install complete! Composer is ready to use."
