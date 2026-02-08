# Release Notes - v0.3.0

## Overview
This release brings major security improvements with **embedded GPG signatures**, enhanced installation progress tracking, and improved UI/UX for the installer wizard.

## Key Changes

### 🔐 Embedded GPG Signature System
- **Self-Contained Packages**: Signatures are now embedded directly in the manifest instead of requiring separate `.sig` files.
- **File Hash Verification**: SHA256 hashes of all package files are stored in `manifest.file_hashes` for integrity verification.
- **Deterministic Serialization**: Changed from `HashMap` to `BTreeMap` for consistent JSON key ordering, ensuring reliable signature verification across all platforms.
- **Canonical JSON**: Introduced `to_canonical_string()` for consistent manifest serialization during signing and verification.

### 📊 Improved Progress Tracking
- **Step-Based Progress Bar**: Progress bar now reflects the entire installation process with phase-based percentages:
  - Extracting files: 0-30%
  - Copying files: 30-60%
  - Setting permissions: 65%
  - Running post-install script: 75%
  - Registering system service: 85%
  - Creating desktop entry: 92%
  - Complete: 100%
- **Synchronized UI**: Progress bar now moves smoothly in sync with installation logs.

### 🎨 UI Improvements
- **Dark Mode Log Viewer**: Installation logs now displayed with a dark terminal-style theme (#1e1e1e background).
- **Enhanced Progress Bar**: New gradient styling for better visibility.
- **Improved Readability**: Monospace font (Consolas/Monaco) for log output with zebra striping.

### 🛡️ Security & Elevation
- **PolicyKit Integration**: System-wide installations support privilege escalation using `pkexec`.
- **GPG Verification by Default**: Embedded signature verification is enabled by default during installation.

### 🔧 Technical Improvements
- Connected extractor progress callback to installer for accurate extraction progress reporting.
- Cleaned up unused variables and compiler warnings.
- Improved error messages for signature verification failures.

---
*Thank you for using INT Installer! For more information, visit our [documentation](README.md).*
