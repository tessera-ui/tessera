use std::{fs, io, process::Command};

use anyhow::{Context, Result, anyhow, bail};
use clap::ValueEnum;
use owo_colors::colored::*;
use serde::Deserialize;

const DEFAULT_ARCH: &str = "arm64";

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum AndroidFormat {
    Apk,
    Aab,
}

impl AndroidFormat {
    fn as_str(self) -> &'static str {
        match self {
            AndroidFormat::Apk => "apk",
            AndroidFormat::Aab => "aab",
        }
    }

    fn from_config(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "apk" => Some(AndroidFormat::Apk),
            "aab" => Some(AndroidFormat::Aab),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct BuildOptions {
    pub release: bool,
    pub arch: Option<String>,
    pub package: Option<String>,
    pub format: Option<AndroidFormat>,
}

#[derive(Debug)]
pub struct DevOptions {
    pub release: bool,
    pub arch: Option<String>,
    pub package: Option<String>,
    pub device: Option<String>,
}

pub fn build(opts: BuildOptions) -> Result<()> {
    let manifest = Manifest::load()?;
    let manifest_package = manifest.package_name();
    let manifest_cfg = manifest.android().unwrap_or_default();

    let package = opts
        .package
        .clone()
        .or_else(|| manifest_cfg.package.clone())
        .or(manifest_package)
        .ok_or_else(|| {
            anyhow!(
                "Unable to determine package name. Provide --package or set \
package.metadata.tessera.android.package in Cargo.toml"
            )
        })?;

    let arch = opts
        .arch
        .clone()
        .or_else(|| manifest_cfg.arch.clone())
        .unwrap_or_else(|| DEFAULT_ARCH.to_string());

    let format = opts
        .format
        .or_else(|| {
            manifest_cfg
                .format
                .as_deref()
                .and_then(AndroidFormat::from_config)
        })
        .unwrap_or(AndroidFormat::Apk);

    println!(
        "{}",
        format!(
            "🤖 Building Android artifact ({}, {}, release: {})",
            package,
            arch,
            if opts.release { "yes" } else { "no" }
        )
        .bright_cyan()
    );

    run_x_command("build", &package, Some(&arch), opts.release, format, None)?;

    println!("\n{} Android build completed!", "✅".green());
    println!("Package: {}", package.bright_green());
    println!(
        "Format : {} ({})",
        format.as_str().bright_yellow(),
        "produced by x build"
    );
    println!(
        "{}",
        "Tip: use `x build -h` for more Android packaging flags.".dimmed()
    );

    Ok(())
}

pub fn dev(opts: DevOptions) -> Result<()> {
    let manifest = Manifest::load()?;
    let manifest_package = manifest.package_name();
    let manifest_cfg = manifest.android().unwrap_or_default();

    let package = opts
        .package
        .clone()
        .or_else(|| manifest_cfg.package.clone())
        .or(manifest_package)
        .ok_or_else(|| {
            anyhow!(
                "Unable to determine package name. Provide --package or set \
package.metadata.tessera.android.package in Cargo.toml"
            )
        })?;

    let arch = opts
        .arch
        .clone()
        .or_else(|| manifest_cfg.arch.clone())
        .unwrap_or_else(|| DEFAULT_ARCH.to_string());

    println!(
        "{}",
        format!(
            "📱 Running Tessera app on Android ({}, arch: {}, release: {})",
            package,
            arch,
            if opts.release { "yes" } else { "no" }
        )
        .bright_cyan()
    );

    run_x_command(
        "run",
        &package,
        Some(&arch),
        opts.release,
        AndroidFormat::Apk,
        opts.device.as_deref(),
    )?;

    println!(
        "{}",
        "✅ App launched via `x run`. Use Ctrl+C to stop or rerun the command after code changes."
            .green()
    );

    Ok(())
}

fn run_x_command(
    subcommand: &str,
    package: &str,
    arch: Option<&str>,
    release: bool,
    format: AndroidFormat,
    device: Option<&str>,
) -> Result<()> {
    let mut cmd = Command::new("x");
    cmd.arg(subcommand)
        .arg("-p")
        .arg(package)
        .arg("--platform")
        .arg("android")
        .arg("--format")
        .arg(format.as_str());

    if let Some(arch) = arch {
        cmd.arg("--arch").arg(arch);
    }

    if let Some(device) = device {
        cmd.arg("--device").arg(device);
    }

    if release {
        cmd.arg("--release");
    }

    let status = match cmd.status() {
        Ok(status) => status,
        Err(err) if err.kind() == io::ErrorKind::NotFound => bail!(
            "`x` (xbuild) was not found. Install it with `cargo install xbuild --features vendored` or open `nix develop .#android`."
        ),
        Err(err) => return Err(err).context(format!("Failed to run `x {subcommand}`")),
    };

    if status.success() {
        Ok(())
    } else if let Some(code) = status.code() {
        bail!("`x {subcommand}` failed (exit code {code}). Run `x doctor` for diagnostics.");
    } else {
        bail!("`x {subcommand}` terminated unexpectedly. Run `x doctor` for diagnostics.");
    }
}

#[derive(Debug, Deserialize)]
struct Manifest {
    package: Option<PackageSection>,
}

impl Manifest {
    fn load() -> Result<Self> {
        let contents = fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;
        toml::from_str(&contents).context("Failed to parse Cargo.toml")
    }

    fn package_name(&self) -> Option<String> {
        self.package.as_ref().and_then(|p| p.name.clone())
    }

    fn android(&self) -> Option<AndroidConfig> {
        self.package
            .as_ref()
            .and_then(|p| p.metadata.as_ref())
            .and_then(|m| m.tessera.as_ref())
            .and_then(|t| t.android.clone())
    }
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    name: Option<String>,
    metadata: Option<MetadataSection>,
}

#[derive(Debug, Deserialize)]
struct MetadataSection {
    tessera: Option<TesseraMetadata>,
}

#[derive(Debug, Deserialize)]
struct TesseraMetadata {
    android: Option<AndroidConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct AndroidConfig {
    package: Option<String>,
    arch: Option<String>,
    format: Option<String>,
}
