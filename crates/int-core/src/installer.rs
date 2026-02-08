/// Installation orchestration
///
/// This module coordinates the complete installation process:
/// - Extracting package
/// - Copying files
/// - Setting permissions
/// - Executing scripts
/// - System integration
use crate::desktop::DesktopIntegration;
use crate::error::{IntError, IntResult};
use crate::extractor::{ExtractedPackage, PackageExtractor};
use crate::manifest::{InstallScope, Manifest};
use crate::service::ServiceManager;
use crate::utils;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

/// Installation configuration
#[derive(Debug, Clone)]
pub struct InstallConfig {
    /// Target installation path (can override manifest)
    pub install_path: Option<PathBuf>,
    /// Whether to start service after installation
    pub start_service: bool,
    /// Whether to create desktop entry
    pub create_desktop_entry: bool,
    /// Dry run (don't actually install)
    pub dry_run: bool,
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            install_path: None,
            start_service: false,
            create_desktop_entry: true,
            dry_run: false,
        }
    }
}

/// Installation progress state
#[derive(Debug, Clone)]
pub enum InstallProgress {
    Extracting { current: u64, total: u64 },
    CopyingFiles { current: usize, total: usize },
    SettingPermissions,
    ExecutingScript { script: String },
    RegisteringService,
    CreatingDesktopEntry,
    Finalizing,
    Completed,
}

/// Installation metadata
///
/// This is saved to track installed packages for uninstallation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallMetadata {
    /// Unique installation ID
    pub install_id: String,
    /// Package name
    pub package_name: String,
    /// Package version
    pub package_version: String,
    /// Installation timestamp
    pub install_date: String,
    /// Installation path
    pub install_path: PathBuf,
    /// Installation scope
    pub install_scope: InstallScope,
    /// Installed files (for uninstallation)
    pub installed_files: Vec<PathBuf>,
    /// Desktop entry path (if created)
    pub desktop_entry: Option<PathBuf>,
    /// Service file path (if created)
    pub service_file: Option<PathBuf>,
    /// Service name (if service)
    pub service_name: Option<String>,
    /// Binary symlink path (if created)
    pub bin_symlink: Option<PathBuf>,
}

impl InstallMetadata {
    /// Save metadata to disk
    pub fn save(&self, scope: InstallScope) -> IntResult<()> {
        let metadata_dir = match scope {
            InstallScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                PathBuf::from(home).join(".local/share/int-installer/installed")
            }
            InstallScope::System => PathBuf::from("/var/lib/int-installer/installed"),
        };

        utils::ensure_dir(&metadata_dir)?;

        let metadata_file = metadata_dir.join(format!("{}.json", self.package_name));

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| IntError::Custom(format!("Failed to serialize metadata: {}", e)))?;

        fs::write(&metadata_file, json).map_err(|e| {
            IntError::Custom(format!(
                "Failed to write metadata to {}: {}",
                metadata_file.display(),
                e
            ))
        })?;

        Ok(())
    }

    /// Load metadata from disk
    pub fn load(package_name: &str, scope: InstallScope) -> IntResult<Self> {
        let metadata_dir = match scope {
            InstallScope::User => {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
                PathBuf::from(home).join(".local/share/int-installer/installed")
            }
            InstallScope::System => PathBuf::from("/var/lib/int-installer/installed"),
        };

        let metadata_file = metadata_dir.join(format!("{}.json", package_name));

        if !metadata_file.exists() {
            return Err(IntError::PackageNotInstalled(package_name.to_string()));
        }

        let content = fs::read_to_string(&metadata_file)
            .map_err(|e| IntError::MetadataCorrupted(e.to_string()))?;

        serde_json::from_str(&content).map_err(|e| IntError::MetadataCorrupted(e.to_string()))
    }
}

/// Package installer
pub struct Installer {
    /// Progress callback
    progress_callback: Option<Box<dyn Fn(InstallProgress) + Send>>,
}

impl Installer {
    /// Create a new installer
    pub fn new() -> Self {
        Self {
            progress_callback: None,
        }
    }

