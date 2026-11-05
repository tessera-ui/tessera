use std::process::Command;

use anyhow::{Context, Result, bail, ensure};
use serde_json::json;

use crate::output;

use super::application::Application;

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
    let mut app = Application::new(package, binary)?;
    let config = &mut app.config;
    let configured_target: Option<String> = config
        .get("targetTriple")
        .map(|value| serde_json::from_value(value.clone()))
        .transpose()?
        .flatten();
    let target = target.or(configured_target.as_deref());
    if target.is_some_and(|target| {
        target.contains("android") || target.contains("ios") || target.contains("wasm")
    }) {
        bail!("Use the Android, iOS or Web build command for this target");
    }
    let executable = app.build(release, target)?;
    let out = app.output_dir();
    let config = &mut app.config;
    config["binaries"] = json!([{ "path": app.binary, "main": true }]);
    config["binariesDir"] = json!(executable.parent());
    config["outDir"] = json!(out);
    let inferred_target = executable
        .strip_prefix(app.target_dir.as_path())
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
            app.package
                .manifest_path
                .parent()
                .context("Missing package directory")?,
        );
    if !formats.is_empty() {
        command.arg("--formats").arg(formats.join(","));
    }
    output::status("Packaging", app.package.name.as_str());
    ensure!(command.status()?.success(), "Application packaging failed");
    output::status("Finished", out.display().to_string());
    Ok(())
}
