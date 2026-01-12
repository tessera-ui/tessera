use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use commands::android::{self, AndroidFormat};

mod commands;
mod template;

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
    /// Start development server with auto rebuild/restart
    Dev {
        /// Enable verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Specify package to run
        #[arg(short, long)]
        package: Option<String>,
        /// Enable release mode
        #[arg(short, long)]
        release: bool,
    },
    /// Build the project for release (native targets)
    Build {
        /// Build in release mode
        #[arg(short, long)]
        release: bool,
        /// Target triple (passed to cargo build)
        #[arg(short, long)]
        target: Option<String>,
        /// Specify package to build
        #[arg(short, long)]
        package: Option<String>,
    },
    /// Android-specific helpers (build/dev)
    Android {
        #[command(subcommand)]
        command: AndroidCommands,
    },
}

#[derive(Subcommand)]
enum AndroidCommands {
    /// Initialize Android project (Gradle) for Tessera app
    Init {
        /// Skip installing Rust targets automatically
        #[arg(long)]
        skip_targets_install: bool,
    },
    /// Build Android artifacts using Gradle
    Build(AndroidBuildArgs),
    /// Run/install the app on an Android device via Gradle
    Dev(AndroidDevArgs),
    /// Build Rust library for a single Android target (used by Gradle)
    RustBuild(AndroidRustBuildArgs),
}

#[derive(Args)]
struct AndroidBuildArgs {
    /// Build in release mode
    #[arg(long, short)]
    release: bool,
    /// Override CPU architecture (default from metadata or arm64)
    #[arg(long)]
    arch: Option<String>,
    /// Override package/binary name (-p)
    #[arg(long, short)]
    package: Option<String>,
    /// Override artifact format (apk or aab)
    #[arg(long, short, value_enum)]
    format: Option<AndroidFormat>,
}

#[derive(Args)]
struct AndroidDevArgs {
    /// Run in release mode
    #[arg(long, short)]
    release: bool,
    /// Override CPU architecture
    #[arg(long)]
    arch: Option<String>,
    /// Override package/binary name (-p)
    #[arg(long, short)]
    package: Option<String>,
    /// Device id used by `x run --device`
    #[arg(long, short)]
    device: Option<String>,
}

#[derive(Args)]
struct AndroidRustBuildArgs {
    /// Build in release mode
    #[arg(long, short)]
    release: bool,
    /// Target triple (e.g. aarch64-linux-android)
    target: String,
    /// Override package/binary name (-p)
    #[arg(long, short)]
    package: Option<String>,
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
            TesseraCommands::Dev {
                verbose,
                package,
                release,
            } => {
                commands::dev::execute(verbose, package.as_deref(), release)?;
            }
            TesseraCommands::Build {
                release,
                target,
                package,
            } => {
                commands::build::execute(release, target.as_deref(), package.as_deref())?;
            }
            TesseraCommands::Android { command } => match command {
                AndroidCommands::Init {
                    skip_targets_install,
                } => {
                    commands::android::init(skip_targets_install)?;
                }
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
                AndroidCommands::RustBuild(build_args) => {
                    android::rust_build(android::RustBuildOptions {
                        release: build_args.release,
                        target: build_args.target.clone(),
                        package: build_args.package.clone(),
                    })?;
                }
            },
        },
    }

    Ok(())
}
