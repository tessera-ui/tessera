use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::channel,
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, bail};
use cargo_metadata::MetadataCommand;
use handlebars::Handlebars;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde_json::json;
use tessera_build::{AssetBackend, load_tessera_config_from_dir, resolve_assets_dir};
use wasm_bindgen_cli_support::Bindgen;

use crate::output;

use super::find_package_dir;

const INDEX_TEMPLATE: &str = include_str!("../../templates/basic/gen/web/index.html.hbs");
const WASM_BINDGEN_DEP: &str = "wasm-bindgen = \"0.2.105\"";
const WASM_DEP_HEADER: &str = "[target.'cfg(target_family = \"wasm\")'.dependencies]";
const README_WEB_SECTION: &str = r#"
## Running On Web

```bash
cargo tessera web dev
```

Then open `http://127.0.0.1:8000`.
"#;
const WASM_START_FN: &str = r#"
#[cfg(target_family = "wasm")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    run()
        .run_web()
        .map_err(|err| wasm_bindgen::JsValue::from_str(&err.to_string()))
}
"#;
const WASM_MAIN_STUB: &str = r#"
#[cfg(target_family = "wasm")]
fn main() {}
"#;
const DESKTOP_MAIN_GUARD: &str = "#[cfg(not(target_family = \"wasm\"))]";
const WASM_TARGET: &str = "wasm32-unknown-unknown";
const WEB_HOST_DIR: &str = "gen/web";
const WEB_OUTPUT_DIR: &str = "gen/web/pkg-web";
const BUILD_DEBOUNCE_WINDOW: Duration = Duration::from_millis(300);

pub fn init(package: Option<&str>) -> Result<()> {
    let project_dir = resolve_project_dir(package)?;
    let metadata = load_project_metadata(&project_dir)?;

    output::status(
        "Initializing",
        format!("web support for `{}`", metadata.package_name),
    );

    let mut changed = false;
    changed |= ensure_index_html(&project_dir, &metadata)?;
    changed |= ensure_pkg_web_ignored(&project_dir)?;
    changed |= ensure_cargo_has_wasm_bindgen(&project_dir)?;
    changed |= ensure_lib_has_wasm_start(&project_dir)?;
    changed |= ensure_main_has_wasm_cfg(&project_dir)?;
    changed |= ensure_readme_has_web_section(&project_dir)?;

    if changed {
        output::status(
            "Updated",
            format!("web support in {}", project_dir.display()),
        );
    } else {
        output::status(
            "UpToDate",
            format!("web support in {}", project_dir.display()),
        );
    }

    output::note("Web host files are now in place.");
    output::step("cargo tessera web dev");

    Ok(())
}

pub fn build(release: bool, package: Option<&str>) -> Result<()> {
    let project_dir = resolve_project_dir(package)?;
    let metadata = load_project_metadata(&project_dir)?;

    output::status("Building", format!("web app `{}`", metadata.package_name));
    build_project(&project_dir, &metadata, release)?;
    output::status(
        "Finished",
        format!(
            "web build in {}",
            project_dir.join(WEB_OUTPUT_DIR).display()
        ),
    );
    Ok(())
}

pub fn dev(release: bool, package: Option<&str>, port: u16) -> Result<()> {
    let project_dir = resolve_project_dir(package)?;
    let metadata = load_project_metadata(&project_dir)?;

    output::status(
        "Starting",
        format!("web dev server for `{}`", metadata.package_name),
    );
    output::status("Serving", format!("http://127.0.0.1:{port}"));
    output::status("Watching", "for file changes");
    output::note("Refresh the browser after rebuilds.");

    build_project(&project_dir, &metadata, release)?;

    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("Failed to bind development server on port {port}"))?;
    listener
        .set_nonblocking(true)
        .context("Failed to configure non-blocking development server socket")?;
    let web_host_dir = project_dir.join(WEB_HOST_DIR);
    spawn_static_file_server(listener, web_host_dir.clone());
    open_browser(format!("http://127.0.0.1:{port}"));

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
        if let Ok(event) = res
            && matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            )
        {
            let _ = tx.send(());
        }
    })?;

    let src_path = project_dir.join("src");
    if src_path.exists() {
        watcher.watch(&src_path, RecursiveMode::Recursive)?;
    } else {
        bail!("Source directory not found: {}", src_path.display());
    }

    for file in [
        "Cargo.toml",
        "build.rs",
        "tessera-app.toml",
        "tessera-config.toml",
    ] {
        let path = project_dir.join(file);
        if path.exists() {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }
    }

    let index_path = project_dir.join(WEB_HOST_DIR).join("index.html");
    if index_path.exists() {
        watcher.watch(&index_path, RecursiveMode::NonRecursive)?;
    }

    if let Some(config) = load_tessera_config_from_dir(&project_dir)?
        && let Some(assets_dir) = resolve_assets_dir(&project_dir, Some(&config))
        && assets_dir.exists()
    {
        watcher.watch(&assets_dir, RecursiveMode::Recursive)?;
    }

    let mut pending_change = false;
    let mut last_change = Instant::now();

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => {
                pending_change = true;
                last_change = Instant::now();
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                bail!("Web file watcher disconnected unexpectedly");
            }
        }

        if pending_change && last_change.elapsed() >= BUILD_DEBOUNCE_WINDOW {
            output::status("Building", "web app");
            match build_project(&project_dir, &metadata, release) {
                Ok(()) => output::status(
                    "Finished",
                    format!(
                        "web build in {}",
                        project_dir.join(WEB_OUTPUT_DIR).display()
                    ),
                ),
                Err(err) => output::error(format!("{err:#}")),
            }
            pending_change = false;
        }
    }
}

