/// INT Installer Core Library
///
/// This library provides core functionality for the INT Installer system:
/// - Package format parsing and validation
/// - Secure package extraction
/// - Installation orchestration
/// - System integration (systemd, desktop entries)
/// - Uninstallation and cleanup
///
/// # Architecture
///
/// The library is organized into modules:
///
/// - `manifest`: Package manifest parsing and validation
/// - `extractor`: Secure tar.gz archive extraction
/// - `installer`: Installation orchestration
/// - `service`: systemd service management
/// - `desktop`: Desktop entry creation
/// - `security`: Security validation and sandboxing
/// - `error`: Error types and handling
/// - `utils`: Utility functions
///
/// # Example Usage
///
/// ```no_run
/// use int_core::{Installer, InstallConfig, PackageExtractor};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// // Extract and validate package
/// let extractor = PackageExtractor::new();
/// let package = extractor.extract("myapp.int")?;
///
/// println!("Installing: {} v{}", 
///     package.manifest.name, 
///     package.manifest.package_version
/// );
///
/// // Install with default configuration
/// let installer = Installer::new();
/// let metadata = installer.install("myapp.int", InstallConfig::default())?;
///
/// println!("Installed to: {}", metadata.install_path.display());
/// # Ok(())
/// # }
/// ```

// Public modules
pub mod desktop;
pub mod error;
pub mod extractor;
pub mod installer;
pub mod manifest;
pub mod security;
pub mod service;
pub mod utils;

// Re-export commonly used types
pub use desktop::DesktopIntegration;
pub use error::{IntError, IntResult};
pub use extractor::{ExtractedPackage, PackageExtractor};
pub use installer::{InstallConfig, InstallMetadata, InstallProgress, Installer};
pub use manifest::{Dependency, DesktopEntry, InstallScope, Manifest};
pub use security::SecurityValidator;
pub use service::ServiceManager;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported manifest version
pub const MANIFEST_VERSION: &str = manifest::MANIFEST_VERSION;

/// Uninstaller for removing installed packages
pub struct Uninstaller;

impl Uninstaller {
    /// Create a new uninstaller
    pub fn new() -> Self {
        Self
    }

    /// Uninstall a package
    ///
    /// This removes all installed files, services, and desktop entries.
    pub fn uninstall(&self, package_name: &str, scope: InstallScope) -> IntResult<()> {
        // Load installation metadata
        let metadata = InstallMetadata::load(package_name, scope)?;

        // Stop and remove service if exists
        if let (Some(service_file), Some(service_name)) =
            (&metadata.service_file, &metadata.service_name)
        {
            let service_manager = ServiceManager::new();
            service_manager.unregister(service_file, service_name, scope)?;
        }

        // Remove desktop entry if exists
        if let Some(ref desktop_entry) = metadata.desktop_entry {
            let desktop_integration = DesktopIntegration::new();
            desktop_integration.remove_entry(desktop_entry)?;
        }

        // Execute pre-uninstall script if it was recorded
        // Note: We don't have access to the original package, so we skip this

        // Remove installed files
        for file in &metadata.installed_files {
            if file.exists() {
                std::fs::remove_file(file).map_err(|e| {
                    IntError::Custom(format!("Failed to remove file {}: {}", file.display(), e))
                })?;
            }
        }

        // Remove installation directory
        if metadata.install_path.exists() {
            utils::remove_dir_safe(&metadata.install_path)?;
        }

        // Remove metadata file
        let metadata_path = metadata
            .install_path
            .parent()
            .and_then(|p| Some(p.join(format!("{}.json", package_name))))
            .ok_or_else(|| IntError::Custom("Invalid metadata path".to_string()))?;

        if metadata_path.exists() {
            std::fs::remove_file(&metadata_path).map_err(|e| {
                IntError::Custom(format!(
                    "Failed to remove metadata {}: {}",
                    metadata_path.display(),
                    e
                ))
            })?;
        }

        Ok(())
    }

    /// List all installed packages
    pub fn list_installed(&self, scope: InstallScope) -> IntResult<Vec<InstallMetadata>> {
        let metadata_dir = match scope {
            InstallScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                std::path::PathBuf::from(home).join(".local/share/int-installer/installed")
            }
            InstallScope::System => std::path::PathBuf::from("/var/lib/int-installer/installed"),
        };

        if !metadata_dir.exists() {
            return Ok(vec![]);
        }

        let mut packages = Vec::new();

        for entry in std::fs::read_dir(&metadata_dir).map_err(IntError::IoError)? {
            let entry = entry.map_err(IntError::IoError)?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| IntError::MetadataCorrupted(e.to_string()))?;

                let metadata: InstallMetadata = serde_json::from_str(&content)
                    .map_err(|e| IntError::MetadataCorrupted(e.to_string()))?;

                packages.push(metadata);
            }
        }

        Ok(packages)
    }
}

impl Default for Uninstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
        assert!(!MANIFEST_VERSION.is_empty());
    }
}
