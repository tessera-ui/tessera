use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, ensure};
use clap::Args;
use plist::{Dictionary, Value};
use serde::Deserialize;
use tessera_build::{AssetBackend, asset_files};

use crate::output;

use super::{app_assets, application::Application};

#[derive(Args)]
pub struct BuildOptions {
    /// Build optimized binaries
    #[arg(short, long)]
    release: bool,
    /// Build a device IPA instead of a simulator application
    #[arg(long)]
    device: bool,
    #[arg(short, long)]
    package: Option<String>,
    #[arg(long)]
    bin: Option<String>,
    /// Write frame profiling JSONL to this application path
    #[arg(long)]
    profiling_output: Option<PathBuf>,
    /// Code-signing certificate name; defaults to ad hoc signing
    #[arg(long)]
    signing_identity: Option<String>,
    /// Embed this provisioning profile for device installation
    #[arg(long)]
    provisioning_profile: Option<PathBuf>,
}

#[derive(Default, Deserialize)]
#[serde(default, rename_all = "kebab-case", deny_unknown_fields)]
struct Config {
    info_plist: Option<PathBuf>,
    asset_catalog: Option<PathBuf>,
    entitlements: Option<PathBuf>,
    resources: Option<PathBuf>,
}

pub(super) fn deployment_target() -> String {
    std::env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "15.0".into())
}

fn run(command: &mut Command) -> Result<()> {
    ensure!(
        command
            .status()
            .with_context(|| format!("Failed to run {command:?}"))?
            .success(),
        "Command failed: {command:?}"
    );
    Ok(())
}

fn copy_resources(source: &Path, destination: &Path) -> Result<()> {
    for file in asset_files(source)? {
        let target = destination.join(file.strip_prefix(source)?);
        fs::create_dir_all(target.parent().context("Missing resource directory")?)?;
        fs::copy(file, target)?;
    }
    Ok(())
}

pub fn build(options: BuildOptions) -> Result<()> {
    ensure!(
        cfg!(target_os = "macos"),
        "iOS packaging requires macOS and Xcode"
    );
    let app = Application::new(options.package.as_deref(), options.bin.as_deref())?;
    let config: Config = serde_json::from_value(
        app.package
            .metadata
            .get("tessera")
            .and_then(|value| value.get("ios"))
            .cloned()
            .unwrap_or_else(|| serde_json::json!({})),
    )?;
    let project = app
        .package
        .manifest_path
        .parent()
        .context("Missing package directory")?
        .as_std_path();
    let target = if options.device {
        "aarch64-apple-ios"
    } else if std::env::consts::ARCH == "aarch64" {
        "aarch64-apple-ios-sim"
    } else {
        "x86_64-apple-ios"
    };
    let sdk = if options.device {
        "iphoneos"
    } else {
        "iphonesimulator"
    };
    run(Command::new("xcrun").args(["--sdk", sdk, "--show-sdk-path"]))?;
    run(Command::new("rustup").args(["target", "add", target]))?;
    let executable = app.build(
        options.release,
        Some(target),
        options.profiling_output.as_deref(),
        AssetBackend::Platform,
    )?;
    let output = app.output_dir().join("ios").join(target);
    let root = if options.device {
        output.join(&app.binary).join("Payload")
    } else {
        output.clone()
    };
    let bundle = root.join(format!("{}.app", app.binary));
    if bundle.exists() {
        fs::remove_dir_all(&bundle)?;
    }
    fs::create_dir_all(&bundle)?;
    fs::copy(&executable, bundle.join(&app.binary))?;
    let resources = executable.with_extension("resources");
    if resources.is_dir() {
        copy_resources(&resources, &bundle)?;
    }
    if let Some(resources) = config.resources {
        copy_resources(&project.join(resources), &bundle)?;
    }
    let version = format!(
        "{}.{}.{}",
        app.package.version.major, app.package.version.minor, app.package.version.patch
    );
    let minimum_version = deployment_target();
    let mut info = Dictionary::new();
    for (key, value) in [
        ("CFBundlePackageType", "APPL"),
        ("MinimumOSVersion", minimum_version.as_str()),
    ] {
        info.insert(key.into(), value.into());
    }
    info.insert(
        "UIDeviceFamily".into(),
        Value::Array(vec![1.into(), 2.into()]),
    );
    info.insert(
        "CFBundleSupportedPlatforms".into(),
        Value::Array(vec![
            if options.device {
                "iPhoneOS"
            } else {
                "iPhoneSimulator"
            }
            .into(),
        ]),
    );
    info.insert("LSRequiresIPhoneOS".into(), true.into());
    info.insert("UILaunchScreen".into(), Dictionary::new().into());
    info.insert(
        "UISupportedInterfaceOrientations".into(),
        Value::Array(
            ["Portrait", "LandscapeLeft", "LandscapeRight"]
                .map(|orientation| format!("UIInterfaceOrientation{orientation}").into())
                .into(),
        ),
    );
    if let Some(path) = config.info_plist {
        info.extend(
            Value::from_file(project.join(path))?
                .into_dictionary()
                .context("Info.plist must be a dictionary")?,
        );
    }
    let generated_catalog = app_assets::ios(&app.identity, &output)?;
    let catalogs = config
        .asset_catalog
        .into_iter()
        .map(|catalog| project.join(catalog))
        .chain(generated_catalog)
        .collect::<Vec<_>>();
    if !catalogs.is_empty() {
        let partial = output.join(format!("{}-assets.plist", app.binary));
        run(Command::new("xcrun")
            .arg("actool")
            .args(catalogs)
            .arg("--compile")
            .arg(&bundle)
            .args([
                "--platform",
                sdk,
                "--minimum-deployment-target",
                &minimum_version,
                "--app-icon",
                "AppIcon",
                "--target-device",
                "iphone",
                "--target-device",
                "ipad",
                "--output-partial-info-plist",
            ])
            .arg(&partial))?;
        info.extend(
            Value::from_file(partial)?
                .into_dictionary()
                .context("Invalid asset catalog metadata")?,
        );
    }
    for (key, value) in [
        ("CFBundleName", app.identity.name.as_str()),
        ("CFBundleDisplayName", app.identity.name.as_str()),
        ("CFBundleShortVersionString", version.as_str()),
        ("CFBundleVersion", version.as_str()),
        ("CFBundleExecutable", app.binary.as_str()),
        ("CFBundleIdentifier", app.identity.identifier.as_str()),
    ] {
        info.insert(key.into(), value.into());
    }
    Value::Dictionary(info).to_file_xml(bundle.join("Info.plist"))?;
    let provisioned = options.provisioning_profile.is_some();
    if let Some(profile) = options.provisioning_profile {
        fs::copy(profile, bundle.join("embedded.mobileprovision"))?;
    }
    let mut sign = Command::new("codesign");
    sign.args([
        "--force",
        "--sign",
        options.signing_identity.as_deref().unwrap_or("-"),
    ]);
    if let Some(path) = config.entitlements {
        sign.arg("--entitlements").arg(project.join(path));
    }
    run(sign.arg(&bundle))?;
    if options.device {
        let archive = output.join(format!("{}.ipa", app.binary));
        run(Command::new("ditto")
            .args(["-c", "-k", "--keepParent"])
            .arg(root)
            .arg(&archive))?;
        if options.signing_identity.is_none() || !provisioned {
            output::note(
                "Device installation requires a signing identity and matching provisioning profile.",
            );
        }
        output::status("Finished", archive.display().to_string());
    } else {
        output::status("Finished", bundle.display().to_string());
    }
    Ok(())
}
