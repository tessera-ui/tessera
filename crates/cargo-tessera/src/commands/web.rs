use std::{
    fs,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::{Sender, channel},
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use cargo_metadata::MetadataCommand;
use handlebars::Handlebars;
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde_json::json;
use tessera_build::{
    AssetBackend, application::AppConfig, load_tessera_config_from_dir, resolve_resource_dirs,
};
use wasm_bindgen_cli_support::Bindgen;

use crate::{call_guard, output};

use super::{
    app_assets,
    assets::{
        collect_platform_asset_candidates, collect_resolved_packages,
        collect_used_platform_asset_paths, sync_assets,
    },
    find_package_dir,
};

const INDEX_TEMPLATE: &str = include_str!("../../templates/basic/gen/web/index.html.hbs");
const WASM_BINDGEN_DEP: &str = "wasm-bindgen = \"0.2.130\"";
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
const NON_DESKTOP_MAIN_STUB: &str = r#"
#[cfg(any(target_family = "wasm", target_os = "android"))]
fn main() {}
"#;
const LEGACY_DESKTOP_MAIN_GUARD: &str = "#[cfg(not(target_family = \"wasm\"))]";
const DESKTOP_MAIN_GUARD: &str =
    "#[cfg(all(not(target_family = \"wasm\"), not(target_os = \"android\")))]";
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
    let identity = AppConfig::load(&project_dir, &metadata.package_name)?;
    sync_web_application(
        &identity,
        &metadata.version,
        &project_dir.join(WEB_HOST_DIR),
    )?;

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

pub fn build(release: bool, package: Option<&str>, features: &[String]) -> Result<()> {
    let project_dir = resolve_project_dir(package)?;
    let metadata = load_project_metadata(&project_dir)?;

    output::status("Building", format!("web app `{}`", metadata.package_name));
    build_project(&project_dir, &metadata, release, features)?;
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

    build_project(&project_dir, &metadata, release, &[])?;

    let listener = TcpListener::bind(("127.0.0.1", port))
        .with_context(|| format!("Failed to bind development server on port {port}"))?;
    listener
        .set_nonblocking(true)
        .context("Failed to configure non-blocking development server socket")?;
    let web_host_dir = project_dir.join(WEB_HOST_DIR);
    let _server = StaticFileServer::new(listener, web_host_dir.clone())?;
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

    let config = load_tessera_config_from_dir(&project_dir)?;
    let identity = AppConfig::load(&project_dir, &metadata.package_name)?;
    if let Some(icon) = identity.icon {
        watcher.watch(&icon, RecursiveMode::NonRecursive)?;
    }
    for directory in resolve_resource_dirs(&project_dir, config.as_ref()) {
        if directory.is_dir() {
            watcher.watch(&directory, RecursiveMode::Recursive)?;
        }
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
            match load_project_metadata(&project_dir)
                .and_then(|metadata| build_project(&project_dir, &metadata, release, &[]))
            {
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

fn web_build_command(project_dir: &Path, release: bool, features: &[String]) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("--manifest-path")
        .arg(project_dir.join("Cargo.toml"))
        .arg("--target")
        .arg(WASM_TARGET)
        .env("TESSERA_ASSET_BACKEND", AssetBackend::Platform.as_str());
    if release {
        for (name, default) in [
            ("CARGO_PROFILE_RELEASE_OPT_LEVEL", "z"),
            ("CARGO_PROFILE_RELEASE_CODEGEN_UNITS", "1"),
            ("CARGO_PROFILE_RELEASE_LTO", "true"),
        ] {
            cmd.env(name, std::env::var(name).unwrap_or_else(|_| default.into()));
        }
        cmd.arg("--release");
    }

    if !features.is_empty() {
        cmd.arg("--features").arg(features.join(","));
    }
    cmd
}

fn build_project(
    project_dir: &Path,
    metadata: &ProjectMetadata,
    release: bool,
    features: &[String],
) -> Result<()> {
    if release {
        let version = Command::new("wasm-opt")
            .arg("--version")
            .output()
            .context("Install Binaryen 110 or newer to build a web release")?;
        let major = String::from_utf8_lossy(&version.stdout)
            .split_whitespace()
            .nth(2)
            .and_then(|number| number.parse::<u32>().ok());
        ensure!(
            version.status.success() && major.is_some_and(|major| major >= 110),
            "Web releases require Binaryen 110 or newer to preserve WebAssembly table exports"
        );
    }
    let mut options =
        call_guard::CheckOptions::new(Some(&metadata.package_name), Some(WASM_TARGET));
    options.features = call_guard::FeatureSelection::Selected {
        features,
        no_default_features: false,
    };
    options.release = release;
    call_guard::run(options)?;
    ensure_index_html(project_dir, metadata)?;
    let identity = AppConfig::load(project_dir, &metadata.package_name)?;
    sync_web_application(
        &identity,
        &metadata.version,
        &project_dir.join(WEB_HOST_DIR),
    )?;

    let mut cmd = web_build_command(project_dir, release, features);
    let status = cmd
        .status()
        .context("Failed to run cargo build for web target")?;
    if !status.success() {
        bail!("Web build failed");
    }
    let output_dir = project_dir.join(WEB_OUTPUT_DIR);
    fs::create_dir_all(&output_dir)
        .with_context(|| format!("Failed to create {}", output_dir.display()))?;

    let profile = if release { "release" } else { "debug" };
    let wasm_path = metadata
        .target_dir
        .join(WASM_TARGET)
        .join(profile)
        .join(format!("{}.wasm", metadata.lib_name.replace('-', "_")));
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

    let packages = collect_resolved_packages(project_dir, None)?;
    let assets_root = project_dir.join(WEB_HOST_DIR).join("tessera");
    let candidates = collect_platform_asset_candidates(&packages, &assets_root)?;
    let wasm = output_dir.join(format!("{}_bg.wasm", metadata.lib_name));
    if release {
        output::status("Optimizing", "WebAssembly for size");
        let optimized = wasm.with_extension("optimized.wasm");
        let status = Command::new("wasm-opt")
            .arg(&wasm)
            .args([
                "-Oz",
                "--enable-bulk-memory",
                "--enable-reference-types",
                "--enable-multivalue",
                "--enable-sign-ext",
                "--enable-nontrapping-float-to-int",
            ])
            .arg("-o")
            .arg(&optimized)
            .status()
            .context("Failed to run wasm-opt")?;
        if !status.success() {
            bail!("wasm-opt failed");
        }
        fs::rename(optimized, &wasm)?;
    }
    let used = collect_used_platform_asset_paths(&[wasm], &candidates)?;
    let (_, total) = sync_assets(&assets_root, &candidates, Some(&used))?;
    output::status("Assets", format!("{} bytes delivered separately", total));

    if !project_dir.join(WEB_HOST_DIR).join("index.html").exists() {
        output::warn(
            "No index.html found; run `cargo tessera web init` to generate a minimal web host",
        );
    }

    Ok(())
}

struct ProjectMetadata {
    target_dir: PathBuf,
    package_name: String,
    project_name_snake: String,
    lib_name: String,
    version: String,
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

    let cargo_metadata = MetadataCommand::new()
        .manifest_path(&cargo_toml_path)
        .no_deps()
        .exec()
        .context("Failed to load cargo metadata for web build")?;
    let version = cargo_metadata
        .packages
        .iter()
        .find(|package| package.name.as_str() == package_name)
        .context("Web application package is missing from Cargo metadata")?
        .version
        .to_string();
    Ok(ProjectMetadata {
        target_dir: cargo_metadata.target_directory.into_std_path_buf(),
        package_name,
        project_name_snake,
        lib_name,
        version,
    })
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

fn sync_web_application(config: &AppConfig, version: &str, host: &Path) -> Result<()> {
    let tags = app_assets::web(config, version, host)?;
    let path = host.join("index.html");
    let original = fs::read_to_string(&path)?;
    let mut html = original.clone();
    let start_marker = "<!-- tessera-app -->";
    let end_marker = "<!-- /tessera-app -->";
    if let Some(start) = html.find(start_marker) {
        let end = html[start..]
            .find(end_marker)
            .context("Unclosed Tessera application metadata in index.html")?
            + start
            + end_marker.len();
        let end = end + usize::from(html[end..].starts_with('\n'));
        html.replace_range(start..end, "");
    }
    let lower = html.to_ascii_lowercase();
    if let Some(start) = lower.find("<title>") {
        let end = lower[start..]
            .find("</title>")
            .context("Unclosed title in index.html")?
            + start
            + "</title>".len();
        html.replace_range(start..end, "");
    }
    let end = html
        .to_ascii_lowercase()
        .find("</head>")
        .context("Web host index.html must contain </head>")?;
    let name = handlebars::html_escape(&config.name);
    html.insert_str(
        end,
        &format!("{start_marker}\n<title>{name}</title>\n{tags}\n{end_marker}\n"),
    );
    if html != original {
        fs::write(path, html)?;
    }
    Ok(())
}

fn ensure_pkg_web_ignored(project_dir: &Path) -> Result<bool> {
    let path = project_dir.join(".gitignore");
    let original = fs::read_to_string(&path).or_else(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Ok(String::new())
        } else {
            Err(error)
        }
    })?;
    let mut content = original.clone();
    for pattern in ["/gen/web/pkg-web", "/gen/web/tessera", "/gen/web/app"] {
        if !content.lines().any(|line| line.trim() == pattern) {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(pattern);
            content.push('\n');
        }
    }
    if content == original {
        return Ok(false);
    }
    fs::write(path, content)?;
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
    if content.contains(LEGACY_DESKTOP_MAIN_GUARD) {
        content = content.replace(LEGACY_DESKTOP_MAIN_GUARD, DESKTOP_MAIN_GUARD);
        changed = true;
    }
    if !content.contains(DESKTOP_MAIN_GUARD) {
        if let Some(pos) = content.find("fn main()") {
            content.insert_str(pos, &format!("{DESKTOP_MAIN_GUARD}\n"));
            changed = true;
        } else {
            output::warn(format!(
                "Skipping desktop guard insertion in {} because `fn main()` was not found",
                main_path.display()
            ));
        }
    }

    if content.contains(WASM_MAIN_STUB.trim()) {
        content = content.replace(WASM_MAIN_STUB.trim(), NON_DESKTOP_MAIN_STUB.trim());
        changed = true;
    }
    if !content.contains(NON_DESKTOP_MAIN_STUB.trim()) {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        content.push_str(NON_DESKTOP_MAIN_STUB.trim_start_matches('\n'));
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

struct StaticFileServer {
    shutdown: Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
}

impl StaticFileServer {
    fn new(listener: TcpListener, project_dir: PathBuf) -> Result<Self> {
        let (shutdown, stopped) = channel();
        let worker = thread::Builder::new()
            .name("tessera-web-server".into())
            .spawn(move || {
                loop {
                    if stopped.try_recv().is_ok() {
                        return;
                    }
                    match listener.accept() {
                        Ok((stream, _)) => {
                            if stream
                                .set_read_timeout(Some(Duration::from_millis(250)))
                                .is_err()
                                || stream
                                    .set_write_timeout(Some(Duration::from_millis(250)))
                                    .is_err()
                            {
                                continue;
                            }
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
            })?;
        Ok(Self {
            shutdown,
            worker: Some(worker),
        })
    }
}

impl Drop for StaticFileServer {
    fn drop(&mut self) {
        let _ = self.shutdown.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn handle_http_request(mut stream: TcpStream, project_dir: &Path) -> Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;
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

    let root = project_dir.canonicalize().ok()?;
    let full_path = full_path.canonicalize().ok()?;
    (full_path.starts_with(root) && full_path.is_file()).then_some(full_path)
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
        "webmanifest" => "application/manifest+json",
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn application_metadata_updates_preserve_the_web_host() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("index.html");
        fs::write(&path, "<!doctype html><html><head><title>Old</title><style>canvas{display:block}</style></head><body><canvas></canvas></body></html>").unwrap();
        let mut config = AppConfig::load(directory.path(), "reader").unwrap();
        config.name = "Reader & Notes".into();
        sync_web_application(&config, "1.2.3", directory.path()).unwrap();
        let first = fs::read_to_string(&path).unwrap();
        sync_web_application(&config, "1.2.3", directory.path()).unwrap();
        assert_eq!(first, fs::read_to_string(&path).unwrap());
        assert!(first.contains("<title>Reader &amp; Notes</title>"));
        assert!(first.contains("<style>canvas{display:block}</style>"));
        assert!(first.contains("<body><canvas></canvas></body>"));
        config.name = "New Name".into();
        sync_web_application(&config, "1.2.3", directory.path()).unwrap();
        let html = fs::read_to_string(path).unwrap();
        assert_eq!(html.matches("<title>").count(), 1);
        assert!(html.contains("<title>New Name</title>"));
    }

    #[test]
    fn web_build_forwards_selected_features_without_changing_default_builds() {
        for (release, features) in [
            (false, Vec::new()),
            (true, vec!["diagnostics".to_string(), "extra".to_string()]),
        ] {
            let command = web_build_command(Path::new("app"), release, &features);
            let actual: Vec<_> = command
                .get_args()
                .map(|arg| arg.to_string_lossy().into_owned())
                .collect();
            let mut expected = vec![
                "build".to_string(),
                "--manifest-path".to_string(),
                Path::new("app")
                    .join("Cargo.toml")
                    .to_string_lossy()
                    .into_owned(),
                "--target".to_string(),
                WASM_TARGET.to_string(),
            ];
            if release {
                expected.push("--release".to_string());
            }
            if !features.is_empty() {
                expected.extend(["--features".to_string(), "diagnostics,extra".to_string()]);
            }
            assert_eq!(actual, expected);
            assert!(
                command
                    .get_envs()
                    .any(|(key, value)| key == "TESSERA_ASSET_BACKEND"
                        && value == Some(std::ffi::OsStr::new("platform")))
            );
        }
    }

    struct Fixture(tempfile::TempDir);

    impl Fixture {
        fn new() -> Self {
            let directory = tempfile::tempdir().expect("isolated web fixture");
            let root = directory.path();
            fs::create_dir_all(root.join("web/nested")).expect("create fixture");
            fs::write(root.join("web/index.html"), "home").expect("write index");
            fs::write(root.join("web/nested/index.html"), "nested").expect("write nested index");
            fs::write(root.join("secret.txt"), "outside web root").expect("write outside file");
            Self(directory)
        }
    }

    #[test]
    fn static_server_releases_its_listener_when_the_owner_is_dropped() {
        let listener = TcpListener::bind(("127.0.0.1", 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let fixture = Fixture::new();
        let server = StaticFileServer::new(listener, fixture.0.path().to_path_buf()).unwrap();
        drop(server);
        assert!(TcpStream::connect(address).is_err());
    }

    #[test]
    fn main_guards_migrate_legacy_templates_and_stay_idempotent() {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.path().join("src")).expect("create src");
        let main = fixture.0.path().join("src/main.rs");
        fs::write(
            &main,
            format!("{LEGACY_DESKTOP_MAIN_GUARD}\nfn main() {{ run(); }}\n{WASM_MAIN_STUB}"),
        )
        .expect("write legacy main");
        assert!(ensure_main_has_wasm_cfg(fixture.0.path()).expect("migrate main"));
        let content = fs::read_to_string(&main).expect("read main");
        assert!(content.contains(DESKTOP_MAIN_GUARD));
        assert!(content.contains(NON_DESKTOP_MAIN_STUB.trim()));
        assert!(!ensure_main_has_wasm_cfg(fixture.0.path()).expect("idempotent migration"));
        assert_eq!(fs::read_to_string(main).expect("read main again"), content);
    }

    #[test]
    fn static_files_stay_inside_web_root() {
        let fixture = Fixture::new();
        let root = fixture.0.path().join("web");
        assert_eq!(
            resolve_served_file(&root, "/"),
            root.join("index.html").canonicalize().ok()
        );
        assert_eq!(
            resolve_served_file(&root, "/nested/"),
            root.join("nested/index.html").canonicalize().ok()
        );
        for path in ["/../secret.txt", "/nested/../../secret.txt", "/missing"] {
            assert!(resolve_served_file(&root, path).is_none(), "{path}");
        }
        fs::create_dir(root.join("empty")).expect("create empty directory");
        assert!(resolve_served_file(&root, "/empty").is_none());
    }

    #[cfg(unix)]
    #[test]
    fn static_files_reject_symlinks_outside_web_root() {
        let fixture = Fixture::new();
        let root = fixture.0.path().join("web");
        std::os::unix::fs::symlink(fixture.0.path().join("secret.txt"), root.join("secret.txt"))
            .expect("create symlink");
        assert!(resolve_served_file(&root, "/secret.txt").is_none());
    }

    #[cfg(windows)]
    #[test]
    fn static_files_reject_junctions_outside_web_root() {
        let fixture = Fixture::new();
        let root = fixture.0.path().join("web");
        let outside = fixture.0.path().join("outside");
        fs::create_dir(&outside).expect("create outside directory");
        fs::write(outside.join("index.html"), "private").expect("write outside index");
        let junction = root.join("escape");
        let output = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&junction)
            .arg(&outside)
            .output()
            .expect("create junction");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(resolve_served_file(&root, "/escape").is_none());
        fs::remove_dir(junction).expect("remove junction");
    }
}
