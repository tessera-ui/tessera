//! `cargo tessera headless`: a long-running supervisor for headless sessions.
//!
//! The supervisor manages one or more headless render workers and speaks a
//! JSONL protocol on stdin/stdout, so automation and agents can drive Tessera
//! applications without a window.
//!
//! ## Protocol
//!
//! One JSON object per line on stdin, one response object per line on stdout.
//! Requests: `spawn`, `list`, `input`, `text`, `render`, `snapshot`, `resize`,
//! `kill`, `rebuild` and `shutdown`. Worker lifecycle commands are handled
//! here; `input`/`text`/`render`/`snapshot`/`resize` are forwarded to the
//! selected worker session.

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

use anyhow::{Result, anyhow, bail};
use cargo_metadata::{MetadataCommand, TargetKind};
use serde::Deserialize;
use serde_json::{Value, json};

/// Runs the headless supervisor loop until stdin closes or `shutdown` arrives.
pub fn execute(
    package: Option<&str>,
    release: bool,
    width: u32,
    height: u32,
    frame_time_ms: u32,
) -> Result<()> {
    let defaults = SupervisorDefaults {
        package: package.map(str::to_string),
        release,
        width,
        height,
        frame_time_ms,
    };

    let mut sessions: HashMap<String, Session> = HashMap::new();
    let mut next_session_id: u64 = 1;

    let stdin = std::io::stdin();
    let reader = BufReader::new(stdin.lock());
    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    write_line(&mut out, json!({"event": "ready"}))?;

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: Request = match serde_json::from_str(line) {
            Ok(request) => request,
            Err(err) => {
                write_line(
                    &mut out,
                    json!({"ok": false, "error": format!("invalid command: {err}")}),
                )?;
                continue;
            }
        };

        let id = request.id();
        let is_shutdown = matches!(request, Request::Shutdown { .. });

        let outcome = match request {
            Request::Spawn {
                name,
                package,
                width,
                height,
                release,
                ..
            } => spawn_session(
                &mut sessions,
                &mut next_session_id,
                &defaults,
                name,
                package,
                width,
                height,
                release,
            ),
            Request::List { .. } => Ok(json!({
                "ok": true,
                "sessions": sessions.values().map(Session::describe).collect::<Vec<_>>(),
            })),
            Request::Input {
                session,
                kind,
                pointer_id,
                x,
                y,
                delta_x,
                delta_y,
                button,
                ..
            } => with_session(&mut sessions, &session, |worker| {
                let mut payload = json!({"cmd": "input", "kind": kind});
                if let Some(map) = payload.as_object_mut() {
                    insert_if_some(map, "pointer_id", pointer_id.map(Value::from));
                    insert_if_some(map, "x", x.map(Value::from));
                    insert_if_some(map, "y", y.map(Value::from));
                    insert_if_some(map, "delta_x", delta_x.map(Value::from));
                    insert_if_some(map, "delta_y", delta_y.map(Value::from));
                    insert_if_some(map, "button", button.map(Value::from));
                }
                worker.send(payload)
            }),
            Request::Text { session, text, .. } => {
                with_session(&mut sessions, &session, |worker| {
                    worker.send(json!({"cmd": "text", "text": text}))
                })
            }
            Request::Render {
                session,
                frames,
                duration_ms,
                out,
                ..
            } => with_session(&mut sessions, &session, |worker| {
                let mut payload = json!({"cmd": "render"});
                if let Some(map) = payload.as_object_mut() {
                    insert_if_some(map, "frames", frames.map(Value::from));
                    insert_if_some(map, "duration_ms", duration_ms.map(Value::from));
                    insert_if_some(
                        map,
                        "out",
                        out.as_deref().map(resolve_out_path).map(Value::from),
                    );
                }
                worker.send(payload)
            }),
            Request::Snapshot { session, .. } => with_session(&mut sessions, &session, |worker| {
                worker.send(json!({"cmd": "snapshot"}))
            }),
            Request::Resize {
                session,
                width,
                height,
                ..
            } => {
                let result = with_session(&mut sessions, &session, |worker| {
                    worker.send(json!({"cmd": "resize", "width": width, "height": height}))
                });
                if result.is_ok()
                    && let Some(session) = sessions.get_mut(&session)
                {
                    session.width = width.max(1);
                    session.height = height.max(1);
                }
                result
            }
            Request::Kill { session, .. } => match sessions.remove(&session) {
                Some(mut worker) => {
                    worker.terminate();
                    Ok(json!({"ok": true, "session": session, "killed": true}))
                }
                None => Err(anyhow!("unknown session `{session}`")),
            },
            Request::Rebuild { session, .. } => rebuild_session(&mut sessions, &session),
            Request::Shutdown { .. } => {
                for (_, mut worker) in sessions.drain() {
                    worker.terminate();
                }
                Ok(json!({"ok": true, "event": "shutdown"}))
            }
        };

        let mut response = match outcome {
            Ok(response) => response,
            Err(err) => json!({"ok": false, "error": format!("{err}")}),
        };
        if let Some(map) = response.as_object_mut() {
            if let Some(id) = id {
                map.insert("id".to_string(), json!(id));
            }
        }
        write_line(&mut out, response)?;

        if is_shutdown {
            break;
        }
    }

    for (_, mut worker) in sessions.drain() {
        worker.terminate();
    }
    Ok(())
}

