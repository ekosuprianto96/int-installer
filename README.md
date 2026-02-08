# INT Installer

**INT Installer** adalah framework instalasi aplikasi untuk Linux yang terinspirasi dari Windows MSI. Sistem ini memungkinkan pengguna untuk menginstal aplikasi dengan double-click pada file `.int`, yang akan membuka GUI wizard installer berbasis Tauri.

## 🎯 Fitur Utama

- ✅ **Package Format**: Format `.int` berbasis tar.gz dengan manifest JSON
- ✅ **GUI Installer**: Wizard interaktif menggunakan Tauri
- ✅ **System Integration**: Desktop entries, systemd services, MIME types
- ✅ **Security**: Path validation, sandboxing, permission checks
- ✅ **Modular Architecture**: Clean, testable, extensible code
- ✅ **Linux Native**: Mengikuti freedesktop.org standards

## 📦 Komponen

### 1. INT Package Format (`.int`)

Package `.int` adalah archive tar.gz dengan struktur standar:

```
package.int
 ├── manifest.json       # Metadata dan konfigurasi
 ├── payload/           # File aplikasi
 ├── scripts/           # Install/uninstall scripts
 └── services/          # systemd service files
```

### 2. INT Engine (Tauri GUI)

Aplikasi Tauri yang bertindak sebagai installer engine:
- Wizard UI multi-step
- Progress tracking
- Error handling
- System integration

### 3. INT Pack (CLI Builder)

Tool untuk membuat package `.int`:

```bash
int-pack build ./myapp --out myapp.int
```

## 🚀 Quick Start

### Prerequisites

- Rust 1.70+ 
- Node.js 18+ (untuk Tauri frontend)
- Linux (Ubuntu 22.04, Fedora 38, atau lebih baru)

### Build dari Source

```bash
# Clone repository
git clone https://github.com/ekosuprianto96/int-installer
cd int-installer

# Build semua components
cargo build --release

# Build Tauri engine dengan GUI
cd crates/int-engine
cargo tauri build

# Binaries ada di target/release/
```

### Install INT Installer

```bash
# Install ke system (requires sudo)
sudo ./scripts/install.sh

# Atau install ke user directory
./scripts/install.sh --user
```

### Membuat Package

```bash
# Buat struktur project
int-pack init myapp

# Edit manifest.json dan tambahkan files ke payload/

# Build package
int-pack build myapp --out myapp.int
```

### Install Package

```bash
# Double-click myapp.int di file manager
# Atau jalankan dari command line:
int-engine myapp.int
```

## 📖 Dokumentasi

- [Architecture](./ARCHITECTURE.md) - Design dan arsitektur sistem
- [Project Structure](./PROJECT_STRUCTURE.md) - Organisasi code
- [Manifest Specification](./docs/manifest-spec.md) - Format manifest.json
- [Developer Guide](./docs/developer-guide.md) - Panduan untuk developer
- [User Guide](./docs/user-guide.md) - Panduan untuk end user
- [Security](./docs/security.md) - Security model dan best practices

## 🏗️ Architecture

INT Installer menggunakan arsitektur modular:

```
┌─────────────────────────────────────┐
│     Tauri Frontend (TypeScript)     │
│  - Wizard UI, Progress, Errors      │
└─────────────┬───────────────────────┘
              │ IPC Commands
┌─────────────▼───────────────────────┐
│     Tauri Backend (Rust)            │
│  - Command handlers, State          │
└─────────────┬───────────────────────┘
              │
┌─────────────▼───────────────────────┐
│     Core Library (Rust)             │
│  - manifest, extractor, installer   │
│  - service, desktop, security       │
└─────────────────────────────────────┘
```

## 🔒 Security

INT Installer dirancang dengan security sebagai prioritas:

- ✅ **Path Validation**: Mencegah path traversal attacks
- ✅ **Sandboxed Execution**: Scripts dijalankan dengan kontrol ketat
- ✅ **Permission Checks**: User vs system scope validation
- ✅ **Signature Verification**: (Planned) GPG signature support
- ✅ **Safe Uninstall**: Prevents deletion of system directories

## 📝 Contoh Manifest

```json
{
  "version": "1.0",
  "name": "myapp",
  "display_name": "My Application",
  "package_version": "1.0.0",
  "description": "A sample application",
  "install_scope": "user",
  "install_path": "/home/user/.local/share/myapp",
  "entry": "myapp",
  "service": true,
  "desktop": {
    "categories": ["Development"],
    "icon": "myapp",
    "show_in_menu": true
  }
}
```

## 🛠️ Development

### Project Structure

```
int-installer/
├── crates/
│   ├── int-core/      # Core library
│   ├── int-engine/    # Tauri GUI installer
│   └── int-pack/      # CLI builder tool
├── docs/              # Documentation
├── examples/          # Example packages
├── integration/       # Linux integration files
└── tests/             # Tests
```

### Running Tests

```bash
# Unit tests
cargo test --all

# Integration tests
cargo test --test integration_tests

# Specific module tests
cargo test -p int-core --lib manifest
```

### Code Quality

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all -- -D warnings

# Check
cargo check --all
```

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

1. Fork the repository
2. Create feature branch (`git checkout -b feature/amazing-feature`)
3. Commit changes (`git commit -m 'Add amazing feature'`)
4. Push to branch (`git push origin feature/amazing-feature`)
5. Open Pull Request

## 📄 License

This project is dual-licensed under:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License 2.0 ([LICENSE-APACHE](LICENSE-APACHE))

You may choose either license for your use.

## 🙏 Acknowledgments

- Inspired by Windows MSI and Linux package managers
- Built with [Tauri](https://tauri.app/)
- Follows [freedesktop.org](https://www.freedesktop.org/) standards

## 📧 Contact

- Project Link: https://github.com/ekosuprianto96/int-installer
- Issues: https://github.com/ekosuprianto96/int-installer/issues
- Discussions: https://github.com/ekosuprianto96/int-installer/discussions

## 🗺️ Roadmap

- [x] Core library implementation
- [x] Basic Tauri GUI
- [x] systemd integration
- [x] Desktop entry support
- [ ] Tauri frontend complete UI
- [ ] PolicyKit integration
- [ ] GPG signature verification
- [ ] Update mechanism
- [ ] Plugin system
- [ ] Flatpak/AppImage conversion
- [ ] Multi-language support

---

**Made with ❤️ for the Linux community**
