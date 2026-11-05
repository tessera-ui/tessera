use std::{path::Path, process::Command};

use anyhow::{Context, Result, bail};
use tessera_build::AssetBackend;

use crate::{call_guard, output};

use super::{build_tessera_ui_features, configure_tessera_ui_features};

pub fn execute(
    release: bool,
    target: Option<&str>,
    package: Option<&str>,
    profiling_output: Option<&Path>,
    debug_dirty_overlay: bool,
    asset_backend_override: Option<AssetBackend>,
) -> Result<()> {
    let feature_overrides = build_tessera_ui_features(profiling_output, None, debug_dirty_overlay);
    call_guard::run(call_guard::CheckOptions {
        features: call_guard::FeatureSelection::Selected {
            features: &feature_overrides,
            no_default_features: false,
        },
        release,
        ..call_guard::CheckOptions::new(package, target)
    })?;

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
    if debug_dirty_overlay {
        details.push("debug dirty overlay".to_string());
    }
    let message = if details.is_empty() {
        "project".to_string()
    } else {
        format!("project ({})", details.join(", "))
    };
    output::status("Building", message);
    let asset_backend = resolve_asset_backend(target, asset_backend_override)?;
    output::status("Assets", format!("backend `{}`", asset_backend.as_str()));

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
    configure_tessera_ui_features(&mut cmd, &feature_overrides, profiling_output, None);
    cmd.env("TESSERA_ASSET_BACKEND", asset_backend.as_str());

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

fn target_is_android(target: Option<&str>) -> bool {
    target.map(is_platform_backend_target).unwrap_or(false)
}

fn resolve_asset_backend(
    target: Option<&str>,
    asset_backend_override: Option<AssetBackend>,
) -> Result<AssetBackend> {
    let supports_platform = target.map(is_platform_backend_target).unwrap_or(false);

    if let Some(backend) = asset_backend_override {
        if backend == AssetBackend::Platform && !supports_platform {
            bail!(
                "Requested platform asset backend is not supported for target `{}`",
                target.unwrap_or("host-default")
            );
        }
        return Ok(backend);
    }

    if supports_platform {
        Ok(AssetBackend::Platform)
    } else {
        Ok(AssetBackend::Embed)
    }
}

fn is_platform_backend_target(target: &str) -> bool {
    target.to_ascii_lowercase().contains("android")
}
