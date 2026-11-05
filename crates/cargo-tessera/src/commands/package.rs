use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use cargo_metadata::MetadataCommand;
use heck::ToLowerCamelCase;
use serde_json::json;
use tessera_build::AssetBackend;

use crate::output;

use super::build;

pub fn execute(
    release: bool,
    target: Option<&str>,
    package: Option<&str>,
    binary: Option<&str>,
    formats: &[String],
) -> Result<()> {
    ensure!(
        Command::new("cargo")
            .args(["packager", "--version"])
            .output()
            .context("Install cargo-packager with `cargo install cargo-packager --locked`")?
            .status
            .success(),
        "Install cargo-packager with `cargo install cargo-packager --locked`"
    );
    let metadata = MetadataCommand::new().no_deps().exec()?;
    let package = if let Some(name) = package {
        metadata
            .packages
            .iter()
            .find(|package| package.name.as_str() == name)
    } else {
        metadata.root_package()
    }
    .context("Select an application package with --package")?;
    let mut config = json!({
        "productName": package.name.as_str(), "version": package.version.to_string(),
        "identifier": format!("com.tessera.{}", package.name.replace('_', "-")),
        "description": package.description, "authors": package.authors, "homepage": package.homepage,
    });
    if let Some(custom) = package
        .metadata
        .get("packager")
        .and_then(|value| value.as_object())
    {
        config
            .as_object_mut()
            .context("Invalid package configuration")?
            .extend(
                custom
                    .iter()
                    .map(|(key, value)| (key.to_lower_camel_case(), value.clone())),
            );
    }
    let configured_target: Option<String> = config
        .get("targetTriple")
        .map(|value| serde_json::from_value(value.clone()))
        .transpose()?
        .flatten();
    let target = target.or(configured_target.as_deref());
    let bins = package
        .targets
        .iter()
        .filter(|target| target.is_bin())
        .collect::<Vec<_>>();
    let name = binary
        .or(package.default_run.as_deref())
        .or_else(|| (bins.len() == 1).then(|| bins[0].name.as_str()))
        .context("Select an application executable with --bin")?;
    ensure!(
        bins.iter().any(|target| target.name == name),
        "Unknown executable `{name}`"
    );
    if target.is_some_and(|target| {
        target.contains("android") || target.contains("ios") || target.contains("wasm")
    }) {
        bail!("Use the Android, iOS or Web build command for this target");
    }
    let binaries = build::execute(
        release,
        target,
        Some(package.name.as_str()),
        Some(name),
        None,
        false,
        Some(AssetBackend::Platform),
    )?;
    let executable = binaries
        .iter()
        .find(|path| path.file_stem().is_some_and(|stem| stem == name))
        .context("Cargo did not produce the selected executable")?;
    let out = metadata
        .target_directory
        .join("packages")
        .join(package.name.as_str());
    config["binaries"] = json!([{ "path": name, "main": true }]);
    config["binariesDir"] = json!(executable.parent());
    config["outDir"] = json!(out);
    let inferred_target = executable
        .strip_prefix(metadata.target_directory.as_std_path())
        .ok()
        .and_then(|path| {
            let mut parts = path.components();
            let target = parts.next()?;
            let profile = parts.next()?;
            (profile.as_os_str() == "debug" || profile.as_os_str() == "release")
                .then(|| target.as_os_str().to_str())
                .flatten()
        });
    config["targetTriple"] = json!(target.or(inferred_target));
    let assets = executable.with_extension("resources").join("tessera");
    if assets.is_dir() {
        if config
            .get("resources")
            .is_none_or(|resources| resources.is_null())
        {
            config["resources"] = json!([]);
        }
        config["resources"]
            .as_array_mut()
            .context("packager.resources must be an array")?
            .push(json!({ "src": assets, "target": "tessera" }));
    }
    let mut command = Command::new("cargo");
    command
        .arg("packager")
        .arg("--config")
        .arg(serde_json::to_string(&config)?)
        .current_dir(
            package
                .manifest_path
                .parent()
                .context("Missing package directory")?,
        );
    if !formats.is_empty() {
        command.arg("--formats").arg(formats.join(","));
    }
    output::status("Packaging", package.name.as_str());
    ensure!(command.status()?.success(), "Application packaging failed");
    output::status("Finished", out.as_str());
    Ok(())
}
