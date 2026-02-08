use anyhow::Result;
use int_core::manifest::Manifest;
use std::path::Path;
use tracing::info;

pub struct PackageValidator;

impl PackageValidator {
    pub fn new() -> Self {
        Self
    }

    pub fn validate(&self, manifest_path: &Path) -> Result<()> {
        info!("Validating manifest: {}", manifest_path.display());

        let manifest = Manifest::from_file(manifest_path)
            .map_err(|e| anyhow::anyhow!("Manifest parse error: {}", e))?;

        manifest.validate()
            .map_err(|e| anyhow::anyhow!("Manifest validation error: {}", e))?;

        info!("✓ Manifest validation passed: {} ({})", manifest.name, manifest.package_version);
        Ok(())
    }
}