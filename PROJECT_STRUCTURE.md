# INT Installer - Project Structure

## Directory Layout

```
int-installer/
├── Cargo.toml                      # Workspace root
├── ARCHITECTURE.md                 # System architecture
├── README.md                       # Project overview
├── LICENSE                         # MIT/Apache-2.0
│
├── crates/                         # Rust crates
│   ├── int-core/                   # Core library
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs             # Library root
│   │   │   ├── manifest.rs        # Manifest parsing & validation
│   │   │   ├── extractor.rs       # Archive extraction
│   │   │   ├── installer.rs       # Installation orchestration
│   │   │   ├── service.rs         # systemd integration
│   │   │   ├── desktop.rs         # Desktop entry creation
│   │   │   ├── security.rs        # Security & validation
│   │   │   ├── error.rs           # Error types & handling
│   │   │   └── utils.rs           # Utility functions
│   │   └── tests/
│   │       ├── manifest_tests.rs
│   │       ├── extractor_tests.rs
│   │       └── integration_tests.rs
│   │
│   ├── int-engine/                 # Tauri GUI installer
│   │   ├── Cargo.toml
│   │   ├── tauri.conf.json        # Tauri configuration
│   │   ├── build.rs               # Build script
│   │   ├── src/
│   │   │   ├── main.rs            # Entry point
│   │   │   ├── commands.rs        # Tauri commands
│   │   │   ├── state.rs           # Application state
│   │   │   └── events.rs          # Event handling
│   │   ├── src-ui/                # Frontend (TypeScript + Vue/React)
│   │   │   ├── package.json
│   │   │   ├── tsconfig.json
│   │   │   ├── vite.config.ts
│   │   │   ├── index.html
│   │   │   └── src/
│   │   │       ├── main.ts
│   │   │       ├── App.vue
│   │   │       ├── components/
│   │   │       │   ├── WizardStep.vue
│   │   │       │   ├── ProgressBar.vue
│   │   │       │   └── InstallOptions.vue
│   │   │       ├── hooks/
│   │   │       │   └── useInstaller.ts
│   │   │       └── styles/
│   │   │           └── main.css
│   │   └── icons/
│   │       └── icon.png
│   │
│   └── int-pack/                   # CLI builder tool
│       ├── Cargo.toml
│       ├── src/
│       │   ├── main.rs            # CLI entry point
│       │   ├── builder.rs         # Package builder
│       │   ├── validator.rs       # Pre-build validation
│       │   └── template.rs        # Project template generator
│       └── tests/
│           └── builder_tests.rs
│
├── integration/                    # Linux integration files
│   ├── mime/
│   │   └── int-installer.xml      # MIME type definition
│   ├── desktop/
│   │   └── int-engine.desktop     # Desktop file handler
│   ├── polkit/
│   │   └── com.intinstaller.install.policy
│   ├── systemd/
│   │   └── int-installer.service  # Optional: auto-update service
│   └── scripts/
│       ├── install-integration.sh # Setup system integration
│       └── uninstall-integration.sh
│
├── examples/                       # Example INT packages
│   ├── hello-world/
│   │   ├── manifest.json
│   │   ├── payload/
│   │   │   └── bin/
│   │   │       └── hello
│   │   └── scripts/
│   │       └── install.sh
│   │
│   ├── web-service/
│   │   ├── manifest.json
│   │   ├── payload/
│   │   ├── scripts/
│   │   └── services/
│   │       └── webapp.service
│   │
│   └── desktop-app/
│       ├── manifest.json
│       ├── payload/
│       └── icons/
│
├── docs/                           # Documentation
│   ├── user-guide.md              # User documentation
│   ├── developer-guide.md         # Developer documentation
│   ├── manifest-spec.md           # Manifest specification
│   └── security.md                # Security guidelines
│
├── tests/                          # End-to-end tests
│   ├── fixtures/                  # Test packages
│   └── e2e_tests.rs
│
└── scripts/                        # Development scripts
    ├── build-release.sh           # Build production binaries
    ├── run-tests.sh               # Run all tests
    └── create-example.sh          # Generate example package
```

