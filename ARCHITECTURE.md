# INT Installer - System Architecture

## Overview

INT Installer is an application installation framework for Linux inspired by Windows MSI. This system allows users to install applications by double-clicking `.int` files, which opens a Tauri-based GUI wizard installer.

## Main Components

### 1. INT Package Format (`.int`)

The `.int` package is a compressed tar.gz file with a defined internal structure:

```
package.int (tar.gz)
 ├── manifest.json          # Metadata and installation configuration
 ├── payload/               # Application files to be installed
 ├── scripts/               # Installation scripts
 └── services/              # Service definitions
```

### 2. INT Engine (Tauri + Rust)

The Engine is a Tauri application that acts as the installer. This component:

- Accepts `.int` files as input
- Validates the package
- Displays the GUI wizard
- Extracts and installs the payload
- Integrates with the Linux system (desktop entry, systemd)

### 3. INT Builder (CLI Tool)

A tool to create `.int` packages from a source directory.

```bash
int-pack build ./myapp --out myapp.int
```

## Software Architecture

### Layer Architecture

```
┌─────────────────────────────────────┐
│     Tauri Frontend (TypeScript)     │
│  - Wizard UI                        │
│  - Progress tracking                │
│  - Error display                    │
└─────────────┬───────────────────────┘
              │ IPC Commands
┌─────────────▼───────────────────────┐
│     Tauri Backend (Rust)            │
│  - Command handlers                 │
│  - State management                 │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│     Core Library (Rust)             │
│                                     │
│  ┌───────────────────────────────┐ │
│  │   manifest.rs                 │ │
│  │   - Parse & validate          │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │   extractor.rs                │ │
│  │   - Extract .int archive      │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │   installer.rs                │ │
│  │   - Copy files                │ │
│  │   - Set permissions           │ │
│  │   - Execute scripts           │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │   service.rs                  │ │
│  │   - systemd integration       │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │   desktop.rs                  │ │
│  │   - .desktop entry creation   │ │
│  └───────────────────────────────┘ │
│                                     │
│  ┌───────────────────────────────┐ │
│  │   security.rs                 │ │
│  │   - Path validation           │ │
│  │   - Permission checks         │ │
│  │   - Signature verification    │ │
│  └───────────────────────────────┘ │
└─────────────────────────────────────┘
```

### Module Design

#### 1. `manifest` Module

Responsible for:
- Parsing `manifest.json`
- Validating the structure and mandatory fields
- Providing typed access to package metadata

**Principles:**
- Immutable data structures
- Strong typing with serde
- Validation at parse time

#### 2. `extractor` Module

Responsible for:
- Opening and validating the `.int` archive
- Extraction to a temporary directory
- Automatic cleanup via RAII

**Principles:**
- Safe extraction (path traversal protection)
- Resource cleanup with Drop trait
- Clear error handling

#### 3. `installer` Module

Responsible for:
- Orchestrating the installation process
- Copying files to the target directory
- Setting ownership and permissions
- Executing pre/post install scripts
- Rollback on failure

**Principles:**
- Transactional installation
- Atomic operations where possible
- Detailed logging

#### 4. `service` Module

Responsible for:
- Registering systemd services
- Enabling/disabling services
- Start/stop operations
- Service status checking

**Principles:**
- No direct systemctl execution from code
- Use D-Bus API when possible
- Graceful fallback

#### 5. `desktop` Module

Responsible for:
- Creating `.desktop` entries
- MIME type registration
- Icon installation
- Application menu integration

**Principles:**
- Follow freedesktop.org standards
- User vs system scope handling
- XDG directory compliance

#### 6. `security` Module

Responsible for:
- Path canonicalization and validation
- Privilege escalation control
- Script execution sandboxing
- Signature verification (future)

**Principles:**
- Defense in depth
- Principle of least privilege
- Fail secure

## Data Flow

### Installation Flow

```
1. User double-clicks .int file
   ↓
2. Desktop environment launches int-engine
   ↓
3. Engine validates file integrity
   ↓
4. Extract to temporary directory
   ↓
5. Parse and validate manifest.json
   ↓
6. Show GUI wizard
   ↓
7. User configures installation
   ↓
8. Check permissions (user vs system install)
   ↓
9. If system install → Request PolicyKit auth
   ↓
10. Create target directory
   ↓
11. Copy payload files
   ↓
12. Set permissions
   ↓
13. Execute install.sh (if exists)
   ↓
14. Register .desktop entry
   ↓
15. Register systemd service (if defined)
   ↓
16. Cleanup temporary files
   ↓
17. Show completion screen
```

### Uninstallation Flow

```
1. User triggers uninstall
   ↓
2. Load installation metadata
   ↓
3. Stop and disable service (if exists)
   ↓
4. Execute uninstall.sh (if exists)
   ↓
5. Remove .desktop entry
   ↓
6. Remove service file
   ↓
7. Remove application files
   ↓
8. Remove installation metadata
   ↓
9. Show completion
```

## Security Model

### Threat Model

**Threats:**
1. Malicious `.int` package with path traversal
2. Privilege escalation via script execution
3. Arbitrary code execution without user consent
4. Data exfiltration via network requests
5. System compromise via malformed manifest

**Mitigations:**

1. **Path Sanitization**
   - All paths are canonicalized
   - Reject absolute paths in payload
   - Reject `..` traversal
   - Enforce target directory boundaries

