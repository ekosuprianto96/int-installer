use anyhow::Result;
use serde_json::json;
use std::fs;
use std::path::{PathBuf};
use tracing::info;

pub struct TemplateGenerator;

impl TemplateGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn create_template(&self, name: &str, output: Option<PathBuf>) -> Result<()> {
        let package_dir = output.unwrap_or_else(|| PathBuf::from(name));
        
        info!("Creating template: {}", name);

        fs::create_dir_all(&package_dir)?;

        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
        let default_install_path = format!("{}/.local/share/{}", home, name);

        // Create manifest.json following int-core structure
        let manifest = json!({
            "version": "1.0",
            "name": name,
            "display_name": name,
            "package_version": "0.1.0",
            "description": format!("A simple INT package: {}", name),
            "author": "Your Name",
            "install_scope": "user",
            "install_path": default_install_path,
            "entry": name,
            "service": false,
            "license": "MIT",
            "homepage": "https://example.com",
            "dependencies": [],
            "desktop": {
                "categories": ["Utility"],
                "mime_types": [],
                "show_in_menu": true,
                "keywords": [name]
            }
        });

        let manifest_path = package_dir.join("manifest.json");
        fs::write(manifest_path, serde_json::to_string_pretty(&manifest)?)?;

        // Create payload directory
        let payload_dir = package_dir.join("payload");
        fs::create_dir_all(&payload_dir)?;

        // Create bin directory inside payload
        fs::create_dir_all(payload_dir.join("bin"))?;

        // Create sample executable placeholder
        let bin_content = "#!/bin/bash\n# Simple placeholder for binary\necho \"Hello from {}\"\n";
        let bin_path = payload_dir.join("bin").join(name);
        fs::write(&bin_path, format!("{}", bin_content.replace("{}", name)))?;
        
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&bin_path, fs::Permissions::from_mode(0o755))?;
        }

        // Create data directory inside payload
        fs::create_dir_all(payload_dir.join("data"))?;

        // Create README
        let readme = format!(
            "# {}\n\nThis is a INT package template for {}.\n\n## Building\n\n```bash\nint-pack build .\n```\n",
            name, name
        );
        fs::write(package_dir.join("README.md"), readme)?;

        info!("✓ Template created at: {}", package_dir.display());
        Ok(())
    }
}