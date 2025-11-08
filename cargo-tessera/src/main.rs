use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;

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
            TesseraCommands::Build { release, target } => {
                commands::build::execute(release, target.as_deref())?;
            }
        },
    }

    Ok(())
}
