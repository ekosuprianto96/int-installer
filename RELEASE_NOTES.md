# Release Notes - v0.2.1

## Overview
This release focuses on improving the user experience for CLI applications by introducing automatic PATH integration and streamlining the installation process.

## Key Changes

### 🚀 Automatic PATH Integration
- **Zero-Config CLI**: Binaries defined as an `entry` in the manifest are now automatically symlinked to the system PATH.
  - User Scope: `~/.local/bin`
  - System Scope: `/usr/local/bin`
- **Smart Cleanup**: Automatically created symlinks are now tracked in metadata and will be cleanly removed during uninstallation.
- **Improved Manifest Spec**: Updated documentation to reflect the new `entry` field behavior.

### 🔧 Installer Improvements
- **Simplified Post-Install**: Removed the need for manual symlink creation in `install.sh` scripts for `composer` and `vscode` examples.
- **Enhanced Metadata**: Added `bin_symlink` tracking to `InstallMetadata` for better package management.

### 🐛 Bug Fixes
- **Installer PATH Bug**: Fixed an issue where installed binaries were not accessible from the command line without manual intervention or PATH modification.

---
*Thank you for using INT Installer! For more information, visit our [documentation](docs/manifest-spec.md).*
