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
    time::{Duration, Instant},
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

fn check_ios(
    udid: &str,
    output: &Path,
    timeout: Duration,
    mut execute: impl FnMut(&mut Command, Duration) -> Result<String>,
) -> Result<()> {
    fs::create_dir_all(output)?;
    let stdout = output.join("ios-ime.stdout.log");
    let stderr = output.join("ios-ime.stderr.log");
    fs::write(&stdout, [])?;
    fs::write(&stderr, [])?;
    let result = (|| {
        execute(
            Command::new("xcrun")
                .args(["simctl", "launch", "--terminate-running-process"])
                .arg(format!("--stdout={}", stdout.display()))
                .arg(format!("--stderr={}", stderr.display()))
                .args([udid, "dev.tessera.ime-test"]),
            Duration::from_secs(120),
        )?;
        let deadline = Instant::now() + timeout;
        loop {
            let bytes = fs::read(&stdout)?;
            let text = String::from_utf8_lossy(&bytes);
            if text.lines().any(|line| line.trim() == "NATIVE_IME_PASS") {
                print!("{text}");
                return Ok(());
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            ensure!(
                !remaining.is_zero(),
                "iOS native IME acceptance did not report NATIVE_IME_PASS within {timeout:?}\nstdout:\n{text}\nstderr:\n{}",
                String::from_utf8_lossy(&fs::read(&stderr)?),
            );
            std::thread::sleep(remaining.min(Duration::from_millis(100)));
        }
    })();
    if result.is_err() {
        for path in [&stdout, &stderr] {
            if let Ok(bytes) = fs::read(path) {
                eprintln!("{}:\n{}", path.display(), String::from_utf8_lossy(&bytes));
            }
        }
        if let Err(error) = execute(
            Command::new("xcrun").args(["simctl", "terminate", udid, "dev.tessera.ime-test"]),
            Duration::from_secs(30),
        ) {
            eprintln!("IME application cleanup failed: {error:#}");
        }
    }
    result
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
        let mut scene = plist::Dictionary::new();
        scene.insert("UISceneConfigurationName".into(), "IME acceptance".into());
        scene.insert("UISceneClassName".into(), "UIWindowScene".into());
        let mut configurations = plist::Dictionary::new();
        configurations.insert(
            "UIWindowSceneSessionRoleApplication".into(),
            plist::Value::Array(vec![plist::Value::Dictionary(scene)]),
        );
        let mut manifest = plist::Dictionary::new();
        manifest.insert("UIApplicationSupportsMultipleScenes".into(), false.into());
        manifest.insert(
            "UISceneConfigurations".into(),
            plist::Value::Dictionary(configurations),
        );
        info.insert(
            "UIApplicationSceneManifest".into(),
            plist::Value::Dictionary(manifest),
        );
        plist::Value::Dictionary(info).to_file_xml(app.join("Info.plist"))?;
        run(
            Command::new("xcrun")
                .args(["simctl", "install", udid])
                .arg(app),
            120,
        )?;
        check_ios(
            udid,
            &root.join("target/mobile-acceptance"),
            Duration::from_secs(90),
            process::run,
        )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ios_ime_requires_the_application_marker_and_preserves_failure_logs() -> Result<()> {
        for success in [false, true] {
            let directory = tempfile::tempdir()?;
            let mut terminated = false;
            let result = check_ios(
                "simulator",
                directory.path(),
                Duration::ZERO,
                |command, _| {
                    let args = command
                        .get_args()
                        .map(|arg| arg.to_string_lossy())
                        .collect::<Vec<_>>();
                    if args[1] == "terminate" {
                        terminated = true;
                        return Ok(String::new());
                    }
                    assert_eq!(args[1], "launch");
                    assert_eq!(args[2], "--terminate-running-process");
                    assert_eq!(args[5], "simulator");
                    assert_eq!(args[6], "dev.tessera.ime-test");
                    let stdout = args[3]
                        .strip_prefix("--stdout=")
                        .expect("application stdout path");
                    let stderr = args[4]
                        .strip_prefix("--stderr=")
                        .expect("application stderr path");
                    fs::write(stdout, if success { "NATIVE_IME_PASS\n" } else { "" })?;
                    fs::write(stderr, "native responder diagnostic")?;
                    Ok("dev.tessera.ime-test: 123\nNATIVE_IME_PASS".into())
                },
            );
            assert_eq!(terminated, !success);
            if success {
                result?;
            } else {
                let error = result.expect_err("launcher output cannot satisfy native acceptance");
                assert!(error.to_string().contains("native responder diagnostic"));
                assert_eq!(
                    fs::read_to_string(directory.path().join("ios-ime.stderr.log"))?,
                    "native responder diagnostic"
                );
            }
        }
        Ok(())
    }

    #[test]
    fn ios_ime_rejects_stale_success_and_retains_launch_errors() -> Result<()> {
        for launch_fails in [false, true] {
            let directory = tempfile::tempdir()?;
            fs::write(
                directory.path().join("ios-ime.stdout.log"),
                "NATIVE_IME_PASS\n",
            )?;
            let error = check_ios(
                "simulator",
                directory.path(),
                Duration::ZERO,
                |command, _| {
                    if command.get_args().nth(1).expect("simctl operation") == "terminate" {
                        anyhow::bail!("cleanup failure");
                    }
                    if launch_fails {
                        anyhow::bail!("launch failure");
                    }
                    Ok("dev.tessera.ime-test: 123".into())
                },
            )
            .expect_err("a previous run cannot satisfy native acceptance");
            assert!(error.to_string().contains(if launch_fails {
                "launch failure"
            } else {
                "did not report NATIVE_IME_PASS"
            }));
            assert_eq!(fs::read(directory.path().join("ios-ime.stdout.log"))?, b"");
        }
        Ok(())
    }
}
