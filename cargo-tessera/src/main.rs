use std::{path::PathBuf, process::ExitCode};

use anyhow::{Result, bail};
use clap::{ArgMatches, Args, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use tessera_build::AssetBackend;

use commands::{
    android::{self, AndroidFormat},
    plugin, web,
};

mod call_guard;
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
    /// Install the bundled tessera-good-taste agent skill (asks for a scope
    /// unless one is given)
    Skill(SkillArgs),
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
        /// Show a live, top-like component profiler in the dev terminal
        #[arg(long)]
        top: bool,
        /// Overlay dirty replay regions with a translucent debug color
        #[arg(long)]
        debug_dirty_overlay: bool,
        /// Override asset backend
        #[arg(long, value_enum)]
        asset_backend: Option<AssetBackendArg>,
    },
    /// Run a non-interactive, session-based headless debug server over JSONL
    /// stdio
    Headless(HeadlessArgs),
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
        /// Overlay dirty replay regions with a translucent debug color
        #[arg(long)]
        debug_dirty_overlay: bool,
        /// Override asset backend
        #[arg(long, value_enum)]
        asset_backend: Option<AssetBackendArg>,
    },
    /// Check compilation and Tessera call boundaries
    Check {
        #[command(flatten)]
        args: CheckArgs,
        /// Explain a Tessera diagnostic code without checking a project
        #[arg(long, value_name = "CODE")]
        explain: Option<call_guard::ErrorCode>,
    },
    /// Apply compiler fixes and machine-applicable Tessera suggestions
    Fix {
        #[command(flatten)]
        args: CheckArgs,
        #[command(flatten)]
        fix: FixArgs,
    },
    /// Profiling utilities
    Profiling {
        #[command(subcommand)]
        command: ProfilingCommands,
    },
    /// Android-specific build and development commands
    Android {
        #[command(subcommand)]
        command: AndroidCommands,
    },
    /// Web-specific initialization and development commands
    Web {
        #[command(subcommand)]
        command: WebCommands,
    },
    /// Create a new Tessera plugin
    Plugin {
        #[command(subcommand)]
        command: PluginCommands,
    },
}

#[derive(Args)]
pub(crate) struct CheckArgs {
    #[command(flatten)]
    lints: LintArgs,
    /// Select a package
    #[arg(short, long)]
    package: Option<String>,
    /// Target triple
    #[arg(short, long)]
    target: Option<String>,
    /// Use release mode
    #[arg(short, long)]
    release: bool,
    /// Select only this package's library
    #[arg(long)]
    lib: bool,
    /// Select all binaries
    #[arg(long)]
    bins: bool,
    /// Select all examples
    #[arg(long)]
    examples: bool,
    /// Select all tests
    #[arg(long)]
    tests: bool,
    /// Select all benches
    #[arg(long)]
    benches: bool,
    /// Select all targets
    #[arg(long)]
    all_targets: bool,
    /// Features to activate
    #[arg(short = 'F', long)]
    features: Vec<String>,
    /// Activate all available features
    #[arg(long)]
    all_features: bool,
    /// Do not activate the `default` feature
    #[arg(long)]
    no_default_features: bool,
    /// Number of parallel jobs
    #[arg(short, long)]
    jobs: Option<String>,
    /// Select all packages in the workspace
    #[arg(long)]
    workspace: bool,
    /// Error format
    #[arg(long)]
    message_format: Option<String>,
    /// Use verbose output
    #[arg(short, long)]
    verbose: bool,
    /// Do not print cargo log messages
    #[arg(short, long)]
    quiet: bool,
}

#[derive(Args)]
pub(crate) struct FixArgs {
    /// Fix code even if the working directory is dirty
    #[arg(long)]
    allow_dirty: bool,
    /// Fix code even if the working directory has staged changes
    #[arg(long)]
    allow_staged: bool,
    /// Fix code even if a VCS was not detected
    #[arg(long)]
    allow_no_vcs: bool,
    /// Fix code even if it already has compiler errors
    #[arg(long)]
    broken_code: bool,
    /// Fix in preparation for the next edition
    #[arg(long)]
    edition: bool,
    /// Fix warnings to migrate to the idioms of an edition
    #[arg(long)]
    edition_idioms: bool,
}

