# 🚀 Quick Start Guide

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
cargo --version
```

## Build & Test

```bash
# Extract archive
tar -xzf int-installer-project.tar.gz
cd int-installer

# Build all tools (requires cargo-tauri if building for production)
cargo build --workspace --release

# Run tests
cargo test --workspace

# Your binaries will be in:
ls -lh target/release/int-*
```

Output:
```
target/release/int-pack     # Package builder
target/release/int-engine   # Installer (GUI & CLI mode)
```

## Usage Examples

### 1. Create a New Package

```bash
# Create package template
./target/release/int-pack init my-app

# This creates:
# my-app/
#   ├── manifest.json
#   ├── payload/
#   ├── scripts/
#   └── services/
```

### 2. Build Package

```bash
# Add your files to my-app/payload/
cp my-binary my-app/payload/bin/

# Build .int package
./target/release/int-pack build my-app --out my-app.int
```

### 3. Install Package

```bash
# Install via GUI (default)
./target/release/int-engine my-app.int

# Install via CLI (if implemented/planned as fallback)
# ./target/release/int-engine --cli my-app.int

# List installed packages
./target/release/int-engine --list

# Uninstall a package
./target/release/int-engine --uninstall my-app
```

## Example: Hello World

```bash
# Build example package
cd examples/hello-world
../../target/release/int-pack build . --out ../../hello-world.int

# Install it
cd ../..
./target/release/int-engine hello-world.int

# Run the installed app
~/.local/bin/hello  # Or defined path in manifest
```

## What's Working Now

✅ **int-pack** - Full CLI implementation
- Create package templates
- Build .int packages
- Validate manifests
- Show package info

✅ **int-engine** - GUI Installer (Tauri)
- Interactive installation wizard
- Package extraction and validation
- desktop entry and service registration
- Progress tracking and error handling

✅ **int-core** - Complete library
- Manifest parsing
- Package extraction
- Installation logic
- systemd integration
- Desktop entries

## What's Not Yet Implemented

🚧 **Advanced Features**
- PolicyKit integration (Native auth dialogs)
- GPG signature verification
- Automatic update mechanism

## Troubleshooting

### Build fails with dependency errors

```bash
# Clean and rebuild
cargo clean
cargo build --workspace --release
```

### Permission denied

```bash
# Make scripts executable
chmod +x install.sh
chmod +x examples/*/scripts/*.sh
```

### "command not found: int-pack"

```bash
# Either use full path:
./target/release/int-pack

# Or add to PATH:
export PATH="$PWD/target/release:$PATH"
int-pack --help
```

## Next Steps

1. ✅ Build project
2. ✅ Create your first package
3. ✅ Test installation
4. 📖 Read [ARCHITECTURE.md](ARCHITECTURE.md) for details
5. 🛡️ Check [RELEASE_NOTES.md](RELEASE_NOTES.md) for the latest updates

## Documentation

- `README.md` - Project overview
- `ARCHITECTURE.md` - System design
- `BUILD_INSTALL.md` - Detailed build guide
- `docs/manifest-spec.md` - Package format
- `RELEASE_NOTES.md` - Version 0.2.0 updates

---

**Pro Tip**: Start with `examples/hello-world` to understand the package format.
