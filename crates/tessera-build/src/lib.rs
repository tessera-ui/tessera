//! Build-time utilities for Tessera packages.
//!
//! ## Usage
//!
//! Load `tessera.toml` and generate resource bindings in build scripts.

#![deny(
    missing_docs,
    clippy::unwrap_used,
    rustdoc::broken_intra_doc_links,
    rustdoc::invalid_rust_codeblocks,
    rustdoc::invalid_html_tags
)]
use std::{
    collections::{BTreeMap, HashMap},
    env,
    fs::{self, ReadDir},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, anyhow, bail, ensure};
use globset::{GlobBuilder, GlobSet, GlobSetBuilder};
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;
use sha2::{Digest, Sha256};

pub mod application;
mod localization;

/// Environment variable selecting the asset backend (`embed` or `platform`).
pub const ASSET_BACKEND_ENV: &str = "TESSERA_ASSET_BACKEND";
/// The Tessera configuration file at a package root.
pub const TESSERA_CONFIG_FILE: &str = "tessera.toml";
/// Generated Rust source filename written under `OUT_DIR`.
pub const GENERATED_ASSET_FILE: &str = "tessera_assets.rs";

/// Backend strategy used when generating asset access code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetBackend {
    /// Assets are directly embedded within the binary executable.
    Embed,
    /// Assets are delivered separately through platform files or HTTP.
    Platform,
}

impl AssetBackend {
    /// Returns the canonical backend name.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Embed => "embed",
            Self::Platform => "platform",
        }
    }

    /// Reads `ASSET_BACKEND_ENV` and parses the backend, defaulting to
    /// `AssetBackend::Embed` when the variable is not set.
    pub fn from_env_or_default() -> Result<Self> {
        match env::var(ASSET_BACKEND_ENV) {
            Ok(value) => value.parse(),
            Err(env::VarError::NotPresent) => Ok(Self::Embed),
            Err(err) => Err(anyhow!("Failed to read {ASSET_BACKEND_ENV}: {err}")),
        }
    }
}

impl std::str::FromStr for AssetBackend {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "embed" => Ok(Self::Embed),
            "platform" => Ok(Self::Platform),
            other => bail!(
                "Invalid value `{other}` for {ASSET_BACKEND_ENV}; expected `embed` or `platform`"
            ),
        }
    }
}

/// Configuration schema loaded from `tessera.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct TesseraConfig {
    /// Application identity, read only from the package being built.
    pub app: Option<application::AppConfig>,
    /// Assets delivered with this package.
    pub assets: Option<AssetsConfig>,
    /// Localization catalogs of this package.
    pub i18n: Option<I18nConfig>,
    /// Platform modules contributed by a plugin package.
    pub plugin: Option<PluginConfig>,
}

/// Embedded localization catalogs and their fallback language.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct I18nConfig {
    /// Catalog directory relative to the package root. Defaults to `locales`.
    pub dir: Option<String>,
    /// Fallback language and default language for the entry package.
    pub fallback: String,
}

/// Asset directory and delivery rules.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct AssetsConfig {
    /// Asset directory relative to the package root. Defaults to `assets`.
    pub dir: Option<String>,
    /// Glob patterns, relative to the asset directory, of assets embedded in
    /// the binary on every target.
    #[serde(default)]
    pub embed: Vec<String>,
    /// Glob patterns of platform assets delivered even when no reference to
    /// them is found in the binary.
    #[serde(default)]
    pub keep: Vec<String>,
}

/// Platform modules contributed by a plugin package.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PluginConfig {
    /// Android library module built from the package's `android` directory.
    pub android: Option<PluginAndroidConfig>,
}

/// Android library module of a plugin package.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct PluginAndroidConfig {
    /// Gradle module name.
    pub module: String,
}

#[derive(Debug, Clone)]
struct AssetEntry {
    embedded: bool,
    absolute_path: PathBuf,
    relative_path: String,
    platform_path: String,
    module_segments: Vec<(String, String)>,
    const_name: String,
}

