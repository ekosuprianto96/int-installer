# Release Notes - v0.2.0

## Overview
This release marks a significant milestone for the INT Installer project, introducing major stability improvements, enhanced installation flexibility, and core framework refinements.

## Key Changes

### 🔧 Installation & Build Improvements
- **Multiple Installation Paths**: Added support for three distinct installation methods:
    1. **Automated Source Build**: Default `./install.sh` for building from scratch.
    2. **GitHub Releases**: Support for installing pre-built binaries via `./install.sh --bin-only`.
    3. **Manual Build**: Documented steps for developers who prefer manual control.
- **Flexible Binary Detection**: The installation script now automatically detects binaries in the project root or the `target/release` directory.
- **System Integration**: Improved automated registration of Desktop Entries and MIME types (`.int` files).

### 🐛 Bug Fixes & Refinements
- **Core Extractor Fix**: Resolved critical compilation errors in `int-core` related to `tempfile` persistence and type mismatches.
- **API Modernization**: Replaced deprecated `into_path()` calls with modern `keep()` API in the extraction logic.
- **Warning Cleanup**: Removed various compiler warnings regarding unused variables and code paths.

### 📁 Project Structure
- **Root Binaries Support**: Facilitated easier access by allowing binaries to reside in the project root while maintaining a clean environment via updated `.gitignore`.
- **Workspace Versioning**: Updated the entire workspace to version `0.2.0`.

## Installation
To install the latest version:
```bash
git clone https://github.com/ekosuprianto96/int-installer
cd int-installer
sudo ./install.sh
```

For pre-built binaries:
```bash
sudo ./install.sh --bin-only
```

---
*For full technical details, refer to the project documentation in the root directory.*
