/// Manifest parsing and validation
///
/// This module handles the manifest.json file that describes an INT package.
/// It provides type-safe parsing, validation, and access to package metadata.

use crate::error::{IntError, IntResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Current supported manifest version
pub const MANIFEST_VERSION: &str = "1.0";

/// Installation scope
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstallScope {
    /// User-level installation (~/.local)
    User,
    /// System-level installation (/opt, /usr/local)
    System,
}

impl InstallScope {
    /// Get default installation path for this scope
    pub fn default_install_path(&self, app_name: &str) -> PathBuf {
        match self {
            InstallScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join(app_name)
            }
            InstallScope::System => PathBuf::from("/opt").join(app_name),
        }
    }

    /// Get desktop entry path for this scope
    pub fn desktop_entry_path(&self) -> PathBuf {
        match self {
            InstallScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                PathBuf::from(home)
                    .join(".local")
                    .join("share")
                    .join("applications")
            }
            InstallScope::System => PathBuf::from("/usr/share/applications"),
        }
    }

    /// Get systemd service path for this scope
    pub fn systemd_service_path(&self) -> PathBuf {
        match self {
            InstallScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                PathBuf::from(home).join(".config/systemd/user")
            }
            InstallScope::System => PathBuf::from("/etc/systemd/system"),
        }
    }
}

/// Package manifest structure
///
/// This represents the complete metadata for an INT package.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    /// Manifest format version
    #[serde(default = "default_version")]
    pub version: String,

    /// Package name (used as identifier)
    pub name: String,

    /// Package display name (optional)
    #[serde(default)]
    pub display_name: Option<String>,

    /// Package version (semver recommended)
    pub package_version: String,

    /// Package description
    #[serde(default)]
    pub description: Option<String>,

    /// Package author/vendor
    #[serde(default)]
    pub author: Option<String>,

    /// Installation scope
    pub install_scope: InstallScope,

    /// Installation path (can be customized by user)
    pub install_path: PathBuf,

    /// Main executable name (relative to install_path/bin)
    #[serde(default)]
    pub entry: Option<String>,

    /// Whether to install as systemd service
    #[serde(default)]
    pub service: bool,

    /// Service name (defaults to package name)
    #[serde(default)]
    pub service_name: Option<String>,

    /// Post-install script path (relative to package root)
    #[serde(default)]
    pub post_install: Option<PathBuf>,

    /// Pre-uninstall script path (relative to package root)
    #[serde(default)]
    pub pre_uninstall: Option<PathBuf>,

    /// Desktop integration settings
    #[serde(default)]
    pub desktop: Option<DesktopEntry>,

    /// Required dependencies
    #[serde(default)]
    pub dependencies: Vec<Dependency>,

    /// Minimum required disk space (bytes)
    #[serde(default)]
    pub required_space: Option<u64>,

    /// Architecture requirement
    #[serde(default)]
    pub architecture: Option<String>,

    /// License identifier
    #[serde(default)]
    pub license: Option<String>,

    /// Homepage URL
    #[serde(default)]
    pub homepage: Option<String>,

    /// Whether to auto-launch after installation
    #[serde(default)]
    pub auto_launch: bool,

    /// Command to launch the application (optional, defaults to entry)
    #[serde(default)]
    pub launch_command: Option<String>,
}

fn default_version() -> String {
    MANIFEST_VERSION.to_string()
}

/// Desktop entry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesktopEntry {
    /// Categories (e.g., "Development;IDE;")
    #[serde(default)]
    pub categories: Vec<String>,

    /// MIME types this application handles
    #[serde(default)]
    pub mime_types: Vec<String>,

    /// Icon name or path
    #[serde(default)]
    pub icon: Option<String>,

    /// Whether to show in application menu
    #[serde(default = "default_true")]
    pub show_in_menu: bool,

    /// Keywords for search
    #[serde(default)]
    pub keywords: Vec<String>,
}

fn default_true() -> bool {
    true
}

/// Package dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Dependency name
    pub name: String,

    /// Minimum version
    #[serde(default)]
    pub min_version: Option<String>,

    /// Check command (e.g., "which docker")
    #[serde(default)]
    pub check_command: Option<String>,
}

impl Manifest {
    /// Parse manifest from JSON string
    pub fn from_str(json: &str) -> IntResult<Self> {
        serde_json::from_str(json).map_err(|e| IntError::ManifestParseError(e.to_string()))
    }

