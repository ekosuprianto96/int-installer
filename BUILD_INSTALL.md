# Build & Install Guide

## 🔨 Build Instructions

### Option 1: Build Core Library Only (Available Now)

```bash
# Di direktori project root
cd int-installer

# Build core library
cargo build --release -p int-core

# Run tests
cargo test -p int-core
```

Output:
- `target/release/libint_core.rlib` - Core library

### Option 2: Build Entire Workspace (Akan Error - Tools Belum Ada)

```bash
# Ini akan GAGAL karena int-engine dan int-pack belum diimplementasi
cargo build --workspace --release
```

**Error yang akan muncul**:
```
error: no bin target named `int-pack`
error: no bin target named `int-engine`
```

Ini normal karena CLI tools belum dibuat.

## ✅ What You Can Do Now

### 1. Explore & Test Core Library

```bash
# Build core
cargo build -p int-core

# Run tests
cargo test -p int-core

# Check documentation
cargo doc -p int-core --open
```

### 2. Use Core Library in Your Code

Tambahkan ke `Cargo.toml` Anda:

```toml
[dependencies]
int-core = { path = "../path/to/int-installer/crates/int-core" }
```

Contoh usage:

```rust
use int_core::{Installer, InstallConfig, PackageExtractor};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Extract package
    let extractor = PackageExtractor::new();
    let package = extractor.extract("myapp.int")?;
    
    println!("Package: {} v{}", 
        package.manifest.name, 
        package.manifest.package_version
    );
    
    // Install
    let installer = Installer::new();
    let metadata = installer.install(
        "myapp.int", 
        InstallConfig::default()
    )?;
    
    println!("Installed to: {}", metadata.install_path.display());
    Ok(())
}
```

## 🚧 Implement Missing Tools

Untuk membuat project fully functional, Anda perlu implement:

### 1. INT Pack (CLI Builder Tool)

**File yang perlu dibuat**: `crates/int-pack/src/main.rs`

```bash
cd crates/int-pack
mkdir -p src

# Edit Cargo.toml
cat > Cargo.toml <<'EOF'
[package]
name = "int-pack"
version.workspace = true
edition.workspace = true

[[bin]]
name = "int-pack"
path = "src/main.rs"

[dependencies]
int-core = { path = "../int-core" }
clap = { version = "4.4", features = ["derive"] }
anyhow.workspace = true
serde_json.workspace = true
EOF

# Create basic main.rs
cat > src/main.rs <<'EOF'
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "int-pack")]
#[command(about = "INT Package Builder", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new package template
    Init { name: String },
    
    /// Build an INT package
    Build {
        /// Source directory
        source: String,
        
        /// Output file
        #[arg(short, long)]
        out: String,
    },
    
    /// Validate a manifest
    Validate { manifest: String },
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Init { name } => {
            println!("Creating package template: {}", name);
            // TODO: Implement
        }
        Commands::Build { source, out } => {
            println!("Building package from {} to {}", source, out);
            // TODO: Implement using int-core
        }
        Commands::Validate { manifest } => {
            println!("Validating manifest: {}", manifest);
            // TODO: Implement using int-core
        }
    }
}
EOF

# Build it
cargo build --release

# Test it
./target/release/int-pack --help
```

### 2. INT Engine (GUI Installer)

**Ini lebih kompleks, perlu Tauri + React**:

```bash
cd crates/int-engine

# Install Tauri CLI
cargo install tauri-cli

# Create Tauri app
cargo tauri init

# Follow prompts, atau lihat NEXT_STEPS.md untuk detail
```

## 📦 Quick Start for Development

### Step 1: Verify Core Library Works

```bash
cd int-installer
cargo test -p int-core
```

Jika semua tests pass ✅, core library siap digunakan.

### Step 2: Create int-pack CLI

Lihat section "Implement Missing Tools" di atas, atau:

```bash
# Copy template dari examples
cp -r examples/int-pack-template crates/int-pack/
cd crates/int-pack
cargo build --release
```

### Step 3: Test with Example Package

```bash
# Build example package (setelah int-pack dibuat)
./target/release/int-pack build examples/hello-world --out hello.int

# Inspect package
tar -tzf hello.int
```

## 🐛 Common Issues

### Error: "found a virtual manifest"

**Problem**: Mencoba `cargo install` di workspace root

**Solution**: 
```bash
# Jangan gunakan cargo install di workspace
# Gunakan cargo build -p <package-name>

cargo build --release -p int-core  # ✅ Correct
cargo install                       # ❌ Wrong
```

### Error: "no bin target named ..."

**Problem**: Tools belum diimplementasi

**Solution**: Implement int-pack dan int-engine dulu (lihat NEXT_STEPS.md)

### Error: Dependencies tidak ditemukan

**Problem**: Network issue atau crates.io down

**Solution**:
```bash
cargo clean
cargo build -p int-core --offline  # Jika sudah pernah build
# atau
cargo build -p int-core            # Re-download dependencies
```

## 📖 Further Reading

- [NEXT_STEPS.md](NEXT_STEPS.md) - Development roadmap
- [ARCHITECTURE.md](ARCHITECTURE.md) - System design
- [docs/manifest-spec.md](docs/manifest-spec.md) - Package format

## 💡 Tips

1. **Untuk sekarang**: Fokus ke int-core library dulu, pastikan tests pass
2. **Next**: Implement int-pack untuk create packages
3. **Last**: Implement int-engine untuk GUI installer

Prioritas:
```
int-core ✅ (Done) → int-pack 🚧 (Next) → int-engine 📅 (Later)
```

## ✅ Success Criteria

Anda berhasil jika:

- ✅ `cargo test -p int-core` semua pass
- ✅ Bisa create package dengan int-pack
- ✅ Bisa install package dengan int-engine

---

**Need Help?** Check NEXT_STEPS.md atau lihat code di `crates/int-core/src/` sebagai reference.
