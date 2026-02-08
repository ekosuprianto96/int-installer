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

# Build all tools
cargo build --workspace --release

# Run tests
cargo test --workspace

# Your binaries will be in:
ls -lh target/release/int-*
```

Output:
```
target/release/int-pack     # Package builder
target/release/int-engine   # Installer (CLI mode)
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
# Install (user scope)
./target/release/int-engine my-app.int

# Install (system scope - requires sudo)
sudo ./target/release/int-engine my-app.int --scope system

# List installed
./target/release/int-engine --list

# Uninstall
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
~/.local/share/hello-world/bin/hello
```

## What's Working Now

✅ **int-pack** - Full CLI implementation
- Create package templates
- Build .int packages
- Validate manifests
- Show package info

✅ **int-engine** - CLI installer (no GUI yet)
- Install packages
- Uninstall packages
- List installed
- Progress tracking

✅ **int-core** - Complete library
- Manifest parsing
- Package extraction
- Installation logic
- systemd integration
- Desktop entries

## What's Not Yet Implemented

❌ **GUI Installer** - Tauri interface not yet built
- Currently CLI-only
- To add GUI: See `NEXT_STEPS.md`

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
chmod +x build.sh
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
5. 🔧 Check [NEXT_STEPS.md](NEXT_STEPS.md) for GUI implementation

## Documentation

- `README.md` - Project overview
- `ARCHITECTURE.md` - System design
- `BUILD_INSTALL.md` - Detailed build guide
- `docs/manifest-spec.md` - Package format
- `NEXT_STEPS.md` - Development roadmap

---

**Pro Tip**: Start with `examples/hello-world` untuk memahami format package.
