use std::process::Command;

use anyhow::Result;

use crate::{color_check, output};

#[allow(clippy::too_many_arguments)]
pub fn execute(
    package: Option<&str>,
    target: Option<&str>,
    release: bool,
    lib: bool,
    bins: bool,
    examples: bool,
    tests: bool,
    benches: bool,
    all_targets: bool,
    features: &[String],
    all_features: bool,
    no_default_features: bool,
    jobs: Option<&str>,
    workspace: bool,
    message_format: Option<&str>,
    verbose: bool,
    quiet: bool,
) -> Result<()> {
    let color_message_format = color_check::MessageFormat::from_cargo_arg(message_format);
    if let Err(err) = color_check::run(color_check::CheckOptions {
        package,
        target,
        target_selection: color_check::TargetSelection {
            lib,
            bins,
            examples,
            tests,
            benches,
            all_targets,
        },
        features: color_check::FeatureSelection::from_cargo_args(
            features,
            all_features,
            no_default_features,
        ),
        message_format: color_message_format,
    }) {
        if color_message_format.is_json() {
            std::process::exit(1);
        }
        return Err(err);
    }

    let mut details = Vec::new();
    if release {
        details.push("release".to_string());
    }
    if let Some(package) = package {
        details.push(format!("package {package}"));
    }
    if let Some(target) = target {
        details.push(format!("target {target}"));
    }
    let message = if details.is_empty() {
        "project".to_string()
    } else {
        format!("project ({})", details.join(", "))
    };
    if !color_message_format.is_json() {
        output::status("Checking", message);
    }

    let mut cmd = Command::new("cargo");
    cmd.arg("check");

    if release {
        cmd.arg("--release");
    }
    if let Some(package) = package {
        cmd.arg("--package").arg(package);
    }
    if let Some(target) = target {
        cmd.arg("--target").arg(target);
    }
    if lib {
        cmd.arg("--lib");
    }
    if bins {
        cmd.arg("--bins");
    }
    if examples {
        cmd.arg("--examples");
    }
    if tests {
        cmd.arg("--tests");
    }
    if benches {
        cmd.arg("--benches");
    }
    if all_targets {
        cmd.arg("--all-targets");
    }
    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }
    if all_features {
        cmd.arg("--all-features");
    }
    if no_default_features {
        cmd.arg("--no-default-features");
    }
    if let Some(jobs) = jobs {
        cmd.arg("--jobs").arg(jobs);
    }
    if workspace {
        cmd.arg("--workspace");
    }
    if let Some(message_format) = message_format {
        cmd.arg("--message-format").arg(message_format);
    }
    if verbose {
        cmd.arg("--verbose");
    }
    if quiet {
        cmd.arg("--quiet");
    }

    let status = cmd.status()?;
    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
