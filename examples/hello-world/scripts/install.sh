#!/bin/bash
# Post-installation script for hello-world

echo "Running post-install script..."
echo "Installation path: $INSTALL_PATH"

# Create config directory
mkdir -p "$HOME/.config/hello-world"

# Create default config
cat > "$HOME/.config/hello-world/config.ini" <<EOF
[general]
message=Hello from INT Installer!
version=1.0.0
EOF

echo "Post-install complete!"
