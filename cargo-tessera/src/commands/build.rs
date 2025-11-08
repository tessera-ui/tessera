use std::process::Command;

use anyhow::{Context, Result, bail};
use clap::ValueEnum;
use owo_colors::colored::*;

mod android;

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum BuildPlatform {
    Native,
    Android,
}

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

#[derive(Debug, Clone)]
pub struct BuildOptions {
    pub release: bool,
    pub target: Option<String>,
    pub platform: BuildPlatform,
    pub android_arch: Option<String>,
    pub android_package: Option<String>,
    pub android_format: Option<AndroidFormat>,
}

pub fn execute(opts: BuildOptions) -> Result<()> {
    match opts.platform {
        BuildPlatform::Native => run_native(opts.release, opts.target.as_deref()),
        BuildPlatform::Android => android::execute(opts),
    }
}

fn run_native(release: bool, target: Option<&str>) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
        println!("Building in {} mode", "release".bright_green());
    }

    if let Some(target) = target {
        cmd.arg("--target").arg(target);
        println!("Target platform: {}", target.bright_yellow());
    }

    let status = cmd.status().context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Build failed");
    }

    println!("\n{} Build completed successfully!", "✅".green());

    if release {
        let binary_path = if let Some(target) = target {
            format!("target/{}/release/", target)
        } else {
            "target/release/".to_string()
        };
        println!("Binary location: {}", binary_path.cyan());
    }

    Ok(())
}
