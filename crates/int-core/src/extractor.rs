/// Package extraction utilities
///
/// This module handles the extraction of .int packages (tar.gz archives)
/// with security validation and progress tracking.
use crate::error::{IntError, IntResult};
use crate::manifest::Manifest;
use crate::security::SecurityValidator;
use flate2::read::GzDecoder;
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use tar::Archive;

/// Extracted package structure
///
/// This represents an extracted .int package with parsed manifest
/// and paths to all components.
pub struct ExtractedPackage {
    /// Path to temporary extraction directory
    pub extract_dir: PathBuf,
    /// Parsed manifest
    pub manifest: Manifest,
    /// Path to payload directory
    pub payload_dir: PathBuf,
    /// Path to scripts directory (if exists)
    pub scripts_dir: Option<PathBuf>,
    /// Path to services directory (if exists)
    pub services_dir: Option<PathBuf>,
}

impl ExtractedPackage {
    /// Get path to a script file
    pub fn script_path(&self, script_name: &str) -> Option<PathBuf> {
        self.scripts_dir.as_ref().map(|dir| dir.join(script_name))
    }

    /// Get path to a service file
    pub fn service_path(&self, service_name: &str) -> Option<PathBuf> {
        self.services_dir.as_ref().map(|dir| dir.join(service_name))
    }

    /// Check if post-install script exists
    pub fn has_post_install(&self) -> bool {
        if let Some(ref script_path) = self.manifest.post_install {
            let full_path = self.extract_dir.join(script_path);
            full_path.exists()
        } else {
            false
        }
    }

    /// Check if pre-uninstall script exists
    pub fn has_pre_uninstall(&self) -> bool {
        if let Some(ref script_path) = self.manifest.pre_uninstall {
            let full_path = self.extract_dir.join(script_path);
            full_path.exists()
        } else {
            false
        }
    }
}

impl Drop for ExtractedPackage {
    /// Cleanup temporary extraction directory when dropped
    fn drop(&mut self) {
        if self.extract_dir.exists() {
            let _ = fs::remove_dir_all(&self.extract_dir);
        }
    }
}

/// Package extractor
pub struct PackageExtractor {
    /// Security validator
    validator: SecurityValidator,
    /// Progress callback
    progress_callback: Option<Box<dyn Fn(u64, u64) + Send>>,
    /// Log callback
    log_callback: Option<Box<dyn Fn(String) + Send>>,
    /// Whether to verify GPG signature
    pub verify_signature: bool,
}

impl PackageExtractor {
    /// Create a new package extractor
    pub fn new() -> Self {
        Self {
            validator: SecurityValidator::new(),
            progress_callback: None,
            log_callback: None,
            verify_signature: false,
        }
    }

    /// Set progress callback
    ///
    /// The callback receives (current_bytes, total_bytes)
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(u64, u64) + Send + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Set log callback
    pub fn with_log<F>(mut self, callback: F) -> Self
    where
        F: Fn(String) + Send + 'static,
    {
        self.log_callback = Some(Box::new(callback));
        self
    }