    /// Set progress callback
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(InstallProgress) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Install a package
    pub fn install<P: AsRef<Path>>(
        &self,
        package_path: P,
        config: InstallConfig,
    ) -> IntResult<InstallMetadata> {
        let package_path = package_path.as_ref();

        // Extract package
        self.report_progress(InstallProgress::Extracting {
            current: 0,
            total: 100,
        });

        let extractor = PackageExtractor::new();
        let extracted = extractor.extract(package_path)?;

        // Determine install path
        let install_path = config
            .install_path
            .unwrap_or_else(|| extracted.manifest.install_path.clone());

        // Check permissions
        self.check_permissions(&extracted.manifest, &install_path)?;

        // Check disk space
        if let Some(required) = extracted.manifest.required_space {
            utils::check_disk_space(&install_path, required)?;
        }

        // Check if already installed - if exists, remove it (overwrite)
        if install_path.exists() && !config.dry_run {
            fs::remove_dir_all(&install_path).map_err(|e| {
                IntError::Custom(format!(
                    "Failed to remove existing installation at {}: {}",
                    install_path.display(),
                    e
                ))
            })?;
        }

        if config.dry_run {
            // Just validate, don't actually install
            return Ok(self.create_metadata(&extracted.manifest, &install_path, vec![]));
        }

        // Copy payload files
        self.report_progress(InstallProgress::CopyingFiles {
            current: 0,
            total: 1,
        });

        utils::ensure_dir(&install_path)?;
        let installed_files = self.copy_payload(&extracted.payload_dir, &install_path)?;

        // Set permissions
        self.report_progress(InstallProgress::SettingPermissions);
        self.set_permissions(&install_path, &extracted.manifest)?;

        // Execute post-install script
        if extracted.has_post_install() {
            if let Some(ref script_path) = extracted.manifest.post_install {
                self.report_progress(InstallProgress::ExecutingScript {
                    script: script_path.display().to_string(),
                });

                let full_script_path = extracted.extract_dir.join(script_path);
                self.execute_script(&full_script_path, &install_path)?;
            }
        }

        // Create desktop entry
        let desktop_entry = if config.create_desktop_entry && extracted.manifest.desktop.is_some() {
            self.report_progress(InstallProgress::CreatingDesktopEntry);
            Some(self.create_desktop_entry(&extracted.manifest, &install_path)?)
        } else {
            None
        };

        // Register service
        let (service_file, service_name) = if extracted.manifest.service {
            self.report_progress(InstallProgress::RegisteringService);
            let (file, name) = self.register_service(&extracted, &install_path)?;

            // Start service if requested
            if config.start_service {
                ServiceManager::new().start(&name, extracted.manifest.install_scope)?;
            }

            (Some(file), Some(name))
        } else {
            (None, None)
        };

        // Create binary symlink if entry is specified
        let bin_symlink = if let Some(ref entry) = extracted.manifest.entry {
            let entry_path = install_path.join("bin").join(entry);
            if entry_path.exists() {
                let bin_dir = extracted.manifest.install_scope.bin_path();
                utils::ensure_dir(&bin_dir)?;
                let symlink_path = bin_dir.join(entry);

                // Create symlink (remove existing if any)
                if symlink_path.exists() {
                    fs::remove_file(&symlink_path).ok();
                }

                #[cfg(unix)]
                {
                    use std::os::unix::fs::symlink;
                    symlink(&entry_path, &symlink_path).map_err(|e| {
                        IntError::Custom(format!("Failed to create symlink: {}", e))
                    })?;
                    Some(symlink_path)
                }
                #[cfg(not(unix))]
                {
                    None // Symlinks not supported/implemented for this platform yet
                }
            } else {
                None
            }
        } else {
            None
        };

        // Create and save metadata
        self.report_progress(InstallProgress::Finalizing);
        let mut metadata =
            self.create_metadata(&extracted.manifest, &install_path, installed_files);
        metadata.desktop_entry = desktop_entry;
        metadata.service_file = service_file;
        metadata.service_name = service_name;
        metadata.bin_symlink = bin_symlink;

        metadata.save(extracted.manifest.install_scope)?;

        self.report_progress(InstallProgress::Completed);

        Ok(metadata)
    }

