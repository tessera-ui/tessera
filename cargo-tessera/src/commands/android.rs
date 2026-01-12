use std::{
    fs,
    path::{Path, PathBuf},
    sync::mpsc::channel,
    time::{Duration, Instant},
};

use anyhow::{Context, Result, anyhow};
use cargo_mobile2::{
    ChildHandle,
    android::{
        self,
        config::{Config as AndroidConfig, Metadata as AndroidMetadata, Raw as RawAndroidConfig},
        device::Device,
        env::Env as AndroidEnv,
        target::Target,
    },
    config::app::{App, Raw as RawAppConfig},
    dot_cargo,
    opts::{FilterLevel, NoiseLevel, Profile},
    os::replace_path_separator,
    target::TargetTrait,
    util,
};
use clap::ValueEnum;
use handlebars::{Handlebars, handlebars_helper};
use include_dir::{Dir, include_dir};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use owo_colors::colored::*;
use serde::Deserialize;
use serde_json::json;

use crate::template::write_template_dir;

use super::find_package_dir;

const DEFAULT_ARCH: &str = "arm64";
const DEFAULT_MIN_SDK_VERSION: u32 = 24;
const DEFAULT_ANDROID_ACTIVITY: &str = ".TesseraGameActivity";
const DEFAULT_ANDROID_THEME_PARENT: &str = "Theme.AppCompat.Light.NoActionBar";
const ANDROID_TEMPLATE_DIR: Dir<'_> =
    include_dir!("$CARGO_MANIFEST_DIR/../tessera-mobile/templates/platforms/android-studio");

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
}

#[derive(Debug)]
pub struct DevOptions {
    pub release: bool,
    pub arch: Option<String>,
    pub package: Option<String>,
    pub device: Option<String>,
}

pub struct RustBuildOptions {
    pub release: bool,
    pub target: String,
    pub package: Option<String>,
}

struct AndroidContext {
    package_dir: Option<PathBuf>,
    package_name: String,
    release: bool,
    format: AndroidFormat,
    arch: Option<String>,
    device: Option<String>,
    activity: String,
    config: AndroidConfig,
    metadata: AndroidMetadata,
}

impl AndroidContext {
    fn from_init() -> Result<Self> {
        Self::from_opts(false, None, None, None, None)
    }
    fn from_build_opts(opts: BuildOptions) -> Result<Self> {
        Self::from_opts(opts.release, opts.arch, opts.package, None, opts.format)
    }

    fn from_dev_opts(opts: DevOptions) -> Result<Self> {
        Self::from_opts(opts.release, opts.arch, opts.package, opts.device, None)
    }

    fn from_rust_build_opts(opts: RustBuildOptions) -> Result<Self> {
        Self::from_opts(opts.release, None, opts.package, None, None)
    }