/// An asset ready for embedding or platform delivery.
pub struct PreparedAsset {
    /// Whether this asset is embedded on every target.
    pub embedded: bool,
    /// Whether this platform asset is delivered without a binary reference.
    pub keep: bool,
    /// Source file.
    pub source: PathBuf,
    /// Logical path relative to the configured asset directory.
    pub relative: String,
}

/// Prepares configured assets for embedding or platform delivery.
pub fn prepare_assets(root: &Path, config: &AssetsConfig) -> Result<Vec<PreparedAsset>> {
    let embed = asset_patterns(&config.embed, "embed")?;
    let keep = asset_patterns(&config.keep, "keep")?;
    asset_files(root)?
        .into_iter()
        .map(|source| {
            let relative = normalize_asset_path(source.strip_prefix(root)?);
            Ok(PreparedAsset {
                embedded: embed.is_match(&relative),
                keep: keep.is_match(&relative),
                source,
                relative,
            })
        })
        .collect()
}

fn asset_patterns(patterns: &[String], key: &str) -> Result<GlobSet> {
    let mut builder = GlobSetBuilder::new();
    for pattern in patterns {
        let normalized = pattern.trim().replace('\\', "/");
        let normalized = normalized.strip_prefix("./").unwrap_or(&normalized);
        ensure!(!normalized.is_empty(), "Empty pattern in `assets.{key}`");
        builder.add(
            GlobBuilder::new(normalized)
                .literal_separator(true)
                .build()
                .with_context(|| format!("Invalid pattern `{pattern}` in `assets.{key}`"))?,
        );
    }
    Ok(builder.build()?)
}

#[derive(Debug, Default)]
struct ModuleNode {
    modules: BTreeMap<String, ModuleNode>,
    module_origins: HashMap<String, String>,
    assets: BTreeMap<String, usize>,
    asset_origins: HashMap<String, String>,
}

impl ModuleNode {
    fn insert(&mut self, entry: &AssetEntry, index: usize) -> Result<()> {
        let mut node = self;
        for (original, module) in &entry.module_segments {
            if let Some(existing) = node.module_origins.get(module) {
                if existing != original {
                    bail!(
                        "Directory name collision under generated modules: `{}` and `{}` both map to `{}`",
                        existing,
                        original,
                        module
                    );
                }
            } else {
                node.module_origins.insert(module.clone(), original.clone());
            }
            node = node.modules.entry(module.clone()).or_default();
        }

        let original_file = entry.relative_path.clone();
        if let Some(existing) = node.asset_origins.get(&entry.const_name) {
            if existing != &original_file {
                bail!(
                    "Asset constant collision: `{}` and `{}` both map to `{}`",
                    existing,
                    original_file,
                    entry.const_name
                );
            }
        } else {
            node.asset_origins
                .insert(entry.const_name.clone(), original_file);
        }
        node.assets.insert(entry.const_name.clone(), index);
        Ok(())
    }
}

