use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Args, Parser, Subcommand};

use commands::{
    android::{self, AndroidFormat},
    plugin,
};

mod commands;
mod output;
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
        /// Enable profiling output and write records to this JSONL file
        /// (desktop only)
        #[arg(long, value_name = "FILE")]
        profiling_output: Option<PathBuf>,
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
        /// Enable profiling output and write records to this JSONL file
        /// (desktop only)
        #[arg(long, value_name = "FILE")]
        profiling_output: Option<PathBuf>,
    },
    /// Profiling utilities
    Profiling {
        #[command(subcommand)]
        command: ProfilingCommands,
    },
    /// Android-specific helpers (build/dev)
    Android {
        #[command(subcommand)]
        command: AndroidCommands,
    },
    /// Create a new Tessera plugin
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
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

#[derive(Subcommand)]
enum PluginCommands {
    /// Create a new Tessera plugin
    New {
        /// Name of the plugin (optional, will prompt if not provided)
        name: Option<String>,
        /// Use a specific template
        #[arg(short, long)]
        template: Option<String>,
    },
}

#[derive(Subcommand)]
enum ProfilingCommands {
    /// Analyze profiler JSONL output
    Analyze {
        /// Path to tessera profiler JSONL output file
        path: PathBuf,
        /// Show top N component entries per section
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// Minimum sample count per component to include in top lists
        #[arg(long, default_value_t = 1)]
        min_count: u64,
        /// Skip non-frame JSON lines that fail parsing
        #[arg(long)]
        skip_invalid: bool,
        /// Export full per-component aggregated stats to CSV
        #[arg(long, value_name = "FILE")]
        csv: Option<PathBuf>,
    },
    /// Pull profiler JSONL from Android via adb and analyze it
    AnalyzeAndroid {
        /// Android app package name (applicationId)
        #[arg(long)]
        package: String,
        /// Device id from `adb devices`
        #[arg(long, short)]
        device: Option<String>,
        /// Path inside app sandbox (default: auto-detect common paths)
        #[arg(long, value_name = "REMOTE_PATH")]
        remote_path: Option<String>,
        /// Local output path for pulled JSONL before analysis
        #[arg(long, value_name = "FILE", default_value = "profiles/android.jsonl")]
        pull_to: PathBuf,
        /// Show top N component entries per section
        #[arg(long, default_value_t = 20)]
        top: usize,
        /// Minimum sample count per component to include in top lists
        #[arg(long, default_value_t = 1)]
        min_count: u64,
        /// Skip non-frame JSON lines that fail parsing
        #[arg(long)]
        skip_invalid: bool,
        /// Export full per-component aggregated stats to CSV
        #[arg(long, value_name = "FILE")]
        csv: Option<PathBuf>,
    },
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
    /// Enable profiling and write JSONL inside app sandbox at this path
    #[arg(long, value_name = "REMOTE_PATH")]
    profiling_output: Option<String>,
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
    /// Device id from `adb devices`
    #[arg(long, short)]
    device: Option<String>,
    /// Enable profiling and write JSONL inside app sandbox at this path
    #[arg(long, value_name = "REMOTE_PATH")]
    profiling_output: Option<String>,
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
    /// Enable profiling and write JSONL inside app sandbox at this path
    #[arg(long, value_name = "REMOTE_PATH")]
    profiling_output: Option<String>,
}

fn main() -> ExitCode {
    if let Err(err) = run() {
        print_error(&err);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run() -> Result<()> {
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
            TesseraCommands::Plugin { command } => match command {
                PluginCommands::New { name, template } => {
                    let name = match name {
                        Some(n) => n,
                        None => plugin::prompt_plugin_name()?,
                    };
                    let template = match template {
                        Some(t) => t,
                        None => plugin::select_template_interactive()?,
                    };
                    plugin::execute(&name, &template)?;
                }
            },
            TesseraCommands::Dev {
                verbose,
                package,
                release,
                profiling_output,
            } => {
                commands::dev::execute(
                    verbose,
                    package.as_deref(),
                    release,
                    profiling_output.as_deref(),
                )?;
            }
            TesseraCommands::Build {
                release,
                target,
                package,
                profiling_output,
            } => {
                commands::build::execute(
                    release,
                    target.as_deref(),
                    package.as_deref(),
                    profiling_output.as_deref(),
                )?;
            }
            TesseraCommands::Profiling { command } => match command {
                ProfilingCommands::Analyze {
                    path,
                    top,
                    min_count,
                    skip_invalid,
                    csv,
                } => {
                    commands::profiling::analyze(
                        &path,
                        top,
                        min_count,
                        skip_invalid,
                        csv.as_deref(),
                    )?;
                }
                ProfilingCommands::AnalyzeAndroid {
                    package,
                    device,
                    remote_path,
                    pull_to,
                    top,
                    min_count,
                    skip_invalid,
                    csv,
                } => {
                    commands::profiling::analyze_android(
                        &package,
                        device.as_deref(),
                        remote_path.as_deref(),
                        &pull_to,
                        top,
                        min_count,
                        skip_invalid,
                        csv.as_deref(),
                    )?;
                }
            },
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
                        profiling_output: build_args.profiling_output.clone(),
                    })?;
                }
                AndroidCommands::Dev(dev_args) => {
                    android::dev(android::DevOptions {
                        release: dev_args.release,
                        arch: dev_args.arch.clone(),
                        package: dev_args.package.clone(),
                        device: dev_args.device.clone(),
                        profiling_output: dev_args.profiling_output.clone(),
                    })?;
                }
                AndroidCommands::RustBuild(build_args) => {
                    android::rust_build(android::RustBuildOptions {
                        release: build_args.release,
                        target: build_args.target.clone(),
                        package: build_args.package.clone(),
                        profiling_output: build_args.profiling_output.clone(),
                    })?;
                }
            },
        },
    }

    Ok(())
}

fn print_error(err: &anyhow::Error) {
    output::error(err.to_string());
    for cause in err.chain().skip(1) {
        output::note(format!("caused by: {cause}"));
    }
}