    /// Extract a .int package to a temporary directory
    ///
    /// Returns an ExtractedPackage with parsed manifest and component paths.
    pub fn extract<P: AsRef<Path>>(&self, package_path: P) -> IntResult<ExtractedPackage> {
        let package_path = package_path.as_ref();

        // Validate package exists
        if !package_path.exists() {
            return Err(IntError::InvalidPackage(format!(
                "Package file not found: {}",
                package_path.display()
            )));
        }

        // Check file extension
        if package_path.extension().and_then(|s| s.to_str()) != Some("int") {
            return Err(IntError::InvalidPackage(
                "Package must have .int extension".to_string(),
            ));
        }

        // Get package size
        let package_size = fs::metadata(package_path)
            .map_err(|e| IntError::IoError(e))?
            .len();

        self.validator.validate_total_size(package_size)?;

        // Create temporary extraction directory
        let temp_dir = tempfile::tempdir()
            .map_err(|e| IntError::Custom(format!("Failed to create temp dir: {}", e)))?;

        // keep() returns PathBuf on some versions or when certain features are enabled.
        // Based on compiler error, it's returning PathBuf directly here.
        let extract_dir = temp_dir.keep();

        // Extract archive
        self.extract_archive(package_path, &extract_dir, package_size)?;

        // Parse manifest
        let manifest_path = extract_dir.join("manifest.json");
        if !manifest_path.exists() {
            return Err(IntError::InvalidPackage(
                "manifest.json not found in package".to_string(),
            ));
        }

        let manifest = Manifest::from_file(&manifest_path)?;
        manifest.validate()?;

        // Verify GPG signature if requested or embedded
        if manifest.signature.is_some() {
            self.verify_embedded_signature(&manifest)?;
        } else if self.verify_signature {
            self.verify_gpg_signature(package_path)?;
        }

        // Verify file hashes if present
        if let Some(ref hashes) = manifest.file_hashes {
            self.verify_file_hashes(&extract_dir, hashes)?;
        }

        // Locate package components
        let payload_dir = extract_dir.join("payload");
        if !payload_dir.exists() {
            return Err(IntError::InvalidPackage(
                "payload directory not found in package".to_string(),
            ));
        }

        let scripts_dir = extract_dir.join("scripts");
        let scripts_dir = if scripts_dir.exists() {
            Some(scripts_dir)
        } else {
            None
        };

        let services_dir = extract_dir.join("services");
        let services_dir = if services_dir.exists() {
            Some(services_dir)
        } else {
            None
        };

        Ok(ExtractedPackage {
            extract_dir: extract_dir.to_path_buf(),
            manifest,
            payload_dir,
            scripts_dir,
            services_dir,
        })
    }

    /// Extract tar.gz archive
    fn extract_archive(
        &self,
        archive_path: &Path,
        extract_dir: &Path,
        total_size: u64,
    ) -> IntResult<()> {
        let file = File::open(archive_path).map_err(IntError::IoError)?;

        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        let mut extracted_size = 0u64;

        for entry_result in archive.entries().map_err(|e| {
            IntError::CorruptedArchive(format!("Failed to read archive entries: {}", e))
        })? {
            let mut entry = entry_result
                .map_err(|e| IntError::CorruptedArchive(format!("Failed to read entry: {}", e)))?;

            // Get entry path
            let entry_path = entry
                .path()
                .map_err(|e| IntError::CorruptedArchive(format!("Invalid entry path: {}", e)))?;

            // Validate path
            let safe_path = self
                .validator
                .validate_extraction_path(&entry_path, extract_dir)?;

            // Validate file size
            let entry_size = entry.header().size().map_err(|e| {
                IntError::CorruptedArchive(format!("Failed to get entry size: {}", e))
            })?;

            self.validator.validate_file_size(entry_size)?;

            // Track total extracted size
            extracted_size += entry_size;
            self.validator.validate_total_size(extracted_size)?;

            // Report progress
            if let Some(ref callback) = self.progress_callback {
                callback(extracted_size, total_size);
            }

            // Report log
            if let Some(ref callback) = self.log_callback {
                callback(format!("Extracting: {}", entry_path.display()));
            }

            // Create parent directories
            if let Some(parent) = safe_path.parent() {
                fs::create_dir_all(parent).map_err(|e| {
                    IntError::DirectoryCreationFailed(format!(
                        "Failed to create directory {}: {}",
                        parent.display(),
                        e
                    ))
                })?;
            }

            // Extract entry
            if entry.header().entry_type().is_dir() {
                fs::create_dir_all(&safe_path).map_err(|e| {
                    IntError::DirectoryCreationFailed(format!(
                        "Failed to create directory {}: {}",
                        safe_path.display(),
                        e
                    ))
                })?;
            } else {
                let mut output_file = File::create(&safe_path).map_err(|e| {
                    IntError::IoError(io::Error::new(
                        e.kind(),
                        format!("Failed to create file {}: {}", safe_path.display(), e),
                    ))
                })?;

                io::copy(&mut entry, &mut output_file).map_err(|e| {
                    IntError::IoError(io::Error::new(
                        e.kind(),
                        format!("Failed to extract {}: {}", safe_path.display(), e),
                    ))
                })?;
            }

            // Set permissions (Unix only)
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Ok(mode) = entry.header().mode() {
                    let perms = fs::Permissions::from_mode(mode);
                    let _ = fs::set_permissions(&safe_path, perms);
                }
            }
        }

