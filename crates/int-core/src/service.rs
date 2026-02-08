/// systemd service integration
///
/// This module handles systemd service registration, management, and cleanup.

use crate::error::{IntError, IntResult};
use crate::extractor::ExtractedPackage;
use crate::manifest::InstallScope;
use crate::utils;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// systemd service manager
pub struct ServiceManager;

impl ServiceManager {
    /// Create a new service manager
    pub fn new() -> Self {
        Self
    }

    /// Register a systemd service
    ///
    /// Copies service file to appropriate systemd directory and enables it.
    pub fn register(
        &self,
        extracted: &ExtractedPackage,
        install_path: &Path,
    ) -> IntResult<(PathBuf, String)> {
        let service_name = extracted.manifest.service_name();
        let scope = extracted.manifest.install_scope;

        // Find service file in extracted package
        let service_file_name = format!("{}.service", service_name);
        let source_service = extracted
            .services_dir
            .as_ref()
            .ok_or_else(|| {
                IntError::ServiceRegistrationFailed("No services directory found".to_string())
            })?
            .join(&service_file_name);

        if !source_service.exists() {
            return Err(IntError::ServiceRegistrationFailed(format!(
                "Service file not found: {}",
                service_file_name
            )));
        }

        // Read and process service file
        let mut service_content = fs::read_to_string(&source_service).map_err(|e| {
            IntError::ServiceRegistrationFailed(format!("Failed to read service file: {}", e))
        })?;

        // Replace installation path placeholder
        service_content =
            service_content.replace("{{INSTALL_PATH}}", &install_path.display().to_string());

        // Determine target service directory
        let service_dir = scope.systemd_service_path();
        utils::ensure_dir(&service_dir)?;

        let target_service = service_dir.join(&service_file_name);

        // Write service file
        fs::write(&target_service, service_content).map_err(|e| {
            IntError::ServiceRegistrationFailed(format!("Failed to write service file: {}", e))
        })?;

        // Reload systemd daemon
        self.reload_daemon(scope)?;

        // Enable service (but don't start it yet)
        self.enable(service_name, scope)?;

        Ok((target_service, service_name.to_string()))
    }

    /// Enable a systemd service
    pub fn enable(&self, service_name: &str, scope: InstallScope) -> IntResult<()> {
        let (systemctl_cmd, user_flag) = self.get_systemctl_command(scope);

        let mut cmd = Command::new(systemctl_cmd);
        cmd.arg("enable").arg(service_name);

        if let Some(flag) = user_flag {
            cmd.arg(flag);
        }

        let output = cmd.output().map_err(|e| {
            IntError::SystemdError(format!("Failed to execute systemctl: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IntError::ServiceRegistrationFailed(format!(
                "Failed to enable service: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Disable a systemd service
    pub fn disable(&self, service_name: &str, scope: InstallScope) -> IntResult<()> {
        let (systemctl_cmd, user_flag) = self.get_systemctl_command(scope);

        let mut cmd = Command::new(systemctl_cmd);
        cmd.arg("disable").arg(service_name);

        if let Some(flag) = user_flag {
            cmd.arg(flag);
        }

        let output = cmd.output().map_err(|e| {
            IntError::SystemdError(format!("Failed to execute systemctl: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IntError::SystemdError(format!(
                "Failed to disable service: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Start a systemd service
    pub fn start(&self, service_name: &str, scope: InstallScope) -> IntResult<()> {
        let (systemctl_cmd, user_flag) = self.get_systemctl_command(scope);

        let mut cmd = Command::new(systemctl_cmd);
        cmd.arg("start").arg(service_name);

        if let Some(flag) = user_flag {
            cmd.arg(flag);
        }

        let output = cmd.output().map_err(|e| {
            IntError::SystemdError(format!("Failed to execute systemctl: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IntError::SystemdError(format!(
                "Failed to start service: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Stop a systemd service
    pub fn stop(&self, service_name: &str, scope: InstallScope) -> IntResult<()> {
        let (systemctl_cmd, user_flag) = self.get_systemctl_command(scope);

        let mut cmd = Command::new(systemctl_cmd);
        cmd.arg("stop").arg(service_name);

        if let Some(flag) = user_flag {
            cmd.arg(flag);
        }

        let output = cmd.output().map_err(|e| {
            IntError::SystemdError(format!("Failed to execute systemctl: {}", e))
        })?;

        // Ignore errors when stopping (service might not be running)
        Ok(())
    }

    /// Check if service is active
    pub fn is_active(&self, service_name: &str, scope: InstallScope) -> bool {
        let (systemctl_cmd, user_flag) = self.get_systemctl_command(scope);

        let mut cmd = Command::new(systemctl_cmd);
        cmd.arg("is-active").arg(service_name);

        if let Some(flag) = user_flag {
            cmd.arg(flag);
        }

        cmd.output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Reload systemd daemon
    fn reload_daemon(&self, scope: InstallScope) -> IntResult<()> {
        let (systemctl_cmd, user_flag) = self.get_systemctl_command(scope);

        let mut cmd = Command::new(systemctl_cmd);
        cmd.arg("daemon-reload");

        if let Some(flag) = user_flag {
            cmd.arg(flag);
        }

        let output = cmd.output().map_err(|e| {
            IntError::SystemdError(format!("Failed to execute systemctl: {}", e))
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IntError::SystemdError(format!(
                "Failed to reload daemon: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Unregister a service
    pub fn unregister(&self, service_path: &Path, service_name: &str, scope: InstallScope) -> IntResult<()> {
        // Stop service if running
        let _ = self.stop(service_name, scope);

        // Disable service
        let _ = self.disable(service_name, scope);

        // Remove service file
        if service_path.exists() {
            fs::remove_file(service_path).map_err(|e| {
                IntError::SystemdError(format!("Failed to remove service file: {}", e))
            })?;
        }

        // Reload daemon
        self.reload_daemon(scope)?;

        Ok(())
    }

    /// Get systemctl command and user flag based on scope
    fn get_systemctl_command(&self, scope: InstallScope) -> (&str, Option<&str>) {
        match scope {
            InstallScope::User => ("systemctl", Some("--user")),
            InstallScope::System => ("systemctl", None),
        }
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_systemctl_command() {
        let manager = ServiceManager::new();

        let (cmd, flag) = manager.get_systemctl_command(InstallScope::User);
        assert_eq!(cmd, "systemctl");
        assert_eq!(flag, Some("--user"));

        let (cmd, flag) = manager.get_systemctl_command(InstallScope::System);
        assert_eq!(cmd, "systemctl");
        assert_eq!(flag, None);
    }
}
