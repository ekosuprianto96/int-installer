# Build & Install Guide

## 🔨 Build Instructions

### Option 1: Build the Entire Workspace (Recommended)

Now that all components are implemented, you can build the entire project at once:

```bash
# In the project root directory
cd int-installer

# Build all crates (core, pack, engine)
cargo build --release
```

Output:
- `target/release/int-pack` - Documentation & Package builder tool
- `target/release/int-engine` - GUI/CLI Installer engine
- `target/release/libint_core.rlib` - Shared core library

### Option 2: Build Specific Crates

```bash
# Build only the core library
cargo build --release -p int-core

# Build only the packaging tool
cargo build --release -p int-pack

# Build only the installer engine
cargo build --release -p int-engine
```

## ✅ What You Can Do Now

### 1. Run Tests

```bash
# Run all tests in the workspace
cargo test --workspace

# Run tests for a specific crate
cargo test -p int-core
```

### 2. Use the Tools

**Create a package template:**
```bash
./target/release/int-pack init my-app
```

**Build a package:**
```bash
./target/release/int-pack build ./my-app --out my-app.int
```

**Install a package:**
```bash
./target/release/int-engine my-app.int
```

## 🚀 Step-by-Step Development Guide

### 1. Core Library (`int-core`)
The core library handles the logic for manifest parsing, package extraction, and system integration.
- Location: `crates/int-core/`
- All tests should pass: `cargo test -p int-core`

### 2. Packaging Tool (`int-pack`)
The CLI tool used by developers to create `.int` packages.
- Location: `crates/int-pack/`
- Usage: `int-pack --help`

### 3. Installer Engine (`int-engine`)
The main GUI application built with Tauri that users interact with.
- Location: `crates/int-engine/`
- Frontend: `crates/int-engine/src-ui/`
- To run development server: `cd crates/int-engine && cargo tauri dev`

## 📦 Distribution Formats

To create system-specific packages (deb, rpm, AppImage):

```bash
cd crates/int-engine
cargo tauri build
```

The resulting bundles will be located in `target/release/bundle/`.

## 🐛 Common Issues

### Error: "command not found: tauri"

**Solution**: Install the Tauri CLI globally or use `cargo tauri`.
```bash
cargo install tauri-cli
```

### Error: CSS/Frontend build fails

**Solution**: Ensure you have Node.js and npm installed, then:
```bash
cd crates/int-engine/src-ui
npm install
npm run build
```

### Permission denied during installation

**Solution**: Ensure the `install.sh` script and individual scripts in the package are executable.
```bash
chmod +x install.sh
```

## 📖 Further Reading

- [ARCHITECTURE.md](ARCHITECTURE.md) - System design and logic
- [QUICK_START.md](QUICK_START.md) - Fast overview of the system
- [RELEASE_NOTES.md](RELEASE_NOTES.md) - Latest features and fixes in v0.2.0

---

**Happy Building!** Check the [Architecture Guide](ARCHITECTURE.md) for deeper technical insights.
