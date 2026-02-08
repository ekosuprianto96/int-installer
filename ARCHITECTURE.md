# INT Installer - Arsitektur Sistem

## Ringkasan

INT Installer adalah framework instalasi aplikasi untuk Linux yang terinspirasi dari Windows MSI. Sistem ini memungkinkan pengguna untuk menginstal aplikasi dengan double-click pada file `.int`, yang akan membuka GUI wizard installer berbasis Tauri.

## Komponen Utama

### 1. INT Package Format (`.int`)

Package `.int` adalah file tar.gz terkompresi dengan struktur internal yang terdefinisi:

```
package.int (tar.gz)
 ├── manifest.json          # Metadata dan konfigurasi instalasi
 ├── payload/               # File aplikasi yang akan diinstal
 │   ├── bin/
 │   ├── lib/
 │   └── assets/
 ├── scripts/               # Script instalasi
 │   ├── install.sh        # Post-install hook
 │   ├── uninstall.sh      # Pre-uninstall hook
 │   └── validate.sh       # Validasi environment
 └── services/              # Service definitions
     └── myapp.service     # systemd unit file
```

### 2. INT Engine (Tauri + Rust)

Engine adalah aplikasi Tauri yang bertindak sebagai installer. Komponen ini:

- Menerima file `.int` sebagai input
- Memvalidasi package
- Menampilkan wizard GUI
- Mengekstrak dan menginstal payload
- Mengintegrasikan dengan sistem Linux (desktop entry, systemd)

### 3. INT Builder (CLI Tool)

Tool untuk membuat package `.int` dari source directory.

```bash
int-pack build ./myapp --out myapp.int
```

## Arsitektur Software

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

Bertanggung jawab untuk:
- Parsing `manifest.json`
- Validasi struktur dan field wajib
- Menyediakan typed access ke metadata package

**Prinsip:**
- Immutable data structures
- Strong typing dengan serde
- Validation di parse time

#### 2. `extractor` Module

Bertanggung jawab untuk:
- Membuka dan memvalidasi archive `.int`
- Ekstraksi ke temporary directory
- Cleanup otomatis via RAII

**Prinsip:**
- Safe extraction (path traversal protection)
- Resource cleanup dengan Drop trait
- Error handling yang jelas

#### 3. `installer` Module

Bertanggung jawab untuk:
- Orchestration proses instalasi
- Copy files ke target directory
- Set ownership dan permissions
- Execute pre/post install scripts
- Rollback on failure

**Prinsip:**
- Transactional installation
- Atomic operations where possible
- Detailed logging

#### 4. `service` Module

Bertanggung jawab untuk:
- Registrasi systemd service
- Enable/disable service
- Start/stop operations
- Service status checking

**Prinsip:**
- No direct systemctl execution dari code
- Use D-Bus API when possible
- Graceful fallback

#### 5. `desktop` Module

Bertanggung jawab untuk:
- Pembuatan `.desktop` entry
- MIME type registration
- Icon installation
- Application menu integration

**Prinsip:**
- Follow freedesktop.org standards
- User vs system scope handling
- XDG directory compliance

#### 6. `security` Module

Bertanggung jawab untuk:
- Path canonicalization dan validation
- Privilege escalation control
- Script execution sandboxing
- Signature verification (future)

**Prinsip:**
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
1. Malicious `.int` package dengan path traversal
2. Privilege escalation melalui script execution
3. Arbitrary code execution tanpa user consent
4. Data exfiltration via network requests
5. System compromise via malformed manifest

**Mitigations:**

1. **Path Sanitization**
   - Semua path di-canonicalize
   - Reject absolute paths di payload
   - Reject `..` traversal
   - Enforce target directory boundaries

2. **Script Execution Control**
   - Scripts di-execute dengan user privileges
   - No automatic root execution
   - Clear permission prompts
   - Audit logging

3. **Manifest Validation**
   - Schema validation dengan serde
   - Type checking
   - Range validation
   - Reject unknown fields (strict mode)

4. **Signature Verification (Future)**
   - GPG signature support
   - Trusted publisher registry
   - Certificate pinning

5. **Sandboxing (Future)**
   - Run scripts dalam restricted environment
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

- **Transactional installation**: Rollback jika gagal
- **Cleanup garanteed**: Menggunakan RAII dan Drop trait
- **User-friendly messages**: Error teknis di-translate ke bahasa sederhana
- **Detailed logging**: Semua error dicatat untuk debugging

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

Untuk mendukung package managers lain:
- APT integration hook
- Flatpak conversion
- AppImage bundling

## Performance Considerations

1. **Streaming Extraction**: Untuk package besar, ekstrak secara streaming
2. **Parallel File Copy**: Gunakan rayon untuk copy paralel
3. **Progress Reporting**: Real-time progress via channels
4. **Memory Usage**: Hindari load entire archive ke memory

## Testing Strategy

1. **Unit Tests**: Setiap module memiliki comprehensive unit tests
2. **Integration Tests**: Test full installation flow
3. **Security Tests**: Penetration testing untuk path traversal, dll
4. **UI Tests**: Tauri WebDriver tests untuk GUI
5. **Package Tests**: Test dengan berbagai manifest configurations

## Deployment

### Distribution

1. **AppImage**: Self-contained int-engine executable
2. **Deb Package**: For Debian/Ubuntu
3. **RPM Package**: For Fedora/RHEL
4. **AUR Package**: For Arch Linux

### Installation Metadata

Setiap instalasi menyimpan metadata di:
- User install: `~/.local/share/int-installer/installed/`
- System install: `/var/lib/int-installer/installed/`

Format metadata:

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

## Kesimpulan

Arsitektur INT Installer dirancang dengan prinsip:

1. **Modular**: Setiap komponen independen dan testable
2. **Secure**: Defense in depth, fail secure
3. **Extensible**: Plugin system untuk custom behaviors
4. **User-friendly**: GUI yang jelas dan error messages yang helpful
5. **Linux-native**: Mengikuti freedesktop.org standards
6. **Professional**: Production-ready code quality

Sistem ini tidak hanya sebagai proof-of-concept, tetapi sebagai installer framework yang siap digunakan untuk distribusi aplikasi Linux secara profesional.
