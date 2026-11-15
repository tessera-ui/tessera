use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow};
use cargo_metadata::MetadataCommand;
use cargo_mobile2::{
    ChildHandle,
    android::{
        self, PROJECT_TEMPLATE as ANDROID_TEMPLATE_DIR,
        config::{Config as AndroidConfig, Metadata as AndroidMetadata, Raw as RawAndroidConfig},
        device::Device,
        env::Env as AndroidEnv,
        target::Target,
    },
    config::app::{App, Raw as RawAppConfig},
    opts::{FilterLevel, NoiseLevel, Profile},
    os::replace_path_separator,
    target::TargetTrait,
    util,
};
use clap::ValueEnum;
use handlebars::{Handlebars, handlebars_helper};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use serde::Deserialize;
use serde_json::{Value, json};
use tessera_build::{
    AssetBackend, application::AppConfig, load_tessera_config_from_dir, resolve_resource_dirs,
};

use crate::{
    call_guard, output,
    template::{write_if_changed, write_template_dir, write_template_file},
};

use super::{
    app_assets,
    assets::{
        PlatformAssetCandidate, collect_platform_asset_candidates, collect_resolved_packages,
        collect_used_platform_asset_paths, sync_assets,
    },
    find_package_dir,
};

const DEFAULT_ARCH: &str = "arm64";
const DEFAULT_MIN_SDK_VERSION: u32 = 24;
const ANDROID_ACTIVITY: &str = "com.tessera.TesseraGameActivity";
const DEFAULT_ANDROID_THEME_PARENT: &str = "Theme.AppCompat.Light.NoActionBar";

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum AndroidFormat {
    Apk,
    Aab,
}

#[derive(Debug)]
pub struct BuildOptions {
    pub release: bool,
    pub arch: String,
    pub package: Option<String>,
    pub format: AndroidFormat,
    pub profiling_output: Option<String>,
    pub debug_dirty_overlay: bool,
    pub asset_backend: Option<AssetBackend>,
}

#[derive(Debug, Clone)]
pub struct DevOptions {
    pub release: bool,
    pub arch: String,
    pub package: Option<String>,
    pub device: String,
    pub profiling_output: Option<String>,
    pub debug_dirty_overlay: bool,
    pub asset_backend: Option<AssetBackend>,
}

pub struct RustBuildOptions {
    pub release: bool,
    pub target: String,
    pub package: Option<String>,
    pub profiling_output: Option<String>,
    pub debug_dirty_overlay: bool,
    pub asset_backend: Option<AssetBackend>,
}

struct AndroidContext {
    package_dir: Option<PathBuf>,
    package_name: String,
    release: bool,
    format: AndroidFormat,
    arch: Option<String>,
    device: Option<String>,
    profiling_output: Option<String>,
    debug_dirty_overlay: bool,
    asset_backend: AssetBackend,
    config: AndroidConfig,
    metadata: AndroidMetadata,
    identity: AppConfig,
    version: String,
    version_code: u32,
}

#[derive(Debug)]
struct AndroidPlugin {
    module: String,
    source_dir: PathBuf,
}

struct AndroidContextOpts {
    release: bool,
    arch: Option<String>,
    package: Option<String>,
    device: Option<String>,
    format: Option<AndroidFormat>,
    profiling_output: Option<String>,
    debug_dirty_overlay: bool,
    asset_backend_override: Option<AssetBackend>,
}

impl AndroidContext {
    fn from_init() -> Result<Self> {
        Self::from_opts(AndroidContextOpts {
            release: false,
            arch: None,
            package: None,
            device: None,
            format: None,
            profiling_output: None,
            debug_dirty_overlay: false,
            asset_backend_override: None,
        })
    }
    fn from_build_opts(opts: BuildOptions) -> Result<Self> {
        Self::from_opts(AndroidContextOpts {
            release: opts.release,
            arch: Some(opts.arch),
            package: opts.package,
            device: None,
            format: Some(opts.format),
            profiling_output: opts.profiling_output,
            debug_dirty_overlay: opts.debug_dirty_overlay,
            asset_backend_override: opts.asset_backend,
        })
    }

    fn from_dev_opts(opts: DevOptions) -> Result<Self> {
        Self::from_opts(AndroidContextOpts {
            release: opts.release,
            arch: Some(opts.arch),
            package: opts.package,
            device: Some(opts.device),
            format: None,
            profiling_output: opts.profiling_output,
            debug_dirty_overlay: opts.debug_dirty_overlay,
            asset_backend_override: opts.asset_backend,
        })
    }

