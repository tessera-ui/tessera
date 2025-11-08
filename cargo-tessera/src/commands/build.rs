use std::process::Command;

use anyhow::{Context, Result};
use colored::*;

pub fn execute(release: bool, target: Option<&str>) -> Result<()> {
    println!("{}", "🔨 Building project...".bright_cyan());

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
        anyhow::bail!("Build failed");
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
