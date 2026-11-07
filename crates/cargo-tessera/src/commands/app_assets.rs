use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::Result;
use image::{Rgba, RgbaImage, imageops};
use serde_json::json;
use tessera_build::application::{AppConfig, AppIcon};

fn png(icon: &AppIcon, path: &Path, size: u32, opaque: bool) -> Result<()> {
    let mut pixels = icon.render(size)?;
    if opaque {
        let mut background = RgbaImage::from_pixel(size, size, Rgba([255, 255, 255, 255]));
        imageops::overlay(&mut background, &pixels, 0, 0);
        pixels = background;
    }
    fs::create_dir_all(path.parent().unwrap_or(Path::new(".")))?;
    if opaque {
        image::DynamicImage::ImageRgba8(pixels)
            .into_rgb8()
            .save(path)?;
    } else {
        pixels.save(path)?;
    }
    Ok(())
}

pub(super) fn desktop(config: &AppConfig, output: &Path) -> Result<Option<Vec<PathBuf>>> {
    let Some(icon) = config.load_icon()? else {
        return Ok(None);
    };
    fs::create_dir_all(output)?;
    let mut paths = Vec::new();
    for size in [16, 32, 64, 128, 256, 512, 1024] {
        let path = output.join(format!("icon-{size}.png"));
        png(&icon, &path, size, false)?;
        paths.push(path);
    }
    let ico = output.join("icon.ico");
    icon.write_ico(&ico)?;
    paths.push(ico);
    Ok(Some(paths))
}

pub(super) fn android(config: &AppConfig, resources: &Path) -> Result<()> {
    let values = resources.join("values");
    fs::create_dir_all(&values)?;
    let name = handlebars::html_escape(&config.name.replace('\\', "\\\\").replace('"', "\\\""));
    fs::write(
        values.join("strings.xml"),
        format!(
            "<resources><string name=\"tessera_app_name\" formatted=\"false\">\"{name}\"</string></resources>\n"
        ),
    )?;
    let Some(icon) = config.load_icon()? else {
        return Ok(());
    };
    for (density, size, adaptive) in [
        ("mdpi", 48, 108),
        ("hdpi", 72, 162),
        ("xhdpi", 96, 216),
        ("xxhdpi", 144, 324),
        ("xxxhdpi", 192, 432),
    ] {
        png(
            &icon,
            &resources.join(format!("mipmap-{density}/tessera_icon.png")),
            size,
            true,
        )?;
        let foreground = resources.join(format!("drawable-{density}"));
        fs::create_dir_all(&foreground)?;
        let artwork_size = adaptive * 44 / 108;
        let artwork = icon.render(artwork_size)?;
        let mut pixels = RgbaImage::new(adaptive, adaptive);
        let offset = i64::from((adaptive - artwork_size) / 2);
        imageops::overlay(&mut pixels, &artwork, offset, offset);
        pixels.save(foreground.join("tessera_icon_foreground.png"))?;
        for pixel in pixels.pixels_mut() {
            pixel.0[..3].fill(0);
        }
        pixels.save(foreground.join("tessera_icon_monochrome.png"))?;
    }
    for (qualifier, monochrome) in [
        ("v26", ""),
        (
            "v33",
            "<monochrome android:drawable=\"@drawable/tessera_icon_monochrome\" />",
        ),
    ] {
        let directory = resources.join(format!("mipmap-anydpi-{qualifier}"));
        fs::create_dir_all(&directory)?;
        fs::write(
            directory.join("tessera_icon.xml"),
            format!(
                r#"<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@android:color/white" />
    <foreground android:drawable="@drawable/tessera_icon_foreground" />
    {monochrome}
</adaptive-icon>
"#
            ),
        )?;
    }
    Ok(())
}

pub(super) fn ios(config: &AppConfig, output: &Path) -> Result<Option<PathBuf>> {
    let Some(icon) = config.load_icon()? else {
        return Ok(None);
    };
    let catalog = output.join("Assets.xcassets");
    let directory = catalog.join("AppIcon.appiconset");
    fs::create_dir_all(&directory)?;
    let info = json!({"version": 1, "author": "tessera"});
    fs::write(
        catalog.join("Contents.json"),
        serde_json::to_vec(&json!({"info": info}))?,
    )?;
    let mut images = Vec::new();
    let mut rendered = HashSet::new();
    for (idiom, size, scales) in [
        ("iphone", 20.0, &[2, 3][..]),
        ("iphone", 29.0, &[2, 3][..]),
        ("iphone", 40.0, &[2, 3][..]),
        ("iphone", 60.0, &[2, 3][..]),
        ("ipad", 20.0, &[1, 2][..]),
        ("ipad", 29.0, &[1, 2][..]),
        ("ipad", 40.0, &[1, 2][..]),
        ("ipad", 76.0, &[1, 2][..]),
        ("ipad", 83.5, &[2][..]),
        ("ios-marketing", 1024.0, &[1][..]),
    ] {
        for scale in scales {
            let pixels = (size * f64::from(*scale)) as u32;
            let filename = format!("icon-{pixels}.png");
            if rendered.insert(pixels) {
                png(&icon, &directory.join(&filename), pixels, true)?;
            }
            images.push(json!({"idiom": idiom, "size": format!("{size}x{size}"), "scale": format!("{scale}x"), "filename": filename}));
        }
    }
    fs::write(
        directory.join("Contents.json"),
        serde_json::to_vec(&json!({"images": images, "info": info}))?,
    )?;
    Ok(Some(catalog))
}

