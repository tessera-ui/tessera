use std::{env, fs, process::Command};

use anyhow::{Context, Result, bail, ensure};
use heck::ToLowerCamelCase;
use serde_json::{Value, json};
use tessera_build::AssetBackend;

use crate::output;

use super::{app_assets, application::Application};

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
    let executable = app.build(release, target, None, AssetBackend::Platform)?;
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
    let host;
    let target = match target.or(inferred_target) {
        Some(target) => target,
        None => {
            let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
            let version = Command::new(rustc)
                .arg("-vV")
                .output()
                .context("Failed to run rustc -vV")?;
            ensure!(
                version.status.success(),
                "rustc -vV failed: {}",
                String::from_utf8_lossy(&version.stderr).trim()
            );
            host = String::from_utf8(version.stdout)?;
            host.lines()
                .find_map(|line| line.strip_prefix("host: "))
                .context("Missing host target in rustc -vV output")?
        }
    };
    let out = app.output_dir();
    let linux = target.contains("linux");
    let (binary, binary_dir) = if linux {
        let binary = app.identity.linux_launcher_name();
        let directory = out.join("bin");
        fs::create_dir_all(&directory)?;
        fs::copy(&executable, directory.join(&binary))?;
        (binary, directory)
    } else {
        (
            app.binary.clone(),
            executable
                .parent()
                .context("Missing binary directory")?
                .to_path_buf(),
        )
    };
    let macos = target.ends_with("-apple-darwin");
    if let Some(icons) = app_assets::desktop(&app.identity, &out.join("icons"), macos)? {
        if target.contains("windows") {
            let ico = icons
                .iter()
                .find(|path| path.extension().is_some_and(|ext| ext == "ico"))
                .context("Missing generated Windows icon")?;
            set_default(&mut app.config, "nsis", "installerIcon", json!(ico))?;
        }
        app.config["icons"] = json!(icons);
    }
    let config = &mut app.config;
    config["binaries"] = json!([{ "path": binary, "main": true }]);
    config["binariesDir"] = json!(binary_dir);
    config["outDir"] = json!(out);
    config["targetTriple"] = json!(target);
    if linux {
        set_default(
            config,
            "deb",
            "packageName",
            json!(app.identity.identifier.to_ascii_lowercase()),
        )?;
    }
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

fn set_default(config: &mut Value, section: &str, key: &str, value: Value) -> Result<()> {
    if config[section].is_null() {
        config[section] = json!({});
    }
    let table = config[section]
        .as_object_mut()
        .with_context(|| format!("packager.{section} must be a table"))?;
    if !table
        .keys()
        .any(|existing| existing.to_lower_camel_case() == key)
    {
        table.insert(key.into(), value);
    }
    Ok(())
}