    fn from_rust_build_opts(opts: RustBuildOptions) -> Result<Self> {
        Self::from_opts(AndroidContextOpts {
            release: opts.release,
            arch: None,
            package: opts.package,
            device: None,
            format: None,
            profiling_output: opts.profiling_output,
            debug_dirty_overlay: opts.debug_dirty_overlay,
            asset_backend_override: opts.asset_backend,
        })
    }

    fn from_opts(opts: AndroidContextOpts) -> Result<Self> {
        let AndroidContextOpts {
            release,
            arch,
            package,
            device,
            format,
            profiling_output,
            debug_dirty_overlay,
            asset_backend_override,
        } = opts;

        let package_dir = package
            .as_deref()
            .and_then(|pkg| find_package_dir(pkg).ok());

        let manifest = if let Some(dir) = &package_dir {
            Manifest::load_from(dir)?
        } else {
            Manifest::load()?
        };

        manifest.reject_platform_metadata()?;
        let manifest_package = manifest.package_name();

        let package_name = package.or(manifest_package).ok_or_else(|| {
            anyhow!(
                "Unable to determine workspace package. Provide --package or run in a package directory."
            )
        })?;
        let format = format.unwrap_or(AndroidFormat::Apk);
        let arch = arch.or_else(|| Some(DEFAULT_ARCH.to_string()));

        let root_dir = package_dir
            .clone()
            .unwrap_or_else(|| PathBuf::from("."))
            .canonicalize()
            .with_context(|| "Failed to resolve project root")?;
        let identity = AppConfig::load(&root_dir, &package_name)?;
        let asset_backend = asset_backend_override.unwrap_or(AssetBackend::Platform);
        let cargo_metadata = MetadataCommand::new()
            .manifest_path(root_dir.join("Cargo.toml"))
            .exec()
            .context("Failed to resolve Cargo target directory")?;
        let version = &cargo_metadata
            .packages
            .iter()
            .find(|package| package.name.as_str() == package_name)
            .context("Application package is missing from Cargo metadata")?
            .version;
        let version_code = android_version_code(version.major, version.minor, version.patch)?;
        let version = version.to_string();
        let target_dir = cargo_metadata.target_directory.into_std_path_buf();

        let lib_name = manifest
            .lib_name()
            .unwrap_or_else(|| package_name.replace('-', "_"));
        let raw_app = RawAppConfig {
            name: package_name.clone(),
            lib_name: Some(lib_name),
            stylized_name: Some(identity.name.clone()),
            identifier: identity.identifier.clone(),
            asset_dir: None,
            template_pack: None,
        };
        let app = App::from_raw(root_dir, raw_app)
            .context("Failed to build Android app metadata")?
            .with_target_dir_resolver(move |triple, profile| {
                target_dir.join(triple).join(profile.as_str())
            });

        let raw_android = RawAndroidConfig {
            min_sdk_version: Some(DEFAULT_MIN_SDK_VERSION),
            project_dir: Some("gen/android".to_string()),
            no_default_features: None,
            features: None,
            logcat_filter_specs: Vec::new(),
        };
        let config = AndroidConfig::from_raw(app, Some(raw_android))
            .context("Failed to build Android config")?;
        let profiling_output = profiling_output
            .map(|path| normalize_android_profiling_output_path(&path, config.app().identifier()));

        let mut cargo_args = Vec::new();
        let mut cargo_features = Vec::new();
        let mut env_vars = HashMap::new();
        if let Some(path) = profiling_output.as_ref() {
            cargo_features.push("tessera-ui/profiling".to_string());
            env_vars.insert("TESSERA_PROFILING_OUTPUT".to_string(), path.clone());
            cargo_args.push("--config".to_string());
            cargo_args.push(format!(
                "env.TESSERA_PROFILING_OUTPUT={}",
                toml::Value::String(path.clone())
            ));
        }
        if debug_dirty_overlay {
            cargo_features.push("tessera-ui/debug-dirty-overlay".to_string());
        }
        env_vars.insert(
            "TESSERA_ASSET_BACKEND".to_string(),
            asset_backend.as_str().to_string(),
        );

        let metadata = AndroidMetadata {
            cargo_args: (!cargo_args.is_empty()).then_some(cargo_args),
            features: (!cargo_features.is_empty()).then_some(cargo_features),
            env_vars: (!env_vars.is_empty()).then_some(env_vars),
            ..AndroidMetadata::default()
        };

        Ok(Self {
            package_dir,
            package_name,
            release,
            format,
            arch,
            device,
            profiling_output,
            debug_dirty_overlay,
            asset_backend,
            config,
            metadata,
            identity,
            version,
            version_code,
        })
    }

