//! Fonts loaded on demand when text uses their family or Unicode ranges.
//!
//! Text uses system fonts unless an application registers [`Fonts`] or a
//! [`FontStylesheet`] at its entry. Web builds bundle a startup font and load
//! additional fonts from a CDN.

use std::{borrow::Cow, cell::RefCell, io, rc::Rc, sync::Arc};

use glyphon::fontdb::{self, Database, FaceInfo};
use tessera_ui::{EntryRegistry, LoadState, Resource, Source, TesseraPackage};

pub use tessera_ui::FontAsset as Font;

use crate::{text::write_font_system, theme::TextStyle};

/// An entry package supplying fonts on demand.
pub struct Fonts(
    /// Font sources available to this application.
    pub &'static [Font],
);

/// An entry package supplying the fonts declared by a CSS stylesheet's
/// `@font-face` rules, such as a web font CDN. Each rule's file is fetched when
/// text uses a character in its `unicode-range`.
pub struct FontStylesheet(
    /// Stylesheet URL.
    pub &'static str,
);

impl TesseraPackage for Fonts {
    fn register(self, _: &mut EntryRegistry) {
        let registry = registry();
        let mut registry = registry.borrow_mut();
        for font in self.0 {
            let font = FontSource::from(*font);
            if !registry
                .sources
                .iter()
                .any(|source| source.source == font.source)
            {
                registry.sources.push(font);
            }
        }
    }
}

impl TesseraPackage for FontStylesheet {
    fn register(self, _: &mut EntryRegistry) {
        let registry = registry();
        let mut registry = registry.borrow_mut();
        if !registry.stylesheets.iter().any(|sheet| sheet.url == self.0) {
            registry.stylesheets.push(Stylesheet::new(self.0));
        }
    }
}

struct Stylesheet {
    url: &'static str,
    faces: Option<Resource<Arc<[FontFace]>>>,
}

impl Stylesheet {
    fn new(url: &'static str) -> Self {
        Self { url, faces: None }
    }
}

#[derive(Clone)]
struct FontFace {
    family: String,
    src: String,
    ranges: Vec<(u32, u32)>,
}

#[derive(Clone)]
struct FontSource {
    source: Source,
    family: Cow<'static, str>,
    fallback: Cow<'static, [(u32, u32)]>,
    coverage: Cow<'static, [(u32, u32)]>,
}

impl From<Font> for FontSource {
    fn from(font: Font) -> Self {
        Self {
            source: font.asset.into(),
            family: font.family.into(),
            fallback: font.fallback.into(),
            coverage: font.coverage.into(),
        }
    }
}

struct FontRequest {
    font: FontSource,
    faces: Resource<Arc<[FaceInfo]>>,
    /// Revision that installed the font; zero when it failed to load.
    installed: Option<u64>,
}

struct FontRegistry {
    sources: Vec<FontSource>,
    stylesheets: Vec<Stylesheet>,
    requests: Vec<FontRequest>,
    revision: u64,
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            #[cfg(target_family = "wasm")]
            stylesheets: [
                "https://cdn.jsdelivr.net/npm/@fontsource-variable/roboto/index.css",
                "https://cdn.jsdelivr.net/npm/@fontsource-variable/noto-sans-sc/index.css",
            ]
            .map(Stylesheet::new)
            .into(),
            #[cfg(not(target_family = "wasm"))]
            stylesheets: Vec::new(),
            requests: Vec::new(),
            revision: 0,
        }
    }
}

fn registry() -> Rc<RefCell<FontRegistry>> {
    tessera_ui::local_runtime_resource::<RefCell<FontRegistry>>()
}

/// Requests the fonts `text` needs and installs those that have loaded.
///
/// Returns `None` while any of them is loading; otherwise a value that changes
/// only when a font relevant to `text` is installed.
pub(crate) fn prepare(text: &str, style: TextStyle) -> Option<u64> {
    let registry = registry();
    let mut registry = registry.borrow_mut();
    let FontRegistry {
        sources,
        stylesheets,
        requests,
        revision,
    } = &mut *registry;
    let mut pending = false;
    stylesheets.retain_mut(|sheet| {
        match sheet
            .faces
            .get_or_insert_with(|| Resource::load(sheet.url, parse_stylesheet))
            .get()
        {
            LoadState::Loading => {
                pending = true;
                return true;
            }
            LoadState::Ready(faces) => {
                sources.extend(faces.iter().map(|face| face.font(sheet.url)));
            }
            LoadState::Failed(_) => {}
        }
        false
    });
    let family = style.font_family;
    let mut key = 0;
    for font in sources.iter().filter(|font| needed(font, family, text)) {
        let index = match requests
            .iter()
            .position(|request| request.font.source == font.source)
        {
            Some(index) => index,
            None => {
                requests.push(FontRequest {
                    font: font.clone(),
                    faces: Resource::load(font.source.clone(), parse_font),
                    installed: None,
                });
                requests.len() - 1
            }
        };
        let installed = match requests[index].installed {
            Some(installed) => installed,
            None => match requests[index].faces.get() {
                LoadState::Loading => {
                    pending = true;
                    continue;
                }
                LoadState::Ready(faces) => {
                    let replace = !requests.iter().any(|request| {
                        request.installed.is_some_and(|installed| installed > 0)
                            && request.font.family == font.family
                    });
                    let mut fonts = write_font_system();
                    let db = fonts.db_mut();
                    if replace {
                        let existing = db
                            .faces()
                            .filter(|face| {
                                face.families.iter().any(|(name, _)| *name == font.family)
                            })
                            .map(|face| face.id)
                            .collect::<Vec<_>>();
                        for id in existing {
                            db.remove_face(id);
                        }
                    }
                    for face in faces.iter() {
                        db.push_face_info(face.clone());
                    }
                    *revision += 1;
                    *revision
                }
                LoadState::Failed(_) => 0,
            },
        };
        requests[index].installed = Some(installed);
        key = key.max(installed);
    }
    (!pending).then_some(key)
}

