//! Shared application identity and icon sources.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use image::{
    ImageFormat, RgbaImage,
    codecs::ico::{IcoEncoder, IcoFrame},
    imageops::FilterType,
};
use proc_macro2::TokenStream;
use quote::quote;
use resvg::{tiny_skia, usvg};
use serde::Deserialize;

/// Application metadata from `tessera-app.toml`.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct AppConfig {
    /// Display name; defaults to the Cargo package name.
    pub name: String,
    /// Reverse-domain application identifier, shared across platforms.
    pub identifier: String,
    /// Square SVG (recommended) or PNG source, relative to the package root.
    pub icon: Option<PathBuf>,
}

impl AppConfig {
    /// Loads and validates application metadata, resolving the icon path.
    pub fn load(project: &Path, package_name: &str) -> Result<Self> {
        let path = project.join("tessera-app.toml");
        let mut config: Self = match fs::read_to_string(&path) {
            Ok(source) => {
                toml::from_str(&source).with_context(|| format!("Invalid {}", path.display()))?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(error) => {
                return Err(error).with_context(|| format!("Failed to read {}", path.display()));
            }
        };
        if config.name.is_empty() {
            config.name = package_name.into();
        }
        ensure!(
            !config.name.trim().is_empty() && !config.name.chars().any(char::is_control),
            "Application name must contain visible text and no control characters"
        );
        if config.identifier.is_empty() {
            let suffix = package_name
                .chars()
                .filter(char::is_ascii_alphanumeric)
                .collect::<String>();
            let prefix = if suffix
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_alphabetic)
            {
                ""
            } else {
                "app"
            };
            config.identifier = format!("com.tessera.{prefix}{suffix}");
        }
        ensure!(
            config.identifier.contains('.')
                && config.identifier.split('.').all(|segment| {
                    segment
                        .as_bytes()
                        .first()
                        .is_some_and(u8::is_ascii_alphabetic)
                        && segment.bytes().all(|byte| byte.is_ascii_alphanumeric())
                }),
            "Application identifier must have at least two dot-separated segments, each starting with an ASCII letter and containing only ASCII letters and digits"
        );
        config.icon = config.icon.map(|path| project.join(path));
        Ok(config)
    }

    /// Loads the configured icon, if present.
    pub fn load_icon(&self) -> Result<Option<AppIcon>> {
        self.icon.as_deref().map(AppIcon::load).transpose()
    }

    /// Linux launcher ID and executable name.
    pub fn linux_launcher_name(&self) -> String {
        self.identifier.replace('.', "-")
    }
}

/// A square icon source rendered independently at each output resolution.
pub struct AppIcon(IconSource);

enum IconSource {
    Svg(usvg::Tree),
    Png(RgbaImage),
}

