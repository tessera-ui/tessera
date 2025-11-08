use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod commands;
use commands::android::{self, AndroidFormat};

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
    /// Build the project for release (native targets)
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        /// Target triple (passed to cargo build)
        #[arg(short, long)]
        target: Option<String>,
    },
    /// Android-specific helpers (build/dev)
    Android {
        #[command(subcommand)]
        command: AndroidCommands,
    },
}

#[derive(Subcommand)]
enum AndroidCommands {
    /// Build Android artifacts using xbuild
    Build(AndroidBuildArgs),
    /// Run/install the app on an Android device via xbuild
    Dev(AndroidDevArgs),
}

#[derive(Args)]
struct AndroidBuildArgs {
    /// Build in release mode
    #[arg(long)]
    release: bool,
    /// Override CPU architecture (default from metadata or arm64)
    #[arg(long = "arch")]
    arch: Option<String>,
    /// Override package/binary name (-p)
    #[arg(long = "package")]
    package: Option<String>,
    /// Override artifact format (apk or aab)
    #[arg(long = "format", value_enum)]
    format: Option<AndroidFormat>,
}

#[derive(Args)]
struct AndroidDevArgs {
    /// Run in release mode
    #[arg(long)]
    release: bool,
    /// Override CPU architecture
    #[arg(long = "arch")]
    arch: Option<String>,
    /// Override package/binary name (-p)
    #[arg(long = "package")]
    package: Option<String>,
    /// Device id used by `x run --device`
    #[arg(long = "device")]
    device: Option<String>,
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
            TesseraCommands::Build { release, target } => {
                commands::build::execute(release, target.as_deref())?;
            }
            TesseraCommands::Android { command } => match command {
                AndroidCommands::Build(build_args) => {
                    android::build(android::BuildOptions {
                        release: build_args.release,
                        arch: build_args.arch.clone(),
                        package: build_args.package.clone(),
                        format: build_args.format,
                    })?;
                }
                AndroidCommands::Dev(dev_args) => {
                    android::dev(android::DevOptions {
                        release: dev_args.release,
                        arch: dev_args.arch.clone(),
                        package: dev_args.package.clone(),
                        device: dev_args.device.clone(),
                    })?;
                }
            },
        },
    }

    Ok(())
}