    fn profile(&self) -> Profile {
        if self.release {
            Profile::Release
        } else {
            Profile::Debug
        }
    }

    fn targets(&self) -> Result<Vec<&'static Target<'static>>> {
        if let Some(arch) = &self.arch {
            let arch = arch.to_ascii_lowercase();
            let target = Target::for_name(&arch)
                .or_else(|| Target::for_arch(&arch))
                .ok_or_else(|| {
                    anyhow!(
                        "Unknown target '{}'. Supported targets: {}",
                        arch,
                        Target::name_list().join(", ")
                    )
                })?;
            Ok(vec![target])
        } else {
            Ok(Target::all().iter().map(|(_, target)| target).collect())
        }
    }

    fn target_by_name_or_triple(&self, value: &str) -> Result<&'static Target<'static>> {
        Target::for_name(value)
            .or_else(|| Target::for_arch(value))
            .or_else(|| {
                Target::all()
                    .iter()
                    .map(|(_, target)| target)
                    .find(|target| target.triple == value)
            })
            .ok_or_else(|| {
                anyhow!(
                    "Unknown target '{}'. Supported targets: {}",
                    value,
                    Target::all()
                        .iter()
                        .map(|(_, target)| target)
                        .map(|target| target.triple)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }
}

// Gradle checks each target when it invokes `rust-build`.
fn run_android_call_guard(ctx: &AndroidContext, target: &str) -> Result<()> {
    let target = ctx.target_by_name_or_triple(target)?.triple;
    let features = ctx.metadata.features().unwrap_or_default();
    call_guard::run(call_guard::CheckOptions {
        target_selection: call_guard::TargetSelection::lib_only(),
        features: call_guard::FeatureSelection::Selected {
            features,
            no_default_features: ctx.metadata.no_default_features(),
        },
        release: ctx.release,
        ..call_guard::CheckOptions::new(Some(&ctx.package_name), Some(target))
    })
}

fn android_version_code(major: u64, minor: u64, patch: u64) -> Result<u32> {
    anyhow::ensure!(
        major < 2100 && minor < 1000 && patch < 1000,
        "Android requires Cargo version major < 2100, minor < 1000 and patch < 1000"
    );
    Ok((major * 1_000_000 + minor * 1_000 + patch + 1) as u32)
}

fn normalize_android_profiling_output_path(path: &str, app_identifier: &str) -> String {
    let path = path.trim().replace('\\', "/");
    if path.is_empty() {
        return format!("/data/user/0/{app_identifier}/files/tessera-profiler.jsonl");
    }
    if path.starts_with('/') {
        return path;
    }
    if path.starts_with("data/") {
        return format!("/{path}");
    }
    format!("/data/user/0/{app_identifier}/{path}")
}

fn android_runtime_env(ctx: &AndroidContext) -> Result<AndroidEnv> {
    Ok(AndroidEnv::new()?
        .with_command_var(
            "TESSERA_ASSET_BACKEND",
            Some(ctx.asset_backend.as_str().into()),
        )
        .with_command_var(
            "TESSERA_PROFILING_OUTPUT",
            ctx.profiling_output.as_ref().map(Into::into),
        )
        .with_command_var(
            "TESSERA_DEBUG_DIRTY_OVERLAY",
            ctx.debug_dirty_overlay.then(|| "1".into()),
        ))
}

/// Generates the Android project, or regenerates its editable files with `force`.
pub fn init(force: bool, skip_targets_install: bool) -> Result<()> {
    let ctx = AndroidContext::from_init()?;
    let project_dir = ctx.config.project_dir();
    let project_exists = ctx.config.project_dir_exists();
    if project_exists && !force {
        output::warn(format!(
            "android project already exists at {}; pass --force to regenerate its editable files",
            display_path(&project_dir)
        ));
    }

    if !skip_targets_install {
        Target::install_all().map_err(|err| anyhow!("Failed to install Android targets: {err}"))?;
    }

    if !project_exists || force {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        register_helpers(&mut handlebars);
        write_template_dir(
            &ANDROID_TEMPLATE_DIR,
            project_dir.as_path(),
            &handlebars,
            &android_template_data(&ctx)?,
        )?;
    }
    sync_android_project(&ctx)?;

    if !project_exists || force {
        output::status(
            "Generated",
            format!("android project at {}", display_path(&project_dir)),
        );
    }
    Ok(())
}

fn register_helpers(handlebars: &mut Handlebars<'static>) {
    handlebars_helper!(quote_and_join: |list: Vec<String>| {
        list.iter()
            .map(|s| format!("\"{}\"", s))
            .collect::<Vec<_>>()
            .join(", ")
    });

    handlebars_helper!(quote_and_join_colon_prefix: |list: Vec<String>| {
        list.iter()
            .map(|s| format!("\":{}\"", s))
            .collect::<Vec<_>>()
            .join(", ")
    });

    handlebars.register_helper("quote-and-join", Box::new(quote_and_join));
    handlebars.register_helper(
        "quote-and-join-colon-prefix",
        Box::new(quote_and_join_colon_prefix),
    );
}

pub fn build(opts: BuildOptions) -> Result<()> {
    let ctx = AndroidContext::from_build_opts(opts)?;
    if !ctx.config.project_dir_exists() {
        return Err(anyhow!(
            "Android project not initialized. Run `cargo tessera android init` first."
        ));
    }
    sync_android_project(&ctx)?;
    let env = android_runtime_env(&ctx)?;
    let targets = ctx.targets()?;
    let profile = ctx.profile();

    output::status(
        "Building",
        format!(
            "android artifact ({}, targets: {}, release: {})",
            ctx.package_name,
            targets
                .iter()
                .map(|t| t.triple)
                .collect::<Vec<_>>()
                .join(", "),
            if ctx.release { "yes" } else { "no" }
        ),
    );
    if let Some(path) = &ctx.profiling_output {
        output::status("Profiling", format!("enabled ({path})"));
    }
    if ctx.debug_dirty_overlay {
        output::status("DirtyOverlay", "enabled");
    }
    output::status(
        "Assets",
        format!("backend `{}`", ctx.asset_backend.as_str()),
    );

    let outputs = match ctx.format {
        AndroidFormat::Apk => android::apk::build(
            &ctx.config,
            &env,
            NoiseLevel::Polite,
            profile,
            targets,
            false,
        )?,
        AndroidFormat::Aab => android::aab::build(
            &ctx.config,
            &env,
            NoiseLevel::Polite,
            profile,
            targets,
            false,
        )?,
    };

    output::status("Finished", "android build");
    output::status("Package", ctx.package_name.clone());
    for out in outputs {
        output::status("Artifact", display_path(&out));
    }
    Ok(())
}

pub fn dev(opts: DevOptions) -> Result<()> {
    let ctx = AndroidContext::from_dev_opts(opts.clone())?;
    if !ctx.config.project_dir_exists() {
        return Err(anyhow!(
            "Android project not initialized. Run `cargo tessera android init` first."
        ));
    }
    let device_id = ctx
        .device
        .clone()
        .ok_or_else(|| anyhow!("--device <adb_serial> is required for android dev"))?;
    sync_android_project(&ctx)?;

    output::status(
        "Running",
        format!(
            "android app {} (release: {})",
            ctx.package_name,
            if ctx.release { "yes" } else { "no" }
        ),
    );
    if let Some(path) = &ctx.profiling_output {
        output::status("Profiling", format!("enabled ({path})"));
    }
    if ctx.debug_dirty_overlay {
        output::status("DirtyOverlay", "enabled");
    }
    output::status(
        "Assets",
        format!("backend `{}`", ctx.asset_backend.as_str()),
    );

    output::status("Watching", "for file changes");

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
        if let Ok(event) = res
            && matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            )
        {
            let _ = tx.send(());
        }
    })?;