/// Loads `tessera.toml` from a package directory.
///
/// Returns `Ok(None)` when the config file does not exist.
pub fn load_tessera_config_from_dir(dir: &Path) -> Result<Option<TesseraConfig>> {
    for (legacy, section) in [
        ("tessera-app.toml", "`[app]`"),
        ("tessera-config.toml", "`[assets]`, `[i18n]`, and `[plugin]`"),
    ] {
        let legacy = dir.join(legacy);
        ensure!(
            !legacy.exists(),
            "{} is no longer read; move its settings to {section} in {}. Platform \
             permissions are declared in the Android manifest and Xcode project instead",
            legacy.display(),
            dir.join(TESSERA_CONFIG_FILE).display()
        );
    }
    let path = dir.join(TESSERA_CONFIG_FILE);
    if !path.exists() {
        return Ok(None);
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
    let config = toml::from_str::<TesseraConfig>(&raw)
        .with_context(|| format!("Failed to parse {}", path.display()))?;
    Ok(Some(config))
}

/// Builds the canonical asset namespace (`{package_name}/{package_version}`).
pub fn asset_namespace(package_name: &str, package_version: &str) -> String {
    format!("{package_name}/{package_version}")
}

/// Returns the content-addressed delivery path for an asset.
pub fn platform_asset_path(namespace: &str, relative_path: &str, bytes: &[u8]) -> String {
    let digest = hex::encode(Sha256::digest(bytes));
    format!("tessera/{namespace}/{digest}/{relative_path}")
}

/// Resolves the configured asset directory path under `manifest_dir`.
///
/// Returns `None` when `config` does not contain an `assets` section.
pub fn resolve_assets_dir(manifest_dir: &Path, config: Option<&TesseraConfig>) -> Option<PathBuf> {
    let assets = config.and_then(|cfg| cfg.assets.as_ref())?;
    let dir = assets.dir.as_deref().unwrap_or("assets");
    Some(manifest_dir.join(dir))
}

/// Lists configured asset and localization directories for development watching.
pub fn resolve_resource_dirs(
    manifest_dir: &Path,
    config: Option<&TesseraConfig>,
) -> impl Iterator<Item = PathBuf> {
    let locales = config
        .and_then(|config| config.i18n.as_ref())
        .map(|config| manifest_dir.join(config.dir.as_deref().unwrap_or("locales")));
    resolve_assets_dir(manifest_dir, config)
        .into_iter()
        .chain(locales)
}

/// Generates application defaults, asset bindings, and localization catalogs.
///
/// The generated file is written to `OUT_DIR/tessera_assets.rs`. Creating a
/// previously absent `tessera.toml` takes effect once the package's
/// `Cargo.toml` changes.
pub fn build() -> Result<()> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").context("Missing CARGO_MANIFEST_DIR for build script")?,
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").context("Missing OUT_DIR for build script")?);
    let package_name =
        env::var("CARGO_PKG_NAME").context("Missing CARGO_PKG_NAME for build script")?;
    let package_version =
        env::var("CARGO_PKG_VERSION").context("Missing CARGO_PKG_VERSION for build script")?;
    let backend = AssetBackend::from_env_or_default()?;

    println!("cargo:rerun-if-env-changed={ASSET_BACKEND_ENV}");
    rerun_if_optional_file_changed(&manifest_dir, &manifest_dir.join(TESSERA_CONFIG_FILE));

    let config = load_tessera_config_from_dir(&manifest_dir)?;
    let assets_dir = resolve_assets_dir(&manifest_dir, config.as_ref());

    let mut entries = Vec::new();
    if let Some(dir) = assets_dir.as_ref() {
        println!("cargo:rerun-if-changed={}", dir.display());
        if !dir.is_dir() {
            bail!(
                "Configured assets directory not found: {}",
                dir.to_string_lossy()
            );
        }
        entries = collect_assets(
            dir,
            &asset_namespace(&package_name, &package_version),
            config
                .as_ref()
                .and_then(|config| config.assets.as_ref())
                .context("Missing assets configuration")?,
        )?;
    }

    let mut generated = generate_asset_file(&entries, backend)?;
    let application =
        application::generate(&manifest_dir, &out_dir, &package_name, config.as_ref())?;
    generated.push_str(&prettyplease::unparse(&syn::parse2::<syn::File>(
        application,
    )?));
    let catalogs = localization::generate(
        &manifest_dir,
        config.as_ref().and_then(|config| config.i18n.as_ref()),
    )?;
    generated.push_str(&prettyplease::unparse(&syn::parse2::<syn::File>(catalogs)?));
    fs::write(out_dir.join(GENERATED_ASSET_FILE), generated)
        .with_context(|| "Failed to write generated asset file")?;
    Ok(())
}

