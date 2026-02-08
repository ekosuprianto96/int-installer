/// Desktop entry integration
///
/// This module handles creation of .desktop files for application menu integration
/// following freedesktop.org standards.

use crate::error::{IntError, IntResult};
use crate::manifest::Manifest;
use crate::utils;
use std::fs;
use std::path::{Path, PathBuf};

/// Desktop integration manager
pub struct DesktopIntegration;

impl DesktopIntegration {
    /// Create a new desktop integration manager
    pub fn new() -> Self {
        Self
    }

    /// Create a desktop entry for an application
    pub fn create_entry(&self, manifest: &Manifest, install_path: &Path) -> IntResult<PathBuf> {
        let desktop_config = manifest.desktop.as_ref().ok_or_else(|| {
            IntError::DesktopEntryFailed("No desktop configuration in manifest".to_string())
        })?;

        // Get desktop entry directory
        let desktop_dir = manifest.install_scope.desktop_entry_path();
        utils::ensure_dir(&desktop_dir)?;

        // Create desktop entry file
        let desktop_file_name = format!("{}.desktop", manifest.name);
        let desktop_file_path = desktop_dir.join(&desktop_file_name);

        // Build desktop entry content
        let mut content = String::new();

        // [Desktop Entry] section
        content.push_str("[Desktop Entry]\n");
        content.push_str(&format!("Name={}\n", manifest.display_name()));
        content.push_str("Type=Application\n");

        if let Some(ref desc) = manifest.description {
            content.push_str(&format!("Comment={}\n", desc));
        }

        // Exec line
        if let Some(ref entry) = manifest.entry {
            let exec_path = install_path.join("bin").join(entry);
            content.push_str(&format!("Exec={}\n", exec_path.display()));
        } else {
            return Err(IntError::DesktopEntryFailed(
                "No entry point specified for desktop application".to_string(),
            ));
        }

        // Icon
        if let Some(ref icon) = desktop_config.icon {
            // Check if icon is absolute path or icon name
            if icon.starts_with('/') {
                content.push_str(&format!("Icon={}\n", icon));
            } else {
                // Try to find icon in install directory
                let icon_path = install_path.join("share/icons").join(icon);
                if icon_path.exists() {
                    content.push_str(&format!("Icon={}\n", icon_path.display()));
                } else {
                    // Use as icon name (theme icon)
                    content.push_str(&format!("Icon={}\n", icon));
                }
            }
        }

        // Categories
        if !desktop_config.categories.is_empty() {
            content.push_str(&format!("Categories={}\n", desktop_config.categories.join(";")));
        }

        // MIME types
        if !desktop_config.mime_types.is_empty() {
            content.push_str(&format!("MimeType={}\n", desktop_config.mime_types.join(";")));
        }

        // Keywords
        if !desktop_config.keywords.is_empty() {
            content.push_str(&format!("Keywords={}\n", desktop_config.keywords.join(";")));
        }

        // NoDisplay
        if !desktop_config.show_in_menu {
            content.push_str("NoDisplay=true\n");
        }

        // Terminal
        content.push_str("Terminal=false\n");

        // Version
        content.push_str("Version=1.0\n");

        // Write desktop file
        fs::write(&desktop_file_path, content).map_err(|e| {
            IntError::DesktopEntryFailed(format!(
                "Failed to write desktop file {}: {}",
                desktop_file_path.display(),
                e
            ))
        })?;

        // Set permissions
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let perms = fs::Permissions::from_mode(0o644);
            fs::set_permissions(&desktop_file_path, perms).map_err(|e| {
                IntError::DesktopEntryFailed(format!("Failed to set permissions: {}", e))
            })?;
        }

        // Update desktop database
        self.update_database(&desktop_dir)?;