    let watch_dir = ctx
        .package_dir
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let src_path = watch_dir.join("src");
    if src_path.exists() {
        watcher.watch(&src_path, RecursiveMode::Recursive)?;
    } else {
        return Err(anyhow!(
            "Source directory not found: {}",
            src_path.display()
        ));
    }

    for file in ["Cargo.toml", "build.rs", "tessera.toml"] {
        let path = watch_dir.join(file);
        if path.exists() {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }
    }
    if let Some(icon) = &ctx.identity.icon {
        watcher.watch(icon, RecursiveMode::NonRecursive)?;
    }
    for directory in collect_resource_dirs(ctx.config.app().root_dir(), ctx.package_dir.as_ref())? {
        watcher.watch(&directory, RecursiveMode::Recursive)?;
    }

    let mut run_child: Option<ChildHandle> = None;
    let mut pending_change = true;
    let mut last_change = Instant::now() - Duration::from_secs(1);
    let debounce_window = Duration::from_millis(300);

    loop {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(_) => {
                pending_change = true;
                last_change = Instant::now();

                if let Some(active_run) = run_child.take() {
                    output::status("Canceling", "in-progress Android deploy due to changes");
                    let _ = active_run.kill();
                    let _ = active_run.wait();
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        if pending_change && run_child.is_none() && last_change.elapsed() >= debounce_window {
            while rx.try_recv().is_ok() {}

            output::status("Deploying", "building and installing on device");

            let deploy = || {
                let ctx = AndroidContext::from_dev_opts(opts.clone())?;
                sync_android_project(&ctx)?;
                let env = android_runtime_env(&ctx)?;
                run_once(&ctx, &env, ctx.profile(), &device_id, NoiseLevel::Polite)
            };
            match deploy() {
                Ok(child) => {
                    run_child = Some(child);
                    pending_change = false;
                }
                Err(e) => {
                    output::error(format!("failed to start deploy: {e}"));
                }
            }
        }

        if let Some(active_run) = run_child.take() {
            match active_run.try_wait() {
                Ok(Some(output)) => {
                    if !output.status.success() {
                        output::warn("deploy failed; waiting for changes");
                    } else {
                        output::status("Running", "app should be running on the device");
                    }
                }
                Ok(None) => {
                    run_child = Some(active_run);
                }
                Err(err) => {
                    output::warn(format!("failed to check deploy status: {err}"));
                }
            }
        }
    }

    if let Some(run) = run_child {
        let _ = run.kill();
        let _ = run.wait();
    }

    Ok(())
}

pub fn rust_build(opts: RustBuildOptions) -> Result<()> {
    let target_name = opts.target.clone();
    let ctx = AndroidContext::from_rust_build_opts(opts)?;
    run_android_call_guard(&ctx, &target_name)?;
    if !ctx.config.project_dir_exists() {
        return Err(anyhow!(
            "Android project not initialized. Run `cargo tessera android init` first."
        ));
    }
    let env = android_runtime_env(&ctx)?;
    let target = ctx.target_by_name_or_triple(&target_name)?;
    let profile = ctx.profile();
    if let Some(path) = &ctx.profiling_output {
        output::status("Profiling", format!("enabled ({path})"));
    }
    if ctx.debug_dirty_overlay {
        output::status("DirtyOverlay", "enabled");
    }
    output::status(
        "Assets",
        format!("backend `{}`", ctx.asset_backend.as_str()),
    );

    target.build(
        &ctx.config,
        &ctx.metadata,
        &env,
        NoiseLevel::Polite,
        true,
        profile,
    )?;
    sync_android_assets_with_binary_pruning(&ctx, profile)?;

    Ok(())
}

fn run_once(
    ctx: &AndroidContext,
    env: &AndroidEnv,
    profile: Profile,
    device_id: &str,
    noise_level: NoiseLevel,
) -> Result<ChildHandle> {
    let device = find_device(env, device_id)?;

    let filter = Some(match noise_level {
        NoiseLevel::Polite => FilterLevel::Info,
        NoiseLevel::LoudAndProud => FilterLevel::Debug,
        NoiseLevel::FranklyQuitePedantic => FilterLevel::Verbose,
    });

    device
        .run(
            &ctx.config,
            env,
            noise_level,
            profile,
            filter,
            false,
            false,
            ANDROID_ACTIVITY.to_string(),
        )
        .map_err(|err| anyhow!("Failed to run on device: {err}"))
}

fn find_device<'a>(env: &'a AndroidEnv, serial: &str) -> Result<Device<'a>> {
    let devices =
        android::adb::device_list(env).map_err(|err| anyhow!("Failed to list devices: {err}"))?;
    devices
        .into_iter()
        .find(|d| d.serial_no() == serial)
        .ok_or_else(|| anyhow!("Device {serial} not found. Run `adb devices` to list targets."))
}

