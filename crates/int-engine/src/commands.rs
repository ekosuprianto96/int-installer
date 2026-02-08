use std::path::PathBuf;
use tauri::{Emitter, State, WebviewWindow};
use int_core::{PackageExtractor, Installer, InstallConfig, InstallProgress, InstallScope, Uninstaller};
use crate::state::AppState;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub display_name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub install_scope: String,
    pub install_path: String,
    pub auto_launch: bool,
    pub launch_command: Option<String>,
}

#[tauri::command]
pub async fn validate_package(path: String, state: State<'_, AppState>) -> Result<PackageInfo, String> {
    let path = PathBuf::from(path);
    let extractor = PackageExtractor::new();
    
    let manifest = extractor.validate_package(&path)
        .map_err(|e| format!("Validation error: {}", e))?;
    
    let info = PackageInfo {
        name: manifest.name.clone(),
        display_name: manifest.display_name().to_string(),
        version: manifest.package_version.clone(),
        description: manifest.description.clone().unwrap_or_default(),
        author: manifest.author.clone().unwrap_or_default(),
        license: manifest.license.clone().unwrap_or_default(),
        install_scope: format!("{:?}", manifest.install_scope),
        install_path: manifest.install_path.to_string_lossy().to_string(),
        auto_launch: manifest.auto_launch,
        launch_command: manifest.launch_command.clone(),
    };

    let mut current = state.current_manifest.lock().unwrap();
    *current = Some(manifest);

    Ok(info)
}

#[tauri::command]
pub async fn install_package(
    window: WebviewWindow,
    path: String,
    install_path: Option<String>,
    start_service: bool
) -> Result<(), String> {
    let path = PathBuf::from(path);
    let config = InstallConfig {
        install_path: install_path.map(PathBuf::from),
        start_service,
        create_desktop_entry: true,
        dry_run: false,
    };

    let installer = Installer::new().with_progress(move |progress| {
        let event_name = match progress {
            InstallProgress::Extracting { .. } => "install-progress-extracting",
            InstallProgress::CopyingFiles { .. } => "install-progress-copying",
            InstallProgress::SettingPermissions => "install-progress-permissions",
            InstallProgress::ExecutingScript { .. } => "install-progress-script",
            InstallProgress::RegisteringService => "install-progress-service",
            InstallProgress::CreatingDesktopEntry => "install-progress-desktop",
            InstallProgress::Finalizing => "install-progress-finalizing",
            InstallProgress::Completed => "install-progress-completed",
        };
        
        let payload = match progress {
            InstallProgress::Extracting { current, total } => {
                serde_json::json!({ "current": current, "total": total })
            },
            InstallProgress::CopyingFiles { current, total } => {
                serde_json::json!({ "current": current as u64, "total": total as u64 })
            },
            _ => serde_json::json!({})
        };

        let _ = window.emit(event_name, payload);
    });

    installer.install(&path, config)
        .map_err(|e| format!("Installation failed: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn list_installed(scope: String) -> Result<Vec<PackageInfo>, String> {
    let scope = match scope.as_str() {
        "system" => InstallScope::System,
        _ => InstallScope::User,
    };
    
    let uninstaller = Uninstaller::new();
    let packages = uninstaller.list_installed(scope)
        .map_err(|e| format!("Failed to list packages: {}", e))?;
    
    Ok(packages.into_iter().map(|p| PackageInfo {
        name: p.package_name.clone(),
        display_name: p.package_name,
        version: p.package_version,
        description: String::new(),
        author: String::new(),
        license: String::new(),
        install_scope: format!("{:?}", scope),
        install_path: String::new(),
        auto_launch: false,
        launch_command: None,
    }).collect())
}

#[tauri::command]
pub async fn uninstall_package(name: String, scope: String) -> Result<(), String> {
    let scope = match scope.as_str() {
        "system" => InstallScope::System,
        _ => InstallScope::User,
    };
    
    let uninstaller = Uninstaller::new();
    uninstaller.uninstall(&name, scope)
        .map_err(|e| format!("Uninstallation failed: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn launch_app(command: String, install_path: String) -> Result<(), String> {
    let install_path = std::path::PathBuf::from(install_path);
    
    // Command can be absolute or relative to install_path/bin
    let full_command = if std::path::Path::new(&command).is_absolute() {
        std::path::PathBuf::from(&command)
    } else {
        install_path.join("bin").join(&command)
    };

    if !full_command.exists() {
        return Err(format!("Launch command not found: {}", full_command.display()));
    }

    std::process::Command::new(full_command)
        .current_dir(install_path)
        .spawn()
        .map_err(|e| format!("Failed to launch application: {}", e))?;

    Ok(())
}

#[tauri::command]
pub async fn exit_app() {
    std::process::exit(0);
}

#[tauri::command]
pub fn get_launch_args() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();
    // In production, the file path is usually the second argument (index 1)
    // In dev, it might be different, but we focus on production behavior for now.
    if args.len() > 1 {
        // Simple check: return the last argument if it looks like a file path
        // This handles cases where there might be other flags
        // For simple association, the OS passes the file as an argument.
         for arg in args.iter().skip(1) {
            if arg.ends_with(".int") {
                return Some(arg.clone());
            }
        }
        // If no .int file found, but there is an arg, maybe it's the file (drag & drop often passes just the path)
        // Let's safe guard it to only return if it looks like a path or specific extension if strictly enforcing
        // For now, let's try to return the first non-flag argument if no .int specific found?
        // Actually, let's stick to .int for safety.
        None
    } else {
        None
    }
}
