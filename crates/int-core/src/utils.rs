/// Utility functions for INT Installer

use crate::error::{IntError, IntResult};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Copy directory recursively
///
/// Copies all files and subdirectories from source to destination.
pub fn copy_dir_recursive(src: &Path, dst: &Path) -> IntResult<()> {
    if !src.exists() {
        return Err(IntError::IoError(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source directory not found: {}", src.display()),
        )));
    }

    if !src.is_dir() {
        return Err(IntError::IoError(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Source is not a directory: {}", src.display()),
        )));
    }

    // Create destination directory
    fs::create_dir_all(dst).map_err(|e| {
        IntError::DirectoryCreationFailed(format!(
            "Failed to create directory {}: {}",
            dst.display(),
            e
        ))
    })?;

    // Walk through source directory
    for entry in WalkDir::new(src).follow_links(false) {
        let entry = entry.map_err(|e| {
            IntError::IoError(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to walk directory: {}", e),
            ))
        })?;

        let entry_path = entry.path();
        let relative_path = entry_path
            .strip_prefix(src)
            .map_err(|e| IntError::Custom(format!("Failed to strip prefix: {}", e)))?;

        let target_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path).map_err(|e| {
                IntError::DirectoryCreationFailed(format!(
                    "Failed to create directory {}: {}",
                    target_path.display(),
                    e
                ))
            })?;
        } else {
            // Ensure parent directory exists
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    IntError::DirectoryCreationFailed(format!(
                        "Failed to create parent directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            // Copy file
            fs::copy(entry_path, &target_path).map_err(|e| {
                IntError::FileCopyFailed {
                    source: entry_path.to_string_lossy().to_string(),
                    dest: target_path.to_string_lossy().to_string(),
                    reason: e.to_string(),
                }
            })?;

            // Preserve permissions on Unix
            #[cfg(unix)]
            {
                let metadata = fs::metadata(entry_path).map_err(IntError::IoError)?;
                fs::set_permissions(&target_path, metadata.permissions())
                    .map_err(IntError::IoError)?;
            }
        }
    }

    Ok(())
}

/// Get available disk space for a path
pub fn get_available_space(path: &Path) -> IntResult<u64> {
    #[cfg(unix)]
    {
        use nix::sys::statvfs::statvfs;

        let path_to_check = if path.exists() {
            path
        } else {
            // Find first existing parent
            let mut current = path;
            while !current.exists() {
                current = current.parent().ok_or_else(|| {
                    IntError::Custom("No existing parent directory found".to_string())
                })?;
            }
            current
        };

        let stat = statvfs(path_to_check).map_err(|e| {
            IntError::Custom(format!("Failed to get filesystem stats: {}", e))
        })?;

        // Available space = block size * available blocks
        Ok(stat.block_size() * stat.blocks_available())
    }

    #[cfg(not(unix))]
    {
        // Fallback: assume enough space
        Ok(u64::MAX)
    }
}

/// Check if path has enough disk space
pub fn check_disk_space(path: &Path, required: u64) -> IntResult<()> {
    let available = get_available_space(path)?;

    if available < required {
        return Err(IntError::DiskSpaceInsufficient {
            required,
            available,
        });
    }

    Ok(())
}

/// Remove directory and all contents
///
/// This is a safe wrapper around fs::remove_dir_all with additional checks.
pub fn remove_dir_safe(path: &Path) -> IntResult<()> {
    use crate::security::SecurityValidator;

    let validator = SecurityValidator::new();

    if !validator.is_safe_to_delete(path) {
        return Err(IntError::Custom(format!(
            "Refusing to delete potentially dangerous path: {}",
            path.display()
        )));
    }

    if !path.exists() {
        return Ok(()); // Already deleted
    }

    fs::remove_dir_all(path).map_err(IntError::IoError)
}

