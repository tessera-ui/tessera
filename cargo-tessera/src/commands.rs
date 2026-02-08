pub mod android;
pub mod build;
pub mod dev;
pub mod new;
pub mod plugin;
pub mod profiling;

use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};

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