impl AppIcon {
    /// Loads an SVG or PNG. SVG text must be converted to paths and external resources embedded.
    pub fn load(path: &Path) -> Result<Self> {
        let bytes =
            fs::read(path).with_context(|| format!("Failed to read icon {}", path.display()))?;
        let source = match path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .as_deref()
        {
            Some("svg") => {
                let xml =
                    std::str::from_utf8(&bytes).context("SVG application icon must be UTF-8")?;
                let document = usvg::roxmltree::Document::parse(xml)
                    .context("Invalid SVG application icon")?;
                for node in document.descendants().filter(|node| node.is_element()) {
                    ensure!(
                        node.tag_name().name() != "text",
                        "Convert SVG application icon text to paths"
                    );
                    for attribute in node
                        .attributes()
                        .filter(|attribute| attribute.name() == "href")
                    {
                        ensure!(
                            attribute.value().starts_with('#')
                                || attribute.value().starts_with("data:"),
                            "Embed external SVG application icon resources"
                        );
                    }
                }
                let options = usvg::Options {
                    image_href_resolver: usvg::ImageHrefResolver {
                        resolve_string: Box::new(|_, _| None),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                let tree = usvg::Tree::from_data(&bytes, &options)
                    .context("Invalid SVG application icon")?;
                ensure!(
                    tree.size().width() == tree.size().height(),
                    "Application icon must be square"
                );
                IconSource::Svg(tree)
            }
            Some("png") => {
                let image =
                    image::load_from_memory_with_format(&bytes, ImageFormat::Png)?.into_rgba8();
                ensure!(
                    image.width() == image.height(),
                    "Application icon must be square"
                );
                IconSource::Png(image)
            }
            _ => bail!(
                "Unsupported application icon {}; use SVG (recommended) or PNG",
                path.display()
            ),
        };
        Ok(Self(source))
    }

    /// Renders straight-alpha RGBA pixels at the requested size.
    pub fn render(&self, size: u32) -> Result<RgbaImage> {
        ensure!(
            (1..=4096).contains(&size),
            "Icon output size must be between 1 and 4096 pixels"
        );
        match &self.0 {
            IconSource::Png(image) => Ok(image::imageops::resize(
                image,
                size,
                size,
                FilterType::Lanczos3,
            )),
            IconSource::Svg(tree) => {
                let mut pixmap = tiny_skia::Pixmap::new(size, size)
                    .context("Failed to allocate application icon")?;
                let scale = size as f32 / tree.size().width();
                resvg::render(
                    tree,
                    tiny_skia::Transform::from_scale(scale, scale),
                    &mut pixmap.as_mut(),
                );
                let pixels = pixmap
                    .pixels()
                    .iter()
                    .flat_map(|pixel| {
                        let pixel = pixel.demultiply();
                        [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
                    })
                    .collect();
                RgbaImage::from_raw(size, size, pixels).context("Invalid rendered application icon")
            }
        }
    }

    /// Writes a Windows icon containing the standard small and large sizes.
    pub fn write_ico(&self, path: &Path) -> Result<()> {
        let frames = [16, 24, 32, 48, 64, 128, 256]
            .into_iter()
            .map(|size| {
                Ok(IcoFrame::as_png(
                    self.render(size)?.as_raw(),
                    size,
                    size,
                    image::ExtendedColorType::Rgba8,
                )?)
            })
            .collect::<Result<Vec<_>>>()?;
        IcoEncoder::new(fs::File::create(path)?).encode_images(&frames)?;
        Ok(())
    }
}

pub(super) fn generate(project: &Path, output: &Path, package_name: &str) -> Result<TokenStream> {
    println!(
        "cargo:rerun-if-changed={}",
        project.join("tessera-app.toml").display()
    );
    let config = AppConfig::load(project, package_name)?;
    let name = &config.name;
    let target_os = std::env::var("CARGO_CFG_TARGET_OS")?;
    let identifier = if target_os == "linux" {
        config.linux_launcher_name()
    } else {
        config.identifier.clone()
    };
    let mut icon = quote!(None);
    if let Some(source) = &config.icon {
        println!("cargo:rerun-if-changed={}", source.display());
    }
    let source = config.load_icon()?;
    if let Some(source) = &source {
        let rgba = output.join("app-icon.rgba");
        fs::write(&rgba, source.render(256)?.as_raw())?;
        let rgba = rgba.to_string_lossy();
        icon = quote!(Some(
            tessera_ui::winit::window::Icon::from_rgba(include_bytes!(#rgba).to_vec(), 256, 256,)
                .expect("Invalid generated application icon")
        ));
    }
    if target_os == "windows" && project.join("tessera-app.toml").is_file() {
        let mut resource = String::from("#pragma code_page(65001)\n");
        if let Some(icon) = &source {
            let ico = output.join("app-icon.ico");
            icon.write_ico(&ico)?;
            resource.push_str(&format!(
                "1 ICON \"{}\"\n",
                ico.to_string_lossy().replace('\\', "/")
            ));
        }
        let version = std::env::var("CARGO_PKG_VERSION")?;
        let numeric = version
            .split(['.', '-', '+'])
            .take(3)
            .map(|part| {
                part.parse::<u16>()
                    .context("Windows application version components must fit in 16 bits")
            })
            .collect::<Result<Vec<_>>>()?;
        let numeric = numeric
            .iter()
            .map(u16::to_string)
            .collect::<Vec<_>>()
            .join(",");
        resource.push_str(&format!(
            r#"1 VERSIONINFO
FILEVERSION {numeric},0
PRODUCTVERSION {numeric},0
BEGIN
    BLOCK "StringFileInfo"
    BEGIN
        BLOCK "040904b0"
        BEGIN
"#
        ));
        for (key, value) in [
            ("ProductName", name.as_str()),
            ("FileDescription", name.as_str()),
            ("InternalName", identifier.as_str()),
            ("FileVersion", version.as_str()),
            ("ProductVersion", version.as_str()),
        ] {
            let value = value.replace('\\', "\\\\").replace('"', "\"\"");
            resource.push_str(&format!("            VALUE \"{key}\", \"{value}\\0\"\n"));
        }
        resource.push_str(
            r#"        END
    END
    BLOCK "VarFileInfo"
    BEGIN
        VALUE "Translation", 0x409, 1200
    END
END
"#,
        );
        let rc = output.join("application.rc");
        fs::write(&rc, resource)?;
        embed_resource::compile(&rc, embed_resource::NONE).manifest_required()?;
    }
    Ok(quote! {
        /// Returns application defaults from `tessera-app.toml`.
        pub fn application() -> tessera_ui::renderer::TesseraConfig {
            #[cfg(not(any(target_os = "android", target_os = "ios", target_family = "wasm")))]
            let icon = #icon;
            #[cfg(any(target_os = "android", target_os = "ios", target_family = "wasm"))]
            let icon = None;
            tessera_ui::renderer::TesseraConfig {
                window_title: #name.into(),
                window: tessera_ui::renderer::WindowConfig {
                    application_id: Some(#identifier.into()),
                    icon,
                    ..Default::default()
                },
                ..Default::default()
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_sources_preserve_color_alpha_and_scale() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let svg = directory.path().join("icon.svg");
        fs::write(
            &svg,
            r##"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2"><path d="M0 0H1V2H0Z" fill="#ff0000" opacity=".5"/></svg>"##,
        )?;
        let icon = AppIcon::load(&svg)?;
        for size in [16, 32, 256] {
            let image = icon.render(size)?;
            assert_eq!(image.dimensions(), (size, size));
            assert_eq!(image.get_pixel(size / 4, size / 2).0, [255, 0, 0, 128]);
            assert_eq!(image.get_pixel(size * 3 / 4, size / 2).0, [0, 0, 0, 0]);
        }
        let png = directory.path().join("icon.png");
        RgbaImage::from_pixel(16, 16, image::Rgba([0, 128, 255, 128])).save(&png)?;
        let image = AppIcon::load(&png)?.render(32)?;
        assert_eq!(image.get_pixel(16, 16).0, [0, 128, 255, 128]);
        Ok(())
    }

    #[test]
    fn icons_reject_sources_that_cannot_render_portably() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let svg = directory.path().join("icon.svg");
        for (body, diagnostic) in [
            (r#"<text x="0" y="1">App</text>"#, "text to paths"),
            (r#"<image href="other.png"/>"#, "Embed external"),
        ] {
            fs::write(
                &svg,
                format!(
                    r#"<svg xmlns="http://www.w3.org/2000/svg" width="2" height="2">{body}</svg>"#
                ),
            )?;
            assert!(
                matches!(AppIcon::load(&svg), Err(error) if error.to_string().contains(diagnostic))
            );
        }
        let png = directory.path().join("icon.png");
        RgbaImage::new(16, 32).save(&png)?;
        assert!(matches!(AppIcon::load(&png), Err(error) if error.to_string().contains("square")));
        Ok(())
    }

    #[test]
    fn shared_identity_defaults_and_validation_are_platform_independent() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let config = AppConfig::load(directory.path(), "my-app")?;
        assert_eq!(config.name, "my-app");
        assert_eq!(config.identifier, "com.tessera.myapp");
        let path = directory.path().join("tessera-app.toml");
        for identifier in ["single", "com.1app", "com.my-app", "com.my_app", "com..app"] {
            fs::write(&path, format!("identifier = {identifier:?}"))?;
            assert!(
                AppConfig::load(directory.path(), "app").is_err(),
                "{identifier}"
            );
        }
        Ok(())
    }
}
