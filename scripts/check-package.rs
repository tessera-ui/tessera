#!/usr/bin/env rust-script
//! ```cargo
//! [package]
//! edition = "2024"
//! [dependencies]
//! anyhow = "1.0.104"
//! clap = { version = "4.6.7", features = ["derive"] }
//! glob = "0.3.3"
//! icns = "0.5.0"
//! ico = "0.6.0"
//! image = { version = "0.25.10", default-features = false, features = ["png", "ico"] }
//! pelite = "0.10.0"
//! plist = "1.10.3"
//! regex = "1.12.3"
//! resvg = { version = "0.49.0", default-features = false }
//! scraper = "0.29.0"
//! semver = "1.0.27"
//! serde_json = "1.0.154"
//! tempfile = "3.27.0"
//! toml = "1.2.1"
//! wait-timeout = "0.2.1"
//! walkdir = "2.5.0"
//! zip = { version = "9.4.2", default-features = false, features = ["deflate-flate2-zlib-rs"] }
//! ```

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

use anyhow::{Context, Result, ensure};
use clap::{Parser, ValueEnum};
use image::{RgbaImage, imageops};
use regex::Regex;
use scraper::{Html, Selector};
use serde_json::Value;
use walkdir::WalkDir;

mod process;

#[derive(Clone, Copy, ValueEnum)]
enum Platform {
    Deb,
    Windows,
    Macos,
    Ios,
    Android,
    Web,
}

#[derive(Parser)]
#[command(about = "Verify the packaged example against its application configuration")]
struct Cli {
    #[arg(value_enum)]
    platform: Platform,
    artifact: PathBuf,
}

struct App {
    name: String,
    identifier: String,
    version: String,
    numeric_version: String,
    icon: PathBuf,
    binary: String,
    library: String,
}

impl App {
    fn load() -> Result<Self> {
        let example = Path::new(env!("RUST_SCRIPT_PATH"))
            .parent()
            .context("Missing script directory")?
            .parent()
            .context("Missing repository root")?
            .join("example");
        let app: toml::Value =
            toml::from_str(&fs::read_to_string(example.join("tessera-app.toml"))?)?;
        let cargo: toml::Value = toml::from_str(&fs::read_to_string(example.join("Cargo.toml"))?)?;
        let string = |value: &toml::Value| -> Result<String> {
            Ok(value
                .as_str()
                .context("Missing string in application configuration")?
                .to_owned())
        };
        let version = string(&cargo["package"]["version"])?;
        let parsed = semver::Version::parse(&version)?;
        Ok(Self {
            name: string(&app["name"])?,
            identifier: string(&app["identifier"])?,
            numeric_version: format!("{}.{}.{}", parsed.major, parsed.minor, parsed.patch),
            version,
            icon: example.join(string(&app["icon"])?),
            binary: string(&cargo["bin"][0]["name"])?,
            library: string(&cargo["lib"]["name"])?,
        })
    }

    fn artwork(&self, size: u32) -> Result<RgbaImage> {
        if self
            .icon
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("svg"))
        {
            let tree = resvg::usvg::Tree::from_data(&fs::read(&self.icon)?, &Default::default())?;
            let mut pixmap =
                resvg::tiny_skia::Pixmap::new(size, size).context("Invalid artwork size")?;
            resvg::render(
                &tree,
                resvg::tiny_skia::Transform::from_scale(
                    size as f32 / tree.size().width(),
                    size as f32 / tree.size().height(),
                ),
                &mut pixmap.as_mut(),
            );
            let bytes = pixmap
                .pixels()
                .iter()
                .flat_map(|pixel| {
                    let pixel = pixel.demultiply();
                    [pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()]
                })
                .collect();
            RgbaImage::from_raw(size, size, bytes).context("Invalid rendered artwork")
        } else {
            Ok(imageops::resize(
                &image::open(&self.icon)?.into_rgba8(),
                size,
                size,
                imageops::FilterType::Lanczos3,
            ))
        }
    }

    fn icon(
        &self,
        actual: RgbaImage,
        inset: bool,
        monochrome: bool,
        size: Option<u32>,
    ) -> Result<()> {
        let (width, height) = actual.dimensions();
        ensure!(
            width == height && width >= 16,
            "Invalid icon dimensions: {width}x{height}"
        );
        if let Some(size) = size {
            equal("icon size", &width, &size)?;
        }
        let artwork_size = if inset { width * 44 / 108 } else { width };
        let mut artwork = self.artwork(artwork_size)?;
        if monochrome {
            for pixel in artwork.pixels_mut() {
                pixel.0[..3].fill(0);
            }
        }
        let mut expected = RgbaImage::new(width, height);
        let offset = i64::from((width - artwork_size) / 2);
        imageops::replace(&mut expected, &artwork, offset, offset);
        let composite = |pixel: &image::Rgba<u8>, channel: usize| {
            let alpha = u32::from(pixel[3]);
            ((u32::from(pixel[channel]) * alpha + 255 * (255 - alpha) + 127) / 255) as f64
        };
        let (mut contrast, mut error) = (0.0, 0.0);
        for (actual, expected) in actual.pixels().zip(expected.pixels()) {
            for channel in 0..3 {
                let reference = composite(expected, channel);
                contrast += (reference - 255.0).abs();
                error += (composite(actual, channel) - reference).abs();
            }
        }
        ensure!(
            contrast > 0.0 && error < contrast * 0.15,
            "Packaged icon differs from {} (error {error}, contrast {contrast})",
            self.icon.display()
        );
        Ok(())
    }

    fn icon_file(&self, path: &Path, size: Option<u32>) -> Result<()> {
        self.icon(image::open(path)?.into_rgba8(), false, false, size)
    }
}

