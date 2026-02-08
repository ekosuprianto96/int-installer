mod commands;
mod state;

use clap::Parser;
use int_core::{InstallConfig, InstallProgress, InstallScope, Installer, Uninstaller};
use state::AppState;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "int-engine")]
#[command(version, about = "INT Package Installer", long_about = None)]
struct Cli {
    /// Package file to install (.int)
    package: Option<PathBuf>,

    /// Uninstall a package
    #[arg(short, long)]
    uninstall: Option<String>,

    /// List installed packages
    #[arg(short, long)]
    list: bool,

    /// Installation scope (user or system)
    #[arg(long, default_value = "user")]
    scope: String,

    /// Custom installation path
    #[arg(long)]
    install_path: Option<PathBuf>,

    /// Start service after installation
    #[arg(long)]
    start_service: bool,

    /// Dry run (don't actually install)
    #[arg(long)]
    dry_run: bool,

    /// Run in GUI mode
    #[arg(short, long)]
    gui: bool,
}

fn main() {
    let cli = Cli::parse();

    if cli.gui || (cli.package.is_none() && !cli.list && cli.uninstall.is_none()) {
        run_gui();
    } else {
        if let Err(e) = run_cli(cli) {
            eprintln!("❌ Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_gui() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::validate_package,
            commands::install_package,
            commands::list_installed,
            commands::uninstall_package,
            commands::launch_app,
            commands::exit_app,
            commands::get_launch_args
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn run_cli(cli: Cli) -> anyhow::Result<()> {
    // Parse scope
    let scope = match cli.scope.as_str() {
        "user" => InstallScope::User,
        "system" => InstallScope::System,
        _ => anyhow::bail!("Invalid scope: {}. Use 'user' or 'system'", cli.scope),
    };

    // Handle commands
    if cli.list {
        cmd_list(scope)?;
    } else if let Some(package_name) = cli.uninstall {
        cmd_uninstall(&package_name, scope)?;
    } else if let Some(package_path) = cli.package {
        let config = InstallConfig {
            install_path: cli.install_path,
            start_service: cli.start_service,
            create_desktop_entry: true,
            dry_run: cli.dry_run,
        };
        cmd_install(&package_path, config)?;
    }

    Ok(())
}

/// Install a package (CLI version)
fn cmd_install(package_path: &PathBuf, config: InstallConfig) -> anyhow::Result<()> {
    use int_core::PackageExtractor;

    println!("📦 Installing package: {}", package_path.display());
    println!();

    // Validate package first
    let extractor = PackageExtractor::new();
    let manifest = extractor.validate_package(package_path)?;

    println!("Package Information:");
    println!("  Name: {}", manifest.display_name());
    println!("  Version: {}", manifest.package_version);
    if let Some(ref desc) = manifest.description {
        println!("  Description: {}", desc);
    }
    println!("  Scope: {:?}", manifest.install_scope);
    println!();

    // Create installer with progress callback
    let installer = Installer::new().with_progress(|progress| match progress {
        InstallProgress::Extracting { current, total } => {
            print!("\r🔄 Extracting... {}/{} bytes", current, total);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        InstallProgress::CopyingFiles { current, total } => {
            print!("\r📁 Copying files... {}/{}", current, total);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        InstallProgress::SettingPermissions => {
            print!("\r🔒 Setting permissions...");
            std::io::Write::flush(&mut std::io::stdout()).unwrap();
        }
        InstallProgress::ExecutingScript { script } => {
            println!("\n🔧 Running script: {}", script);
        }
        InstallProgress::RegisteringService => {
            println!("\n⚙️  Registering service...");
        }
        InstallProgress::CreatingDesktopEntry => {
            println!("\n🖥️  Creating desktop entry...");
        }
        InstallProgress::Finalizing => {
            println!("\n✨ Finalizing...");
        }
        InstallProgress::Log { message } => {
            println!("📝 {}", message);
        }
        InstallProgress::Completed => {
            println!("\n✅ Installation completed!");
        }
    });

    // Install
    let metadata = installer.install(package_path, config)?;

    println!();
    println!("Installation Details:");
    println!("  Installed to: {}", metadata.install_path.display());
    println!("  Files installed: {}", metadata.installed_files.len());

    if let Some(ref desktop) = metadata.desktop_entry {
        println!("  Desktop entry: {}", desktop.display());
    }

    if let Some(ref service) = metadata.service_name {
        println!("  Service: {}", service);
    }

    println!();
    println!("🎉 Package installed successfully!");

    Ok(())
}

/// Uninstall a package (CLI version)
fn cmd_uninstall(package_name: &str, scope: InstallScope) -> anyhow::Result<()> {
    println!("🗑️  Uninstalling package: {}", package_name);

    let uninstaller = Uninstaller::new();
    uninstaller.uninstall(package_name, scope)?;

    println!("✅ Package uninstalled successfully!");

    Ok(())
}

/// List installed packages (CLI version)
fn cmd_list(scope: InstallScope) -> anyhow::Result<()> {
    let uninstaller = Uninstaller::new();
    let packages = uninstaller.list_installed(scope)?;

    if packages.is_empty() {
        println!("No packages installed ({:?} scope)", scope);
        return Ok(());
    }

    println!("Installed Packages ({:?} scope):", scope);
    println!();

    for pkg in packages {
        println!("📦 {} v{}", pkg.package_name, pkg.package_version);
        println!("   Path: {}", pkg.install_path.display());
        println!("   Installed: {}", pkg.install_date);
        if let Some(ref service) = pkg.service_name {
            println!("   Service: {}", service);
        }
        println!();
    }

    Ok(())
}
