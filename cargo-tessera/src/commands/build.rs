use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::output;

pub fn execute(release: bool, target: Option<&str>, package: Option<&str>) -> Result<()> {
    let mut details = Vec::new();
    if release {
        details.push("release".to_string());
    }
    if let Some(target) = target {
        details.push(format!("target {target}"));
    }
    if let Some(package) = package {
        details.push(format!("package {package}"));
    }
    let message = if details.is_empty() {
        "project".to_string()
    } else {
        format!("project ({})", details.join(", "))
    };
    output::status("Building", message);

    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
    }

    if let Some(target) = target {
        cmd.arg("--target").arg(target);
    }

    if let Some(package) = package {
        cmd.arg("-p").arg(package);
    }

    let status = cmd.status().context("Failed to run cargo build")?;

    if !status.success() {
        bail!("Build failed");
    }

    if release {
        let binary_path = if let Some(target) = target {
            format!("target/{}/release/", target)
        } else {
            "target/release/".to_string()
        };
        output::status("Binary", binary_path);
    }

    Ok(())
}