2. **Script Execution Control**
   - Scripts are executed with user privileges
   - No automatic root execution
   - Clear permission prompts
   - Audit logging

3. **Manifest Validation**
   - Schema validation with serde
   - Type checking
   - Range validation
   - Reject unknown fields (strict mode)

4. **Signature Verification (Future)**
   - GPG signature support
   - Trusted publisher registry
   - Certificate pinning

5. **Sandboxing (Future)**
   - Run scripts in a restricted environment
   - Network isolation option
   - Filesystem access limitation

### Permission Model

**User Install:**
- Target: `~/.local/share/applications/`
- Desktop entry: `~/.local/share/applications/`
- Service: `~/.config/systemd/user/`
- No sudo required

**System Install:**
- Target: `/opt/` or `/usr/local/`
- Desktop entry: `/usr/share/applications/`
- Service: `/etc/systemd/system/`
- Requires PolicyKit authentication

## Linux Integration

### MIME Type Registration

File: `/usr/share/mime/packages/int-installer.xml`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<mime-info xmlns="http://www.freedesktop.org/standards/shared-mime-info">
  <mime-type type="application/x-int-package">
    <comment>INT Installer Package</comment>
    <glob pattern="*.int"/>
    <magic priority="50">
      <match type="string" offset="257" value="ustar"/>
    </magic>
  </mime-type>
</mime-info>
```

### Desktop Handler

File: `/usr/share/applications/int-engine.desktop`

```desktop
[Desktop Entry]
Type=Application
Name=INT Installer
Comment=Install INT packages
Exec=int-engine %f
Icon=int-installer
NoDisplay=true
MimeType=application/x-int-package;
```

### PolicyKit Policy

File: `/usr/share/polkit-1/actions/com.intinstaller.install.policy`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC
 "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <action id="com.intinstaller.install.system">
    <description>Install application system-wide</description>
    <message>Authentication is required to install application system-wide</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin_keep</allow_active>
    </defaults>
  </action>
</policyconfig>
```

## Error Handling Strategy

### Error Types

```rust
pub enum IntError {
    // Package errors
    InvalidPackage(String),
    ManifestParseError(String),
    CorruptedArchive(String),
    
    // Installation errors
    InsufficientPermissions,
    TargetPathExists,
    DiskSpaceInsufficient,
    
    // Integration errors
    ServiceRegistrationFailed(String),
    DesktopEntryFailed(String),
    
    // Security errors
    PathTraversalAttempt,
    InvalidSignature,
    UntrustedPublisher,
    
    // System errors
    IoError(std::io::Error),
    SystemdError(String),
}
```

### Error Recovery

- **Transactional installation**: Rollback on failure
- **Cleanup guaranteed**: Using RAII and Drop trait
- **User-friendly messages**: Technical errors translated to simple language
- **Detailed logging**: All errors recorded for debugging

## Extensibility

### Plugin System (Future)

```rust
pub trait InstallHook {
    fn pre_install(&self, manifest: &Manifest) -> Result<(), IntError>;
    fn post_install(&self, manifest: &Manifest) -> Result<(), IntError>;
    fn pre_uninstall(&self, manifest: &Manifest) -> Result<(), IntError>;
    fn post_uninstall(&self, manifest: &Manifest) -> Result<(), IntError>;
}
```

### Custom Validators

```rust
pub trait ManifestValidator {
    fn validate(&self, manifest: &Manifest) -> Result<(), ValidationError>;
}
```

### Backend Support

To support other package managers:
- APT integration hook
- Flatpak conversion
- AppImage bundling

## Performance Considerations

1. **Streaming Extraction**: For large packages, extract via streaming
2. **Parallel File Copy**: Use rayon for parallel copying
3. **Progress Reporting**: Real-time progress via channels
4. **Memory Usage**: Avoid loading the entire archive into memory

## Testing Strategy

1. **Unit Tests**: Every module has comprehensive unit tests
2. **Integration Tests**: Test full installation flow
3. **Security Tests**: Penetration testing for path traversal, etc.
4. **UI Tests**: Tauri WebDriver tests for GUI
5. **Package Tests**: Test with various manifest configurations

## Deployment

### Distribution

1. **AppImage**: Self-contained int-engine executable
2. **Deb Package**: For Debian/Ubuntu
3. **RPM Package**: For Fedora/RHEL
4. **AUR Package**: For Arch Linux

### Installation Metadata

Each installation stores metadata in:
- User install: `~/.local/share/int-installer/installed/`
- System install: `/var/lib/int-installer/installed/`

Metadata format:

```json
{
  "package_name": "myapp",
  "version": "1.0.0",
  "install_date": "2025-02-07T10:30:00Z",
  "install_path": "/opt/myapp",
  "files": [
    "/opt/myapp/bin/myapp",
    "/opt/myapp/lib/libmyapp.so"
  ],
  "desktop_entry": "/usr/share/applications/myapp.desktop",
  "service": "/etc/systemd/system/myapp.service"
}
```

## Conclusion

The INT Installer architecture is designed with the following principles:

1. **Modular**: Each component is independent and testable
2. **Secure**: Defense in depth, fail secure
3. **Extensible**: Plugin system for custom behaviors
4. **User-friendly**: Clear GUI and helpful error messages
5. **Linux-native**: Follows freedesktop.org standards
6. **Professional**: Production-ready code quality

This system is not just a proof-of-concept, but an installer framework ready to be used for professional Linux application distribution.
