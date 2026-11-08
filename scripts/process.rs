use std::{
    fs::File,
    io::{Read, Seek, SeekFrom},
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Context, Result, bail, ensure};
use wait_timeout::ChildExt;

pub fn run(command: &mut Command, timeout: Duration) -> Result<String> {
    capture(command, timeout, false)
}

#[allow(dead_code)]
pub fn run_allow_failure(command: &mut Command, timeout: Duration) -> Result<String> {
    capture(command, timeout, true)
}

fn capture(command: &mut Command, timeout: Duration, allow_failure: bool) -> Result<String> {
    let mut stdout = tempfile::tempfile()?;
    let mut stderr = tempfile::tempfile()?;
    let mut child = command
        .stdin(Stdio::null())
        .stdout(stdout.try_clone()?)
        .stderr(stderr.try_clone()?)
        .spawn()
        .with_context(|| format!("Failed to start {command:?}"))?;
    let status = child.wait_timeout(timeout)?;
    if status.is_none() {
        let _ = child.kill();
        let _ = child.wait();
    }
    fn read(file: &mut File) -> Result<String> {
        file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        Ok(String::from_utf8_lossy(&bytes).into_owned())
    }
    let stdout = read(&mut stdout)?;
    let stderr = read(&mut stderr)?;
    let Some(status) = status else {
        bail!("{command:?} timed out after {timeout:?}\n{stdout}\n{stderr}");
    };
    ensure!(
        status.success() || allow_failure,
        "{command:?}: {status}\n{stdout}\n{stderr}"
    );
    if !stderr.is_empty() && !allow_failure {
        eprint!("{stderr}");
    }
    Ok(stdout.trim().to_owned())
}