// Cargo reruns a build script on every build while a watched path is missing,
// so an absent optional file is tracked through the package manifest instead.
fn rerun_if_optional_file_changed(manifest_dir: &Path, path: &Path) {
    let watched = if path.exists() {
        path.to_path_buf()
    } else {
        manifest_dir.join("Cargo.toml")
    };
    println!("cargo:rerun-if-changed={}", watched.display());
}

/// Lists files recursively beneath an asset directory.
pub fn asset_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut file_paths = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_type = entry
                .file_type()
                .with_context(|| format!("Failed to inspect {}", path.display()))?;
            if file_type.is_dir() {
                stack.push(path);
                continue;
            }
            if file_type.is_file() {
                file_paths.push(path);
            }
        }
    }

    file_paths.sort();
    Ok(file_paths)
}

fn collect_assets(
    assets_dir: &Path,
    namespace: &str,
    config: &AssetsConfig,
) -> Result<Vec<AssetEntry>> {
    let files = prepare_assets(assets_dir, config)?;

    let mut entries = Vec::with_capacity(files.len());
    for file in files {
        let absolute_path = file.source;
        let relative_path = file.relative;
        let relative = Path::new(&relative_path);
        let file_name = relative
            .file_name()
            .ok_or_else(|| anyhow!("Missing file name for {}", absolute_path.display()))?
            .to_string_lossy()
            .into_owned();
        let const_name = mangle_file_name(&file_name);

        let mut module_segments = Vec::new();
        if let Some(parent) = relative.parent() {
            for segment in parent {
                let original = segment.to_string_lossy().into_owned();
                let module = mangle_module_name(&original);
                module_segments.push((original, module));
            }
        }

        let platform_path =
            platform_asset_path(namespace, &relative_path, &fs::read(&absolute_path)?);
        entries.push(AssetEntry {
            embedded: file.embedded,
            absolute_path,
            relative_path,
            platform_path,
            module_segments,
            const_name,
        });
    }
    Ok(entries)
}

fn generate_asset_file(entries: &[AssetEntry], backend: AssetBackend) -> Result<String> {
    let mut root = ModuleNode::default();
    for (index, entry) in entries.iter().enumerate() {
        root.insert(entry, index)?;
    }

    let module_body_tokens = generate_module_body_tokens(&root, entries, backend)?;
    let file_tokens = quote! {
        pub use tessera_ui::Asset;

        #module_body_tokens
    };

    let file = syn::parse2::<syn::File>(file_tokens)
        .context("Failed to build generated asset syntax tree")?;
    let pretty = prettyplease::unparse(&file);
    Ok(format!(
        "// @generated by tessera-build; do not edit.\n{pretty}"
    ))
}

fn generate_module_body_tokens(
    node: &ModuleNode,
    entries: &[AssetEntry],
    backend: AssetBackend,
) -> Result<TokenStream> {
    let mut items = Vec::new();

    for (const_name, index) in &node.assets {
        let ident = format_ident!("{const_name}");
        let index = *index;
        let entry = entries
            .get(index)
            .ok_or_else(|| anyhow!("Asset index out of range while generating module body"))?;
        let value_tokens = generate_asset_value_tokens(entry, backend);
        items.push(quote! {
            pub const #ident: Asset = #value_tokens;
        });
    }

    for (module_name, child) in &node.modules {
        let module_ident = format_ident!("{module_name}");
        let child_body = generate_module_body_tokens(child, entries, backend)?;
        items.push(quote! {
            pub mod #module_ident {
                use super::Asset;
                #child_body
            }
        });
    }

    Ok(quote! {
        #(#items)*
    })
}

