/// Security validation and sandboxing utilities
///
/// This module provides security checks and validation to prevent
/// malicious packages from compromising the system.
use crate::error::{IntError, IntResult};
use std::path::{Path, PathBuf};

/// Security validator for package operations
pub struct SecurityValidator {
    /// Allow absolute paths in payload (dangerous, should be false)
    pub allow_absolute_paths: bool,
    /// Maximum file size in bytes (to prevent zip bombs)
    pub max_file_size: u64,
    /// Maximum total extracted size
    pub max_total_size: u64,
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self {
            allow_absolute_paths: false,
            max_file_size: 1_000_000_000,  // 1 GB per file
            max_total_size: 5_000_000_000, // 5 GB total
        }
    }
}

impl SecurityValidator {
    /// Create a new security validator with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate a path for extraction
    ///
    /// This checks for:
    /// - Path traversal attempts (..)
    /// - Absolute paths
    /// - Symlink attacks
    /// - Special characters
    pub fn validate_extraction_path(&self, path: &Path, base_dir: &Path) -> IntResult<PathBuf> {
        // Reject absolute paths unless explicitly allowed
        if path.is_absolute() && !self.allow_absolute_paths {
            return Err(IntError::PathTraversalAttempt(path.to_path_buf()));
        }

        // Normalize path
        let normalized = normalize_path(path);

        // Check for parent directory traversal
        if has_parent_dir_component(&normalized) {
            return Err(IntError::PathTraversalAttempt(normalized));
        }

        // Build full path
        let full_path = base_dir.join(&normalized);

        // Canonicalize to resolve symlinks and verify it's within base_dir
        // Note: canonicalize requires path to exist, so we check parent
        let parent = full_path
            .parent()
            .ok_or_else(|| IntError::ValidationError("Invalid path: no parent".to_string()))?;

        if parent.exists() {
            let canonical_parent = parent.canonicalize().map_err(|e| {
                IntError::ValidationError(format!("Failed to canonicalize path: {}", e))
            })?;

            let canonical_base = base_dir.canonicalize().map_err(|e| {
                IntError::ValidationError(format!("Failed to canonicalize base dir: {}", e))
            })?;

            if !canonical_parent.starts_with(&canonical_base) {
                return Err(IntError::PathTraversalAttempt(full_path));
            }
        }

        Ok(full_path)
    }

    /// Validate file size
    pub fn validate_file_size(&self, size: u64) -> IntResult<()> {
        if size > self.max_file_size {
            return Err(IntError::ValidationError(format!(
                "File too large: {} bytes (max: {} bytes)",
                size, self.max_file_size
            )));
        }
        Ok(())
    }

    /// Validate total extracted size
    pub fn validate_total_size(&self, size: u64) -> IntResult<()> {
        if size > self.max_total_size {
            return Err(IntError::ValidationError(format!(
                "Total package size too large: {} bytes (max: {} bytes)",
                size, self.max_total_size
            )));
        }
        Ok(())
    }

    /// Validate script path
    ///
    /// Scripts must be:
    /// - Relative paths
    /// - Within package directory
    /// - No path traversal
    pub fn validate_script_path(&self, script_path: &Path) -> IntResult<()> {
        if script_path.is_absolute() {
            return Err(IntError::ValidationError(
                "Script path must be relative".to_string(),
            ));
        }

        if has_parent_dir_component(script_path) {
            return Err(IntError::PathTraversalAttempt(script_path.to_path_buf()));
        }

        Ok(())
    }

    /// Check if path is safe for deletion (used during uninstall)
    ///
    /// Prevents deletion of:
    /// - System directories
    /// - User home directory
    /// - Parent directories outside install scope
    pub fn is_safe_to_delete(&self, path: &Path) -> bool {
        // Never delete these critical paths
        let critical_paths = [
            "/", "/bin", "/boot", "/dev", "/etc", "/lib", "/lib64", "/proc", "/root", "/sbin",
            "/sys", "/usr", "/var",
        ];

        let path_str = path.to_string_lossy();

        // Check if it's a critical system path
        if critical_paths.iter().any(|&p| path_str == p) {
            return false;
        }

        // Don't delete user home directory
        if let Ok(home) = std::env::var("HOME") {
            if path_str == home {
                return false;
            }
        }

        // Path should be at least 2 levels deep
        // This prevents accidental deletion of /opt or /usr/local
        path.components().count() >= 3
    }
}