fn equal<T: PartialEq + std::fmt::Debug + ?Sized>(
    field: &str,
    actual: &T,
    expected: &T,
) -> Result<()> {
    ensure!(
        actual == expected,
        "{field}: expected {expected:?}, got {actual:?}"
    );
    Ok(())
}

fn run(command: &mut Command) -> Result<String> {
    process::run(command, Duration::from_secs(120))
}

fn paths(pattern: &Path) -> Result<Vec<PathBuf>> {
    glob::glob(pattern.to_str().context("Non-UTF-8 artifact path")?)?
        .map(|path| Ok(path?))
        .collect()
}

fn one(paths: Vec<PathBuf>) -> Result<PathBuf> {
    ensure!(paths.len() == 1, "Expected one artifact, found {paths:?}");
    Ok(paths.into_iter().next().unwrap())
}

fn deb(app: &App, directory: &Path, scratch: &Path) -> Result<()> {
    let package = one(paths(&directory.join("*.deb"))?)?;
    equal(
        "Debian Package",
        &run(Command::new("dpkg-deb")
            .arg("--field")
            .arg(&package)
            .arg("Package"))?,
        &app.identifier.to_lowercase(),
    )?;
    equal(
        "Debian Version",
        &run(Command::new("dpkg-deb")
            .arg("--field")
            .arg(&package)
            .arg("Version"))?,
        &app.version,
    )?;
    run(Command::new("dpkg-deb")
        .arg("--extract")
        .arg(package)
        .arg(scratch))?;
    let launcher = app.identifier.replace('.', "-");
    let desktop =
        fs::read_to_string(scratch.join(format!("usr/share/applications/{launcher}.desktop")))?;
    let mut in_entry = false;
    let mut entry = BTreeMap::new();
    for line in desktop.lines().map(str::trim) {
        if line.starts_with('[') {
            in_entry = line == "[Desktop Entry]";
        } else if in_entry && !line.starts_with('#') {
            if let Some((key, value)) = line.split_once('=') {
                entry.insert(key, value);
            }
        }
    }
    equal(
        "desktop Name",
        entry.get("Name").copied().unwrap_or(""),
        app.name.as_str(),
    )?;
    equal(
        "desktop executable",
        entry
            .get("Exec")
            .and_then(|value| value.split_whitespace().next())
            .unwrap_or(""),
        launcher.as_str(),
    )?;
    ensure!(
        scratch.join("usr/bin").join(&launcher).is_file(),
        "Missing desktop executable"
    );
    let icon = entry.get("Icon").context("Missing desktop icon")?;
    let icons = paths(&scratch.join(format!("usr/share/icons/hicolor/*x*/apps/{icon}.png")))?;
    ensure!(!icons.is_empty(), "Missing desktop icon {icon}");
    for icon in icons {
        app.icon_file(&icon, None)?;
    }
    Ok(())
}