        Ok(())
    }

    /// Validate package without extracting
    ///
    /// This performs a quick validation by checking the manifest only.
    pub fn validate_package<P: AsRef<Path>>(&self, package_path: P) -> IntResult<Manifest> {
        let package_path = package_path.as_ref();

        if !package_path.exists() {
            return Err(IntError::InvalidPackage(
                "Package file not found".to_string(),
            ));
        }

        let file = File::open(package_path).map_err(IntError::IoError)?;
        let decoder = GzDecoder::new(file);
        let mut archive = Archive::new(decoder);

        // Find and parse manifest
        for entry_result in archive
            .entries()
            .map_err(|e| IntError::CorruptedArchive(format!("Failed to read archive: {}", e)))?
        {
            let mut entry = entry_result
                .map_err(|e| IntError::CorruptedArchive(format!("Failed to read entry: {}", e)))?;

            let entry_path = entry
                .path()
                .map_err(|e| IntError::CorruptedArchive(format!("Invalid entry path: {}", e)))?;

            if entry_path == Path::new("manifest.json") {
                let mut content = String::new();
                entry
                    .read_to_string(&mut content)
                    .map_err(|e| IntError::ManifestParseError(e.to_string()))?;

                let manifest = Manifest::from_str(&content)?;
                manifest.validate()?;
                return Ok(manifest);
            }
        }

        Err(IntError::InvalidPackage(
            "manifest.json not found in package".to_string(),
        ))
    }

    /// Verify GPG signature of a package (detached)
    fn verify_gpg_signature(&self, package_path: &Path) -> IntResult<()> {
        let sig_path = package_path.with_extension("int.sig");
        if !sig_path.exists() {
            return Err(IntError::InvalidSignature(format!(
                "Signature file not found: {}",
                sig_path.display()
            )));
        }

        if let Some(ref callback) = self.log_callback {
            callback(format!(
                "Verifying external GPG signature for {}...",
                package_path.display()
            ));
        }

        use std::process::Command;
        let output = Command::new("gpg")
            .arg("--verify")
            .arg(&sig_path)
            .arg(package_path)
            .output()
            .map_err(|e| IntError::Custom(format!("Failed to execute gpg: {}", e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(IntError::InvalidSignature(format!(
                "GPG verification failed: {}",
                err
            )));
        }

        if let Some(ref callback) = self.log_callback {
            callback("GPG signature verified successfully.".to_string());
        }

        Ok(())
    }

    /// Verify embedded signature in manifest
    fn verify_embedded_signature(&self, manifest: &Manifest) -> IntResult<()> {
        let signature = match manifest.signature {
            Some(ref s) => s,
            None => return Ok(()),
        };

        if let Some(ref callback) = self.log_callback {
            callback("Verifying embedded GPG signature...".to_string());
        }

        // Create a manifest copy without the signature to verify it
        let mut manifest_to_verify = manifest.clone();
        manifest_to_verify.signature = None;
        let canonical_json = manifest_to_verify.to_canonical_string()?;

        use std::io::Write;
        use std::process::Command;

        // We use gpg --verify by stdin for the signature and file for the data
        // Or simpler: put signature in temp file, data in temp file
        let mut sig_file = tempfile::NamedTempFile::new()
            .map_err(|e| IntError::Custom(format!("Failed to create temp sig file: {}", e)))?;
        sig_file
            .write_all(signature.as_bytes())
            .map_err(|e| IntError::IoError(e))?;

        let mut data_file = tempfile::NamedTempFile::new()
            .map_err(|e| IntError::Custom(format!("Failed to create temp data file: {}", e)))?;
        data_file
            .write_all(canonical_json.as_bytes())
            .map_err(|e| IntError::IoError(e))?;

        let output = Command::new("gpg")
            .arg("--verify")
            .arg(sig_file.path())
            .arg(data_file.path())
            .output()
            .map_err(|e| IntError::Custom(format!("Failed to execute gpg: {}", e)))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            return Err(IntError::InvalidSignature(format!(
                "Embedded GPG verification failed: {}",
                err
            )));
        }

        if let Some(ref callback) = self.log_callback {
            callback("Embedded GPG signature verified successfully.".to_string());
        }

        Ok(())
    }

    /// Verify file hashes against extracted files
    fn verify_file_hashes(
        &self,
        extract_dir: &Path,
        hashes: &std::collections::BTreeMap<String, String>,
    ) -> IntResult<()> {
        if let Some(ref callback) = self.log_callback {
            callback(format!("Verifying hashes for {} files...", hashes.len()));
        }

        for (rel_path, expected_hash) in hashes {
            let full_path = extract_dir.join(rel_path);
            if !full_path.exists() {
                return Err(IntError::InvalidPackage(format!(
                    "File missing from package: {}",
                    rel_path
                )));
            }

            // Calculate SHA256
            let hash = self.calculate_sha256(&full_path)?;
            if hash != *expected_hash {
                return Err(IntError::InvalidSignature(format!(
                    "Hash mismatch for file {}: expected {}, found {}",
                    rel_path, expected_hash, hash
                )));
            }
        }

        if let Some(ref callback) = self.log_callback {
            callback("All file hashes verified successfully.".to_string());
        }

        Ok(())
    }

    /// Calculate SHA256 hash of a file
    fn calculate_sha256(&self, path: &Path) -> IntResult<String> {
        use sha2::{Digest, Sha256};
        let mut file = File::open(path).map_err(IntError::IoError)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let count = file.read(&mut buffer).map_err(IntError::IoError)?;
            if count == 0 {
                break;
            }
            hasher.update(&buffer[..count]);
        }

        Ok(format!("{:x}", hasher.finalize()))
    }
}

