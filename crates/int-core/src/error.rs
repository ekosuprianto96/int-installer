// ...existing code...
/// Error types for INT Installer core library
///
/// This module defines all possible errors that can occur during
/// package parsing, extraction, installation, and system integration.
use std::path::PathBuf;
use std::{error::Error as StdError, fmt};

/// Main error type for INT Installer operations
#[derive(Debug)]
pub enum IntError {
    // ===== Package Errors =====
    /// Package file is invalid or corrupted
    InvalidPackage(String),

    /// Manifest parsing failed
    ManifestParseError(String),

    /// Archive is corrupted or incomplete
    CorruptedArchive(String),

    /// Required field missing in manifest
    MissingField(String),

    // ===== Installation Errors =====
    /// Insufficient permissions for operation
    InsufficientPermissions(String),

    /// Target installation path already exists
    TargetPathExists(PathBuf),

    /// Not enough disk space for installation
    DiskSpaceInsufficient { required: u64, available: u64 },

    /// Installation directory creation failed
    DirectoryCreationFailed(String),

    /// File copy operation failed
    FileCopyFailed {
        source: String,
        dest: String,
        reason: String,
    },

    // ===== System Integration Errors =====
    /// systemd service registration failed
    ServiceRegistrationFailed(String),

    /// Desktop entry creation failed
    DesktopEntryFailed(String),

    /// MIME type registration failed
    MimeRegistrationFailed(String),

    // ===== Security Errors =====
    /// Path traversal attempt detected
    PathTraversalAttempt(PathBuf),

    /// Invalid or unverified signature
    InvalidSignature(String),

    /// Publisher not in trusted list
    UntrustedPublisher(String),

    /// Invalid or malicious script detected
    InvalidScript(String),

    // ===== Script Execution Errors =====
    /// Script execution failed
    ScriptExecutionFailed { script: String, exit_code: i32 },

    /// Script timeout
    ScriptTimeout(String),

    // ===== System Errors =====
    /// Generic I/O error
    IoError(std::io::Error),

    /// systemd interaction error
    SystemdError(String),

    /// Permission setting error
    PermissionError(String),

    /// User/group lookup error
    UserLookupError(String),

    // ===== Validation Errors =====
    /// Manifest validation failed
    ValidationError(String),

    /// Unsupported manifest version
    UnsupportedVersion { found: String, expected: String },

    /// Invalid installation scope
    InvalidScope(String),

    // ===== Uninstallation Errors =====
    /// Package not found in installation registry
    PackageNotInstalled(String),

    /// Installation metadata corrupted
    MetadataCorrupted(String),

    // ===== Generic Errors =====
    /// Generic error with custom message
    Custom(String),

    /// Unexpected error
    Unexpected(String),
}

/// Result type alias for INT operations
pub type IntResult<T> = Result<T, IntError>;

/// Validation-specific errors
#[derive(Debug)]
pub enum ValidationError {
    InvalidValue {
        field: String,
        value: String,
    },

    OutOfRange {
        field: String,
        min: i64,
        max: i64,
        value: i64,
    },

    MalformedPath(String),

    UnsupportedFileType(String),

    ChecksumMismatch {
        expected: String,
        actual: String,
    },
}

// Implement Display for IntError (replacing thiserror derive to avoid AsDynError generation)
impl fmt::Display for IntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntError::InvalidPackage(s) => write!(f, "Invalid package: {}", s),
            IntError::ManifestParseError(s) => write!(f, "Failed to parse manifest: {}", s),
            IntError::CorruptedArchive(s) => write!(f, "Corrupted archive: {}", s),
            IntError::MissingField(s) => write!(f, "Missing required field in manifest: {}", s),

            IntError::InsufficientPermissions(s) => write!(f, "Insufficient permissions: {}", s),
            IntError::TargetPathExists(p) => {
                write!(f, "Target path already exists: {}", p.display())
            }
            IntError::DiskSpaceInsufficient {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient disk space: required {} bytes, available {} bytes",
                    required, available
                )
            }
            IntError::DirectoryCreationFailed(s) => {
                write!(f, "Failed to create installation directory: {}", s)
            }
            IntError::FileCopyFailed {
                source,
                dest,
                reason,
            } => {
                write!(
                    f,
                    "Failed to copy file from {} to {}: {}",
                    source, dest, reason
                )
            }

            IntError::ServiceRegistrationFailed(s) => {
                write!(f, "Failed to register systemd service: {}", s)
            }
            IntError::DesktopEntryFailed(s) => write!(f, "Failed to create desktop entry: {}", s),
            IntError::MimeRegistrationFailed(s) => write!(f, "Failed to register MIME type: {}", s),

            IntError::PathTraversalAttempt(p) => {
                write!(f, "Path traversal attempt detected: {}", p.display())
            }
            IntError::InvalidSignature(s) => write!(f, "Invalid package signature: {}", s),
            IntError::UntrustedPublisher(s) => write!(f, "Untrusted publisher: {}", s),
            IntError::InvalidScript(s) => write!(f, "Invalid script: {}", s),

            IntError::ScriptExecutionFailed { script, exit_code } => {
                write!(
                    f,
                    "Script execution failed: {} (exit code: {})",
                    script, exit_code
                )
            }
            IntError::ScriptTimeout(s) => write!(f, "Script execution timeout: {}", s),

            IntError::IoError(e) => write!(f, "I/O error: {}", e),
            IntError::SystemdError(s) => write!(f, "systemd error: {}", s),
            IntError::PermissionError(s) => write!(f, "Failed to set permissions: {}", s),
            IntError::UserLookupError(s) => write!(f, "Failed to lookup user/group: {}", s),

            IntError::ValidationError(s) => write!(f, "Manifest validation failed: {}", s),
            IntError::UnsupportedVersion { found, expected } => {
                write!(
                    f,
                    "Unsupported manifest version: {}, expected: {}",
                    found, expected
                )
            }
            IntError::InvalidScope(s) => write!(
                f,
                "Invalid installation scope: {} (expected: user or system)",
                s
            ),

            IntError::PackageNotInstalled(s) => write!(f, "Package not installed: {}", s),
            IntError::MetadataCorrupted(s) => write!(f, "Installation metadata corrupted: {}", s),

            IntError::Custom(s) => write!(f, "{}", s),
            IntError::Unexpected(s) => write!(f, "Unexpected error: {}", s),
        }
    }
}

