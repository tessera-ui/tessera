use std::{
    fs, io,
    process::{Command, ExitStatus, Stdio},
};

use anyhow::{Context, Result, anyhow, bail};
use owo_colors::colored::*;
use serde::Deserialize;

use super::{AndroidFormat, BuildOptions, BuildPlatform};

const DEFAULT_ARCH: &str = "arm64";

pub fn execute(opts: BuildOptions) -> Result<()> {
    if opts.platform != BuildPlatform::Android {
        bail!("Android builder invoked with non-android platform");
    }

    if opts.target.is_some() {
        bail!("`--target` is not supported for Android builds. Use `--android-arch` instead.");
    }

    ensure_x_available()?;

    let manifest = Manifest::load()?;
    let manifest_package = manifest.package_name();
    let manifest_cfg = manifest.android().unwrap_or_default();

    let package = opts
        .android_package
        .clone()
        .or_else(|| manifest_cfg.package.clone())
        .or(manifest_package)
        .ok_or_else(|| {
            anyhow!(
                "Unable to determine package name. Provide --android-package or set \
package.metadata.tessera.android.package in Cargo.toml"
            )
        })?;

    let arch = opts
        .android_arch
        .clone()
        .or_else(|| manifest_cfg.arch.clone())
        .unwrap_or_else(|| DEFAULT_ARCH.to_string());

    let format = opts
        .android_format
        .or_else(|| {
            manifest_cfg
                .format
                .as_deref()
                .and_then(AndroidFormat::from_config)
        })
        .unwrap_or(AndroidFormat::Apk);

    let skip_doctor = opts.android_skip_doctor || manifest_cfg.skip_doctor.unwrap_or(false);

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

    if skip_doctor {
        println!("{}", "Skipping `x doctor` (requested)".dimmed());
    } else {
        run_x_doctor()?;
    }

    run_x_build(&package, &arch, format, opts.release)?;

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

fn ensure_x_available() -> Result<()> {
    match Command::new("x")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == io::ErrorKind::NotFound => bail!(
            "`x` (xbuild) was not found. Install it with `cargo install xbuild --features vendored` \
or open `nix develop .#android`."
        ),
        Err(err) => Err(err.into()),
    }
}

fn run_x_doctor() -> Result<()> {
    println!(
        "{}",
        "Running `x doctor` to validate Android toolchain...".dimmed()
    );
    let status = Command::new("x")
        .arg("doctor")
        .status()
        .context("Failed to run `x doctor`")?;

    if !status.success() {
        bail!("`x doctor` reported issues. Resolve them or pass --android-skip-doctor.");
    }

    Ok(())
}

fn run_x_build(package: &str, arch: &str, format: AndroidFormat, release: bool) -> Result<()> {
    let mut cmd = Command::new("x");
    cmd.arg("build")
        .arg("-p")
        .arg(package)
        .arg("--platform")
        .arg("android")
        .arg("--arch")
        .arg(arch)
        .arg("--format")
        .arg(format.as_str());

    if release {
        cmd.arg("--release");
    }

    let status = cmd.status().context("Failed to run `x build`")?;
    ensure_success(status, "`x build` failed")?;
    Ok(())
}

fn ensure_success(status: ExitStatus, message: &str) -> Result<()> {
    if status.success() {
        Ok(())
    } else if let Some(code) = status.code() {
        bail!("{message} (exit code {code})");
    } else {
        bail!("{message}");
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
    skip_doctor: Option<bool>,
}