fn pe_metadata(app: &App, path: &Path, application: bool) -> Result<()> {
    let bytes = fs::read(path)?;
    let pe = pelite::PeFile::from_bytes(&bytes)?;
    let resources = pe.resources()?;
    let version = resources.version_info()?;
    let file_info = version.file_info();
    let value = |key: &str| {
        file_info
            .strings
            .values()
            .find_map(|table| table.get(key))
            .cloned()
            .with_context(|| format!("Missing PE {key}"))
    };
    for (key, expected) in [("ProductName", &app.name), ("ProductVersion", &app.version)] {
        equal(key, &value(key)?, expected)?;
    }
    if application {
        equal("InternalName", &value("InternalName")?, &app.identifier)?;
    }
    let (_, group) = resources
        .icons()
        .next()
        .context("Missing PE icon group")??;
    let entries = group.entries();
    equal(
        "PE icon frame count",
        &entries.len(),
        &(group.header().idCount as usize),
    )?;
    let frame = entries
        .iter()
        .max_by_key(|entry| {
            let width = if entry.bWidth == 0 {
                256
            } else {
                u32::from(entry.bWidth)
            };
            let height = if entry.bHeight == 0 {
                256
            } else {
                u32::from(entry.bHeight)
            };
            width * height
        })
        .context("Empty PE icon")?;
    app.icon(
        decode_pe_icon(frame, group.image(frame.nId)?)?,
        false,
        false,
        None,
    )
}

fn decode_pe_icon(
    entry: &pelite::resources::group::image::GRPICONDIRENTRY,
    pixels: &[u8],
) -> Result<RgbaImage> {
    // PE group entries can report stale byte lengths; use the actual image resource size.
    let mut ico = Vec::with_capacity(22 + pixels.len());
    ico.extend_from_slice(&[0, 0, 1, 0, 1, 0]);
    ico.extend_from_slice(&[
        entry.bWidth,
        entry.bHeight,
        entry.bColorCount,
        entry.bReserved,
    ]);
    ico.extend_from_slice(&entry.wPlanes.to_le_bytes());
    ico.extend_from_slice(&entry.wBitCount.to_le_bytes());
    ico.extend_from_slice(&u32::try_from(pixels.len())?.to_le_bytes());
    ico.extend_from_slice(&22u32.to_le_bytes());
    ico.extend_from_slice(pixels);
    let dir = ico::IconDir::read(Cursor::new(ico))?;
    let frame = dir.entries()[0].decode()?;
    RgbaImage::from_raw(frame.width(), frame.height(), frame.rgba_data().to_vec())
        .context("Invalid PE icon")
}

fn windows(app: &App, directory: &Path, scratch: &Path) -> Result<()> {
    let installer = one(paths(&directory.join("*-setup.exe"))?)?;
    pe_metadata(app, &installer, false)?;
    let executable = format!("{}.exe", app.binary);
    let seven_zip = std::env::var_os("ProgramFiles")
        .map(|folder| PathBuf::from(folder).join("7-Zip/7z.exe"))
        .filter(|path| path.is_file())
        .unwrap_or_else(|| PathBuf::from("7z"));
    run(Command::new(seven_zip)
        .arg("e")
        .arg(installer)
        .arg(format!("-o{}", scratch.display()))
        .arg("-y")
        .arg(&executable))?;
    pe_metadata(app, &scratch.join(executable), true)
}