fn build_project(project_dir: &Path, metadata: &ProjectMetadata, release: bool) -> Result<()> {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .arg("--target")
        .arg(WASM_TARGET)
        .env("TESSERA_ASSET_BACKEND", AssetBackend::Embed.as_str());
    if release {
        cmd.arg("--release");
    }

    let status = cmd
        .status()
        .context("Failed to run cargo build for web target")?;
    if !status.success() {
        bail!("Web build failed");
    }

    let output_dir = project_dir.join(WEB_OUTPUT_DIR);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create {}", output_dir.display()))?;

    let wasm_path = resolve_wasm_artifact_path(project_dir, metadata, release)?;
    if !wasm_path.exists() {
        bail!(
            "Compiled wasm artifact not found at {}",
            wasm_path.display()
        );
    }

    let mut bindgen = Bindgen::new();
    bindgen
        .input_path(&wasm_path)
        .out_name(&metadata.lib_name)
        .omit_default_module_path(false)
        .web(true)
        .context("Failed to configure web bindgen target")?
        .debug(!release)
        .typescript(true)
        .generate(&output_dir)
        .with_context(|| {
            format!(
                "Failed to generate browser bindings into {}",
                output_dir.display()
            )
        })?;

    if !project_dir.join(WEB_HOST_DIR).join("index.html").exists() {
        output::warn(
            "No index.html found; run `cargo tessera web init` to generate a minimal web host",
        );
    }

    Ok(())
}

struct ProjectMetadata {
    manifest_path: PathBuf,
    package_name: String,
    project_name_snake: String,
    lib_name: String,
}

fn resolve_project_dir(package: Option<&str>) -> Result<PathBuf> {
    if let Some(package) = package {
        return find_package_dir(package);
    }

    let cargo_toml = PathBuf::from("Cargo.toml");
    if !cargo_toml.exists() {
        bail!("No Cargo.toml found in the current directory");
    }

    let content = fs::read_to_string(&cargo_toml).context("Failed to read Cargo.toml")?;
    let manifest: toml::Value =
        toml::from_str(&content).context("Failed to parse current Cargo.toml")?;
    if manifest.get("package").is_some() {
        return Ok(PathBuf::from("."));
    }

    bail!("Could not determine app package automatically; rerun with --package <name>")
}

fn load_project_metadata(project_dir: &Path) -> Result<ProjectMetadata> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;
    let manifest: toml::Value = toml::from_str(&content)
        .with_context(|| format!("Failed to parse {}", cargo_toml_path.display()))?;

    let package_name = manifest
        .get("package")
        .and_then(|package| package.get("name"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| anyhow!("Missing package.name in {}", cargo_toml_path.display()))?
        .to_string();
    let project_name_snake = package_name.replace('-', "_");
    let lib_name = manifest
        .get("lib")
        .and_then(|lib| lib.get("name"))
        .and_then(toml::Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| format!("{project_name_snake}_lib"));

    Ok(ProjectMetadata {
        manifest_path: cargo_toml_path,
        package_name,
        project_name_snake,
        lib_name,
    })
}

fn resolve_wasm_artifact_path(
    _project_dir: &Path,
    metadata: &ProjectMetadata,
    release: bool,
) -> Result<PathBuf> {
    let cargo_metadata = MetadataCommand::new()
        .manifest_path(&metadata.manifest_path)
        .exec()
        .context("Failed to load cargo metadata for web build")?;

    let profile_dir = if release { "release" } else { "debug" };
    let artifact_name = format!("{}.wasm", metadata.lib_name.replace('-', "_"));
    Ok(cargo_metadata
        .target_directory
        .as_std_path()
        .join(WASM_TARGET)
        .join(profile_dir)
        .join(artifact_name))
}