impl Default for PackageExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use tempfile::TempDir;

    fn create_test_package() -> (TempDir, PathBuf) {
        use flate2::write::GzEncoder;
        use flate2::Compression;
        use tar::Builder;

        let temp_dir = TempDir::new().unwrap();
        let package_path = temp_dir.path().join("test.int");

        // Create package content
        let manifest = r#"{
            "version": "1.0",
            "name": "test-app",
            "package_version": "1.0.0",
            "install_scope": "user",
            "install_path": "/home/user/.local/share/test-app"
        }"#;

        // Create tar.gz
        let file = File::create(&package_path).unwrap();
        let encoder = GzEncoder::new(file, Compression::default());
        let mut builder = Builder::new(encoder);

        // Add manifest
        let mut header = tar::Header::new_gnu();
        header.set_path("manifest.json").unwrap();
        header.set_size(manifest.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, manifest.as_bytes()).unwrap();

        // Add payload directory
        let mut header = tar::Header::new_gnu();
        header.set_path("payload/").unwrap();
        header.set_size(0);
        header.set_mode(0o755);
        header.set_entry_type(tar::EntryType::Directory);
        header.set_cksum();
        builder.append(&header, &[][..]).unwrap();

        // Add a test file
        let test_content = b"test file content";
        let mut header = tar::Header::new_gnu();
        header.set_path("payload/test.txt").unwrap();
        header.set_size(test_content.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder.append(&header, &test_content[..]).unwrap();

        builder.finish().unwrap();

        (temp_dir, package_path)
    }

    #[test]
    fn test_extract_package() {
        let (_temp, package_path) = create_test_package();

        let extractor = PackageExtractor::new();
        let extracted = extractor.extract(&package_path).unwrap();

        assert_eq!(extracted.manifest.name, "test-app");
        assert!(extracted.payload_dir.exists());
        assert!(extracted.payload_dir.join("test.txt").exists());
    }

    #[test]
    fn test_validate_package() {
        let (_temp, package_path) = create_test_package();

        let extractor = PackageExtractor::new();
        let manifest = extractor.validate_package(&package_path).unwrap();

        assert_eq!(manifest.name, "test-app");
        assert_eq!(manifest.package_version, "1.0.0");
    }

    #[test]
    fn test_progress_callback() {
        let (_temp, package_path) = create_test_package();

        let progress_called = Arc::new(AtomicBool::new(false));
        let progress_called_clone = Arc::clone(&progress_called);

        let extractor = PackageExtractor::new().with_progress(move |current, total| {
            assert!(current <= total);
            progress_called_clone.store(true, Ordering::SeqCst);
        });

        let _extracted = extractor.extract(&package_path).unwrap();
        assert!(progress_called.load(Ordering::SeqCst));
    }
}