fn display_path(path: &Path) -> String {
    let text = path.as_os_str().to_string_lossy().into_owned();
    if let Some(rest) = text.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", rest)
    } else if let Some(rest) = text.strip_prefix(r"\\?\") {
        rest.to_string()
    } else {
        text
    }
}

/// Values rendered into the editable project files at initialization.
fn android_template_data(ctx: &AndroidContext) -> Result<Value> {
    let mut abi_list = Vec::new();
    let mut arch_list = Vec::new();
    let mut target_list = Vec::new();
    for target in Target::all().iter().map(|(_, target)| target) {
        abi_list.push(target.abi.to_string());
        arch_list.push(target.arch.to_string());
        target_list.push(target.triple.to_string());
    }
    Ok(json!({
        "app": serde_json::to_value(ctx.config.app())
            .context("Failed to serialize app metadata for templates")?,
        "android-app-theme-parent": DEFAULT_ANDROID_THEME_PARENT,
        "abi_list": abi_list,
        "arch_list": arch_list,
        "target_list": target_list,
    }))
}

/// Regenerates the files cargo-tessera maintains inside the Android project.
fn sync_android_project(ctx: &AndroidContext) -> Result<()> {
    let project_dir = ctx.config.project_dir();
    let plugins = collect_android_plugins(ctx)?;

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    register_helpers(&mut handlebars);
    let data = android_template_data(ctx)?;
    for template in [
        "buildSrc/build.gradle.kts.hbs",
        "buildSrc/src/main/kotlin/BuildTask.kt.hbs",
        "buildSrc/src/main/kotlin/RustPlugin.kt.hbs",
    ] {
        write_template_file(
            &ANDROID_TEMPLATE_DIR,
            Path::new(template),
            project_dir.as_path(),
            &handlebars,
            &data,
        )?;
    }

    let rust_root = util::relativize_path(ctx.config.app().root_dir(), project_dir.join("app"));
    let rust_root = replace_path_separator(rust_root.into_os_string())
        .to_string_lossy()
        .into_owned();
    let mut properties = String::from(GENERATED_HEADER.replace("//", "#"));
    for (key, value) in [
        ("applicationId", ctx.config.app().identifier().to_string()),
        ("versionCode", ctx.version_code.to_string()),
        ("versionName", ctx.version.clone()),
        ("libName", ctx.config.app().lib_name()),
        ("rustRoot", rust_root),
    ] {
        properties.push_str(&format!("{key}={}\n", escape_property(&value)));
    }
    write_if_changed(
        &project_dir.join("tessera.properties"),
        properties.as_bytes(),
    )?;

    let mut settings = GENERATED_HEADER.to_string();
    let mut dependencies = format!("{GENERATED_HEADER}dependencies {{\n");
    for plugin in &plugins {
        settings.push_str(&format!(
            "include ':plugins:{0}'\nproject(':plugins:{0}').projectDir = file('{PLUGIN_DIR}/{0}')\n",
            plugin.module
        ));
        dependencies.push_str(&format!(
            "    add(\"implementation\", project(\":plugins:{}\"))\n",
            plugin.module
        ));
    }
    dependencies.push_str("}\n");
    write_if_changed(
        &project_dir.join("tessera.settings.gradle"),
        settings.as_bytes(),
    )?;
    write_if_changed(
        &project_dir.join("tessera.build.gradle.kts"),
        dependencies.as_bytes(),
    )?;
    sync_android_plugins(&project_dir, &plugins)?;

    let resources = project_dir.join("app/src/main/tessera-res");
    if resources.exists() {
        fs::remove_dir_all(&resources)
            .with_context(|| format!("Failed to remove {}", resources.display()))?;
    }
    app_assets::android(&ctx.identity, &resources)?;

    warn_unreferenced_generated_files(&project_dir);
    Ok(())
}

