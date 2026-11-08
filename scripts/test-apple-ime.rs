#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! anyhow = "1.0.104"
//! plist = "1.10.3"
//! serde_json = "1.0.154"
//! tempfile = "3.27.0"
//! wait-timeout = "0.2.1"
//! ```

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{Context, Result, ensure};
use serde_json::Value;

mod process;

fn run(command: &mut Command, seconds: u64) -> Result<String> {
    process::run(command, Duration::from_secs(seconds))
}

fn build(root: &Path, target: Option<&str>) -> Result<PathBuf> {
    let mut command = Command::new("cargo");
    command
        .args(["build", "--manifest-path"])
        .arg(root.join("crates/tessera-winit/tests/apple-ime/Cargo.toml"))
        .arg("--message-format=json")
        .env("CARGO_TARGET_DIR", root.join("target"));
    if let Some(target) = target {
        command.args(["--target", target]);
    }
    let output = run(&mut command, 900)?;
    output
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<std::result::Result<Vec<_>, _>>()?
        .iter()
        .rev()
        .find(|message| {
            message["reason"] == "compiler-artifact" && message["executable"].is_string()
        })
        .and_then(|message| message["executable"].as_str())
        .map(PathBuf::from)
        .context("Build produced no executable")
}

fn check(command: &mut Command) -> Result<()> {
    let output = run(command, 90)?;
    println!("{output}");
    ensure!(
        output.contains("NATIVE_IME_PASS"),
        "Native IME edits did not reach the application"
    );
    Ok(())
}

fn main() -> Result<()> {
    ensure!(
        cfg!(target_os = "macos"),
        "Apple IME acceptance requires macOS and Xcode"
    );
    let root = Path::new(env!("RUST_SCRIPT_PATH"))
        .parent()
        .context("Missing script directory")?
        .parent()
        .context("Missing repository root")?;
    check(&mut Command::new(build(root, None)?))?;
    let target = if std::env::consts::ARCH == "aarch64" {
        "aarch64-apple-ios-sim"
    } else {
        "x86_64-apple-ios"
    };
    run(Command::new("rustup").args(["target", "add", target]), 120)?;
    let executable = build(root, Some(target))?;
    let devices: Value = serde_json::from_str(&run(
        Command::new("xcrun").args(["simctl", "list", "devices", "available", "--json"]),
        120,
    )?)?;
    let phone = devices["devices"]
        .as_object()
        .context("Missing simulator devices")?
        .iter()
        .filter(|(runtime, _)| runtime.contains(".iOS-"))
        .flat_map(|(_, group)| group.as_array().into_iter().flatten())
        .find(|device| {
            device["name"]
                .as_str()
                .is_some_and(|name| name.starts_with("iPhone"))
        })
        .context("No available iPhone simulator")?;
    let udid = phone["udid"].as_str().context("Missing simulator UDID")?;
    let booted = phone["state"] == "Booted";
    let result = (|| {
        if !booted {
            run(Command::new("xcrun").args(["simctl", "boot", udid]), 120)?;
        }
        run(
            Command::new("xcrun").args(["simctl", "bootstatus", udid, "-b"]),
            600,
        )?;
        if let Some(bundle) = std::env::args_os().nth(1) {
            let output = run(
                Command::new("rust-script")
                    .arg(root.join("scripts/check-mobile.rs"))
                    .args(["--ios", udid])
                    .arg(bundle),
                2400,
            )?;
            println!("{output}");
        }
        let directory = tempfile::tempdir()?;
        let app = directory.path().join("ImeTest.app");
        fs::create_dir(&app)?;
        fs::copy(&executable, app.join("ime-test"))?;
        let mut info = plist::Dictionary::new();
        for (key, value) in [
            ("CFBundleIdentifier", "dev.tessera.ime-test"),
            ("CFBundleExecutable", "ime-test"),
            ("CFBundleName", "Tessera IME Test"),
            ("CFBundlePackageType", "APPL"),
            ("CFBundleVersion", "1"),
            ("CFBundleShortVersionString", "1.0"),
            ("MinimumOSVersion", "13.0"),
        ] {
            info.insert(key.into(), value.into());
        }
        info.insert(
            "UIDeviceFamily".into(),
            plist::Value::Array(vec![1.into(), 2.into()]),
        );
        plist::Value::Dictionary(info).to_file_xml(app.join("Info.plist"))?;
        run(
            Command::new("xcrun")
                .args(["simctl", "install", udid])
                .arg(app),
            120,
        )?;
        check(Command::new("xcrun").args([
            "simctl",
            "launch",
            "--console-pty",
            "--terminate-running-process",
            udid,
            "dev.tessera.ime-test",
        ]))
    })();
    if !booted {
        if let Err(error) = run(
            Command::new("xcrun").args(["simctl", "shutdown", udid]),
            120,
        ) {
            eprintln!("Simulator shutdown failed: {error:#}");
        }
    }
    result
}
