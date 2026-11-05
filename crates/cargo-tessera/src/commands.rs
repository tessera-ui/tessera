pub mod android;
mod assets;
pub mod build;
pub mod check;
pub mod dev;
pub mod headless;
pub mod new;
pub mod package;
pub mod plugin;
pub mod profiling;
pub mod skill;
pub mod web;

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Result, anyhow};
use inquire::{Text, error::CustomUserError, validator::Validation};

/// Find the directory of a package by name in the workspace
pub fn find_package_dir(package_name: &str) -> Result<PathBuf> {
    let cargo_toml = fs::read_to_string("Cargo.toml")?;
    let root_toml: toml::Value = toml::from_str(&cargo_toml)?;

    // Check if current directory is the target package
    if let Some(name) = root_toml
        .get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        && name == package_name
    {
        return Ok(PathBuf::from("."));
    }

    // Search workspace members
    let members = root_toml
        .get("workspace")
        .and_then(|w| w.get("members"))
        .and_then(|m| m.as_array())
        .ok_or_else(|| anyhow!("No workspace members found"))?;

    for member in members.iter().filter_map(|m| m.as_str()) {
        let member_path = PathBuf::from(member);
        let member_cargo = member_path.join("Cargo.toml");

        let Ok(content) = fs::read_to_string(&member_cargo) else {
            continue;
        };
        let Ok(member_toml) = toml::from_str::<toml::Value>(&content) else {
            continue;
        };

        let Some(name) = member_toml
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
        else {
            continue;
        };

        if name == package_name {
            return Ok(member_path);
        }
    }

    Err(anyhow!("Package '{}' not found in workspace", package_name))
}

pub fn prompt_name(kind: &'static str, placeholder: &str) -> Result<String> {
    let validator = move |input: &str| -> Result<Validation, CustomUserError> {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Ok(Validation::Invalid(
                format!("{kind} name cannot be empty").into(),
            ));
        }

        if !trimmed
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        {
            return Ok(Validation::Invalid(
                "Only lowercase letters, digits, '-' and '_' are allowed".into(),
            ));
        }

        if !trimmed
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_lowercase())
        {
            return Ok(Validation::Invalid(
                format!("{kind} name must start with a lowercase letter").into(),
            ));
        }

        if Path::new(trimmed).exists() {
            return Ok(Validation::Invalid(
                format!("Directory '{trimmed}' already exists").into(),
            ));
        }

        Ok(Validation::Valid)
    };

    let name = Text::new(&format!("{kind} name"))
        .with_help_message("lowercase, numbers, '-' or '_', must start with a letter")
        .with_placeholder(placeholder)
        .with_validator(validator)
        .prompt()?;

    Ok(name.trim().to_string())
}

pub(super) fn build_tessera_ui_features(
    profiling_output: Option<&Path>,
    profiling_ipc: Option<&str>,
    debug_dirty_overlay: bool,
) -> Vec<String> {
    let mut features = Vec::new();
    if profiling_output.is_some() || profiling_ipc.is_some() {
        features.push("tessera-ui/profiling".to_string());
    }
    if debug_dirty_overlay {
        features.push("tessera-ui/debug-dirty-overlay".to_string());
    }
    features
}

pub(super) fn configure_tessera_ui_features(
    cmd: &mut Command,
    features: &[String],
    profiling_output: Option<&Path>,
    profiling_ipc: Option<&str>,
) {
    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }
    if let Some(output_path) = profiling_output {
        cmd.env("TESSERA_PROFILING_OUTPUT", output_path);
    }
    if let Some(name) = profiling_ipc {
        cmd.env("TESSERA_PROFILING_IPC", name);
    }
}