const GENERATED_HEADER: &str =
    "// @generated by cargo-tessera from tessera.toml and Cargo.toml; do not edit.\n";
const PLUGIN_DIR: &str = "tessera-plugins";

fn escape_property(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('=', "\\=")
        .replace(':', "\\:")
}

/// Warns when an edited project no longer reads a maintained file.
fn warn_unreferenced_generated_files(project_dir: &Path) {
    for (file, generated, effect) in [
        (
            "settings.gradle",
            "tessera.settings.gradle",
            "plugin modules are not included",
        ),
        (
            "app/build.gradle.kts",
            "tessera.properties",
            "the application ID, version, and Rust project location from tessera.toml and Cargo.toml are not applied",
        ),
        (
            "app/build.gradle.kts",
            "tessera.build.gradle.kts",
            "plugin module dependencies are not added",
        ),
    ] {
        let references = fs::read_to_string(project_dir.join(file))
            .is_ok_and(|contents| contents.contains(generated));
        if !references {
            output::warn(format!(
                "gen/android/{file} does not reference {generated}, so {effect}; \
                 run `cargo tessera android init --force` to regenerate it"
            ));
        }
    }
}

fn collect_android_plugins(ctx: &AndroidContext) -> Result<Vec<AndroidPlugin>> {
    let packages =
        collect_resolved_packages(ctx.config.app().root_dir(), ctx.package_dir.as_ref())?;
    let mut plugins = Vec::new();
    let mut modules = HashSet::new();

    for package in packages.into_iter().filter(|package| !package.is_root) {
        let Some(config) = load_tessera_config_from_dir(&package.dir)? else {
            continue;
        };
        let module = config
            .plugin
            .as_ref()
            .and_then(|plugin| plugin.android.as_ref())
            .map(|android| android.module.clone());
        let Some(module) = module else {
            continue;
        };

        if !modules.insert(module.clone()) {
            return Err(anyhow!(
                "Duplicate Android plugin module '{}' found in package `{}`",
                module,
                package.name
            ));
        }

        let source_dir = package.dir.join("android");
        if !source_dir.is_dir() {
            return Err(anyhow!(
                "Android module directory not found for plugin '{}': {}",
                module,
                source_dir.display()
            ));
        }
        plugins.push(AndroidPlugin { module, source_dir });
    }

    plugins.sort_by(|a, b| a.module.cmp(&b.module));
    Ok(plugins)
}