struct SupervisorDefaults {
    package: Option<String>,
    release: bool,
    width: u32,
    height: u32,
    frame_time_ms: u32,
}

struct Session {
    name: String,
    package: String,
    release: bool,
    width: u32,
    height: u32,
    frame_time_ms: u32,
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Session {
    fn describe(&self) -> Value {
        json!({
            "name": self.name,
            "package": self.package,
            "release": self.release,
            "width": self.width,
            "height": self.height,
            "pid": self.child.id(),
        })
    }

    /// Sends one request to the worker and returns its response.
    fn send(&mut self, request: Value) -> Result<Value> {
        let encoded = serde_json::to_string(&request)?;
        self.stdin.write_all(encoded.as_bytes())?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;

        let mut line = String::new();
        if self.stdout.read_line(&mut line)? == 0 {
            bail!("headless worker `{}` closed its stdout", self.name);
        }
        // Snapshots nest one level per component, which easily exceeds the
        // default serde_json recursion limit.
        let mut deserializer = serde_json::Deserializer::from_str(line.trim());
        deserializer.disable_recursion_limit();
        let value = Value::deserialize(&mut deserializer)?;
        deserializer.end()?;
        Ok(value)
    }

    fn terminate(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[allow(clippy::too_many_arguments)]
fn spawn_session(
    sessions: &mut HashMap<String, Session>,
    next_session_id: &mut u64,
    defaults: &SupervisorDefaults,
    name: Option<String>,
    package: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    release: Option<bool>,
) -> Result<Value> {
    let name = name.unwrap_or_else(|| {
        let id = *next_session_id;
        *next_session_id += 1;
        format!("session-{id}")
    });
    if sessions.contains_key(&name) {
        bail!("session `{name}` already exists");
    }

    let package = package
        .or_else(|| defaults.package.clone())
        .ok_or_else(|| {
            anyhow!(
                "no package selected; pass `--package <name>` to `cargo tessera headless` \
             or a `package` field to `spawn`"
            )
        })?;
    let release = release.unwrap_or(defaults.release);
    let width = width.unwrap_or(defaults.width).max(1);
    let height = height.unwrap_or(defaults.height).max(1);

    let worker = launch_worker(
        &name,
        &package,
        release,
        width,
        height,
        defaults.frame_time_ms,
    )?;
    let description = worker.describe();
    sessions.insert(name, worker);

    Ok(json!({"ok": true, "session": description}))
}

fn rebuild_session(sessions: &mut HashMap<String, Session>, name: &str) -> Result<Value> {
    let (package, release, width, height, frame_time_ms) = {
        let session = sessions
            .get(name)
            .ok_or_else(|| anyhow!("unknown session `{name}`"))?;
        (
            session.package.clone(),
            session.release,
            session.width,
            session.height,
            session.frame_time_ms,
        )
    };

    if let Some(mut old) = sessions.remove(name) {
        old.terminate();
    }

    let mut worker = launch_worker(name, &package, release, width, height, frame_time_ms)?;
    let description = worker.describe();
    worker.send(json!({"cmd": "resize", "width": width, "height": height}))?;
    sessions.insert(name.to_string(), worker);

    Ok(json!({"ok": true, "session": description, "rebuilt": true}))
}

fn with_session<F>(sessions: &mut HashMap<String, Session>, name: &str, action: F) -> Result<Value>
where
    F: FnOnce(&mut Session) -> Result<Value>,
{
    let session = sessions
        .get_mut(name)
        .ok_or_else(|| anyhow!("unknown session `{name}`"))?;
    let mut response = action(session)?;
    if let Some(map) = response.as_object_mut() {
        map.entry("ok").or_insert(json!(true));
        map.insert("session".to_string(), json!(name));
    }
    Ok(response)
}

fn launch_worker(
    name: &str,
    package: &str,
    release: bool,
    width: u32,
    height: u32,
    frame_time_ms: u32,
) -> Result<Session> {
    build_package(package, release)?;
    let binary = resolve_binary(package, release)?;

    let mut child = Command::new(&binary)
        .env("TESSERA_HEADLESS", "1")
        .env("TESSERA_HEADLESS_WIDTH", width.to_string())
        .env("TESSERA_HEADLESS_HEIGHT", height.to_string())
        .env("TESSERA_HEADLESS_FRAME_TIME_MS", frame_time_ms.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|err| anyhow!("failed to spawn `{}`: {err}", binary.display()))?;

    let stdin = child
        .stdin
        .take()
        .ok_or_else(|| anyhow!("failed to capture worker stdin"))?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("failed to capture worker stdout"))?;

    let mut stdout = BufReader::new(stdout);
    let mut ready = String::new();
    if stdout.read_line(&mut ready)? == 0 {
        let _ = child.kill();
        let _ = child.wait();
        bail!("headless worker `{name}` exited during startup");
    }

    Ok(Session {
        name: name.to_string(),
        package: package.to_string(),
        release,
        width,
        height,
        frame_time_ms,
        child,
        stdin,
        stdout,
    })
}

fn build_package(package: &str, release: bool) -> Result<()> {
    eprintln!("[headless] building package `{package}`");
    let mut command = Command::new("cargo");
    command.arg("build").arg("-p").arg(package);
    if release {
        command.arg("--release");
    }
    let status = command
        .status()
        .map_err(|err| anyhow!("failed to run cargo build: {err}"))?;
    if !status.success() {
        bail!("cargo build -p {package} failed with status {status}");
    }
    Ok(())
}

fn resolve_binary(package: &str, release: bool) -> Result<PathBuf> {
    let metadata = MetadataCommand::new().exec()?;
    let pkg = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name.as_str() == package)
        .ok_or_else(|| anyhow!("package `{package}` not found in cargo metadata"))?;

    let binary = pkg
        .targets
        .iter()
        .filter(|target| {
            target
                .kind
                .iter()
                .any(|kind| matches!(kind, TargetKind::Bin))
        })
        .min_by_key(|target| usize::from(target.name.as_str() != package))
        .ok_or_else(|| anyhow!("package `{package}` has no binary target"))?;

    let profile = if release { "release" } else { "debug" };
    let file_name = format!("{}{}", binary.name, std::env::consts::EXE_SUFFIX);
    let path = metadata.target_directory.join(profile).join(file_name);
    if !path.exists() {
        bail!(
            "expected worker binary at `{}`, but it does not exist",
            path
        );
    }
    Ok(path.into_std_path_buf())
}

fn resolve_out_path(path: &str) -> String {
    let candidate = Path::new(path);
    if candidate.is_absolute() {
        return candidate.to_string_lossy().into_owned();
    }
    match std::env::current_dir() {
        Ok(cwd) => cwd.join(candidate).to_string_lossy().into_owned(),
        Err(_) => path.to_string(),
    }
}

fn insert_if_some(map: &mut serde_json::Map<String, Value>, key: &str, value: Option<Value>) {
    if let Some(value) = value {
        map.insert(key.to_string(), value);
    }
}

fn write_line(out: &mut impl Write, value: Value) -> Result<()> {
    let encoded = serde_json::to_string(&value)?;
    out.write_all(encoded.as_bytes())?;
    out.write_all(b"\n")?;
    out.flush()?;
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
enum Request {
    Spawn {
        #[serde(default)]
        id: Option<u64>,
        #[serde(default)]
        name: Option<String>,
        #[serde(default)]
        package: Option<String>,
        #[serde(default)]
        width: Option<u32>,
        #[serde(default)]
        height: Option<u32>,
        #[serde(default)]
        release: Option<bool>,
    },
    List {
        #[serde(default)]
        id: Option<u64>,
    },
    Input {
        #[serde(default)]
        id: Option<u64>,
        session: String,
        kind: String,
        #[serde(default)]
        pointer_id: Option<u64>,
        #[serde(default)]
        x: Option<f32>,
        #[serde(default)]
        y: Option<f32>,
        #[serde(default)]
        delta_x: Option<f32>,
        #[serde(default)]
        delta_y: Option<f32>,
        #[serde(default)]
        button: Option<String>,
    },
    Text {
        #[serde(default)]
        id: Option<u64>,
        session: String,
        text: String,
    },
    Render {
        #[serde(default)]
        id: Option<u64>,
        session: String,
        #[serde(default)]
        frames: Option<u32>,
        #[serde(default)]
        duration_ms: Option<u64>,
        #[serde(default)]
        out: Option<String>,
    },
    Snapshot {
        #[serde(default)]
        id: Option<u64>,
        session: String,
    },
    Resize {
        #[serde(default)]
        id: Option<u64>,
        session: String,
        width: u32,
        height: u32,
    },
    Kill {
        #[serde(default)]
        id: Option<u64>,
        session: String,
    },
    Rebuild {
        #[serde(default)]
        id: Option<u64>,
        session: String,
    },
    Shutdown {
        #[serde(default)]
        id: Option<u64>,
    },
}

impl Request {
    fn id(&self) -> Option<u64> {
        match self {
            Request::Spawn { id, .. }
            | Request::List { id }
            | Request::Input { id, .. }
            | Request::Text { id, .. }
            | Request::Render { id, .. }
            | Request::Snapshot { id, .. }
            | Request::Resize { id, .. }
            | Request::Kill { id, .. }
            | Request::Rebuild { id, .. }
            | Request::Shutdown { id } => *id,
        }
    }
}