fn ensure_index_html(project_dir: &Path, metadata: &ProjectMetadata) -> Result<bool> {
    let host_dir = project_dir.join(WEB_HOST_DIR);
    fs::create_dir_all(&host_dir)
        .with_context(|| format!("Failed to create {}", host_dir.display()))?;
    let index_path = host_dir.join("index.html");
    if index_path.exists() {
        return Ok(false);
    }

    let legacy_index_path = project_dir.join("index.html");
    if legacy_index_path.exists() {
        fs::copy(&legacy_index_path, &index_path).with_context(|| {
            format!(
                "Failed to migrate {} to {}",
                legacy_index_path.display(),
                index_path.display()
            )
        })?;
        return Ok(true);
    }

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    let rendered = handlebars
        .render_template(
            INDEX_TEMPLATE,
            &json!({
                "project_name": metadata.package_name,
                "project_name_snake": metadata.project_name_snake,
                "lib_name": metadata.lib_name,
            }),
        )
        .context("Failed to render web host template")?;
    fs::write(&index_path, rendered)
        .with_context(|| format!("Failed to write {}", index_path.display()))?;
    Ok(true)
}

fn ensure_pkg_web_ignored(project_dir: &Path) -> Result<bool> {
    let gitignore_path = project_dir.join(".gitignore");
    let mut content = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)
            .with_context(|| format!("Failed to read {}", gitignore_path.display()))?
    } else {
        String::new()
    };

    let normalized = content.replace("/pkg-web", "/gen/web/pkg-web");
    if normalized != content {
        fs::write(&gitignore_path, normalized)
            .with_context(|| format!("Failed to write {}", gitignore_path.display()))?;
        return Ok(true);
    }

    if content
        .lines()
        .any(|line| line.trim() == "/gen/web/pkg-web")
    {
        return Ok(false);
    }

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }
    content.push_str("/gen/web/pkg-web\n");
    fs::write(&gitignore_path, content)
        .with_context(|| format!("Failed to write {}", gitignore_path.display()))?;
    Ok(true)
}

fn ensure_cargo_has_wasm_bindgen(project_dir: &Path) -> Result<bool> {
    let cargo_toml_path = project_dir.join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;

    if content.contains("wasm-bindgen") {
        return Ok(false);
    }

    let updated = if let Some(header_pos) = content.find(WASM_DEP_HEADER) {
        let insert_at = header_pos + WASM_DEP_HEADER.len();
        let mut next = String::with_capacity(content.len() + WASM_BINDGEN_DEP.len() + 2);
        next.push_str(&content[..insert_at]);
        next.push('\n');
        next.push_str(WASM_BINDGEN_DEP);
        next.push_str(&content[insert_at..]);
        next
    } else {
        let mut next = content;
        if !next.ends_with('\n') {
            next.push('\n');
        }
        next.push('\n');
        next.push_str(WASM_DEP_HEADER);
        next.push('\n');
        next.push_str(WASM_BINDGEN_DEP);
        next.push('\n');
        next
    };

    fs::write(&cargo_toml_path, updated)
        .with_context(|| format!("Failed to write {}", cargo_toml_path.display()))?;
    Ok(true)
}