/// Set file permissions (Unix only)
#[cfg(unix)]
pub fn set_permissions(path: &Path, mode: u32) -> IntResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let perms = fs::Permissions::from_mode(mode);
    fs::set_permissions(path, perms).map_err(|e| {
        IntError::PermissionError(format!(
            "Failed to set permissions on {}: {}",
            path.display(),
            e
        ))
    })
}

#[cfg(not(unix))]
pub fn set_permissions(_path: &Path, _mode: u32) -> IntResult<()> {
    Ok(()) // No-op on non-Unix platforms
}

/// Make file executable
#[cfg(unix)]
pub fn make_executable(path: &Path) -> IntResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let metadata = fs::metadata(path).map_err(IntError::IoError)?;
    let mut perms = metadata.permissions();
    let mode = perms.mode();

    // Add execute bit for owner, group, and others if they have read permission
    let new_mode = mode | ((mode & 0o444) >> 2);
    perms.set_mode(new_mode);

    fs::set_permissions(path, perms).map_err(|e| {
        IntError::PermissionError(format!(
            "Failed to make file executable {}: {}",
            path.display(),
            e
        ))
    })
}

#[cfg(not(unix))]
pub fn make_executable(_path: &Path) -> IntResult<()> {
    Ok(()) // No-op on non-Unix platforms
}

/// Format bytes as human-readable string
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= THRESHOLD && unit_index < UNITS.len() - 1 {
        size /= THRESHOLD;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

/// Get current username
pub fn get_current_username() -> Option<String> {
    #[cfg(unix)]
    {
        use nix::unistd::{getuid, User};

        User::from_uid(getuid())
            .ok()
            .flatten()
            .map(|user| user.name)
    }

    #[cfg(not(unix))]
    {
        std::env::var("USER")
            .or_else(|_| std::env::var("USERNAME"))
            .ok()
    }
}

/// Ensure directory exists with proper permissions
pub fn ensure_dir(path: &Path) -> IntResult<()> {
    if path.exists() {
        if !path.is_dir() {
            return Err(IntError::Custom(format!(
                "Path exists but is not a directory: {}",
                path.display()
            )));
        }
        return Ok(());
    }

    fs::create_dir_all(path).map_err(|e| {
        IntError::DirectoryCreationFailed(format!(
            "Failed to create directory {}: {}",
            path.display(),
            e
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_copy_dir_recursive() {
        let temp = TempDir::new().unwrap();
        let src = temp.path().join("src");
        let dst = temp.path().join("dst");

        // Create source structure
        fs::create_dir_all(src.join("subdir")).unwrap();
        File::create(src.join("file1.txt"))
            .unwrap()
            .write_all(b"content1")
            .unwrap();
        File::create(src.join("subdir/file2.txt"))
            .unwrap()
            .write_all(b"content2")
            .unwrap();

        // Copy
        copy_dir_recursive(&src, &dst).unwrap();

        // Verify
        assert!(dst.join("file1.txt").exists());
        assert!(dst.join("subdir/file2.txt").exists());

        let content = fs::read_to_string(dst.join("subdir/file2.txt")).unwrap();
        assert_eq!(content, "content2");
    }

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1_048_576), "1.00 MB");
        assert_eq!(format_bytes(1_073_741_824), "1.00 GB");
    }

    #[test]
    fn test_ensure_dir() {
        let temp = TempDir::new().unwrap();
        let dir = temp.path().join("test/nested/dir");

        ensure_dir(&dir).unwrap();
        assert!(dir.exists());
        assert!(dir.is_dir());

        // Should not fail if already exists
        ensure_dir(&dir).unwrap();
    }

    #[test]
    #[cfg(unix)]
    fn test_make_executable() {
        use std::os::unix::fs::PermissionsExt;

        let temp = TempDir::new().unwrap();
        let file = temp.path().join("script.sh");
        File::create(&file).unwrap();

        make_executable(&file).unwrap();

        let metadata = fs::metadata(&file).unwrap();
        let perms = metadata.permissions();
        assert!(perms.mode() & 0o111 != 0); // At least one execute bit set
    }
}