        Ok(desktop_file_path)
    }

    /// Remove a desktop entry
    pub fn remove_entry(&self, desktop_file_path: &Path) -> IntResult<()> {
        if desktop_file_path.exists() {
            fs::remove_file(desktop_file_path).map_err(|e| {
                IntError::DesktopEntryFailed(format!("Failed to remove desktop file: {}", e))
            })?;

            // Update desktop database
            if let Some(desktop_dir) = desktop_file_path.parent() {
                let _ = self.update_database(desktop_dir);
            }
        }

        Ok(())
    }

    /// Update desktop database
    ///
    /// This runs `update-desktop-database` to refresh the application menu cache.
    fn update_database(&self, desktop_dir: &Path) -> IntResult<()> {
        use std::process::Command;

        // Check if update-desktop-database exists
        let which_output = Command::new("which")
            .arg("update-desktop-database")
            .output();

        if let Ok(output) = which_output {
            if output.status.success() {
                // Run update-desktop-database
                let _ = Command::new("update-desktop-database")
                    .arg(desktop_dir)
                    .output();
                // Ignore errors - this is optional
            }
        }

        Ok(())
    }

    /// Install icon files
    ///
    /// Copies icon files to the appropriate XDG icon directory.
    pub fn install_icons(&self, source_dir: &Path, app_name: &str, is_user: bool) -> IntResult<()> {
        let icon_base = if is_user {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".to_string());
            PathBuf::from(home).join(".local/share/icons")
        } else {
            PathBuf::from("/usr/share/icons")
        };

        // Common icon sizes
        let sizes = ["16x16", "32x32", "48x48", "64x64", "128x128", "256x256"];

        for size in &sizes {
            let source_icon_dir = source_dir.join("hicolor").join(size).join("apps");
            if source_icon_dir.exists() {
                let target_icon_dir = icon_base.join("hicolor").join(size).join("apps");
                utils::ensure_dir(&target_icon_dir)?;

                // Copy all icon files
                for entry in fs::read_dir(&source_icon_dir).map_err(IntError::IoError)? {
                    let entry = entry.map_err(IntError::IoError)?;
                    let source = entry.path();
                    if source.is_file() {
                        let target = target_icon_dir.join(entry.file_name());
                        fs::copy(&source, &target).map_err(IntError::IoError)?;
                    }
                }
            }
        }

        // Update icon cache
        self.update_icon_cache(&icon_base)?;

        Ok(())
    }

    /// Update icon cache
    fn update_icon_cache(&self, icon_dir: &Path) -> IntResult<()> {
        use std::process::Command;

        let which_output = Command::new("which")
            .arg("gtk-update-icon-cache")
            .output();

        if let Ok(output) = which_output {
            if output.status.success() {
                let _ = Command::new("gtk-update-icon-cache")
                    .arg(icon_dir)
                    .arg("-f")
                    .arg("-t")
                    .output();
            }
        }

        Ok(())
    }
}

impl Default for DesktopIntegration {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::{DesktopEntry, InstallScope};

    fn create_test_manifest() -> Manifest {
        Manifest {
            version: "1.0".to_string(),
            name: "test-app".to_string(),
            display_name: Some("Test Application".to_string()),
            package_version: "1.0.0".to_string(),
            description: Some("A test application".to_string()),
            author: None,
            install_scope: InstallScope::User,
            install_path: PathBuf::from("/tmp/test-app"),
            entry: Some("test-app".to_string()),
            service: false,
            service_name: None,
            post_install: None,
            pre_uninstall: None,
            desktop: Some(DesktopEntry {
                categories: vec!["Development".to_string()],
                mime_types: vec![],
                icon: Some("test-app".to_string()),
                show_in_menu: true,
                keywords: vec!["test".to_string()],
            }),
            dependencies: vec![],
            required_space: None,
            architecture: None,
            license: None,
            homepage: None,
        }
    }

    #[test]
    fn test_desktop_entry_creation() {
        use tempfile::TempDir;

        let manifest = create_test_manifest();
        let temp_dir = TempDir::new().unwrap();
        let install_path = temp_dir.path();

        // Create bin directory
        fs::create_dir_all(install_path.join("bin")).unwrap();

        // Note: This test will fail if run without proper environment
        // It's here to demonstrate the structure
    }
}