fn ensure_lib_has_wasm_start(project_dir: &Path) -> Result<bool> {
    let lib_path = project_dir.join("src/lib.rs");
    let mut content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;

    if content.contains("wasm_bindgen(start)") {
        return Ok(false);
    }

    if !content.contains("fn run(") {
        output::warn(format!(
            "Skipping {} because no `run()` entry function was found",
            lib_path.display()
        ));
        return Ok(false);
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(WASM_START_FN.trim_start_matches('\n'));
    fs::write(&lib_path, content)
        .with_context(|| format!("Failed to write {}", lib_path.display()))?;
    Ok(true)
}

fn ensure_main_has_wasm_cfg(project_dir: &Path) -> Result<bool> {
    let main_path = project_dir.join("src/main.rs");
    let mut content = fs::read_to_string(&main_path)
        .with_context(|| format!("Failed to read {}", main_path.display()))?;

    let mut changed = false;
    if !content.contains(DESKTOP_MAIN_GUARD) {
        if let Some(pos) = content.find("fn main()") {
            content.insert_str(pos, "#[cfg(not(target_family = \"wasm\"))]\n");
            changed = true;
        } else {
            output::warn(format!(
                "Skipping desktop guard insertion in {} because `fn main()` was not found",
                main_path.display()
            ));
        }
    }

    if !content.contains("cfg(target_family = \"wasm\")") || !content.contains("fn main() {}") {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(WASM_MAIN_STUB.trim_start_matches('\n'));
        changed = true;
    }

    if changed {
        fs::write(&main_path, content)
            .with_context(|| format!("Failed to write {}", main_path.display()))?;
    }

    Ok(changed)
}

fn ensure_readme_has_web_section(project_dir: &Path) -> Result<bool> {
    let readme_path = project_dir.join("README.md");
    if !readme_path.exists() {
        return Ok(false);
    }

    let mut content = fs::read_to_string(&readme_path)
        .with_context(|| format!("Failed to read {}", readme_path.display()))?;
    if content.contains("## Running On Web") {
        let normalized = content.replace(
            "wasm-pack build . --target web --dev --out-dir pkg-web\npython -m http.server 8000",
            "cargo tessera web dev",
        );
        let normalized = normalized.replace(
            "cargo tessera web init\ncargo tessera web dev",
            "cargo tessera web dev",
        );
        if normalized != content {
            fs::write(&readme_path, normalized)
                .with_context(|| format!("Failed to write {}", readme_path.display()))?;
            return Ok(true);
        }
        return Ok(false);
    }

    if !content.ends_with('\n') {
        content.push('\n');
    }
    content.push('\n');
    content.push_str(README_WEB_SECTION.trim_start_matches('\n'));
    content.push('\n');
    fs::write(&readme_path, content)
        .with_context(|| format!("Failed to write {}", readme_path.display()))?;
    Ok(true)
}

fn open_browser(url: String) {
    let spawn_result = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", &url]).spawn()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(&url).spawn()
    } else {
        Command::new("xdg-open").arg(&url).spawn()
    };

    if let Err(err) = spawn_result {
        output::warn(format!("failed to open browser automatically: {err}"));
    }
}

fn spawn_static_file_server(listener: TcpListener, project_dir: PathBuf) {
    thread::spawn(move || {
        loop {
            match listener.accept() {
                Ok((stream, _)) => {
                    if let Err(err) = handle_http_request(stream, &project_dir) {
                        output::warn(format!("web server request failed: {err:#}"));
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(25));
                }
                Err(err) => {
                    output::error(format!("web server failed: {err}"));
                    break;
                }
            }
        }
    });
}

fn handle_http_request(mut stream: TcpStream, project_dir: &Path) -> Result<()> {
    let mut buffer = [0_u8; 4096];
    let bytes_read = stream
        .read(&mut buffer)
        .context("Failed to read HTTP request")?;
    if bytes_read == 0 {
        return Ok(());
    }

    let request = String::from_utf8_lossy(&buffer[..bytes_read]);
    let Some(request_line) = request.lines().next() else {
        return Ok(());
    };
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or("/");

    if method != "GET" && method != "HEAD" {
        return write_response(
            &mut stream,
            "405 Method Not Allowed",
            "text/plain; charset=utf-8",
            b"Method Not Allowed",
        );
    }

    let requested_path = target.split(['?', '#']).next().unwrap_or("/");
    let Some(file_path) = resolve_served_file(project_dir, requested_path) else {
        return write_response(
            &mut stream,
            "404 Not Found",
            "text/plain; charset=utf-8",
            b"Not Found",
        );
    };

    let body =
        fs::read(&file_path).with_context(|| format!("Failed to read {}", file_path.display()))?;
    let content_type = content_type_for_path(&file_path);

    if method == "HEAD" {
        write_headers(&mut stream, "200 OK", content_type, body.len())
    } else {
        write_response(&mut stream, "200 OK", content_type, &body)
    }
}

fn resolve_served_file(project_dir: &Path, requested_path: &str) -> Option<PathBuf> {
    let relative = if requested_path == "/" {
        PathBuf::from("index.html")
    } else {
        let mut relative = PathBuf::new();
        for component in Path::new(requested_path.trim_start_matches('/')).components() {
            match component {
                std::path::Component::Normal(part) => relative.push(part),
                _ => return None,
            }
        }
        relative
    };

    let mut full_path = project_dir.join(relative);
    if full_path.is_dir() {
        full_path = full_path.join("index.html");
    }

    full_path.exists().then_some(full_path)
}

fn content_type_for_path(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
    {
        "html" => "text/html; charset=utf-8",
        "js" => "text/javascript; charset=utf-8",
        "wasm" => "application/wasm",
        "json" => "application/json; charset=utf-8",
        "map" => "application/json; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "ico" => "image/x-icon",
        "txt" => "text/plain; charset=utf-8",
        _ => "application/octet-stream",
    }
}

fn write_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    body: &[u8],
) -> Result<()> {
    write_headers(stream, status, content_type, body.len())?;
    stream
        .write_all(body)
        .context("Failed to write HTTP response body")
}

fn write_headers(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    content_length: usize,
) -> Result<()> {
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {content_length}\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n"
    )
    .context("Failed to write HTTP response headers")
}