## Dependency Graph

```
int-pack  ──────┐
                ├──► int-core
int-engine ─────┘
```

## Build Outputs

```
target/
├── debug/
│   ├── int-pack               # Builder CLI (debug)
│   └── int-engine             # Installer GUI (debug)
└── release/
    ├── int-pack               # Builder CLI (release)
    ├── int-engine             # Installer GUI (release)
    └── bundle/
        ├── int-engine.AppImage
        ├── deb/
        │   └── int-installer_1.0.0_amd64.deb
        └── rpm/
            └── int-installer-1.0.0.x86_64.rpm
```

## Distributable Package Structure

When int-engine is bundled for distribution:

```
int-installer-1.0.0/
├── bin/
│   ├── int-engine             # Tauri GUI
│   └── int-pack               # Builder CLI
├── share/
│   ├── applications/
│   │   └── int-engine.desktop
│   ├── mime/
│   │   └── packages/
│   │       └── int-installer.xml
│   ├── icons/
│   │   └── hicolor/
│   │       └── 256x256/
│   │           └── apps/
│   │               └── int-installer.png
│   └── polkit-1/
│       └── actions/
│           └── com.intinstaller.install.policy
└── lib/
    └── int-installer/
        └── (bundled dependencies if needed)
```

## Installation Locations

### User Install (--user flag)

```
~/.local/
├── bin/
│   ├── int-engine
│   └── int-pack
└── share/
    ├── applications/
    │   └── int-engine.desktop
    └── int-installer/
        └── installed/           # Installation metadata
```

### System Install (default, requires sudo)

```
/usr/local/
├── bin/
│   ├── int-engine
│   └── int-pack
└── share/
    └── applications/
        └── int-engine.desktop

/var/lib/
└── int-installer/
    └── installed/               # Installation metadata

/usr/share/
├── mime/
│   └── packages/
│       └── int-installer.xml
└── polkit-1/
    └── actions/
        └── com.intinstaller.install.policy
```

## Development Workflow

### Setup Development Environment

```bash
# Clone repository
git clone <repo-url>
cd int-installer

# Build Rust dependencies
cargo build

# Setup frontend (int-engine)
cd crates/int-engine/src-ui
npm install
cd ../../..

# Run tests
cargo test --all
```

### Build Package

```bash
# Build debug
cargo build

# Build release
cargo build --release

# Build Tauri bundles (deb, rpm, AppImage)
cd crates/int-engine
cargo tauri build
```

### Create Example Package

```bash
# Build int-pack tool
cargo build --release -p int-pack

# Create package
./target/release/int-pack build examples/hello-world --out hello-world.int

# Test installation
./target/release/int-engine hello-world.int
```

## Configuration Files

### Workspace Cargo.toml

Defines workspace and shared dependencies.

### Tauri Configuration

`crates/int-engine/tauri.conf.json` - Window, permissions, bundling configuration.

### Frontend Configuration

- `package.json` - npm dependencies
- `tsconfig.json` - TypeScript configuration
- `vite.config.ts` - Build tool configuration

## Module Responsibilities

| Module | Responsibility | External Dependencies |
|--------|---------------|----------------------|
| int-core | Core logic, no GUI | serde, tar, flate2, toml |
| int-engine | GUI installer | tauri, int-core |
| int-pack | CLI builder | clap, int-core |

## Notes

1. **Separation of Concerns**: Core logic is separate from the UI, allowing a CLI-only installer if needed.
2. **Testability**: Each crate can be tested independently.
3. **Reusability**: int-core can be used by other tools.
4. **Distribution**: Multiple distribution formats (AppImage, deb, rpm).
5. **Standards Compliance**: Follows Linux/freedesktop.org standards.
