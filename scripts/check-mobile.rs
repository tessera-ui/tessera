#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! anyhow = "1.0.104"
//! clap = { version = "4.6.7", features = ["derive"] }
//! plist = "1.10.3"
//! regex = "1.12.3"
//! serde = "1.0.229"
//! serde_json = { version = "1.0.154", features = ["unbounded_depth"] }
//! serde_stacker = "0.1.14"
//! tempfile = "3.27.0"
//! wait-timeout = "0.2.1"
//! walkdir = "2.5.0"
//! ```

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, ensure};
use clap::Parser;
use regex::Regex;
use serde::Deserialize;
use serde_json::{Value, json};
use walkdir::WalkDir;

mod process;

const COMMAND_TIMEOUT: Duration = Duration::from_secs(120);

#[derive(Parser)]
#[command(about = "Verify mobile rendering and application resume")]
struct Cli {
    #[arg(long, conflicts_with = "ios", required_unless_present = "ios")]
    android: Option<PathBuf>,
    #[arg(long, num_args = 2, value_names = ["UDID", "APP"])]
    ios: Vec<String>,
}

fn events(text: &str) -> Vec<Value> {
    text.lines()
        .filter_map(|line| {
            let mut deserializer = serde_json::Deserializer::from_str(line);
            deserializer.disable_recursion_limit();
            Value::deserialize(serde_stacker::Deserializer::new(&mut deserializer)).ok()
        })
        .collect()
}

fn frame(records: &[Value]) -> Option<&Value> {
    records.iter().rev().find(|item| item["type"] == "frame")
}

fn valid(record: &Value) -> bool {
    fn text(nodes: &Value) -> bool {
        nodes.as_array().is_some_and(|nodes| {
            nodes.iter().any(|node| {
                node["fn_name"]
                    .as_str()
                    .is_some_and(|name| name.split("::").last() == Some("text"))
                    || text(&node["children"])
            })
        })
    }
    record["render_time_ns"].as_u64().unwrap_or(0) > 0 && text(&record["components"])
}

fn ready(records: &[Value], require_runtime_redraw: bool) -> bool {
    if !require_runtime_redraw {
        return frame(records).is_some_and(valid);
    }
    records
        .iter()
        .filter(|record| record["type"] == "frame" && valid(record))
        .skip(1)
        .any(|record| {
            record["redraw_reasons"].as_array().is_some_and(|reasons| {
                reasons
                    .iter()
                    .any(|reason| reason == "runtime_invalidation")
            })
        })
}

fn wait(
    read: &mut impl FnMut() -> Result<String>,
    expected: &str,
    predicate: impl Fn(&[Value]) -> bool,
) -> Result<Vec<Value>> {
    let deadline = Instant::now() + Duration::from_secs(90);
    loop {
        let records = events(&read()?);
        if predicate(&records) {
            return Ok(records);
        }
        ensure!(
            Instant::now() < deadline,
            "Application did not publish {expected}; latest={:?}",
            &records[records.len().saturating_sub(5)..]
        );
        thread::sleep(Duration::from_millis(200));
    }
}

fn verify(
    mut read: impl FnMut() -> Result<String>,
    mut background: impl FnMut() -> Result<()>,
    mut foreground: impl FnMut() -> Result<()>,
    require_runtime_redraw: bool,
) -> Result<()> {
    let initial = wait(&mut read, "the initial rendered frame", |records| {
        ready(records, require_runtime_redraw)
    })?;
    background()?;
    let suspended = wait(
        &mut read,
        "a suspended event after backgrounding",
        |records| {
            records[initial.len().min(records.len())..]
                .iter()
                .any(|item| item["type"] == "runtime" && item["kind"] == "suspended")
        },
    )?;
    foreground()?;
    let restored = wait(
        &mut read,
        "a resumed frame after foregrounding",
        |records| {
            let following = &records[suspended.len().min(records.len())..];
            following
                .iter()
                .position(|item| item["type"] == "runtime" && item["kind"] == "resumed")
                .is_some_and(|position| ready(&following[position + 1..], require_runtime_redraw))
        },
    )?;
    println!(
        "{}",
        json!({"first_frame": frame(&initial).unwrap()["frame"], "restored_frame": frame(&restored).unwrap()["frame"]})
    );
    Ok(())
}