fn generate_asset_value_tokens(entry: &AssetEntry, backend: AssetBackend) -> TokenStream {
    let platform_path = Literal::string(&entry.platform_path);
    match if entry.embedded {
        AssetBackend::Embed
    } else {
        backend
    } {
        AssetBackend::Embed => {
            let path = entry.absolute_path.to_string_lossy().into_owned();
            let path_literal = Literal::string(&path);
            quote! {
                Asset::embedded(#platform_path, include_bytes!(#path_literal) as &[u8])
            }
        }
        AssetBackend::Platform => {
            quote! {
                Asset::platform(#platform_path)
            }
        }
    }
}

fn read_dir(path: &Path) -> Result<ReadDir> {
    fs::read_dir(path).with_context(|| format!("Failed to read {}", path.display()))
}

/// Returns an asset path with slash separators.
pub fn normalize_asset_path(path: &Path) -> String {
    path.iter()
        .map(|segment| segment.to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

fn mangle_module_name(input: &str) -> String {
    let mut text = mangle_segment(input, false);
    if is_keyword(&text) {
        text.push('_');
    }
    text
}

fn mangle_file_name(input: &str) -> String {
    mangle_segment(input, true)
}

fn mangle_segment(input: &str, uppercase: bool) -> String {
    let mut out = String::new();
    let mut prev_underscore = false;
    for ch in input.chars() {
        let mapped = if ch.is_ascii_alphanumeric() {
            if uppercase {
                ch.to_ascii_uppercase()
            } else {
                ch.to_ascii_lowercase()
            }
        } else {
            '_'
        };

        if mapped == '_' {
            if !prev_underscore {
                out.push('_');
                prev_underscore = true;
            }
        } else {
            out.push(mapped);
            prev_underscore = false;
        }
    }

    while out.starts_with('_') {
        out.remove(0);
    }
    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        out = if uppercase {
            "ASSET".to_string()
        } else {
            "asset".to_string()
        };
    }
    if out
        .chars()
        .next()
        .map(|ch| ch.is_ascii_digit())
        .unwrap_or(false)
    {
        out.insert(0, '_');
    }
    out
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "gen"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            | "dyn"
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_names_escape_rust_2024_keywords() {
        for keyword in ["gen", "async", "await", "self", "type", "try"] {
            assert_eq!(mangle_module_name(keyword), format!("{keyword}_"));
        }
        assert_eq!(mangle_module_name("images"), "images");
    }

    #[test]
    fn asset_globs_select_embedded_and_kept_assets() -> Result<()> {
        let root = tempfile::tempdir()?;
        for file in ["fonts/a.ttf", "fonts/nested/b.ttf", "licenses/x/OFL.txt", "logo.svg"] {
            let path = root.path().join(file);
            fs::create_dir_all(path.parent().context("asset directory")?)?;
            fs::write(path, file)?;
        }
        let config: AssetsConfig =
            toml::from_str("embed = [\"fonts/*.ttf\"]\nkeep = [\"./licenses/**\"]")?;
        let assets = prepare_assets(root.path(), &config)?
            .into_iter()
            .map(|asset| (asset.relative, asset.embedded, asset.keep))
            .collect::<Vec<_>>();
        assert_eq!(
            assets,
            [
                ("fonts/a.ttf".to_string(), true, false),
                ("fonts/nested/b.ttf".to_string(), false, false),
                ("licenses/x/OFL.txt".to_string(), false, true),
                ("logo.svg".to_string(), false, false),
            ]
        );
        Ok(())
    }

    #[test]
    fn configuration_rejects_unknown_keys_and_legacy_files() -> Result<()> {
        let package = tempfile::tempdir()?;
        let config = package.path().join(TESSERA_CONFIG_FILE);
        fs::write(&config, "[assets]\nembedded = []\n")?;
        let error = load_tessera_config_from_dir(package.path()).unwrap_err();
        assert!(format!("{error:#}").contains("unknown field `embedded`"));

        fs::write(&config, "[assets]\n")?;
        fs::write(package.path().join("tessera-config.toml"), "")?;
        let error = load_tessera_config_from_dir(package.path()).unwrap_err();
        assert!(error.to_string().contains("tessera-config.toml is no longer read"));
        Ok(())
    }
}