fn apple(app: &App, bundle: &Path, scratch: &Path, ios: bool) -> Result<()> {
    let contents = if ios {
        bundle.to_owned()
    } else {
        bundle.join("Contents")
    };
    let info = plist::Value::from_file(contents.join("Info.plist"))?;
    let info = info.as_dictionary().context("Invalid bundle plist")?;
    for (key, expected) in [
        ("CFBundleName", &app.name),
        ("CFBundleDisplayName", &app.name),
        ("CFBundleIdentifier", &app.identifier),
        (
            "CFBundleShortVersionString",
            if ios {
                &app.numeric_version
            } else {
                &app.version
            },
        ),
    ] {
        equal(
            key,
            info.get(key)
                .and_then(plist::Value::as_string)
                .unwrap_or(""),
            expected.as_str(),
        )?;
    }
    let executable = info
        .get("CFBundleExecutable")
        .and_then(plist::Value::as_string)
        .context("Missing bundle executable")?;
    ensure!(
        (if ios {
            contents.clone()
        } else {
            contents.join("MacOS")
        })
        .join(executable)
        .is_file(),
        "Missing bundle executable"
    );
    if ios {
        equal(
            "iOS build version",
            info.get("CFBundleVersion")
                .and_then(plist::Value::as_string)
                .unwrap_or(""),
            app.numeric_version.as_str(),
        )?;
        equal(
            "iOS bundle platform",
            info.get("CFBundleSupportedPlatforms")
                .context("Missing supported platforms")?,
            &plist::Value::Array(vec!["iPhoneSimulator".into()]),
        )?;
        run(Command::new("codesign")
            .args(["--verify", "--strict"])
            .arg(bundle))?;
        ensure!(
            fs::metadata(contents.join("Assets.car"))?.len() > 0,
            "Empty asset catalog"
        );
        for key in ["CFBundleIcons", "CFBundleIcons~ipad"] {
            let files = info
                .get(key)
                .and_then(plist::Value::as_dictionary)
                .and_then(|icons| icons.get("CFBundlePrimaryIcon"))
                .and_then(plist::Value::as_dictionary)
                .and_then(|primary| primary.get("CFBundleIconFiles"))
                .and_then(plist::Value::as_array)
                .context(format!("{key} has no primary icon files"))?;
            ensure!(!files.is_empty(), "{key} has no primary icon files");
            for name in files {
                let name = name.as_string().context("Invalid icon name")?;
                let stem = name.strip_suffix(".png").unwrap_or(name);
                let mut icons = paths(&contents.join(format!("{stem}.png")))?;
                icons.extend(paths(&contents.join(format!("{stem}@*.png")))?);
                ensure!(!icons.is_empty(), "{key} points to missing icon {name}");
                for icon in icons {
                    let decoded = scratch.join("decoded.png");
                    run(Command::new("sips")
                        .args(["-s", "format", "png"])
                        .arg(icon)
                        .arg("--out")
                        .arg(&decoded))?;
                    app.icon_file(&decoded, None)?;
                }
            }
        }
    } else {
        let icon = Path::new(
            info.get("CFBundleIconFile")
                .and_then(plist::Value::as_string)
                .context("Missing macOS icon")?,
        );
        let icon = contents
            .join("Resources")
            .join(if icon.extension().is_some() {
                icon.to_owned()
            } else {
                icon.with_extension("icns")
            });
        let family = icns::IconFamily::read(fs::File::open(icon)?)?;
        let kind = family
            .available_icons()
            .into_iter()
            .max_by_key(|kind| kind.pixel_width() * kind.pixel_height())
            .context("Empty macOS icon")?;
        let image = family
            .get_icon_with_type(kind)?
            .convert_to(icns::PixelFormat::RGBA);
        app.icon(
            RgbaImage::from_raw(image.width(), image.height(), image.data().to_vec())
                .context("Invalid macOS icon")?,
            false,
            false,
            None,
        )?;
    }
    Ok(())
}