fn run(command: &mut Command) -> Result<String> {
    process::run(command, COMMAND_TIMEOUT)
}

fn android(directory: &Path) -> Result<()> {
    let sdk = std::env::var_os("ANDROID_HOME")
        .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
        .context("Set ANDROID_HOME or ANDROID_SDK_ROOT")?;
    let mut tools = WalkDir::new(Path::new(&sdk).join("build-tools"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() == if cfg!(windows) { "aapt.exe" } else { "aapt" })
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();
    tools.sort();
    let aapt = tools.last().context("No Android aapt tool found")?;
    let abi = run(Command::new("adb").args(["shell", "getprop", "ro.product.cpu.abi"]))?;
    let paths = if directory.is_file() {
        vec![directory.to_owned()]
    } else {
        WalkDir::new(directory)
            .into_iter()
            .filter_map(Result::ok)
            .map(|entry| entry.into_path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "apk"))
            .collect()
    };
    let mut matches = Vec::new();
    for path in paths {
        let metadata = run(Command::new(aapt).args(["dump", "badging"]).arg(&path))?;
        if metadata.contains(&format!("'{abi}'")) {
            matches.push((fs::metadata(&path)?.modified()?, path, metadata));
        }
    }
    matches.sort_by_key(|item| item.0);
    let (_, apk, metadata) = matches.last().context("No APK matches the device ABI")?;
    let identifier = Regex::new(r"package: name='([^']+)'")?
        .captures(metadata)
        .context("Missing APK identifier")?[1]
        .to_owned();
    let activity = Regex::new(r"launchable-activity: name='([^']+)'")?
        .captures(metadata)
        .context("Missing APK launch activity")?[1]
        .to_owned();
    run(Command::new("adb").args(["install", "-r"]).arg(apk))?;
    run(Command::new("adb").args(["shell", "am", "force-stop", &identifier]))?;
    run(Command::new("adb").args([
        "shell",
        "run-as",
        &identifier,
        "rm",
        "-f",
        "files/acceptance.jsonl",
    ]))?;
    let launch = || {
        run(Command::new("adb").args([
            "shell",
            "am",
            "start",
            "-W",
            "-n",
            &format!("{identifier}/{activity}"),
        ]))
    };
    let read = || {
        let mut command = Command::new("adb");
        command.args([
            "shell",
            "run-as",
            &identifier,
            "cat",
            "files/acceptance.jsonl",
        ]);
        process::run_allow_failure(&mut command, Duration::from_secs(15))
    };
    let result = (|| {
        launch()?;
        let pid = run(Command::new("adb").args(["shell", "pidof", &identifier]))?;
        verify(
            read,
            || {
                run(Command::new("adb").args([
                    "shell",
                    "am",
                    "start",
                    "-W",
                    "-a",
                    "android.intent.action.MAIN",
                    "-c",
                    "android.intent.category.HOME",
                ]))?;
                Ok(())
            },
            || launch().map(|_| ()),
            false,
        )?;
        ensure!(
            run(Command::new("adb").args(["shell", "pidof", &identifier]))? == pid,
            "Application restarted instead of resuming"
        );
        run(Command::new("adb").args(["shell", "am", "force-stop", &identifier]))?;
        Ok(())
    })();
    fs::create_dir_all("target/mobile-acceptance")?;
    fs::write("target/mobile-acceptance/android.jsonl", read()?)?;
    result
}

