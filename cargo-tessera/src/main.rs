use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
use commands::build::{AndroidFormat, BuildOptions, BuildPlatform};

#[derive(Parser)]
#[command(name = "cargo-tessera")]
#[command(bin_name = "cargo")]
#[command(version, about = "CLI tool for Tessera UI framework", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Tessera CLI commands
    #[command(name = "tessera")]
    Tessera(TesseraArgs),
}

#[derive(Parser)]
struct TesseraArgs {
    #[command(subcommand)]
    command: TesseraCommands,
}

#[derive(Subcommand)]
enum TesseraCommands {
    /// Create a new Tessera project
    New {
        /// Name of the project (optional, will prompt if not provided)
        name: Option<String>,
        /// Use a specific template
        #[arg(short, long)]
        template: Option<String>,
    },
    /// Start development server with hot reload
    Dev {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Build the project for release
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        /// Target platform
        #[arg(short, long)]
        target: Option<String>,
        /// Select build platform (native or android)
        #[arg(long, value_enum, default_value_t = BuildPlatform::Native)]
        platform: BuildPlatform,
        /// Override Android CPU architecture (e.g. arm64, armeabi-v7a)
        #[arg(long = "android-arch")]
        android_arch: Option<String>,
        /// Override Android package/binary name passed to xbuild (-p)
        #[arg(long = "android-package")]
        android_package: Option<String>,
        /// Override Android artifact format (apk or aab)
        #[arg(long = "android-format", value_enum)]
        android_format: Option<AndroidFormat>,
    },
}

fn main() -> Result<()> {
    let Cli { command } = Cli::parse();

    match command {
        Commands::Tessera(args) => match args.command {
            TesseraCommands::New { name, template } => {
                let name = match name {
                    Some(n) => n,
                    None => commands::new::prompt_project_name()?,
                };
                let template = match template {
                    Some(t) => t,
                    None => commands::new::select_template_interactive()?,
                };
                commands::new::execute(&name, &template)?;
            }
            TesseraCommands::Dev { verbose } => {
                commands::dev::execute(verbose)?;
            }
            TesseraCommands::Build {
                release,
                target,
                platform,
                android_arch,
                android_package,
                android_format,
            } => {
                let opts = BuildOptions {
                    release,
                    target,
                    platform,
                    android_arch,
                    android_package,
                    android_format,
                };
                commands::build::execute(opts)?;
            }
        },
    }

    Ok(())
}