fn collect_resource_dirs(root_dir: &Path, package_dir: Option<&PathBuf>) -> Result<Vec<PathBuf>> {
    let packages = collect_resolved_packages(root_dir, package_dir)?;
    let mut dirs = Vec::new();
    for package in packages {
        let config = load_tessera_config_from_dir(&package.dir)?;
        dirs.extend(
            resolve_resource_dirs(&package.dir, config.as_ref()).filter(|dir| dir.is_dir()),
        );
    }
    dirs.sort();
    dirs.dedup();
    Ok(dirs)
}

fn sync_android_assets_with_binary_pruning(ctx: &AndroidContext, profile: Profile) -> Result<()> {
    if ctx.asset_backend != AssetBackend::Platform {
        return Ok(());
    }

    let packages =
        collect_resolved_packages(ctx.config.app().root_dir(), ctx.package_dir.as_ref())?;
    let tessera_assets_root = ctx.config.project_dir().join("app/src/main/assets/tessera");
    let candidates = collect_platform_asset_candidates(&packages, &tessera_assets_root)?;
    let binary_paths = collect_existing_android_libs(ctx, profile);
    if candidates.is_empty() || binary_paths.is_empty() {
        return sync_android_assets_internal(ctx, None, Some(&candidates));
    }

    let used_paths = collect_used_platform_asset_paths(&binary_paths, &candidates)?;
    sync_android_assets_internal(ctx, Some(&used_paths), Some(&candidates))
}

fn sync_android_assets_internal(
    ctx: &AndroidContext,
    used_platform_paths: Option<&HashSet<String>>,
    precomputed_candidates: Option<&[PlatformAssetCandidate]>,
) -> Result<()> {
    let root = ctx.config.project_dir().join("app/src/main/assets/tessera");
    let collected;
    let candidates = if ctx.asset_backend != AssetBackend::Platform {
        &[]
    } else if let Some(candidates) = precomputed_candidates {
        candidates
    } else {
        let packages =
            collect_resolved_packages(ctx.config.app().root_dir(), ctx.package_dir.as_ref())?;
        collected = collect_platform_asset_candidates(&packages, &root)?;
        &collected
    };
    let (count, bytes) = sync_assets(&root, candidates, used_platform_paths)?;
    output::status(
        "Assets",
        format!(
            "delivered {count} assets ({bytes} bytes) to {}",
            display_path(&root)
        ),
    );
    Ok(())
}

fn collect_existing_android_libs(ctx: &AndroidContext, profile: Profile) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    let so_name = ctx.config.so_name();
    for target in Target::all().iter().map(|(_, target)| target) {
        let path = ctx
            .config
            .app()
            .target_dir(target.triple, profile)
            .join(&so_name);
        if path.is_file() {
            paths.push(path);
        }
    }
    paths.sort();
    paths.dedup();
    paths
}

