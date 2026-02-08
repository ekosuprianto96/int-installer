use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing_subscriber;

mod builder;
mod template;
mod validator;

use builder::PackageBuilder;
use template::TemplateGenerator;
use validator::PackageValidator;

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

        /// Sign the package with GPG
        #[arg(short, long)]
        sign: bool,

        /// GPG key ID to use for signing
        #[arg(short, long)]
        key: Option<String>,
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
    tracing_subscriber::fmt().with_env_filter(log_level).init();

    match cli.command {
        Commands::Init { name, output } => {
            let generator = TemplateGenerator::new();
            generator.create_template(&name, output)?;
            println!("✓ Package template created successfully");
        }

        Commands::Build {
            path,
            output,
            compress,
            sign,
            key,
        } => {
            let builder = PackageBuilder::new(path);
            let output_path = builder.build(output, compress, sign, key).await?;
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