fn ios_with(
    udid: &str,
    bundle: &Path,
    output: &Path,
    mut execute: impl FnMut(&mut Command, Duration) -> Result<String>,
) -> Result<()> {
    let info = plist::Value::from_file(bundle.join("Info.plist"))?;
    let identifier = info
        .as_dictionary()
        .and_then(|info| info.get("CFBundleIdentifier"))
        .and_then(plist::Value::as_string)
        .context("Missing bundle identifier")?;
    execute(
        Command::new("xcrun")
            .args(["simctl", "install", udid])
            .arg(bundle),
        COMMAND_TIMEOUT,
    )?;
    let container = execute(
        Command::new("xcrun").args(["simctl", "get_app_container", udid, identifier, "data"]),
        COMMAND_TIMEOUT,
    )?;
    let trace = Path::new(&container).join("Documents/acceptance.jsonl");
    if trace.exists() {
        fs::remove_file(&trace)?;
    }
    fs::create_dir_all(output)?;
    let stdout = output.join("ios.stdout.log");
    let stderr = output.join("ios.stderr.log");
    let launch = || {
        let mut command = Command::new("xcrun");
        command
            .args(["simctl", "launch"])
            .arg(format!("--stdout={}", stdout.display()))
            .arg(format!("--stderr={}", stderr.display()))
            .args([udid, identifier])
            .env("SIMCTL_CHILD_TESSERA_PROFILING_OUTPUT", &trace);
        command
    };
    let read = || {
        if trace.exists() {
            Ok(fs::read_to_string(&trace)?)
        } else {
            Ok(String::new())
        }
    };
    let result = (|| {
        let pid = execute(&mut launch(), Duration::from_secs(300))?
            .rsplit(':')
            .next()
            .context("Missing launched PID")?
            .trim()
            .to_owned();
        let execute = std::cell::RefCell::new(&mut execute);
        verify(
            read,
            || {
                execute.borrow_mut()(
                    Command::new("xcrun").args([
                        "simctl",
                        "launch",
                        udid,
                        "com.apple.mobilesafari",
                    ]),
                    COMMAND_TIMEOUT,
                )?;
                // simctl launch can start Safari without making it the foreground app.
                execute.borrow_mut()(
                    Command::new("xcrun").args(["simctl", "openurl", udid, "https://127.0.0.1/"]),
                    COMMAND_TIMEOUT,
                )?;
                Ok(())
            },
            || execute.borrow_mut()(&mut launch(), COMMAND_TIMEOUT).map(|_| ()),
            true,
        )?;
        let current = execute.borrow_mut()(&mut launch(), COMMAND_TIMEOUT)?;
        ensure!(
            current.rsplit(':').next().unwrap_or("").trim() == pid,
            "Application restarted instead of resuming"
        );
        Ok(())
    })();
    if result.is_err() {
        for path in [&stdout, &stderr] {
            if let Ok(bytes) = fs::read(path) {
                eprintln!(
                    "{}:\n{}",
                    path.display(),
                    String::from_utf8_lossy(&bytes[bytes.len().saturating_sub(12000)..])
                );
            }
        }
    }
    if let Err(error) = execute(
        Command::new("xcrun").args(["simctl", "terminate", udid, identifier]),
        Duration::from_secs(30),
    ) {
        eprintln!("Simulator cleanup failed: {error:#}");
    }
    fs::write(output.join("ios.jsonl"), read()?)?;
    result
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if let Some(directory) = cli.android {
        android(&directory)
    } else {
        ios_with(
            &cli.ios[0],
            Path::new(&cli.ios[1]),
            &std::env::current_dir()?.join("target/mobile-acceptance"),
            process::run,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Result<(tempfile::TempDir, PathBuf, PathBuf)> {
        let directory = tempfile::tempdir()?;
        let bundle = directory.path().join("Example.app");
        fs::create_dir(&bundle)?;
        let mut info = plist::Dictionary::new();
        info.insert("CFBundleIdentifier".into(), "dev.tessera.example".into());
        plist::Value::Dictionary(info).to_file_xml(bundle.join("Info.plist"))?;
        let container = directory.path().join("container");
        fs::create_dir_all(container.join("Documents"))?;
        Ok((directory, bundle, container))
    }

    #[test]
    fn initial_launch_failure_retains_trace_even_when_cleanup_fails() -> Result<()> {
        let (directory, bundle, container) = fixture()?;
        let output = directory.path().join("diagnostics");
        let trace = container.join("Documents/acceptance.jsonl");
        let error = ios_with("simulator", &bundle, &output, |command, timeout| {
            let args = command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            match args[1].as_str() {
                "get_app_container" => Ok(container.display().to_string()),
                "launch" => {
                    assert_eq!(timeout, Duration::from_secs(300));
                    fs::write(&trace, "{\"type\":\"runtime\",\"kind\":\"resumed\"}\n")?;
                    for option in &args[2..4] {
                        fs::write(option.split_once('=').unwrap().1, "startup diagnostic")?;
                    }
                    anyhow::bail!("launch timeout")
                }
                "terminate" => anyhow::bail!("cleanup timeout"),
                _ => Ok(String::new()),
            }
        })
        .unwrap_err();
        assert!(error.to_string().contains("launch timeout"));
        assert_eq!(fs::read(output.join("ios.jsonl"))?, fs::read(trace)?);
        assert_eq!(
            fs::read_to_string(output.join("ios.stderr.log"))?,
            "startup diagnostic"
        );
        Ok(())
    }

    #[test]
    fn ios_lifecycle_activates_safari_and_rejects_process_restart() -> Result<()> {
        use std::io::Write;

        for restarted in [false, true] {
            let (directory, bundle, container) = fixture()?;
            let output = directory.path().join("diagnostics");
            let trace = container.join("Documents/acceptance.jsonl");
            let mut launches = 0;
            let result = ios_with("simulator", &bundle, &output, |command, timeout| {
                let args = command
                    .get_args()
                    .map(|arg| arg.to_string_lossy().into_owned())
                    .collect::<Vec<_>>();
                match args[1].as_str() {
                    "get_app_container" => return Ok(container.display().to_string()),
                    "openurl" => {
                        assert_eq!(args[3], "https://127.0.0.1/");
                        let mut file = fs::OpenOptions::new().append(true).open(&trace)?;
                        writeln!(file, "{}", json!({"type":"runtime", "kind":"suspended"}))?;
                        return Ok(String::new());
                    }
                    "launch" => {}
                    _ => return Ok(String::new()),
                }
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&trace)?;
                if args.last().unwrap() == "com.apple.mobilesafari" {
                    return Ok("safari: 456".into());
                }
                launches += 1;
                assert_eq!(timeout.as_secs(), if launches == 1 { 300 } else { 120 });
                assert!(command.get_envs().any(|(key, value)| key
                    == "SIMCTL_CHILD_TESSERA_PROFILING_OUTPUT"
                    && value == Some(trace.as_os_str())));
                writeln!(file, "{}", json!({"type":"runtime", "kind":"resumed"}))?;
                for (offset, reason) in [(0, "window_event"), (1, "runtime_invalidation")] {
                    writeln!(
                        file,
                        "{}",
                        json!({"type":"frame", "frame":launches * 2 + offset, "render_time_ns":1, "components":[{"fn_name":"text"}], "redraw_reasons":[reason]})
                    )?;
                }
                Ok(format!(
                    "example: {}",
                    if restarted && launches > 1 { 789 } else { 123 }
                ))
            });
            if restarted {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("Application restarted")
                );
            } else {
                result?;
            }
            assert_eq!(fs::read(output.join("ios.jsonl"))?, fs::read(trace)?);
        }
        Ok(())
    }

    #[test]
    fn runtime_redraw_requires_a_second_rendered_frame_with_text() {
        let first = json!({"type":"frame", "render_time_ns":1, "components":[{"fn_name":"text"}], "redraw_reasons":["runtime_invalidation"]});
        assert!(!ready(&[first.clone()], true));
        let mut second = first.clone();
        second["render_time_ns"] = json!(0);
        assert!(!ready(&[first.clone(), second.clone()], true));
        second["render_time_ns"] = json!(1);
        second["components"] = json!([]);
        assert!(!ready(&[first.clone(), second.clone()], true));
        second["components"] = json!([{"children":[{"fn_name":"module::text"}]}]);
        assert!(ready(&[first, second], true));
    }

    #[test]
    fn deeply_nested_mobile_frames_remain_visible_to_lifecycle_checks() {
        let component = format!(
            "{}{{\"fn_name\":\"text\"}}{}",
            "{\"children\":[".repeat(160),
            "]}".repeat(160)
        );
        let frame = |index, reason| {
            format!(
                "{{\"type\":\"frame\",\"frame\":{index},\"render_time_ns\":1,\"components\":[{component}],\"redraw_reasons\":[\"{reason}\"]}}"
            )
        };
        let records = events(&format!(
            "{}\n{}\n",
            frame(0, "startup"),
            frame(1, "runtime_invalidation")
        ));
        assert_eq!(records.len(), 2);
        assert!(ready(&records, true));
    }
}