// Implement std::error::Error for IntError with explicit source handling
impl StdError for IntError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            IntError::IoError(e) => Some(e),
            _ => None,
        }
    }
}

// Provide From<std::io::Error> so `?` still works where expected
impl From<std::io::Error> for IntError {
    fn from(e: std::io::Error) -> Self {
        IntError::IoError(e)
    }
}

// Implement Display and Error for ValidationError
impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValidationError::InvalidValue { field, value } => {
                write!(f, "Invalid field value: {} = {}", field, value)
            }
            ValidationError::OutOfRange {
                field,
                min,
                max,
                value,
            } => {
                write!(
                    f,
                    "Field out of range: {} (min: {}, max: {}, got: {})",
                    field, min, max, value
                )
            }
            ValidationError::MalformedPath(s) => write!(f, "Malformed path: {}", s),
            ValidationError::UnsupportedFileType(s) => write!(f, "Unsupported file type: {}", s),
            ValidationError::ChecksumMismatch { expected, actual } => {
                write!(
                    f,
                    "Checksum mismatch: expected {}, got {}",
                    expected, actual
                )
            }
        }
    }
}

impl StdError for ValidationError {}

// ...existing code...
impl IntError {
    /// Create a custom error with a message
    pub fn custom<S: Into<String>>(msg: S) -> Self {
        IntError::Custom(msg.into())
    }

    /// Check if error is recoverable
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            IntError::TargetPathExists(_)
                | IntError::ScriptExecutionFailed { .. }
                | IntError::ValidationError(_)
        )
    }

    /// Check if error requires elevated permissions
    pub fn requires_elevation(&self) -> bool {
        matches!(
            self,
            IntError::InsufficientPermissions(_) | IntError::PermissionError(_)
        )
    }

    /// Get user-friendly error message
    ///
    /// This converts technical errors into messages suitable for end users
    pub fn user_message(&self) -> String {
        match self {
            IntError::InvalidPackage(_) => {
                "File package tidak valid. Pastikan file .int tidak rusak.".to_string()
            }
            IntError::InsufficientPermissions(_) => {
                "Izin tidak cukup. Coba install sebagai user atau minta akses administrator."
                    .to_string()
            }
            IntError::TargetPathExists(path) => {
                format!(
                    "Direktori tujuan sudah ada: {}. Hapus terlebih dahulu atau pilih lokasi lain.",
                    path.display()
                )
            }
            IntError::DiskSpaceInsufficient {
                required,
                available,
            } => {
                format!(
                    "Ruang disk tidak cukup. Dibutuhkan {} MB, tersedia {} MB.",
                    required / 1_000_000,
                    available / 1_000_000
                )
            }
            IntError::ServiceRegistrationFailed(_) => {
                "Gagal mendaftarkan service. Periksa konfigurasi systemd.".to_string()
            }
            IntError::PathTraversalAttempt(_) => {
                "Package mengandung path berbahaya. Instalasi dibatalkan untuk keamanan."
                    .to_string()
            }
            _ => format!("Terjadi kesalahan: {}", self),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = IntError::custom("test error");
        assert!(matches!(err, IntError::Custom(_)));
    }

    #[test]
    fn test_recoverable_check() {
        let err = IntError::TargetPathExists(PathBuf::from("/tmp/test"));
        assert!(err.is_recoverable());

        let err = IntError::InvalidSignature("test".to_string());
        assert!(!err.is_recoverable());
    }

    #[test]
    fn test_user_message() {
        let err = IntError::DiskSpaceInsufficient {
            required: 1_000_000_000,
            available: 500_000_000,
        };
        let msg = err.user_message();
        assert!(msg.contains("Ruang disk tidak cukup"));
    }
}
// ...existing code...