    /// Check if we have sufficient permissions
    fn check_permissions(&self, manifest: &Manifest, install_path: &Path) -> IntResult<()> {
        use crate::security;

        if manifest.install_scope == InstallScope::System {
            // System install requires root or polkit
            if !security::has_root_privileges() {
                // Check if we can write to system directories
                if !security::can_write_system_dir(install_path) {
                    return Err(IntError::InsufficientPermissions(
                        "System installation requires administrator privileges".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Copy payload to installation directory
    fn copy_payload(&self, payload_dir: &Path, install_path: &Path) -> IntResult<Vec<PathBuf>> {
        use walkdir::WalkDir;

        let mut installed_files = Vec::new();

        for entry in WalkDir::new(payload_dir).follow_links(false) {
            let entry = entry.map_err(|e| {
                IntError::Custom(format!("Failed to walk payload directory: {}", e))
            })?;

            let src_path = entry.path();
            let relative = src_path
                .strip_prefix(payload_dir)
                .map_err(|e| IntError::Custom(format!("Failed to get relative path: {}", e)))?;

            let dst_path = install_path.join(relative);

            if entry.file_type().is_dir() {
                utils::ensure_dir(&dst_path)?;
            } else {
                if let Some(parent) = dst_path.parent() {
                    utils::ensure_dir(parent)?;
                }

                fs::copy(src_path, &dst_path).map_err(|e| IntError::FileCopyFailed {
                    source: src_path.display().to_string(),
                    dest: dst_path.display().to_string(),
                    reason: e.to_string(),
                })?;

                installed_files.push(dst_path);
            }
        }

        Ok(installed_files)
    }

    /// Set permissions on installed files
    fn set_permissions(&self, install_path: &Path, manifest: &Manifest) -> IntResult<()> {
        // Make entry executable if specified
        if let Some(ref entry) = manifest.entry {
            let entry_path = install_path.join("bin").join(entry);
            if entry_path.exists() {
                utils::make_executable(&entry_path)?;
            }
        }

        Ok(())
    }

    /// Execute installation script
    fn execute_script(&self, script_path: &Path, install_path: &Path) -> IntResult<()> {
        // Make script executable
        utils::make_executable(script_path)?;

        // Execute script with install_path as working directory
        let output = Command::new(script_path)
            .current_dir(install_path)
            .env("INSTALL_PATH", install_path)
            .output()
            .map_err(|e| IntError::Custom(format!("Failed to execute script: {}", e)))?;

        if !output.status.success() {
            let exit_code = output.status.code().unwrap_or(-1);
            return Err(IntError::ScriptExecutionFailed {
                script: script_path.display().to_string(),
                exit_code,
            });
        }

        Ok(())
    }

    /// Create desktop entry
    fn create_desktop_entry(&self, manifest: &Manifest, install_path: &Path) -> IntResult<PathBuf> {
        let desktop_integration = DesktopIntegration::new();
        desktop_integration.create_entry(manifest, install_path)
    }

    /// Register systemd service
    fn register_service(
        &self,
        extracted: &ExtractedPackage,
        install_path: &Path,
    ) -> IntResult<(PathBuf, String)> {
        let service_manager = ServiceManager::new();
        service_manager.register(extracted, install_path)
    }

    /// Create installation metadata
    fn create_metadata(
        &self,
        manifest: &Manifest,
        install_path: &Path,
        installed_files: Vec<PathBuf>,
    ) -> InstallMetadata {
        InstallMetadata {
            install_id: Uuid::new_v4().to_string(),
            package_name: manifest.name.clone(),
            package_version: manifest.package_version.clone(),
            install_date: Utc::now().to_rfc3339(),
            install_path: install_path.to_path_buf(),
            install_scope: manifest.install_scope,
            installed_files,
            desktop_entry: None,
            service_file: None,
            service_name: None,
            bin_symlink: None,
        }
    }

    /// Report progress
    fn report_progress(&self, progress: InstallProgress) {
        if let Some(ref callback) = self.progress_callback {
            callback(progress);
        }
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self::new()
    }
}
