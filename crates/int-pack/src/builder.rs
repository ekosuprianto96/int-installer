use anyhow::{anyhow, Result};
use flate2::write::GzEncoder;
use flate2::Compression;
use int_core::manifest::Manifest;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use tar::Builder;
use tracing::info;
use walkdir::WalkDir;

pub struct PackageBuilder {
    source_dir: PathBuf,
}

impl PackageBuilder {
    pub fn new(source_dir: PathBuf) -> Self {
        Self { source_dir }
    }

    /// Build a .int package from directory
    pub async fn build(
        &self,
        output: Option<PathBuf>,
        _compress: bool,
        sign: bool,
        key: Option<String>,
    ) -> Result<PathBuf> {
        // Force compression for .int packages to be compatible with int-core
        info!("Starting package build from: {}", self.source_dir.display());

        // Use int-core to parse and validate manifest
        let manifest_path = self.source_dir.join("manifest.json");
        let mut manifest = Manifest::from_file(&manifest_path)
            .map_err(|e| anyhow!("Failed to read manifest for build: {}", e))?;

        // Calculate file hashes for all files that will be included
        info!("Calculating file hashes...");
        let hashes = self.collect_file_hashes(&self.source_dir)?;
        manifest.file_hashes = Some(hashes);

        // Sign manifest if requested
        if sign {
            info!("Signing manifest...");
            let signature = self.sign_manifest(&manifest, key)?;
            manifest.signature = Some(signature);
        }

        manifest
            .validate()
            .map_err(|e| anyhow!("Manifest validation failed: {}", e))?;

        // Determine output path based on name and version
        let ext = ".int";
        let default_name = format!("{}-{}{}", manifest.name, manifest.package_version, ext);
        let output_path = output
            .clone()
            .unwrap_or_else(|| PathBuf::from(default_name));

        // We need to write the UPDATED manifest to a temporary location or
        // handle it specially during tar creation.
        // Let's create a temporary manifest file.
        // IMPORTANT: Use to_canonical_string() to ensure the manifest in the archive
        // matches exactly what was signed (same format used in sign_manifest).
        let temp_manifest_dir = tempfile::tempdir()?;
        let temp_manifest_path = temp_manifest_dir.path().join("manifest.json");
        std::fs::write(&temp_manifest_path, manifest.to_canonical_string()?)?;

        // Create tar archive
        let tar_file = File::create(&output_path)?;
        let encoder = GzEncoder::new(tar_file, Compression::default());
        let mut tar_builder = Builder::new(encoder);

        // Add updated manifest first
        tar_builder.append_path_with_name(&temp_manifest_path, "manifest.json")?;

        // Add rest of the files (skipping original manifest)
        self.add_directory_to_tar(&mut tar_builder, &self.source_dir, true)?;
        tar_builder.finish()?;

        info!("Package built: {}", output_path.display());
        Ok(output_path)
    }

    /// Sign manifest content using GPG
    fn sign_manifest(&self, manifest: &Manifest, key: Option<String>) -> Result<String> {
        // We sign a copy without the signature field (which should be None anyway)
        let mut manifest_to_sign = manifest.clone();
        manifest_to_sign.signature = None;
        let content = manifest_to_sign.to_canonical_string()?;

        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut cmd = Command::new("gpg");
        cmd.arg("--detach-sign")
            .arg("--armor")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(key_id) = key {
            cmd.arg("--local-user").arg(key_id);
        }

        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow!("Failed to execute gpg: {}", e))?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open stdin"))?;
        stdin.write_all(content.as_bytes())?;
        drop(stdin);

        let output = child.wait_with_output()?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("GPG signing failed: {}", err));
        }

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }

    /// Collect SHA256 hashes of all files in a directory
    fn collect_file_hashes(&self, dir: &Path) -> Result<BTreeMap<String, String>> {
        let mut hashes = BTreeMap::new();

        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() || path.file_name().and_then(|s| s.to_str()) == Some("manifest.json") {
                continue;
            }

            let relative = path
                .strip_prefix(dir)?
                .to_str()
                .ok_or_else(|| anyhow!("Invalid path encoding"))?
                .to_string();

            // Skip common temporary/vcs files
            if relative.starts_with(".git") || relative.starts_with("target") {
                continue;
            }

            let hash = self.calculate_sha256(path)?;
            hashes.insert(relative, hash);
        }

        Ok(hashes)
    }

    /// Calculate SHA256 hash of a file
    fn calculate_sha256(&self, path: &Path) -> Result<String> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let count = file.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Add directory contents to tar archive
    fn add_directory_to_tar<W: std::io::Write>(
        &self,
        tar: &mut Builder<W>,
        dir: &Path,
        skip_manifest: bool,
    ) -> Result<()> {
        for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if path == dir {
                continue;
            }

            let relative = path.strip_prefix(dir)?;
            let rel_str = relative.to_str().unwrap_or("");

            // Skip manifest.json if requested (because we already added the updated one)
            if skip_manifest && rel_str == "manifest.json" {
                continue;
            }

            // Skip common temporary/vcs files if they accidentally exist
            if rel_str.starts_with(".git") || rel_str.starts_with("target") {
                continue;
            }

            if path.is_dir() {
                tar.append_dir(relative, path)?;
            } else {
                let mut file = File::open(path)?;
                tar.append_file(relative, &mut file)?;
            }
        }
        Ok(())
    }

    /// Show package information
    pub async fn show_info(&self) -> Result<()> {
        let manifest_path = if self.source_dir.is_file() {
            // If it's a file, it might be a .int package, but for now int-pack info
            // seems designed for source directories.
            // TODO: Support reading from .int archive directly
            return Err(anyhow!("Currently 'info' command only supports package source directories. Reading from .int files coming soon."));
        } else {
            self.source_dir.join("manifest.json")
        };

        let manifest = Manifest::from_file(manifest_path)
            .map_err(|e| anyhow!("Failed to read manifest: {}", e))?;

        println!("\n📦 Package Information:\n");
        println!("Name:         {}", manifest.name);
        println!("Display Name: {}", manifest.display_name());
        println!("Version:      {}", manifest.package_version);
        println!(
            "Description:  {}",
            manifest.description.as_deref().unwrap_or("N/A")
        );
        println!(
            "Author:       {}",
            manifest.author.as_deref().unwrap_or("unknown")
        );
        println!(
            "License:      {}",
            manifest.license.as_deref().unwrap_or("unknown")
        );
        println!("Install Path: {}", manifest.install_path.display());
        println!("Scope:        {:?}", manifest.install_scope);
        println!(
            "Auto-Launch:  {} (Command: {})",
            manifest.auto_launch,
            manifest
                .launch_command
                .as_deref()
                .or(manifest.entry.as_deref())
                .unwrap_or("none")
        );

        if let Some(ref desktop) = manifest.desktop {
            println!("UI Categories: {:?}", desktop.categories);
        }

        Ok(())
    }
}
