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

pub const ASSET_BACKEND_ENV: &str = "TESSERA_ASSET_BACKEND";
pub const TESSERA_CONFIG_FILE: &str = "tessera-config.toml";
pub const GENERATED_ASSET_FILE: &str = "tessera_assets.rs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetBackend {
    Embed,
    Platform,
}

impl AssetBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Embed => "embed",
            Self::Platform => "platform",
        }
    }

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

#[derive(Debug, Clone, Deserialize, Default)]
pub struct TesseraConfig {
    #[serde(default)]
    pub permissions: Vec<String>,
    pub assets: Option<AssetsConfig>,
    pub plugin: Option<PluginConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct AssetsConfig {
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginConfig {
    pub android: Option<PluginAndroidConfig>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct PluginAndroidConfig {
    pub module: Option<String>,
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

pub fn asset_namespace(package_name: &str, package_version: &str) -> String {
    format!("{package_name}/{package_version}")
}

pub fn resolve_assets_dir(manifest_dir: &Path, config: Option<&TesseraConfig>) -> Option<PathBuf> {
    let assets = config.and_then(|cfg| cfg.assets.as_ref())?;
    let dir = assets.dir.as_deref().unwrap_or("assets");
    Some(manifest_dir.join(dir))
}

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
        AssetBackend::Embed => generate_embed_backend_tokens(entries),
        AssetBackend::Platform => generate_platform_backend_tokens(entries),
    };
    let module_body_tokens = generate_module_body_tokens(&root)?;

    let file_tokens = quote! {
        use std::io;
        use std::sync::Arc;

        #[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
        pub struct Asset {
            index: usize,
        }

        impl Asset {
            const fn new(index: usize) -> Self {
                Self { index }
            }
        }

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

fn generate_embed_backend_tokens(entries: &[AssetEntry]) -> TokenStream {
    let len = entries.len();
    let bytes = entries.iter().map(|entry| {
        let path = entry.absolute_path.to_string_lossy().into_owned();
        let path_literal = Literal::string(&path);
        quote! { include_bytes!(#path_literal) as &[u8] }
    });

    quote! {
        const __TESSERA_ASSET_BYTES: [&[u8]; #len] = [#(#bytes,)*];

        impl tessera_ui::AssetExt for Asset {
            fn read(self) -> io::Result<Arc<[u8]>> {
                tessera_ui::asset::read_with_lru_cache::<Asset, _>(self.index as u64, || {
                    if let Some(bytes) = __TESSERA_ASSET_BYTES.get(self.index) {
                        return Ok(Arc::<[u8]>::from(*bytes));
                    }
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "asset index out of range",
                    ))
                })
            }
        }
    }
}

fn generate_platform_backend_tokens(entries: &[AssetEntry]) -> TokenStream {
    let len = entries.len();
    let paths = entries.iter().map(|entry| {
        let path_literal = Literal::string(&entry.platform_path);
        quote! { #path_literal }
    });

    quote! {
        const __TESSERA_ASSET_PATHS: [&str; #len] = [#(#paths,)*];

        fn __tessera_read_platform_asset(path: &str) -> io::Result<Arc<[u8]>> {
            #[cfg(target_os = "android")]
            {
                return __tessera_read_android_asset(path);
            }

            #[cfg(not(target_os = "android"))]
            {
                let root = std::env::current_exe()?
                    .parent()
                    .map(std::path::PathBuf::from)
                    .ok_or_else(|| io::Error::other("Failed to resolve executable directory"))?;
                let file_path = root.join("assets").join(path);
                let bytes = std::fs::read(&file_path)?;
                Ok(Arc::from(bytes))
            }
        }

        #[cfg(target_os = "android")]
        fn __tessera_read_android_asset(path: &str) -> io::Result<Arc<[u8]>> {
            use std::ffi::CString;
            use tessera_ui::jni::{objects::JObject, JavaVM};

            fn map_android_error(message: impl Into<String>) -> io::Error {
                io::Error::other(message.into())
            }

            let android_context = tessera_ui::ndk_context::android_context();
            let vm = unsafe { JavaVM::from_raw(android_context.vm().cast()) }
                .map_err(|err| map_android_error(format!("Failed to load JavaVM: {err}")))?;
            let mut env = vm.attach_current_thread().map_err(|err| {
                map_android_error(format!("Failed to attach JNI thread: {err}"))
            })?;

            let context = unsafe { JObject::from_raw(android_context.context().cast()) };
            let asset_manager_object = env
                .call_method(
                    &context,
                    "getAssets",
                    "()Landroid/content/res/AssetManager;",
                    &[],
                )
                .and_then(|value| value.l())
                .map_err(|err| {
                    map_android_error(format!("Failed to get AssetManager: {err}"))
                })?;

            if asset_manager_object.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Android AssetManager was null",
                ));
            }

            let manager = unsafe {
                tessera_ui::ndk_sys::AAssetManager_fromJava(
                    env.get_native_interface(),
                    asset_manager_object.into_raw(),
                )
            };
            if manager.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    "Failed to convert AssetManager handle",
                ));
            }

            let c_path = CString::new(path).map_err(|err| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid asset path `{path}`: {err}"),
                )
            })?;
            let asset = unsafe {
                tessera_ui::ndk_sys::AAssetManager_open(
                    manager,
                    c_path.as_ptr(),
                    tessera_ui::ndk_sys::AASSET_MODE_BUFFER as i32,
                )
            };
            if asset.is_null() {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("Asset not found: {path}"),
                ));
            }

            let length = unsafe { tessera_ui::ndk_sys::AAsset_getLength64(asset) };
            if length < 0 {
                unsafe {
                    tessera_ui::ndk_sys::AAsset_close(asset);
                }
                return Err(io::Error::other(format!(
                    "Invalid asset length for `{path}`"
                )));
            }

            let mut bytes = vec![0u8; length as usize];
            let mut offset = 0usize;
            while offset < bytes.len() {
                let read = unsafe {
                    tessera_ui::ndk_sys::AAsset_read(
                        asset,
                        bytes[offset..].as_mut_ptr().cast(),
                        (bytes.len() - offset) as _,
                    )
                };
                if read <= 0 {
                    break;
                }
                offset += read as usize;
            }
            unsafe {
                tessera_ui::ndk_sys::AAsset_close(asset);
            }

            if offset != bytes.len() {
                bytes.truncate(offset);
            }
            Ok(Arc::from(bytes))
        }

        impl tessera_ui::AssetExt for Asset {
            fn read(self) -> io::Result<Arc<[u8]>> {
                tessera_ui::asset::read_with_lru_cache::<Asset, _>(self.index as u64, || {
                    if let Some(path) = __TESSERA_ASSET_PATHS.get(self.index) {
                        return __tessera_read_platform_asset(path);
                    }
                    Err(io::Error::new(
                        io::ErrorKind::NotFound,
                        "asset index out of range",
                    ))
                })
            }
        }
    }
}

fn generate_module_body_tokens(node: &ModuleNode) -> Result<TokenStream> {
    let mut items = Vec::new();

    for (const_name, index) in &node.assets {
        let ident = format_ident!("{const_name}");
        let index = *index;
        items.push(quote! {
            pub const #ident: Asset = Asset::new(#index);
        });
    }

    for (module_name, child) in &node.modules {
        let module_ident = format_ident!("{module_name}");
        let child_body = generate_module_body_tokens(child)?;
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