fn android(app: &App, directory: &Path) -> Result<()> {
    let sdk = std::env::var_os("ANDROID_HOME")
        .or_else(|| std::env::var_os("ANDROID_SDK_ROOT"))
        .context("Set ANDROID_HOME or ANDROID_SDK_ROOT")?;
    let tool = if cfg!(windows) { "aapt2.exe" } else { "aapt2" };
    let digits = Regex::new(r"\d+")?;
    let aapt = WalkDir::new(Path::new(&sdk).join("build-tools"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name() == tool)
        .map(|entry| entry.into_path())
        .max_by_key(|path| {
            digits
                .find_iter(
                    &path
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_string_lossy(),
                )
                .filter_map(|number| number.as_str().parse::<u32>().ok())
                .collect::<Vec<_>>()
        })
        .context("No Android aapt2 tool found")?;
    let apks = if directory.is_file() {
        vec![directory.to_owned()]
    } else {
        WalkDir::new(directory)
            .into_iter()
            .filter_map(Result::ok)
            .map(|entry| entry.into_path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "apk"))
            .collect()
    };
    ensure!(!apks.is_empty(), "No APKs under {}", directory.display());
    let version = semver::Version::parse(&app.version)?;
    let version_code =
        (version.major * 1_000_000 + version.minor * 1_000 + version.patch + 1).to_string();
    let fields = Regex::new(r"(\w+)='([^']*)'")?;
    let icon_id = Regex::new(r":icon\([^)]*\)=@(0x[0-9a-f]+)")?;
    let sections = Regex::new(r"(?m)^ +resource ")?;
    let files = Regex::new(r"\(file\) (\S+)")?;
    let layer =
        Regex::new(r"E: (foreground|monochrome)[^\n]*\n\s+A: .*:drawable\([^)]*\)=@(0x[0-9a-f]+)")?;
    for apk in apks {
        println!("Checking {}", apk.display());
        let dump = |kind: &str| run(Command::new(&aapt).args(["dump", kind]).arg(&apk));
        let badging = dump("badging")?;
        let package = badging
            .lines()
            .find(|line| line.starts_with("package: "))
            .context("Missing APK package")?;
        let fields = fields
            .captures_iter(package)
            .map(|capture| (capture[1].to_owned(), capture[2].to_owned()))
            .collect::<BTreeMap<_, _>>();
        for (key, expected) in [
            ("name", &app.identifier),
            ("versionName", &app.version),
            ("versionCode", &version_code),
        ] {
            equal(
                key,
                fields.get(key).context(format!("Missing {key}"))?,
                expected,
            )?;
        }
        let label = badging
            .lines()
            .find_map(|line| line.strip_prefix("application-label:"))
            .context("Missing APK label")?;
        equal(
            "APK application label",
            label.trim_matches('\''),
            app.name.as_str(),
        )?;
        let xml = |path: &str| {
            run(Command::new(&aapt)
                .args(["dump", "xmltree"])
                .arg(&apk)
                .args(["--file", path]))
        };
        let manifest = xml("AndroidManifest.xml")?;
        let icon = icon_id
            .captures(&manifest)
            .context("Missing APK icon resource")?[1]
            .to_owned();
        let resources = dump("resources")?;
        let resources = sections
            .split(&resources)
            .skip(1)
            .map(|section| {
                (
                    section.split_whitespace().next().unwrap_or("").to_owned(),
                    files
                        .captures_iter(section)
                        .map(|capture| capture[1].to_owned())
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut archive = zip::ZipArchive::new(fs::File::open(&apk)?)?;
        let icons = resources
            .get(&icon)
            .context("Missing launcher icon resource")?;
        ensure!(
            icons.iter().any(|path| path.ends_with(".png")),
            "APK has no legacy launcher icon"
        );
        let mut layers = BTreeSet::new();
        for icon in icons {
            let mut decode = |path: &str| -> Result<RgbaImage> {
                let mut bytes = Vec::new();
                archive.by_name(path)?.read_to_end(&mut bytes)?;
                Ok(image::load_from_memory(&bytes)?.into_rgba8())
            };
            if icon.ends_with(".png") {
                app.icon(decode(icon)?, false, false, None)?;
                continue;
            }
            for capture in layer.captures_iter(&xml(icon)?) {
                layers.insert(capture[1].to_owned());
                let paths = resources
                    .get(&capture[2])
                    .context("Missing adaptive icon resource")?;
                ensure!(
                    !paths.is_empty(),
                    "Empty adaptive icon layer {}",
                    &capture[1]
                );
                for path in paths {
                    app.icon(decode(path)?, true, &capture[1] == "monochrome", None)?;
                }
            }
        }
        equal(
            "adaptive icon layers",
            &layers,
            &BTreeSet::from(["foreground".to_owned(), "monochrome".to_owned()]),
        )?;
    }
    Ok(())
}

fn web(app: &App, host: &Path) -> Result<()> {
    let html = Html::parse_document(&fs::read_to_string(host.join("index.html"))?);
    let title = html
        .select(&Selector::parse("title").unwrap())
        .flat_map(|element| element.text())
        .collect::<String>();
    equal("Web title", &title, &app.name)?;
    let links = html
        .select(&Selector::parse("link[rel][href]").unwrap())
        .map(|element| {
            (
                element.value().attr("rel").unwrap(),
                element.value().attr("href").unwrap(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let manifest_path = host.join(links.get("manifest").context("Missing Web manifest link")?);
    let manifest: Value = serde_json::from_str(&fs::read_to_string(&manifest_path)?)?;
    for (key, expected) in [
        ("name", &app.name),
        ("id", &app.identifier),
        ("version", &app.version),
    ] {
        equal(key, manifest[key].as_str().unwrap_or(""), expected.as_str())?;
    }
    app.icon_file(
        &host.join(links.get("icon").context("Missing favicon link")?),
        None,
    )?;
    app.icon_file(
        &host.join(
            links
                .get("apple-touch-icon")
                .context("Missing Apple touch icon link")?,
        ),
        Some(180),
    )?;
    let icons = manifest["icons"]
        .as_array()
        .context("Missing Web manifest icons")?;
    let sizes = icons
        .iter()
        .filter_map(|icon| icon["sizes"].as_str())
        .collect::<BTreeSet<_>>();
    equal(
        "Web manifest icon sizes",
        &sizes,
        &BTreeSet::from(["192x192", "512x512"]),
    )?;
    for icon in icons {
        let size = icon["sizes"]
            .as_str()
            .context("Missing icon size")?
            .split('x')
            .next()
            .unwrap()
            .parse()?;
        app.icon_file(
            &manifest_path
                .parent()
                .unwrap()
                .join(icon["src"].as_str().context("Missing icon source")?),
            Some(size),
        )?;
    }
    let mut header = [0; 8];
    fs::File::open(host.join(format!("pkg-web/{}_bg.wasm", app.library)))?
        .read_exact(&mut header)?;
    equal("WebAssembly header", &header, b"\0asm\x01\0\0\0")
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let app = App::load()?;
    let scratch = tempfile::tempdir()?;
    println!("Checking artifacts at {}", cli.artifact.display());
    match cli.platform {
        Platform::Deb => deb(&app, &cli.artifact, scratch.path()),
        Platform::Windows => windows(&app, &cli.artifact, scratch.path()),
        Platform::Macos => apple(&app, &cli.artifact, scratch.path(), false),
        Platform::Ios => apple(&app, &cli.artifact, scratch.path(), true),
        Platform::Android => android(&app, &cli.artifact),
        Platform::Web => web(&app, &cli.artifact),
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;

    fn app_with_icon(directory: &Path) -> Result<App> {
        let mut app = App::load()?;
        app.icon = directory.join("source.png");
        RgbaImage::from_pixel(32, 32, image::Rgba([192, 32, 64, 255])).save(&app.icon)?;
        Ok(app)
    }

    #[test]
    fn icon_comparison_accepts_source_and_rejects_wrong_pixels() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let app = app_with_icon(directory.path())?;
        app.icon(
            RgbaImage::from_pixel(64, 64, image::Rgba([192, 32, 64, 255])),
            false,
            false,
            Some(64),
        )?;
        let wrong = RgbaImage::from_pixel(64, 64, image::Rgba([255, 255, 255, 255]));
        assert!(app.icon(wrong, false, false, Some(64)).is_err());
        Ok(())
    }

    #[test]
    fn web_inspection_checks_icons_manifest_and_wasm_header() -> Result<()> {
        let host = tempfile::tempdir()?;
        let app = app_with_icon(host.path())?;
        fs::create_dir_all(host.path().join("pkg-web"))?;
        fs::write(
            host.path().join("index.html"),
            format!(
                "<title>{}</title><link rel=\"manifest\" href=\"manifest.json\"><link rel=\"icon\" href=\"icon.png\"><link rel=\"apple-touch-icon\" href=\"apple.png\">",
                app.name
            ),
        )?;
        let manifest = serde_json::json!({
            "name":app.name, "id":app.identifier, "version":app.version,
            "icons":[
                {"sizes":"192x192", "src":"192.png"},
                {"sizes":"512x512", "src":"512.png"}
            ]
        });
        fs::write(
            host.path().join("manifest.json"),
            serde_json::to_vec(&manifest)?,
        )?;
        for (name, size) in [
            ("icon.png", 64),
            ("apple.png", 180),
            ("192.png", 192),
            ("512.png", 512),
        ] {
            RgbaImage::from_pixel(size, size, image::Rgba([192, 32, 64, 255]))
                .save(host.path().join(name))?;
        }
        fs::File::create(host.path().join(format!("pkg-web/{}_bg.wasm", app.library)))?
            .write_all(b"\0asm\x01\0\0\0")?;
        web(&app, host.path())?;
        let mut bad = manifest;
        bad["version"] = serde_json::json!("0.0.0");
        fs::write(host.path().join("manifest.json"), serde_json::to_vec(&bad)?)?;
        assert!(web(&app, host.path()).is_err());
        Ok(())
    }
}