fn needed(font: &FontSource, family: Option<&str>, text: &str) -> bool {
    let contains = |ranges: &[(u32, u32)], c: char| {
        ranges
            .iter()
            .any(|&(start, end)| (start..=end).contains(&(c as u32)))
    };
    let named = family == Some(&*font.family);
    (named && font.coverage.is_empty())
        || text.chars().any(|c| {
            (font.coverage.is_empty() || contains(&font.coverage, c))
                && (named || contains(&font.fallback, c))
        })
}

fn parse_font(bytes: Arc<[u8]>) -> io::Result<Arc<[FaceInfo]>> {
    let bytes: Arc<[u8]> = if bytes.starts_with(b"wOF2") {
        wuff::decompress_woff2(&bytes)
            .map_err(|error| io::Error::other(format!("{error:?}")))?
            .into()
    } else {
        bytes
    };
    let mut db = Database::new();
    db.load_font_source(fontdb::Source::Binary(Arc::new(bytes)));
    let faces = db.faces().cloned().collect::<Arc<[_]>>();
    if faces.is_empty() {
        return Err(io::Error::other("font contains no supported faces"));
    }
    Ok(faces)
}

fn parse_stylesheet(bytes: Arc<[u8]>) -> io::Result<Arc<[FontFace]>> {
    let css = std::str::from_utf8(&bytes).map_err(io::Error::other)?;
    Ok(css
        .split("@font-face")
        .skip(1)
        .filter_map(|rule| {
            let rule = &rule[rule.find('{')? + 1..rule.find('}')?];
            let property = |name: &str| {
                rule.split(';').find_map(|declaration| {
                    let (key, value) = declaration.split_once(':')?;
                    (key.trim() == name).then(|| value.trim())
                })
            };
            let src = property("src")?;
            let src = &src[src.find("url(")? + 4..];
            let src = src[..src.find(')')?].trim_matches(['"', '\'']);
            let ranges = property("unicode-range").map_or_else(
                || vec![(0, 0x10ffff)],
                |ranges| {
                    ranges
                        .split(',')
                        .filter_map(|range| {
                            let range = range.trim().strip_prefix("U+")?;
                            let (start, end) = range.split_once('-').unwrap_or((range, range));
                            let parse = |value: &str, fill| {
                                u32::from_str_radix(&value.replace('?', fill), 16).ok()
                            };
                            Some((parse(start, "0")?, parse(end, "F")?))
                        })
                        .collect()
                },
            );
            Some(FontFace {
                family: property("font-family")?
                    .trim_matches(['"', '\''])
                    .to_owned(),
                src: src.to_owned(),
                ranges,
            })
        })
        .collect())
}

impl FontFace {
    fn font(&self, stylesheet: &str) -> FontSource {
        let url = if self.src.contains("://") {
            self.src.clone()
        } else {
            let base = &stylesheet[..stylesheet.rfind('/').map_or(0, |end| end + 1)];
            format!("{base}{}", self.src.trim_start_matches("./"))
        };
        FontSource {
            source: url.into(),
            family: self.family.clone().into(),
            fallback: self.ranges.clone().into(),
            coverage: self.ranges.clone().into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stylesheet_rules_resolve_files_and_unicode_ranges() {
        let css = br#"
            /* latin */
            @font-face {
              font-family: 'Example Sans';
              src: url(./files/latin.woff2) format('woff2');
              unicode-range: U+0000-00FF, U+4??, U+2014;
            }
            @font-face { font-family: "Other"; src: url("https://cdn.test/other.ttf"); }
        "#;
        let fonts = parse_stylesheet(Arc::from(&css[..]))
            .unwrap()
            .iter()
            .map(|face| face.font("https://cdn.test/sans/index.css"))
            .collect::<Vec<_>>();
        assert!(fonts[0].source == Source::from("https://cdn.test/sans/files/latin.woff2"));
        assert_eq!(fonts[0].family, "Example Sans");
        assert_eq!(
            *fonts[0].coverage,
            [(0, 0xff), (0x400, 0x4ff), (0x2014, 0x2014)]
        );
        assert!(fonts[1].source == Source::from("https://cdn.test/other.ttf"));
        assert_eq!(*fonts[1].coverage, [(0, 0x10ffff)]);
    }
}
