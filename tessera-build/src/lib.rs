//! Build-time utilities for Tessera packages.
//!
//! ## Usage
//!
//! Load `tessera-config.toml` and generate compiled asset bindings in build
//! scripts.

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

use anyhow::{Context, Result, anyhow, bail};
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote};
use serde::Deserialize;

use backend_embed::generate_embed_backend_tokens;
use backend_platform::generate_platform_backend_tokens;

mod backend_embed;
mod backend_platform;

/// Environment variable selecting the asset backend (`embed` or `platform`).
pub const ASSET_BACKEND_ENV: &str = "TESSERA_ASSET_BACKEND";
/// The conventional filename for Tessera project configurations.
pub const TESSERA_CONFIG_FILE: &str = "tessera-config.toml";
/// Generated Rust source filename written under `OUT_DIR`.
pub const GENERATED_ASSET_FILE: &str = "tessera_assets.rs";

/// Backend strategy used when generating asset access code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetBackend {
    /// Assets are directly embedded within the binary executable.
    Embed,
    /// Assets are loaded using the platform's native asset management system
    /// (e.g. Android assets).
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

/// Configuration schema loaded from `tessera-config.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct TesseraConfig {
    /// Permissions requested by the package.
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Asset collection and filtering configuration.
    pub assets: Option<AssetsConfig>,
    /// Plugin wrapper generation settings.
    pub plugin: Option<PluginConfig>,
}

/// Asset directory and filtering configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AssetsConfig {
    /// Asset directory relative to the package root. Defaults to `assets`.
    pub dir: Option<String>,
    /// Tree-shaking rules for excluding matched assets.
    pub tree_shaking: Option<AssetsTreeShakingConfig>,
}

impl AssetsConfig {
    /// Returns configured exclusion patterns or an empty slice.
    pub fn tree_shaking_exclude_patterns(&self) -> &[String] {
        self.tree_shaking
            .as_ref()
            .map_or(&[], |tree_shaking| tree_shaking.exclude.as_slice())
    }
}

/// Tree-shaking rules applied during asset discovery.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct AssetsTreeShakingConfig {
    /// Patterns of asset paths to exclude from the generated bundle.
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Plugin-specific configuration options.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginConfig {
    /// Android-specific plugin integration settings.
    pub android: Option<PluginAndroidConfig>,
}

/// Android plugin integration metadata.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginAndroidConfig {
    /// Android module path used by generated plugin glue code.
    pub module: Option<String>,
    /// Java/Kotlin package name used for Android integration.
    pub package: Option<String>,
}

#[derive(Debug, Clone)]
struct AssetEntry {
    absolute_path: PathBuf,
    relative_path: String,
    platform_path: String,
    module_segments: Vec<(String, String)>,
    const_name: String,
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

/// Loads `tessera-config.toml` from a package directory.
///
/// Returns `Ok(None)` when the config file does not exist.
pub fn load_tessera_config_from_dir(dir: &Path) -> Result<Option<TesseraConfig>> {
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

/// Resolves the configured asset directory path under `manifest_dir`.
///
/// Returns `None` when `config` does not contain an `assets` section.
pub fn resolve_assets_dir(manifest_dir: &Path, config: Option<&TesseraConfig>) -> Option<PathBuf> {
    let assets = config.and_then(|cfg| cfg.assets.as_ref())?;
    let dir = assets.dir.as_deref().unwrap_or("assets");
    Some(manifest_dir.join(dir))
}

/// Generates the asset bindings source file during build.
///
/// The generated file is written to `OUT_DIR/tessera_assets.rs`.
pub fn generate_assets() -> Result<()> {
    let manifest_dir = PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").context("Missing CARGO_MANIFEST_DIR for build script")?,
    );
    let out_dir = PathBuf::from(env::var("OUT_DIR").context("Missing OUT_DIR for build script")?);
    let package_name =
        env::var("CARGO_PKG_NAME").context("Missing CARGO_PKG_NAME for build script")?;
    let package_version =
        env::var("CARGO_PKG_VERSION").context("Missing CARGO_PKG_VERSION for build script")?;
    let backend = AssetBackend::from_env_or_default()?;

    let config_path = manifest_dir.join(TESSERA_CONFIG_FILE);
    println!("cargo:rerun-if-env-changed={ASSET_BACKEND_ENV}");
    println!("cargo:rerun-if-changed={}", config_path.display());

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
        entries = collect_assets(dir, &asset_namespace(&package_name, &package_version))?;
    }

    let generated = generate_asset_file(&entries, backend)?;
    fs::write(out_dir.join(GENERATED_ASSET_FILE), generated)
        .with_context(|| "Failed to write generated asset file")?;
    Ok(())
}

fn collect_assets(assets_dir: &Path, namespace: &str) -> Result<Vec<AssetEntry>> {
    let mut file_paths = Vec::new();
    let mut stack = vec![assets_dir.to_path_buf()];
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

    file_paths.sort_by(|left, right| {
        normalize_relative_path(
            left.strip_prefix(assets_dir)
                .expect("left file must be under assets directory"),
        )
        .cmp(&normalize_relative_path(
            right
                .strip_prefix(assets_dir)
                .expect("right file must be under assets directory"),
        ))
    });

    let mut entries = Vec::with_capacity(file_paths.len());
    for absolute_path in file_paths {
        let relative = absolute_path
            .strip_prefix(assets_dir)
            .with_context(|| format!("Failed to relativize {}", absolute_path.display()))?;
        let relative_path = normalize_relative_path(relative);
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

        let platform_path = format!("tessera/{namespace}/{relative_path}");
        entries.push(AssetEntry {
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

    let backend_tokens = match backend {
        AssetBackend::Embed => generate_embed_backend_tokens(),
        AssetBackend::Platform => generate_platform_backend_tokens(),
    };
    let module_body_tokens = generate_module_body_tokens(&root, entries, backend)?;

    let file_tokens = quote! {
        use std::io;
        use std::sync::Arc;

        #backend_tokens

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
        let value_tokens = generate_asset_value_tokens(entry, index, backend);
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

fn generate_asset_value_tokens(
    entry: &AssetEntry,
    index: usize,
    backend: AssetBackend,
) -> TokenStream {
    match backend {
        AssetBackend::Embed => {
            let path = entry.absolute_path.to_string_lossy().into_owned();
            let path_literal = Literal::string(&path);
            quote! {
                Asset::new_embed(#index, include_bytes!(#path_literal) as &[u8])
            }
        }
        AssetBackend::Platform => {
            let path_literal = Literal::string(&entry.platform_path);
            quote! {
                Asset::new_platform(#index, #path_literal)
            }
        }
    }
}

fn read_dir(path: &Path) -> Result<ReadDir> {
    fs::read_dir(path).with_context(|| format!("Failed to read {}", path.display()))
}

fn normalize_relative_path(path: &Path) -> String {
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
