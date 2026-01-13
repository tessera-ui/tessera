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
        self,
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
use include_dir::{Dir, include_dir};
use notify::{Event, EventKind, RecursiveMode, Watcher};
use owo_colors::colored::*;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::template::{write_template_dir, write_template_file};

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

#[derive(Debug)]
struct AndroidPlugin {
    module: String,
    source_dir: PathBuf,
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
        let target_dir = MetadataCommand::new()
            .manifest_path(root_dir.join("Cargo.toml"))
            .exec()
            .context("Failed to resolve Cargo target directory")?
            .target_directory
            .into_std_path_buf();

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

        let app_toml = load_app_toml(config.app().root_dir())?;
        let app_permissions =
            map_tessera_permissions(app_toml.permissions.as_deref().unwrap_or_default())?;
        let mut merged_permissions = manifest_cfg.permissions.clone().unwrap_or_default();
        merged_permissions.extend(app_permissions);
        merged_permissions.sort();
        merged_permissions.dedup();

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
            app_permissions: Some(merged_permissions),
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
    let android_plugins = collect_android_plugins(&ctx)?;
    let android_plugin_modules = collect_android_plugin_modules(&android_plugins);

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
    } else {
        write_template_file(
            &ANDROID_TEMPLATE_DIR,
            Path::new("build.gradle.kts.hbs"),
            ctx.config.project_dir().as_path(),
            &handlebars,
            &data,
        )?;
        write_template_file(
            &ANDROID_TEMPLATE_DIR,
            Path::new("settings.gradle.hbs"),
            ctx.config.project_dir().as_path(),
            &handlebars,
            &data,
        )?;
        write_template_file(
            &ANDROID_TEMPLATE_DIR,
            Path::new("app/build.gradle.kts.hbs"),
            ctx.config.project_dir().as_path(),
            &handlebars,
            &data,
        )?;
    }

    sync_android_plugins(&ctx.config.project_dir(), &android_plugins)?;

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
    sync_android_project(&ctx)?;
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
    sync_android_project(&ctx)?;

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
        let kotlin_plugin = "org.jetbrains.kotlin:kotlin-gradle-plugin:1.8.22".to_string();
        if !project_dependencies.iter().any(|dep| dep == &kotlin_plugin) {
            project_dependencies.push(kotlin_plugin);
        }
    }
    project_dependencies.sort();
    project_dependencies.dedup();

    let mut abi_list = Vec::new();
    let mut arch_list = Vec::new();
    let mut target_list = Vec::new();
    for target in Target::all().values() {
        abi_list.push(target.abi.to_string());
        arch_list.push(target.arch.to_string());
        target_list.push(target.triple.to_string());
    }

    Ok(json!({
        "app": app,
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
        Path::new("app/build.gradle.kts.hbs"),
        project_dir.as_path(),
        &handlebars,
        &data,
    )?;

    sync_android_plugins(&project_dir, &android_plugins)?;
    Ok(())
}

fn collect_android_plugins(ctx: &AndroidContext) -> Result<Vec<AndroidPlugin>> {
    let manifest_path = ctx.config.app().root_dir().join("Cargo.toml");
    let metadata = MetadataCommand::new()
        .manifest_path(&manifest_path)
        .exec()
        .context("Failed to run cargo metadata")?;

    let root_manifest = ctx
        .package_dir
        .as_ref()
        .map(|dir| dir.join("Cargo.toml"))
        .unwrap_or_else(|| manifest_path.clone())
        .canonicalize()
        .with_context(|| "Failed to resolve root Cargo.toml")?;

    let root_pkg = metadata
        .packages
        .iter()
        .find(|pkg| {
            pkg.manifest_path
                .clone()
                .into_std_path_buf()
                .canonicalize()
                .map(|path| path == root_manifest)
                .unwrap_or(false)
        })
        .ok_or_else(|| anyhow!("Failed to resolve the root package from cargo metadata"))?;

    let resolve = metadata
        .resolve
        .as_ref()
        .ok_or_else(|| anyhow!("cargo metadata missing dependency graph"))?;

    let mut nodes = HashMap::new();
    for node in &resolve.nodes {
        nodes.insert(node.id.clone(), node);
    }

    let mut visited = HashSet::new();
    let mut stack = vec![root_pkg.id.clone()];
    while let Some(id) = stack.pop() {
        if visited.insert(id.clone())
            && let Some(node) = nodes.get(&id)
        {
            for dep in &node.deps {
                stack.push(dep.pkg.clone());
            }
        }
    }

    let mut plugins = Vec::new();
    let mut modules = HashSet::new();
    for pkg in metadata
        .packages
        .iter()
        .filter(|pkg| visited.contains(&pkg.id) && pkg.id != root_pkg.id)
    {
        let pkg_dir = pkg
            .manifest_path
            .clone()
            .into_std_path_buf()
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| anyhow!("Failed to locate plugin root for {}", pkg.manifest_path))?;
        let plugin_manifest_path = pkg_dir.join("tessera-plugin.toml");
        if !plugin_manifest_path.exists() {
            continue;
        }
        let contents = fs::read_to_string(&plugin_manifest_path)
            .with_context(|| format!("Failed to read {}", plugin_manifest_path.display()))?;
        let manifest: PluginManifest = toml::from_str(&contents)
            .with_context(|| format!("Failed to parse {}", plugin_manifest_path.display()))?;
        let Some(android) = manifest.android else {
            continue;
        };
        let module = android.module;
        if !modules.insert(module.clone()) {
            return Err(anyhow!(
                "Duplicate Android plugin module '{}' found in {}",
                module,
                plugin_manifest_path.display()
            ));
        }
        let source_dir = pkg_dir.join("android");
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

#[derive(Debug, Deserialize, Default)]
struct AppToml {
    permissions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct PluginManifest {
    #[allow(dead_code)]
    name: Option<String>,
    #[allow(dead_code)]
    version: Option<u32>,
    #[allow(dead_code)]
    permissions: Option<Vec<String>>,
    android: Option<PluginAndroid>,
}

#[derive(Debug, Deserialize)]
struct PluginAndroid {
    module: String,
    #[allow(dead_code)]
    package: Option<String>,
}

fn load_app_toml(root: &Path) -> Result<AppToml> {
    let path = root.join("tessera-app.toml");
    if !path.exists() {
        return Ok(AppToml::default());
    }
    let contents =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    toml::from_str(&contents).with_context(|| format!("Failed to parse {}", path.display()))
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
                    "Unknown tessera permission '{other}' in tessera-app.toml"
                ));
            }
        }
    }
    mapped.sort();
    mapped.dedup();
    Ok(mapped)
}
