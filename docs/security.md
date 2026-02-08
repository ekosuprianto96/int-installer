# Security Model

INT Installer is designed with security as a priority to protect the system from malicious packages.

## Core Security Mechanisms

### 2. Embedded GPG Signature Verification
Each `.int` package is self-signed to ensure its authenticity and integrity.
- **Mechanism**: The `manifest.json` within the package contains an `signature` field (the GPG signature of the manifest itself) and a `file_hashes` field (SHA256 hashes of all other files).
- **Verification Alogrithm**:
    1. Extract `manifest.json`.
    2. Verify the `signature` against the manifest content using GPG.
    3. Calculate SHA256 hashes for all extracted files and compare them with `file_hashes`.
- **Enforcement**: If verification fails or the signature is missing, the installation is blocked.

### 3. Path Validation (Path Traversal Protection)
The installer validates all paths within the package to prevent files from being written outside the intended installation directory.
- **Protection**: Any path containing `..` or leading to an outside directory is rejected.

### 3. Secure Privilege Escalation (PolicyKit)
For system-wide installations (`/opt` or `/usr`), the installer uses **PolicyKit (pkexec)**.
- The GUI runs with regular user privileges.
- Only the core installation process is elevated after successful user authentication.
- Policies are defined in `com.intinstaller.install.policy`.

### 4. Sandboxing & Script Execution
- `post_install` and `pre_uninstall` scripts are executed with strict controls.
- All scripts are validated for existence within the package before execution.

### 5. Safe Uninstall
The uninstallation process is validated to ensure no critical system directories are accidentally deleted.
- The installer tracks all installed files in metadata.
- Security checks ensure only application files are removed.

## Developer Guide: Signing Packages

You can sign your packages automatically using the `int-pack` tool:

```bash
# Build and sign with default GPG key
int-pack build myapp --sign

# Build and sign with a specific key ID
int-pack build myapp --sign --key 0x12345678
```

Alternatively, you can manually sign `.int` packages using GPG:

```bash
gpg --detach-sign --armor myapp-1.0.0.int
```

This will produce a `myapp-1.0.0.int.sig` file. Ensure this file is distributed along with the package.
