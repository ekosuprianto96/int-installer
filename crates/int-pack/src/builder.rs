use anyhow::{anyhow, Result};
use flate2::Compression;
use flate2::write::GzEncoder;
use int_core::manifest::Manifest;
use std::fs::{File};
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
    pub async fn build(&self, output: Option<PathBuf>, _compress: bool) -> Result<PathBuf> {
        // Force compression for .int packages to be compatible with int-core
        let compress = true;
        info!("Starting package build from: {}", self.source_dir.display());

        // Use int-core to parse and validate manifest
        let manifest_path = self.source_dir.join("manifest.json");
        let manifest = Manifest::from_file(&manifest_path)
            .map_err(|e| anyhow!("Failed to read manifest for build: {}", e))?;
        
        manifest.validate()
            .map_err(|e| anyhow!("Manifest validation failed: {}", e))?;

        // Determine output path based on name and version
        let ext = ".int";
        let default_name = format!("{}-{}{}", manifest.name, manifest.package_version, ext);
        let output_path = output.unwrap_or_else(|| PathBuf::from(default_name));

        // Create tar archive
        let tar_file = File::create(&output_path)?;
        if compress {
            let encoder = GzEncoder::new(tar_file, Compression::default());
            let mut tar_builder = Builder::new(encoder);
            self.add_directory_to_tar(&mut tar_builder, &self.source_dir)?;
            tar_builder.finish()?;
        } else {
            let mut tar_builder = Builder::new(tar_file);
            self.add_directory_to_tar(&mut tar_builder, &self.source_dir)?;
            tar_builder.finish()?;
        }

        info!("Package built: {}", output_path.display());
        Ok(output_path)
    }

    /// Add directory contents to tar archive
    fn add_directory_to_tar<W: std::io::Write>(
        &self,
        tar: &mut Builder<W>,
        dir: &Path,
    ) -> Result<()> {
        for entry in WalkDir::new(dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path == dir {
                continue;
            }
            
            let relative = path.strip_prefix(dir)?;
            
            // Skip common temporary/vcs files if they accidentally exist
            if relative.starts_with(".git") || relative.starts_with("target") {
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
        println!("Description:  {}", manifest.description.as_deref().unwrap_or("N/A"));
        println!("Author:       {}", manifest.author.as_deref().unwrap_or("unknown"));
        println!("License:      {}", manifest.license.as_deref().unwrap_or("unknown"));
        println!("Install Path: {}", manifest.install_path.display());
        println!("Scope:        {:?}", manifest.install_scope);
        println!("Auto-Launch:  {} (Command: {})", 
            manifest.auto_launch, 
            manifest.launch_command.as_deref().or(manifest.entry.as_deref()).unwrap_or("none")
        );
        
        if let Some(ref desktop) = manifest.desktop {
            println!("UI Categories: {:?}", desktop.categories);
        }

        Ok(())
    }
}