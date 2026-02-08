use std::{path::Path, process::Command};

use anyhow::{Context, Result, bail};

use crate::output;

pub fn execute(
    release: bool,
    target: Option<&str>,
    package: Option<&str>,
    profiling_output: Option<&Path>,
) -> Result<()> {
    if profiling_output.is_some() && target_is_android(target) {
        bail!("--profiling-output is not supported for Android targets");
    }

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
    if let Some(path) = profiling_output {
        details.push(format!("profiling {}", path.display()));
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
    if let Some(path) = profiling_output {
        enable_profiling(&mut cmd, path);
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

fn enable_profiling(cmd: &mut Command, output_path: &Path) {
    cmd.arg("--features").arg("tessera-ui/profiling");
    cmd.env("TESSERA_PROFILING_OUTPUT", output_path);
}

fn target_is_android(target: Option<&str>) -> bool {
    target
        .map(|triple| triple.to_ascii_lowercase().contains("android"))
        .unwrap_or(false)
}
