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
        config::{
            Config as AndroidConfig, DEFAULT_VULKAN_VALIDATION, Metadata as AndroidMetadata,
            Raw as RawAndroidConfig,
        },
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
    template::{write_template_dir, write_template_file},
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
const DEFAULT_ANDROID_ACTIVITY: &str = ".TesseraGameActivity";
const DEFAULT_ANDROID_THEME_PARENT: &str = "Theme.AppCompat.Light.NoActionBar";

#[derive(Debug, Copy, Clone, PartialEq, Eq, ValueEnum)]
pub enum AndroidFormat {
    Apk,
    Aab,
}

impl AndroidFormat {
    fn from_config(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "apk" => Some(AndroidFormat::Apk),
            "aab" => Some(AndroidFormat::Aab),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct BuildOptions {
    pub release: bool,
    pub arch: Option<String>,
    pub package: Option<String>,
    pub format: Option<AndroidFormat>,
    pub profiling_output: Option<String>,
    pub debug_dirty_overlay: bool,
    pub asset_backend: Option<AssetBackend>,
}

#[derive(Debug, Clone)]
pub struct DevOptions {
    pub release: bool,
    pub arch: Option<String>,
    pub package: Option<String>,
    pub device: Option<String>,
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
    activity: String,
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
            arch: opts.arch,
            package: opts.package,
            device: None,
            format: opts.format,
            profiling_output: opts.profiling_output,
            debug_dirty_overlay: opts.debug_dirty_overlay,
            asset_backend_override: opts.asset_backend,
        })
    }

    fn from_dev_opts(opts: DevOptions) -> Result<Self> {
        Self::from_opts(AndroidContextOpts {
            release: opts.release,
            arch: opts.arch,
            package: opts.package,
            device: opts.device,
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

        let manifest_package = manifest.package_name();
        let manifest_cfg = manifest.android().unwrap_or_default();

        let package_name = package.or(manifest_package).ok_or_else(|| {
            anyhow!(
                "Unable to determine workspace package. Provide --package or run in a package directory."
            )
        })?;

        let format = format
            .or_else(|| {
                manifest_cfg
                    .format
                    .as_deref()
                    .and_then(AndroidFormat::from_config)
            })
            .unwrap_or(AndroidFormat::Apk);

        let arch = arch
            .or_else(|| manifest_cfg.arch.clone())
            .or_else(|| Some(DEFAULT_ARCH.to_string()));

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

        let tessera_permissions = collect_tessera_permissions(&root_dir, package_dir.as_ref())?;
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
            min_sdk_version: Some(manifest_cfg.min_sdk.unwrap_or(DEFAULT_MIN_SDK_VERSION)),
            project_dir: Some("gen/android".to_string()),
            no_default_features: None,
            features: None,
            logcat_filter_specs: Vec::new(),
        };
        let config = AndroidConfig::from_raw(app, Some(raw_android))
            .context("Failed to build Android config")?;
        let profiling_output = profiling_output
            .map(|path| normalize_android_profiling_output_path(&path, config.app().identifier()));

        let app_permissions = map_tessera_permissions(&tessera_permissions)?;
        let mut merged_permissions = manifest_cfg.permissions.clone().unwrap_or_default();
        merged_permissions.extend(app_permissions);
        merged_permissions.sort();
        merged_permissions.dedup();

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
            supported: true,
            no_default_features: false,
            cargo_args: (!cargo_args.is_empty()).then_some(cargo_args),
            features: (!cargo_features.is_empty()).then_some(cargo_features),
            app_sources: None,
            app_plugins: manifest_cfg.app_plugins.clone(),
            project_dependencies: manifest_cfg.project_dependencies.clone(),
            app_dependencies: manifest_cfg.app_dependencies.clone(),
            app_dependencies_platform: manifest_cfg.app_dependencies_platform.clone(),
            asset_packs: None,
            app_activity_name: Some(DEFAULT_ANDROID_ACTIVITY.to_string()),
            app_permissions: Some(merged_permissions),
            app_theme_parent: Some(DEFAULT_ANDROID_THEME_PARENT.to_string()),
            env_vars: (!env_vars.is_empty()).then_some(env_vars),
            vulkan_validation: None,
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
            activity: DEFAULT_ANDROID_ACTIVITY.to_string(),
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

fn run_android_call_guard(ctx: &AndroidContext, target: Option<&str>) -> Result<()> {
    let target = if let Some(target) = target {
        ctx.target_by_name_or_triple(target)?.triple
    } else {
        ctx.targets()?
            .first()
            .ok_or_else(|| anyhow!("No Android Rust targets are available"))?
            .triple
    };
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

pub fn init(skip_targets_install: bool) -> Result<()> {
    let ctx = AndroidContext::from_init()?;
    let android_plugins = collect_android_plugins(&ctx)?;
    let android_plugin_modules = collect_android_plugin_modules(&android_plugins);

    let project_exists = ctx.config.project_dir_exists();
    if project_exists {
        output::warn(format!(
            "android project already exists at {}",
            display_path(&ctx.config.project_dir())
        ));
    }

    if !skip_targets_install {
        Target::install_all().map_err(|err| anyhow!("Failed to install Android targets: {err}"))?;
    }

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    register_helpers(&mut handlebars);
    let data = build_android_template_data(&ctx, &android_plugin_modules)?;

    if !project_exists {
        write_template_dir(
            &ANDROID_TEMPLATE_DIR,
            ctx.config.project_dir().as_path(),
            &handlebars,
            &data,
        )?;

        let asset_dir = ctx.config.project_dir().join("app/src/main/assets");
        fs::create_dir_all(&asset_dir)
            .with_context(|| format!("Failed to create assets dir {}", asset_dir.display()))?;
    }

    sync_android_project(&ctx)?;

    if !project_exists {
        output::status(
            "Generated",
            format!(
                "android project at {}",
                display_path(&ctx.config.project_dir())
            ),
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

    handlebars_helper!(snake_case: |s: str| {
        s.chars()
            .map(|c| {
                if c == '-' || c == ' ' {
                    '_'
                } else {
                    c.to_ascii_lowercase()
                }
            })
            .collect::<String>()
    });

    handlebars_helper!(html_escape: |s: str| {
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('\"', "&quot;")
            .replace('\'', "&#x27;")
    });

    handlebars.register_helper("quote-and-join", Box::new(quote_and_join));
    handlebars.register_helper(
        "quote-and-join-colon-prefix",
        Box::new(quote_and_join_colon_prefix),
    );
    handlebars.register_helper("snake-case", Box::new(snake_case));
    handlebars.register_helper("html-escape", Box::new(html_escape));
}

pub fn build(opts: BuildOptions) -> Result<()> {
    let ctx = AndroidContext::from_build_opts(opts)?;
    run_android_call_guard(&ctx, None)?;
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
    run_android_call_guard(&ctx, None)?;
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

    for file in [
        "Cargo.toml",
        "build.rs",
        "tessera-app.toml",
        "tessera-config.toml",
    ] {
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
    run_android_call_guard(&ctx, Some(&target_name))?;
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
            ctx.activity.clone(),
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

fn collect_android_plugin_modules(plugins: &[AndroidPlugin]) -> Vec<String> {
    plugins.iter().map(|plugin| plugin.module.clone()).collect()
}

fn build_android_template_data(
    ctx: &AndroidContext,
    android_plugin_modules: &[String],
) -> Result<Value> {
    let app_dir = ctx.config.project_dir().join("app");
    let root_dir_rel = util::relativize_path(ctx.config.app().root_dir(), app_dir);
    let root_dir_rel = replace_path_separator(root_dir_rel.into_os_string())
        .to_string_lossy()
        .into_owned();

    let app = serde_json::to_value(ctx.config.app())
        .context("Failed to serialize app metadata for templates")?;

    let asset_packs: Vec<String> = ctx
        .metadata
        .asset_packs
        .as_ref()
        .map(|packs| packs.iter().map(|pack| pack.name.clone()).collect())
        .unwrap_or_default();
    let has_asset_packs = !asset_packs.is_empty();
    let app_permissions = ctx.metadata.app_permissions.clone().unwrap_or_default();
    let app_activity_name = ctx
        .metadata
        .app_activity_name
        .clone()
        .unwrap_or_else(|| DEFAULT_ANDROID_ACTIVITY.to_string());
    let app_theme_parent = ctx
        .metadata
        .app_theme_parent
        .clone()
        .unwrap_or_else(|| DEFAULT_ANDROID_THEME_PARENT.to_string());

    let mut project_dependencies = ctx
        .metadata
        .project_dependencies
        .clone()
        .unwrap_or_default();
    if !android_plugin_modules.is_empty() {
        let kotlin_plugin = "org.jetbrains.kotlin:kotlin-gradle-plugin:2.2.21".to_string();
        if !project_dependencies.iter().any(|dep| dep == &kotlin_plugin) {
            project_dependencies.push(kotlin_plugin);
        }
    }
    project_dependencies.sort();
    project_dependencies.dedup();

    let mut abi_list = Vec::new();
    let mut arch_list = Vec::new();
    let mut target_list = Vec::new();
    for target in Target::all().iter().map(|(_, target)| target) {
        abi_list.push(target.abi.to_string());
        arch_list.push(target.arch.to_string());
        target_list.push(target.triple.to_string());
    }

    Ok(json!({
        "app": app,
        "version": ctx.version,
        "version-code": ctx.version_code,
        "app-icon": ctx.identity.icon.is_some(),
        "android": {
            "min-sdk-version": ctx.config.min_sdk_version(),
        },
        "root-dir-rel": root_dir_rel,
        "android-app-plugins": ctx.metadata.app_plugins.clone().unwrap_or_default(),
        "android-project-dependencies": project_dependencies,
        "android-app-dependencies": ctx.metadata.app_dependencies.clone().unwrap_or_default(),
        "android-app-dependencies-platform": ctx
            .metadata
            .app_dependencies_platform
            .clone()
            .unwrap_or_default(),
        "android-plugin-modules": android_plugin_modules,
        "android-app-permissions": app_permissions,
        "android-app-activity-name": app_activity_name,
        "android-app-theme-parent": app_theme_parent,
        "has-code": true,
        "asset-packs": asset_packs,
        "has-asset-packs": has_asset_packs,
        "android-vulkan-validation": ctx
            .metadata
            .vulkan_validation
            .unwrap_or(DEFAULT_VULKAN_VALIDATION),
        "abi_list": abi_list,
        "arch_list": arch_list,
        "target_list": target_list,
    }))
}

fn sync_android_project(ctx: &AndroidContext) -> Result<()> {
    let android_plugins = collect_android_plugins(ctx)?;
    let android_plugin_modules = collect_android_plugin_modules(&android_plugins);

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    register_helpers(&mut handlebars);
    let data = build_android_template_data(ctx, &android_plugin_modules)?;

    let project_dir = ctx.config.project_dir();
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("build.gradle.kts.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("settings.gradle.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("buildSrc/build.gradle.kts.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("buildSrc/src/main/kotlin/BuildTask.kt.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("buildSrc/src/main/kotlin/RustPlugin.kt.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("app/src/main/AndroidManifest.xml.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("app/build.gradle.kts.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;
    write_template_file(
        &ANDROID_TEMPLATE_DIR,
        Path::new("app/src/main/java/TesseraGameActivity.java.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;

    app_assets::android(&ctx.identity, &project_dir.join("app/src/main/tessera-res"))?;
    sync_android_plugins(&project_dir, &android_plugins)?;
    Ok(())
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
            .and_then(|android| android.module.as_ref())
            .cloned();
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

fn collect_tessera_permissions(
    root_dir: &Path,
    package_dir: Option<&PathBuf>,
) -> Result<Vec<String>> {
    let packages = collect_resolved_packages(root_dir, package_dir)?;
    let mut permissions = Vec::new();
    for package in packages {
        let Some(config) = load_tessera_config_from_dir(&package.dir)? else {
            continue;
        };
        permissions.extend(config.permissions);
    }
    permissions.sort();
    permissions.dedup();
    Ok(permissions)
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

fn sync_android_plugins(project_dir: &Path, plugins: &[AndroidPlugin]) -> Result<()> {
    if plugins.is_empty() {
        return Ok(());
    }
    let plugins_dir = project_dir.join("plugins");
    fs::create_dir_all(&plugins_dir)
        .with_context(|| format!("Failed to create {}", plugins_dir.display()))?;

    for plugin in plugins {
        let target_dir = plugins_dir.join(&plugin.module);
        if target_dir.exists() {
            fs::remove_dir_all(&target_dir)
                .with_context(|| format!("Failed to remove {}", target_dir.display()))?;
        }
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

    fn android(&self) -> Option<AndroidManifestConfig> {
        self.package
            .as_ref()
            .and_then(|p| p.metadata.as_ref())
            .and_then(|m| m.tessera.as_ref())
            .and_then(|t| t.android.clone())
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
    android: Option<AndroidManifestConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct AndroidManifestConfig {
    arch: Option<String>,
    format: Option<String>,
    min_sdk: Option<u32>,
    permissions: Option<Vec<String>>,
    app_plugins: Option<Vec<String>>,
    project_dependencies: Option<Vec<String>>,
    app_dependencies: Option<Vec<String>>,
    app_dependencies_platform: Option<Vec<String>>,
}

fn map_tessera_permissions(perms: &[String]) -> Result<Vec<String>> {
    let mut mapped = Vec::new();
    for perm in perms {
        match perm.as_str() {
            "notifications" => mapped.push("android.permission.POST_NOTIFICATIONS".to_string()),
            "camera" => mapped.push("android.permission.CAMERA".to_string()),
            "microphone" => mapped.push("android.permission.RECORD_AUDIO".to_string()),
            "location" => {
                mapped.push("android.permission.ACCESS_COARSE_LOCATION".to_string());
                mapped.push("android.permission.ACCESS_FINE_LOCATION".to_string());
            }
            "bluetooth" => {
                mapped.push("android.permission.BLUETOOTH_CONNECT".to_string());
                mapped.push("android.permission.BLUETOOTH_SCAN".to_string());
            }
            other => {
                return Err(anyhow!(
                    "Unknown tessera permission '{other}' in tessera-config.toml"
                ));
            }
        }
    }
    mapped.sort();
    mapped.dedup();
    Ok(mapped)
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