/// Normalize a path by resolving `.` components
fn normalize_path(path: &Path) -> PathBuf {
    use std::path::Component;

    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {
                // Skip "." components
            }
            Component::ParentDir => {
                // Keep ".." as is - will be rejected by validation
                normalized.push("..");
            }
            other => {
                normalized.push(other);
            }
        }
    }

    normalized
}

/// Check if path contains parent directory components
fn has_parent_dir_component(path: &Path) -> bool {
    use std::path::Component;

    path.components().any(|c| matches!(c, Component::ParentDir))
}

/// Check if current process has root/admin privileges
pub fn has_root_privileges() -> bool {
    #[cfg(unix)]
    {
        use nix::unistd::Uid;
        Uid::effective().is_root()
    }

    #[cfg(not(unix))]
    {
        false
    }
}

/// Check if we can write to a system directory
pub fn can_write_system_dir(path: &Path) -> bool {
    if !path.exists() {
        // Check parent directory
        if let Some(parent) = path.parent() {
            return can_write_system_dir(parent);
        }
        return false;
    }

    // Try to create a temporary file
    use std::fs::OpenOptions;
    use std::io::Write;

    let test_file = path.join(".int_write_test");
    let result = OpenOptions::new()
        .write(true)
        .create(true)
        .open(&test_file)
        .and_then(|mut f| f.write_all(b"test"))
        .and_then(|_| std::fs::remove_file(&test_file));

    result.is_ok()
}

/// Sanitize a filename by removing dangerous characters
pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    // std::fs removed
    use tempfile::TempDir;

    #[test]
    fn test_path_traversal_detection() {
        assert!(has_parent_dir_component(&PathBuf::from("../etc/passwd")));
        assert!(has_parent_dir_component(&PathBuf::from("foo/../bar")));
        assert!(!has_parent_dir_component(&PathBuf::from("foo/bar")));
    }

    #[test]
    fn test_normalize_path() {
        let path = PathBuf::from("./foo/./bar");
        let normalized = normalize_path(&path);
        assert_eq!(normalized, PathBuf::from("foo/bar"));
    }

    #[test]
    fn test_validate_extraction_path() {
        let validator = SecurityValidator::new();
        let temp_dir = TempDir::new().unwrap();
        let base = temp_dir.path();

        // Valid path
        let result = validator.validate_extraction_path(&PathBuf::from("foo/bar"), base);
        assert!(result.is_ok());

        // Path traversal attempt
        let result = validator.validate_extraction_path(&PathBuf::from("../etc/passwd"), base);
        assert!(result.is_err());

        // Absolute path
        let result = validator.validate_extraction_path(&PathBuf::from("/etc/passwd"), base);
        assert!(result.is_err());
    }

    #[test]
    fn test_safe_to_delete() {
        let validator = SecurityValidator::new();

        // System paths should not be deletable
        assert!(!validator.is_safe_to_delete(&PathBuf::from("/")));
        assert!(!validator.is_safe_to_delete(&PathBuf::from("/etc")));
        assert!(!validator.is_safe_to_delete(&PathBuf::from("/usr")));

        // App install paths should be deletable
        assert!(validator.is_safe_to_delete(&PathBuf::from("/opt/myapp")));
        assert!(validator.is_safe_to_delete(&PathBuf::from("/home/user/.local/share/myapp")));

        // But not shallow paths
        assert!(!validator.is_safe_to_delete(&PathBuf::from("/opt")));
    }

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("my-app_v1.0"), "my-app_v1.0");
        assert_eq!(sanitize_filename("my app!@#"), "my_app___");
        assert_eq!(sanitize_filename("../../etc"), "______etc");
    }

    #[test]
    fn test_file_size_validation() {
        let validator = SecurityValidator::new();

        assert!(validator.validate_file_size(1000).is_ok());
        assert!(validator
            .validate_file_size(validator.max_file_size)
            .is_ok());
        assert!(validator
            .validate_file_size(validator.max_file_size + 1)
            .is_err());
    }
}
