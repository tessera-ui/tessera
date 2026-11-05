use std::{
    io::BufReader,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result, bail};
use cargo_metadata::{Message, MetadataCommand};
use tessera_build::AssetBackend;

use crate::{call_guard, output};

use super::{
    assets::{
        collect_platform_asset_candidates, collect_resolved_packages,
        collect_used_platform_asset_paths, sync_assets,
    },
    build_tessera_ui_features, configure_tessera_ui_features,
};

pub fn execute(
    release: bool,
    target: Option<&str>,
    package: Option<&str>,
    binary: Option<&str>,
    profiling_output: Option<&Path>,
    debug_dirty_overlay: bool,
    asset_backend_override: Option<AssetBackend>,
) -> Result<Vec<PathBuf>> {
    let android = target.is_some_and(|target| target.contains("android"));
    if profiling_output.is_some() && android {
        bail!("--profiling-output is not supported for Android targets");
    }
    let backend = asset_backend_override.unwrap_or(if android {
        AssetBackend::Platform
    } else {
        AssetBackend::Embed
    });
    let features = build_tessera_ui_features(profiling_output, None, debug_dirty_overlay);
    call_guard::run(call_guard::CheckOptions {
        features: call_guard::FeatureSelection::Selected {
            features: &features,
            no_default_features: false,
        },
        release,
        target_selection: call_guard::TargetSelection {
            bin: binary.map(str::to_owned),
            ..Default::default()
        },
        ..call_guard::CheckOptions::new(package, target)
    })?;
    output::status("Building", package.unwrap_or("project"));
    output::status("Assets", format!("backend `{}`", backend.as_str()));
    let mut command = Command::new("cargo");
    command.args(["build", "--message-format=json-render-diagnostics"]);
    if release {
        command.arg("--release");
    }
    if let Some(target) = target {
        command.args(["--target", target]);
    }
    if let Some(package) = package {
        command.args(["-p", package]);
    }
    if let Some(binary) = binary {
        command.args(["--bin", binary]);
    }
    configure_tessera_ui_features(&mut command, &features, profiling_output, None);
    command
        .env("TESSERA_ASSET_BACKEND", backend.as_str())
        .stdout(Stdio::piped());
    let mut child = command.spawn().context("Failed to run cargo build")?;
    let mut artifacts = Vec::new();
    for message in Message::parse_stream(BufReader::new(
        child.stdout.take().context("Missing Cargo output")?,
    )) {
        match message? {
            Message::CompilerArtifact(artifact) if artifact.target.is_bin() => {
                if artifact.executable.is_some() {
                    artifacts.push(artifact);
                }
            }
            Message::TextLine(line) => println!("{line}"),
            _ => {}
        }
    }
    if !child.wait()?.success() {
        bail!("Build failed");
    }
    if backend == AssetBackend::Platform
        && !android
        && !target.is_some_and(|target| target.contains("wasm"))
    {
        let metadata = MetadataCommand::new().no_deps().exec()?;
        for artifact in &artifacts {
            let binary = artifact
                .executable
                .as_ref()
                .context("Missing executable")?
                .as_std_path();
            let project = metadata
                .packages
                .iter()
                .find(|package| package.id == artifact.package_id)
                .context("Built package is absent from Cargo metadata")?
                .manifest_path
                .parent()
                .context("Missing package directory")?
                .to_path_buf()
                .into_std_path_buf();
            let packages = collect_resolved_packages(Path::new("."), Some(&project))?;
            let root = binary.with_extension("resources").join("tessera");
            let assets = collect_platform_asset_candidates(&packages, &root)?;
            let used = collect_used_platform_asset_paths(&[binary.to_path_buf()], &assets)?;
            let (_, bytes) = sync_assets(&root, &assets, Some(&used))?;
            output::status("Assets", format!("{bytes} bytes in {}", root.display()));
        }
    }
    Ok(artifacts
        .into_iter()
        .filter_map(|artifact| artifact.executable.map(|path| path.into_std_path_buf()))
        .collect())
}