    fn from_opts(
        release: bool,
        arch: Option<String>,
        package: Option<String>,
        device: Option<String>,
        format: Option<AndroidFormat>,
    ) -> Result<Self> {
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

        let package_name = package
            .or_else(|| manifest_cfg.package.clone())
            .or(manifest_package)
            .ok_or_else(|| {
                anyhow!(
                    "Unable to determine package name. Provide --package or set \
package.metadata.tessera.android.package in Cargo.toml"
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

        let identifier = manifest_cfg
            .package
            .clone()
            .unwrap_or_else(|| default_identifier(&package_name));
        let raw_app = RawAppConfig {
            name: package_name.clone(),
            lib_name: Some(package_name.replace('-', "_")),
            stylized_name: None,
            identifier: sanitize_identifier(&identifier),
            asset_dir: None,
            template_pack: None,
        };
        let app =
            App::from_raw(root_dir, raw_app).context("Failed to build Android app metadata")?;

        let raw_android = RawAndroidConfig {
            min_sdk_version: Some(manifest_cfg.min_sdk.unwrap_or(DEFAULT_MIN_SDK_VERSION)),
            project_dir: Some("gen/android".to_string()),
            no_default_features: None,
            features: None,
            logcat_filter_specs: Vec::new(),
        };
        let config = AndroidConfig::from_raw(app, Some(raw_android))
            .context("Failed to build Android config")?;

        let metadata = AndroidMetadata {
            supported: true,
            no_default_features: false,
            cargo_args: None,
            features: None,
            app_sources: None,
            app_plugins: manifest_cfg.app_plugins.clone(),
            project_dependencies: manifest_cfg.project_dependencies.clone(),
            app_dependencies: manifest_cfg.app_dependencies.clone(),
            app_dependencies_platform: manifest_cfg.app_dependencies_platform.clone(),
            asset_packs: None,
            app_activity_name: Some(DEFAULT_ANDROID_ACTIVITY.to_string()),
            app_permissions: manifest_cfg.permissions.clone(),
            app_theme_parent: Some(DEFAULT_ANDROID_THEME_PARENT.to_string()),
            env_vars: None,
            vulkan_validation: None,
        };

        Ok(Self {
            package_dir,
            package_name,
            release,
            format,
            arch,
            device,
            activity: DEFAULT_ANDROID_ACTIVITY.to_string(),
            config,
            metadata,
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
            Ok(Target::all().values().collect())
        }
    }

    fn target_by_name_or_triple(&self, value: &str) -> Result<&'static Target<'static>> {
        Target::for_name(value)
            .or_else(|| Target::for_arch(value))
            .or_else(|| Target::all().values().find(|target| target.triple == value))
            .ok_or_else(|| {
                anyhow!(
                    "Unknown target '{}'. Supported targets: {}",
                    value,
                    Target::all()
                        .values()
                        .map(|target| target.triple)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })
    }
}

fn default_identifier(package_name: &str) -> String {
    let sanitized = package_name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    format!("com.example.{sanitized}")
}

fn sanitize_identifier(identifier: &str) -> String {
    identifier
        .chars()
        .map(|c| match c {
            '-' => '_',
            c if c.is_ascii_alphanumeric() || c == '_' || c == '.' => c,
            _ => '_',
        })
        .collect()
}

pub fn init(skip_targets_install: bool) -> Result<()> {
    let ctx = AndroidContext::from_init()?;

    let project_exists = ctx.config.project_dir_exists();
    if project_exists {
        println!(
            "{}",
            format!(
                "Android project already exists at {}",
                display_path(&ctx.config.project_dir())
            )
            .bright_yellow()
        );
    }

    if !skip_targets_install {
        Target::install_all().map_err(|err| anyhow!("Failed to install Android targets: {err}"))?;
    }

    let env =
        AndroidEnv::new().map_err(|err| anyhow!("Failed to load Android SDK/NDK env: {err}"))?;

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    register_helpers(&mut handlebars);

    let root_dir_rel = util::relativize_path(
        ctx.config.app().root_dir(),
        ctx.config.project_dir().join("app"),
    );
    let root_dir_rel = replace_path_separator(root_dir_rel.into_os_string());

    let data = json!({
        "app": {
            "identifier": ctx.config.app().identifier(),
            "name": ctx.config.app().name(),
            "stylized-name": ctx.config.app().stylized_name(),
        },
        "android": {
            "min-sdk-version": ctx.config.min_sdk_version(),
        },
        "android-vulkan-validation": false,
        "android-app-activity-name": DEFAULT_ANDROID_ACTIVITY,
        "android-app-theme-parent": DEFAULT_ANDROID_THEME_PARENT,
        "android-app-permissions": ctx.metadata.app_permissions().unwrap_or_default(),
        "android-app-plugins": ctx.metadata.app_plugins().unwrap_or_default(),
        "android-project-dependencies": ctx.metadata.project_dependencies().unwrap_or_default(),
        "android-app-dependencies": ctx.metadata.app_dependencies().unwrap_or_default(),
        "android-app-dependencies-platform": ctx.metadata
            .app_dependencies_platform()
            .unwrap_or_default(),
        "has-code": true,
        "has-asset-packs": false,
        "asset-packs": Vec::<String>::new(),
        "abi-list": Target::all().values().map(|t| t.abi).collect::<Vec<_>>(),
        "arch-list": Target::all().values().map(|t| t.arch).collect::<Vec<_>>(),
        "target-list": Target::all().keys().collect::<Vec<_>>(),
        "root-dir-rel": Path::new(&root_dir_rel).display().to_string(),
        "windows": cfg!(windows),
    });

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

    let mut cargo_config = dot_cargo::DotCargo::load(ctx.config.app())
        .with_context(|| "Failed to load .cargo/config.toml")?;
    for target in Target::all().values() {
        let dot_target = target
            .generate_cargo_config(&ctx.config, &env)
            .map_err(|err| {
                anyhow!(
                    "Failed to generate cargo config for {}: {err}",
                    target.triple
                )
            })?;
        cargo_config.insert_target(target.triple.to_owned(), dot_target);
    }
    cargo_config
        .write(ctx.config.app())
        .with_context(|| "Failed to write .cargo/config.toml")?;

    if !project_exists {
        println!(
            "{}",
            format!(
                "Android project generated at {}",
                display_path(&ctx.config.project_dir())
            )
            .bright_green()
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
    if !ctx.config.project_dir_exists() {
        return Err(anyhow!(
            "Android project not initialized. Run `cargo tessera android init` first."
        ));
    }
    let env = AndroidEnv::new()?;
    let targets = ctx.targets()?;
    let profile = ctx.profile();

    println!(
        "{}",
        format!(
            "Building Android artifact ({}, targets: {}, release: {})",
            ctx.package_name,
            targets
                .iter()
                .map(|t| t.triple)
                .collect::<Vec<_>>()
                .join(", "),
            if ctx.release { "yes" } else { "no" }
        )
        .bright_cyan()
    );

    for target in &targets {
        target.build(
            &ctx.config,
            &ctx.metadata,
            &env,
            NoiseLevel::Polite,
            true,
            profile,
        )?;
    }

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

    println!("\n{}", "Android build complete".green());
    println!("Package: {}", ctx.package_name.bright_green());
    for out in outputs {
        println!("Artifact: {}", display_path(&out).bright_yellow());
    }
    Ok(())
}

pub fn dev(opts: DevOptions) -> Result<()> {
    let ctx = AndroidContext::from_dev_opts(opts)?;
    if !ctx.config.project_dir_exists() {
        return Err(anyhow!(
            "Android project not initialized. Run `cargo tessera android init` first."
        ));
    }
    let device_id = ctx
        .device
        .as_deref()
        .ok_or_else(|| anyhow!("--device <adb_serial> is required for android dev"))?;

    println!(
        "{}",
        format!(
            "Running Tessera app on Android ({}, release: {})",
            ctx.package_name,
            if ctx.release { "yes" } else { "no" }
        )
        .bright_cyan()
    );

    let env = AndroidEnv::new()?;
    let profile = ctx.profile();

    println!("{}", "Watching for file changes...".dimmed());

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

    for file in ["Cargo.toml", "build.rs"] {
        let path = watch_dir.join(file);
        if path.exists() {
            watcher.watch(&path, RecursiveMode::NonRecursive)?;
        }
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
                    println!(
                        "\n{}",
                        "Change detected, canceling in-progress Android deploy...".bright_yellow()
                    );
                    let _ = active_run.kill();
                    let _ = active_run.wait();
                }
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
            Err(_) => break,
        }

        if pending_change && run_child.is_none() && last_change.elapsed() >= debounce_window {
            while rx.try_recv().is_ok() {}

            println!(
                "\n{}",
                "Building and deploying to Android device...".bright_yellow()
            );

            match run_once(&ctx, &env, profile, device_id, NoiseLevel::Polite) {
                Ok(child) => {
                    run_child = Some(child);
                    pending_change = false;
                }
                Err(e) => {
                    println!("{} Failed to start deploy: {}", "Error".red(), e);
                }
            }
        }

        if let Some(active_run) = run_child.take() {
            match active_run.try_wait() {
                Ok(Some(output)) => {
                    if !output.status.success() {
                        println!("{}", "Deploy failed, waiting for changes...".red());
                    } else {
                        println!(
                            "{}",
                            "Deploy finished. App should be running on the device.".green()
                        );
                    }
                }
                Ok(None) => {
                    run_child = Some(active_run);
                }
                Err(err) => {
                    println!("{} Failed to check deploy status: {}", "⚠️".yellow(), err);
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
    if !ctx.config.project_dir_exists() {
        return Err(anyhow!(
            "Android project not initialized. Run `cargo tessera android init` first."
        ));
    }
    let env = AndroidEnv::new()?;
    let target = ctx.target_by_name_or_triple(&target_name)?;
    let profile = ctx.profile();

    target.build(
        &ctx.config,
        &ctx.metadata,
        &env,
        NoiseLevel::Polite,
        true,
        profile,
    )?;

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
    let target = device.target();

    target.build(&ctx.config, &ctx.metadata, env, noise_level, true, profile)?;

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

// xbuild helpers removed in favor of tessera-mobile integration.

#[derive(Debug, Deserialize)]
struct Manifest {
    package: Option<PackageSection>,
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
struct MetadataSection {
    tessera: Option<TesseraMetadata>,
}

#[derive(Debug, Deserialize)]
struct TesseraMetadata {
    android: Option<AndroidManifestConfig>,
}

#[derive(Debug, Deserialize, Default, Clone)]
struct AndroidManifestConfig {
    package: Option<String>,
    arch: Option<String>,
    format: Option<String>,
    min_sdk: Option<u32>,
    permissions: Option<Vec<String>>,
    app_plugins: Option<Vec<String>>,
    project_dependencies: Option<Vec<String>>,
    app_dependencies: Option<Vec<String>>,
    app_dependencies_platform: Option<Vec<String>>,
}