// Plugin sources are copied because Gradle writes build output into each
// module directory, which may be a read-only Cargo registry checkout.
fn sync_android_plugins(project_dir: &Path, plugins: &[AndroidPlugin]) -> Result<()> {
    let plugins_dir = project_dir.join(PLUGIN_DIR);
    if plugins_dir.exists() {
        fs::remove_dir_all(&plugins_dir)
            .with_context(|| format!("Failed to remove {}", plugins_dir.display()))?;
    }
    if plugins.is_empty() {
        return Ok(());
    }
    fs::create_dir_all(&plugins_dir)
        .with_context(|| format!("Failed to create {}", plugins_dir.display()))?;

    for plugin in plugins {
        let target_dir = plugins_dir.join(&plugin.module);
        copy_dir_all(&plugin.source_dir, &target_dir).with_context(|| {
            format!(
                "Failed to copy plugin module {} to {}",
                plugin.module,
                target_dir.display()
            )
        })?;
    }
    Ok(())
}

fn copy_dir_all(source: &Path, target: &Path) -> Result<()> {
    fs::create_dir_all(target).with_context(|| format!("Failed to create {}", target.display()))?;
    for entry in
        fs::read_dir(source).with_context(|| format!("Failed to read {}", source.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let dest = target.join(entry.file_name());
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            copy_dir_all(&path, &dest)?;
        } else {
            fs::copy(&path, &dest).with_context(|| format!("Failed to copy {}", path.display()))?;
        }
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
struct Manifest {
    package: Option<PackageSection>,
    lib: Option<LibSection>,
}

impl Manifest {
    fn load() -> Result<Self> {
        let contents = fs::read_to_string("Cargo.toml").context("Failed to read Cargo.toml")?;
        toml::from_str(&contents).context("Failed to parse Cargo.toml")
    }

    fn load_from(dir: &Path) -> Result<Self> {
        let cargo_path = dir.join("Cargo.toml");
        let contents = fs::read_to_string(&cargo_path)
            .with_context(|| format!("Failed to read Cargo.toml from {}", dir.display()))?;
        toml::from_str(&contents).context("Failed to parse Cargo.toml")
    }

    fn package_name(&self) -> Option<String> {
        self.package.as_ref().and_then(|p| p.name.clone())
    }

    fn lib_name(&self) -> Option<String> {
        self.lib.as_ref().and_then(|l| l.name.clone())
    }

    fn reject_platform_metadata(&self) -> Result<()> {
        let tessera = self
            .package
            .as_ref()
            .and_then(|package| package.metadata.as_ref())
            .and_then(|metadata| metadata.tessera.as_ref());
        if let Some(tessera) = tessera {
            reject_legacy_platform_metadata(tessera)?;
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct PackageSection {
    name: Option<String>,
    metadata: Option<MetadataSection>,
}

#[derive(Debug, Deserialize)]
struct LibSection {
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MetadataSection {
    tessera: Option<TesseraMetadata>,
}

#[derive(Debug, Deserialize)]
struct TesseraMetadata {
    android: Option<toml::Value>,
    ios: Option<toml::Value>,
}

fn reject_legacy_platform_metadata(metadata: &TesseraMetadata) -> Result<()> {
    anyhow::ensure!(
        metadata.android.is_none(),
        "`[package.metadata.tessera.android]` is no longer read. Edit the generated Android \
         project instead: `minSdk`, Gradle plugins, and dependencies in \
         gen/android/app/build.gradle.kts, permissions in \
         gen/android/app/src/main/AndroidManifest.xml. Pass `--arch` and `--format` on the \
         command line"
    );
    anyhow::ensure!(
        metadata.ios.is_none(),
        "`[package.metadata.tessera.ios]` is no longer read. Run `cargo tessera apple init` \
         and edit the generated Xcode project in gen/apple instead"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_versions_produce_positive_ordered_android_codes() {
        assert_eq!(android_version_code(0, 0, 0).unwrap(), 1);
        assert_eq!(android_version_code(1, 2, 3).unwrap(), 1_002_004);
        assert!(
            android_version_code(1, 999, 999).unwrap() < android_version_code(2, 0, 0).unwrap()
        );
        assert_eq!(android_version_code(2099, 999, 999).unwrap(), 2_100_000_000);
        for (major, minor, patch) in [(2100, 0, 0), (0, 1000, 0), (0, 0, 1000), (u64::MAX, 0, 0)] {
            assert!(android_version_code(major, minor, patch).is_err());
        }
    }
}