#[derive(Args)]
struct LintArgs {
    /// Allow a Tessera lint or group
    #[arg(short = 'A', long, value_name = "LINT")]
    allow: Vec<call_guard::LintTarget>,
    /// Warn on a Tessera lint or group
    #[arg(short = 'W', long, value_name = "LINT")]
    warn: Vec<call_guard::LintTarget>,
    /// Deny a Tessera lint or group
    #[arg(short = 'D', long, value_name = "LINT")]
    deny: Vec<call_guard::LintTarget>,
    /// Forbid a Tessera lint or group
    #[arg(long, value_name = "LINT")]
    forbid: Vec<call_guard::LintTarget>,
    /// Warn even when a Tessera lint is allowed or capped
    #[arg(long, value_name = "LINT", value_parser = parse_force_warn)]
    force_warn: Vec<call_guard::LintTarget>,
    /// Cap ordinary Tessera lint levels at allow, warn, deny, or forbid
    #[arg(long, value_name = "LEVEL", value_parser = parse_lint_cap)]
    cap_lints: Option<call_guard::Level>,
}

impl LintArgs {
    fn options(&self, matches: &ArgMatches) -> call_guard::LintOptions {
        let check = matches
            .subcommand_matches("tessera")
            .and_then(|matches| matches.subcommand())
            .map(|(_, matches)| matches)
            .expect("parsed check or fix command");
        let mut overrides = Vec::new();
        for (arg, level, targets) in [
            ("allow", call_guard::Level::Allow, &self.allow),
            ("warn", call_guard::Level::Warn, &self.warn),
            ("deny", call_guard::Level::Deny, &self.deny),
            ("forbid", call_guard::Level::Forbid, &self.forbid),
            ("force_warn", call_guard::Level::ForceWarn, &self.force_warn),
        ] {
            if let Some(indices) = check.indices_of(arg) {
                overrides.extend(
                    indices
                        .zip(targets.iter().copied())
                        .map(|(index, target)| (index, (target, level))),
                );
            }
        }
        overrides.sort_by_key(|(index, _)| *index);
        call_guard::LintOptions {
            overrides: overrides.into_iter().map(|(_, value)| value).collect(),
            cap: self.cap_lints,
        }
    }
}

fn parse_lint_cap(value: &str) -> Result<call_guard::Level, String> {
    let level = value.parse()?;
    match level {
        call_guard::Level::ForceWarn => {
            Err("--cap-lints accepts allow, warn, deny, or forbid".into())
        }
        level => Ok(level),
    }
}

fn parse_force_warn(value: &str) -> Result<call_guard::LintTarget, String> {
    match value.parse()? {
        call_guard::LintTarget::Warnings => Err("--force-warn does not accept `warnings`".into()),
        target => Ok(target),
    }
}