pub(super) fn web(config: &AppConfig, version: &str, host: &Path) -> Result<String> {
    let output = host.join("app");
    fs::create_dir_all(&output)?;
    let mut icons = Vec::new();
    let mut tags = String::new();
    if let Some(icon) = config.load_icon()? {
        for size in [180, 192, 512] {
            png(&icon, &output.join(format!("icon-{size}.png")), size, false)?;
        }
        for size in [192, 512] {
            icons.push(json!({"src": format!("icon-{size}.png"), "sizes": format!("{size}x{size}"), "type": "image/png", "purpose": "any"}));
        }
        icon.write_ico(&output.join("favicon.ico"))?;
        tags.push_str("<link rel=\"icon\" href=\"./app/favicon.ico\">\n<link rel=\"apple-touch-icon\" href=\"./app/icon-180.png\">\n");
    }
    fs::write(
        output.join("manifest.webmanifest"),
        serde_json::to_vec(&json!({
            "id": config.identifier, "name": config.name, "short_name": config.name, "version": version,
            "start_url": "../", "scope": "../", "display": "standalone", "icons": icons,
        }))?,
    )?;
    tags.push_str("<link rel=\"manifest\" href=\"./app/manifest.webmanifest\">");
    Ok(tags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_source_produces_platform_icon_artifacts() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::write(root.join("icon.svg"), r##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="8"><path d="M2 2H6V6H2Z" fill="#ff0000" opacity=".5"/></svg>"##).unwrap();
        fs::write(
            root.join("tessera-app.toml"),
            "name = \"Reader & Notes\"\nidentifier = \"org.example.reader\"\nicon = \"icon.svg\"\n",
        )
        .unwrap();
        let config = AppConfig::load(root, "fixture").unwrap();
        let paths = desktop(&config, &root.join("desktop")).unwrap().unwrap();
        let ico = paths.last().unwrap();
        let windows = image::open(ico).unwrap().into_rgba8();
        assert_eq!(windows.dimensions(), (256, 256));
        assert_eq!(windows.get_pixel(128, 128).0, [255, 0, 0, 128]);

        let res = root.join("android");
        android(&config, &res).unwrap();
        let foreground = image::open(res.join("drawable-xxxhdpi/tessera_icon_foreground.png"))
            .unwrap()
            .into_rgba8();
        assert_eq!(foreground.dimensions(), (432, 432));
        assert_eq!(foreground.get_pixel(216, 216).0, [255, 0, 0, 128]);
        assert_eq!(foreground.get_pixel(0, 0).0, [0, 0, 0, 0]);
        let monochrome = image::open(res.join("drawable-xxxhdpi/tessera_icon_monochrome.png"))
            .unwrap()
            .into_rgba8();
        assert_eq!(monochrome.get_pixel(216, 216).0, [0, 0, 0, 128]);
        assert_eq!(
            image::open(res.join("mipmap-mdpi/tessera_icon.png"))
                .unwrap()
                .into_rgba8()
                .dimensions(),
            (48, 48)
        );

        let catalog = ios(&config, root)
            .unwrap()
            .unwrap()
            .join("AppIcon.appiconset");
        let metadata: serde_json::Value =
            serde_json::from_slice(&fs::read(catalog.join("Contents.json")).unwrap()).unwrap();
        for entry in metadata["images"].as_array().unwrap() {
            let image = image::open(catalog.join(entry["filename"].as_str().unwrap())).unwrap();
            assert_eq!(image.color(), image::ColorType::Rgb8);
        }
        let marketing = image::open(catalog.join("icon-1024.png"))
            .unwrap()
            .into_rgb8();
        assert_eq!(marketing.get_pixel(0, 0).0, [255, 255, 255]);
        assert_eq!(marketing.get_pixel(512, 512).0, [255, 127, 127]);

        let host = root.join("web");
        web(&config, "1.2.3", &host).unwrap();
        let manifest: serde_json::Value =
            serde_json::from_slice(&fs::read(host.join("app/manifest.webmanifest")).unwrap())
                .unwrap();
        assert_eq!(manifest["name"], "Reader & Notes");
        assert_eq!(manifest["id"], "org.example.reader");
        for icon in manifest["icons"].as_array().unwrap() {
            assert!(
                host.join("app")
                    .join(icon["src"].as_str().unwrap())
                    .is_file()
            );
        }
    }
}
