use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber;

mod builder;
mod validator;
mod template;

use builder::PackageBuilder;
use validator::PackageValidator;
use template::TemplateGenerator;

#[derive(Parser)]
#[command(name = "int-pack")]
#[command(about = "INT Package Builder - Create .int packages", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(global = true, short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new package template
    Init {
        /// Package name
        name: String,

        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Build a .int package
    Build {
        /// Package directory or manifest path
        path: PathBuf,

        /// Output .int file path
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Compress with gzip
        #[arg(short, long)]
        compress: bool,
    },

    /// Validate manifest
    Validate {
        /// Manifest file path
        manifest: PathBuf,
    },

    /// Show package information
    Info {
        /// Package directory
        path: PathBuf,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .init();

    match cli.command {
        Commands::Init { name, output } => {
            let generator = TemplateGenerator::new();
            generator.create_template(&name, output)?;
            println!("✓ Package template created successfully");
        }

        Commands::Build { path, output, compress } => {
            let builder = PackageBuilder::new(path);
            let output_path = builder.build(output, compress).await?;
            println!("✓ Package built successfully: {}", output_path.display());
        }

        Commands::Validate { manifest } => {
            let validator = PackageValidator::new();
            validator.validate(&manifest)?;
            println!("✓ Manifest is valid and compatible with int-core");
        }

        Commands::Info { path } => {
            let builder = PackageBuilder::new(path);
            builder.show_info().await?;
        }
    }

    Ok(())
}