#[derive(Args)]
struct SkillArgs {
    /// Project directory for local installation (default: current directory)
    #[arg(long, conflicts_with = "global")]
    path: Option<PathBuf>,
    /// Install into the user's ~/.agents/skills directory without prompting
    #[arg(long, conflicts_with = "local")]
    global: bool,
    /// Install into the project's .agents/skills directory without prompting
    #[arg(long)]
    local: bool,
    /// Replace locally modified files belonging to the bundled skill
    #[arg(long)]
    force: bool,
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
enum WebCommands {
    /// Initialize minimal browser host and wasm entry support for a Tessera app
    Init {
        /// Override package/binary name (-p)
        #[arg(long, short)]
        package: Option<String>,
    },
    /// Build a Tessera app for the browser
    Build {
        /// Build in release mode
        #[arg(long, short)]
        release: bool,
        /// Override package/binary name (-p)
        #[arg(long, short)]
        package: Option<String>,
        /// Features to activate for Tessera checking and the web build
        #[arg(short = 'F', long, value_delimiter = ',')]
        features: Vec<String>,
    },
    /// Rebuild and serve a Tessera app for browser development
    Dev {
        /// Build in release mode
        #[arg(long, short)]
        release: bool,
        /// Override package/binary name (-p)
        #[arg(long, short)]
        package: Option<String>,
        /// Port for the local static file server
        #[arg(long, default_value_t = 8000)]
        port: u16,
    },
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
        /// Show full profiling summary and all ranking sections
        #[arg(short, long)]
        verbose: bool,
        /// Show top N component entries per section (default: 5 compact, 20
        /// verbose)
        #[arg(long)]
        top: Option<usize>,
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
        /// Show full profiling summary and all ranking sections
        #[arg(short, long)]
        verbose: bool,
        /// Show top N component entries per section (default: 5 compact, 20
        /// verbose)
        #[arg(long)]
        top: Option<usize>,
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
enum AssetBackendArg {
    Embed,
    Platform,
}

impl AssetBackendArg {
    fn to_backend(self) -> AssetBackend {
        match self {
            Self::Embed => AssetBackend::Embed,
            Self::Platform => AssetBackend::Platform,
        }
    }
}

#[derive(Args)]
struct HeadlessArgs {
    /// Package to run as headless workers (defaults to the workspace root
    /// package)
    #[arg(long, short)]
    package: Option<String>,
    /// Build workers in release mode
    #[arg(long, short)]
    release: bool,
    /// Default offscreen width in physical pixels for new sessions
    #[arg(long, default_value_t = 800)]
    width: u32,
    /// Default offscreen height in physical pixels for new sessions
    #[arg(long, default_value_t = 600)]
    height: u32,
    /// Virtual frame step in milliseconds for new sessions
    #[arg(long, default_value_t = 16)]
    frame_time_ms: u32,
    /// Enable profiling output and write records to this JSONL file
    #[arg(long, value_name = "FILE")]
    profiling_output: Option<PathBuf>,
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
    /// Overlay dirty replay regions with a translucent debug color
    #[arg(long)]
    debug_dirty_overlay: bool,
    /// Override asset backend
    #[arg(long, value_enum)]
    asset_backend: Option<AssetBackendArg>,
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
    /// Overlay dirty replay regions with a translucent debug color
    #[arg(long)]
    debug_dirty_overlay: bool,
    /// Override asset backend
    #[arg(long, value_enum)]
    asset_backend: Option<AssetBackendArg>,
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
    /// Overlay dirty replay regions with a translucent debug color
    #[arg(long)]
    debug_dirty_overlay: bool,
    /// Override asset backend
    #[arg(long, value_enum)]
    asset_backend: Option<AssetBackendArg>,
}

fn main() -> ExitCode {
    if let Err(err) = run() {
        print_error(&err);
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run() -> Result<()> {
    let matches = Cli::command().get_matches();
    let Cli { command } = Cli::from_arg_matches(&matches)?;

    match command {
        Commands::Tessera(args) => match args.command {
            TesseraCommands::Skill(SkillArgs {
                path,
                global,
                local,
                force,
            }) => {
                commands::skill::execute(path.as_deref(), force, global, local)?;
            }
            TesseraCommands::New { name, template } => {
                let name = match name {
                    Some(n) => n,
                    None => commands::prompt_name("Project", "my-tessera-app")?,
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
                        None => commands::prompt_name("Plugin", "tessera-plugin")?,
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
                top,
                debug_dirty_overlay,
                asset_backend,
            } => {
                commands::dev::execute(
                    verbose,
                    package.as_deref(),
                    release,
                    profiling_output.as_deref(),
                    debug_dirty_overlay,
                    asset_backend.map(AssetBackendArg::to_backend),
                    top,
                )?;
            }
            TesseraCommands::Headless(args) => {
                commands::headless::execute(
                    args.package.as_deref(),
                    args.release,
                    args.width,
                    args.height,
                    args.frame_time_ms,
                    args.profiling_output.as_deref(),
                )?;
            }
            TesseraCommands::Build {
                release,
                target,
                package,
                profiling_output,
                debug_dirty_overlay,
                asset_backend,
            } => {
                commands::build::execute(
                    release,
                    target.as_deref(),
                    package.as_deref(),
                    profiling_output.as_deref(),
                    debug_dirty_overlay,
                    asset_backend.map(AssetBackendArg::to_backend),
                )?;
            }
            TesseraCommands::Check { args, explain } => {
                if let Some(code) = explain {
                    print!("{}", code.explanation());
                    return Ok(());
                }
                let lints = args.lints.options(&matches);
                commands::check::execute(&args, lints, None)?;
            }
            TesseraCommands::Fix { args, fix } => {
                let lints = args.lints.options(&matches);
                commands::check::execute(&args, lints, Some(&fix))?;
            }
            TesseraCommands::Profiling { command } => match command {
                ProfilingCommands::Analyze {
                    path,
                    verbose,
                    top,
                    min_count,
                    skip_invalid,
                    csv,
                } => {
                    let top = top.unwrap_or(if verbose { 20 } else { 5 });
                    if top == 0 {
                        bail!("--top must be greater than 0");
                    }
                    commands::profiling::analyze(
                        &path,
                        verbose,
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
                    verbose,
                    top,
                    min_count,
                    skip_invalid,
                    csv,
                } => {
                    let top = top.unwrap_or(if verbose { 20 } else { 5 });
                    if top == 0 {
                        bail!("--top must be greater than 0");
                    }
                    commands::profiling::analyze_android(
                        commands::profiling::AnalyzeAndroidOptions {
                            package: &package,
                            device: device.as_deref(),
                            remote_path: remote_path.as_deref(),
                            pull_to: &pull_to,
                            verbose,
                            top,
                            min_count,
                            skip_invalid,
                            csv: csv.as_deref(),
                        },
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
                        debug_dirty_overlay: build_args.debug_dirty_overlay,
                        asset_backend: build_args.asset_backend.map(AssetBackendArg::to_backend),
                    })?;
                }
                AndroidCommands::Dev(dev_args) => {
                    android::dev(android::DevOptions {
                        release: dev_args.release,
                        arch: dev_args.arch.clone(),
                        package: dev_args.package.clone(),
                        device: dev_args.device.clone(),
                        profiling_output: dev_args.profiling_output.clone(),
                        debug_dirty_overlay: dev_args.debug_dirty_overlay,
                        asset_backend: dev_args.asset_backend.map(AssetBackendArg::to_backend),
                    })?;
                }
                AndroidCommands::RustBuild(build_args) => {
                    android::rust_build(android::RustBuildOptions {
                        release: build_args.release,
                        target: build_args.target.clone(),
                        package: build_args.package.clone(),
                        profiling_output: build_args.profiling_output.clone(),
                        debug_dirty_overlay: build_args.debug_dirty_overlay,
                        asset_backend: build_args.asset_backend.map(AssetBackendArg::to_backend),
                    })?;
                }
            },
            TesseraCommands::Web { command } => match command {
                WebCommands::Init { package } => {
                    web::init(package.as_deref())?;
                }
                WebCommands::Build {
                    release,
                    package,
                    features,
                } => {
                    web::build(release, package.as_deref(), &features)?;
                }
                WebCommands::Dev {
                    release,
                    package,
                    port,
                } => {
                    web::dev(release, package.as_deref(), port)?;
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

#[cfg(test)]
mod cli_tests {
    use super::*;

    fn skill_args(extra: &[&str]) -> Result<SkillArgs, clap::Error> {
        let mut args = vec!["cargo", "tessera", "skill"];
        args.extend(extra);
        let cli = Cli::try_parse_from(args)?;
        let Commands::Tessera(args) = cli.command;
        let TesseraCommands::Skill(skill) = args.command else {
            panic!("expected the skill command");
        };
        Ok(skill)
    }

    #[test]
    fn skill_accepts_installation_options_directly() {
        let interactive = skill_args(&[]).unwrap();
        assert!(!interactive.global && !interactive.local && interactive.path.is_none());

        let local = skill_args(&["--local", "--path", "app", "--force"]).unwrap();
        assert!(local.local && local.force && !local.global);
        assert_eq!(local.path, Some(PathBuf::from("app")));

        let global = skill_args(&["--global"]).unwrap();
        assert!(global.global && !global.local && global.path.is_none());
    }

    #[test]
    fn removed_install_subcommand_is_rejected() {
        for extra in [&["install"][..], &["install", "--local"]] {
            let error = match skill_args(extra) {
                Ok(_) => panic!("`skill install` must no longer parse"),
                Err(error) => error,
            };
            assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
        }
    }

    #[test]
    fn skill_scope_flags_conflict_and_global_rejects_an_explicit_path() {
        for flags in [&["--global", "--local"][..], &["--global", "--path", "app"]] {
            let error = match skill_args(flags) {
                Ok(_) => panic!("conflicting options must fail"),
                Err(error) => error,
            };
            assert_eq!(error.kind(), clap::error::ErrorKind::ArgumentConflict);
        }
    }

    #[test]
    fn check_and_fix_preserve_interleaved_lints_and_keep_f_for_features() {
        for command in ["check", "fix"] {
            let matches = Cli::command()
                .try_get_matches_from([
                    "cargo",
                    "tessera",
                    command,
                    "-Awarnings",
                    "-F",
                    "demo",
                    "--deny=tessera::redundant_render_slot",
                    "--allow",
                    "tessera::all",
                    "-Wwarnings",
                    "--forbid",
                    "tessera::redundant_render_slot",
                    "--force-warn=tessera::all",
                    "--cap-lints",
                    "warn",
                ])
                .unwrap();
            let Cli {
                command: Commands::Tessera(args),
            } = Cli::from_arg_matches(&matches).unwrap();
            let args = match args.command {
                TesseraCommands::Check { args, .. } | TesseraCommands::Fix { args, .. } => args,
                _ => panic!("expected check or fix"),
            };
            let options = args.lints.options(&matches);
            let lint = "tessera::redundant_render_slot".parse().unwrap();
            assert_eq!(args.features, ["demo"]);
            assert_eq!(options.cap, Some(call_guard::Level::Warn));
            assert_eq!(
                options.overrides,
                [
                    (call_guard::LintTarget::Warnings, call_guard::Level::Allow),
                    (lint, call_guard::Level::Deny),
                    (call_guard::LintTarget::All, call_guard::Level::Allow),
                    (call_guard::LintTarget::Warnings, call_guard::Level::Warn),
                    (lint, call_guard::Level::Forbid),
                    (call_guard::LintTarget::All, call_guard::Level::ForceWarn),
                ]
            );
        }
    }

    #[test]
    fn check_validates_lint_targets_and_caps_without_exposing_them_on_other_commands() {
        for args in [
            ["cargo", "tessera", "check", "-A", "T0001"],
            ["cargo", "tessera", "check", "-W", "tessera::missing"],
            ["cargo", "tessera", "check", "--cap-lints", "force-warn"],
            ["cargo", "tessera", "check", "--force-warn", "warnings"],
        ] {
            assert_eq!(
                Cli::command()
                    .try_get_matches_from(args)
                    .unwrap_err()
                    .kind(),
                clap::error::ErrorKind::ValueValidation
            );
        }
        let error = Cli::command()
            .try_get_matches_from(["cargo", "tessera", "build", "-A", "warnings"])
            .unwrap_err();
        assert_eq!(error.kind(), clap::error::ErrorKind::UnknownArgument);
    }
}