    /// Parse manifest from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> IntResult<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            IntError::ManifestParseError(format!("Failed to read manifest file: {}", e))
        })?;
        Self::from_str(&content)
    }

    /// Validate manifest
    ///
    /// Performs comprehensive validation to ensure the manifest is valid and safe.
    pub fn validate(&self) -> IntResult<()> {
        // Check version compatibility
        if self.version != MANIFEST_VERSION {
            return Err(IntError::UnsupportedVersion {
                found: self.version.clone(),
                expected: MANIFEST_VERSION.to_string(),
            });
        }

        // Validate package name
        if self.name.is_empty() {
            return Err(IntError::MissingField("name".to_string()));
        }

        if !is_valid_package_name(&self.name) {
            return Err(IntError::ValidationError(format!(
                "Invalid package name: {}. Must contain only alphanumeric characters, hyphens, and underscores",
                self.name
            )));
        }

        // Validate version
        if self.package_version.is_empty() {
            return Err(IntError::MissingField("package_version".to_string()));
        }

        // Validate install path
        if !self.install_path.is_absolute() {
            return Err(IntError::ValidationError(
                "install_path must be absolute".to_string(),
            ));
        }

        // Check for path traversal in install path
        if has_path_traversal(&self.install_path) {
            return Err(IntError::PathTraversalAttempt(self.install_path.clone()));
        }

        // Validate script paths
        if let Some(ref script) = self.post_install {
            if script.is_absolute() {
                return Err(IntError::ValidationError(
                    "post_install script path must be relative".to_string(),
                ));
            }
            if has_path_traversal(script) {
                return Err(IntError::PathTraversalAttempt(script.to_path_buf()));
            }
        }

        if let Some(ref script) = self.pre_uninstall {
            if script.is_absolute() {
                return Err(IntError::ValidationError(
                    "pre_uninstall script path must be relative".to_string(),
                ));
            }
            if has_path_traversal(script) {
                return Err(IntError::PathTraversalAttempt(script.to_path_buf()));
            }
        }

        // Validate auto-launch
        if self.auto_launch && self.launch_command.is_none() && self.entry.is_none() {
            return Err(IntError::ValidationError(
                "auto_launch is true but neither launch_command nor entry is specified"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Get display name or fallback to name
    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }

    /// Get service name or fallback to name
    pub fn service_name(&self) -> &str {
        self.service_name.as_deref().unwrap_or(&self.name)
    }

    /// Check if package requires system-level installation
    pub fn requires_system_install(&self) -> bool {
        self.install_scope == InstallScope::System
    }

    /// Get installation metadata path for this package
    pub fn metadata_path(&self, scope: InstallScope) -> PathBuf {
        match scope {
            InstallScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                PathBuf::from(home)
                    .join(".local/share/int-installer/installed")
                    .join(format!("{}.json", self.name))
            }
            InstallScope::System => {
                PathBuf::from("/var/lib/int-installer/installed")
                    .join(format!("{}.json", self.name))
            }
        }
    }

    /// Serialize to JSON string
    pub fn to_string(&self) -> IntResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| IntError::Custom(format!("Failed to serialize manifest: {}", e)))
    }
}

/// Validate package name format
fn is_valid_package_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
}

/// Check if path contains traversal attempts (..)
fn has_path_traversal(path: &Path) -> bool {
    path.components().any(|c| matches!(c, std::path::Component::ParentDir))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_manifest() -> Manifest {
        Manifest {
            version: MANIFEST_VERSION.to_string(),
            name: "test-app".to_string(),
            display_name: Some("Test Application".to_string()),
            package_version: "1.0.0".to_string(),
            description: Some("A test application".to_string()),
            author: Some("Test Author".to_string()),
            install_scope: InstallScope::User,
            install_path: PathBuf::from("/home/user/.local/share/test-app"),
            entry: Some("test-app".to_string()),
            service: false,
            service_name: None,
            post_install: None,
            pre_uninstall: None,
            desktop: None,
            dependencies: vec![],
            required_space: Some(10_000_000),
            architecture: Some("x86_64".to_string()),
            license: Some("MIT".to_string()),
            homepage: Some("https://example.com".to_string()),
        }
    }

    #[test]
    fn test_manifest_validation() {
        let manifest = create_test_manifest();
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_invalid_version() {
        let mut manifest = create_test_manifest();
        manifest.version = "99.0".to_string();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_path_traversal_detection() {
        assert!(has_path_traversal(&PathBuf::from("../etc/passwd")));
        assert!(has_path_traversal(&PathBuf::from("/etc/../etc/passwd")));
        assert!(!has_path_traversal(&PathBuf::from("bin/myapp")));
    }

    #[test]
    fn test_package_name_validation() {
        assert!(is_valid_package_name("my-app"));
        assert!(is_valid_package_name("my_app_123"));
        assert!(!is_valid_package_name("my app"));
        assert!(!is_valid_package_name("my/app"));
        assert!(!is_valid_package_name(""));
    }

    #[test]
    fn test_serialization() {
        let manifest = create_test_manifest();
        let json = manifest.to_string().unwrap();
        let parsed = Manifest::from_str(&json).unwrap();
        assert_eq!(manifest.name, parsed.name);
        assert_eq!(manifest.package_version, parsed.package_version);
    }

    #[test]
    fn test_install_scope_paths() {
        let user_scope = InstallScope::User;
        let system_scope = InstallScope::System;

        assert!(user_scope.default_install_path("myapp").to_string_lossy().contains(".local"));
        assert_eq!(system_scope.default_install_path("myapp"), PathBuf::from("/opt/myapp"));
    }
}
