//! Fonts loaded when their family or fallback coverage is used.
//!
//! Register additional fonts with [`Fonts`] at the application entry.

use std::{cell::RefCell, sync::Arc};

use tessera_ui::{Asset, EntryRegistry, LoadState, Resource, TesseraPackage};

use crate::{res, text::write_font_system, theme::TextStyle};

/// An application font and the Unicode ranges for which it supplies fallback.
#[derive(Clone, Copy)]
pub struct Font {
    /// The generated font asset.
    pub asset: Asset,
    /// The family name used by text styles.
    pub family: &'static str,
    /// Inclusive fallback ranges, independent of the selected family.
    pub fallback: &'static [(u32, u32)],
}

/// An entry package supplying fonts on demand.
pub struct Fonts(
    /// Font sources available to this application.
    pub &'static [Font],
);

impl TesseraPackage for Fonts {
    fn register(self, _: &mut EntryRegistry) {
        let registry = tessera_ui::__private::local_runtime_resource::<RefCell<FontRegistry>>();
        let mut registry = registry.borrow_mut();
        for font in self.0 {
            if !registry
                .sources
                .iter()
                .any(|source| source.asset == font.asset)
            {
                registry.sources.push(*font);
            }
        }
    }
}

struct FontRequest {
    font: Font,
    resource: Resource<Arc<[u8]>>,
    installed: Option<bool>,
}

struct FontRegistry {
    sources: Vec<Font>,
    requests: Vec<FontRequest>,
    revision: u64,
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self {
            sources: vec![
                Font {
                    asset: res::ROBOTO_REGULAR_TTF,
                    family: "Roboto",
                    fallback: &[],
                },
                Font {
                    asset: res::ROBOTO_MEDIUM_TTF,
                    family: "Roboto",
                    fallback: &[],
                },
                Font {
                    asset: res::ROBOTO_BOLD_TTF,
                    family: "Roboto",
                    fallback: &[],
                },
                Font {
                    asset: res::NOTOSANSSC_REGULAR_OTF,
                    family: "Noto Sans SC",
                    fallback: &[
                        (0x2e80, 0x9fff),
                        (0xac00, 0xd7af),
                        (0xf900, 0xfaff),
                        (0xff00, 0xffef),
                        (0x20000, 0x323af),
                    ],
                },
                Font {
                    asset: res::NOTOCOLOREMOJI_COLRV0_TTF,
                    family: "Noto Color Emoji",
                    fallback: &[
                        (0x20e3, 0x20e3),
                        (0x2300, 0x27ff),
                        (0x2b00, 0x2bff),
                        (0xfe0f, 0xfe0f),
                        (0x1f000, 0x1faff),
                    ],
                },
            ],
            requests: Vec::new(),
            revision: 0,
        }
    }
}

pub(crate) fn prepare(text: &str, style: TextStyle) -> u64 {
    let registry = tessera_ui::__private::local_runtime_resource::<RefCell<FontRegistry>>();
    {
        let mut registry = registry.borrow_mut();
        let family = style.font_family.unwrap_or("Roboto");
        let FontRegistry {
            sources, requests, ..
        } = &mut *registry;
        for font in sources.iter() {
            if !requests
                .iter()
                .any(|request| request.font.asset == font.asset)
                && (font.family == family
                    || !font.fallback.is_empty()
                        && text.chars().any(|c| {
                            font.fallback
                                .iter()
                                .any(|&(start, end)| (start..=end).contains(&(c as u32)))
                        }))
            {
                requests.push(FontRequest {
                    font: *font,
                    resource: font.asset.load(),
                    installed: None,
                });
            }
        }
    }
    revision()
}

pub(crate) fn revision() -> u64 {
    let registry = tessera_ui::__private::local_runtime_resource::<RefCell<FontRegistry>>();
    let mut registry = registry.borrow_mut();
    for i in 0..registry.requests.len() {
        if registry.requests[i].installed.is_some() {
            continue;
        }
        let font = registry.requests[i].font;
        let installed = match registry.requests[i].resource.get() {
            LoadState::Loading => continue,
            LoadState::Ready(bytes) => {
                let replace = !registry.requests.iter().any(|request| {
                    request.installed == Some(true) && request.font.family == font.family
                });
                let mut fonts = write_font_system();
                let db = fonts.db_mut();
                let existing = db
                    .faces()
                    .filter(|face| {
                        replace && face.families.iter().any(|(name, _)| name == font.family)
                    })
                    .map(|face| face.id)
                    .collect::<Vec<_>>();
                let added = db.load_font_source(glyphon::fontdb::Source::Binary(Arc::new(bytes)));
                if added.is_empty() {
                    tracing::warn!(
                        asset = font.asset.path(),
                        "font contains no supported faces"
                    );
                    false
                } else {
                    for id in existing {
                        db.remove_face(id);
                    }
                    registry.revision += 1;
                    true
                }
            }
            LoadState::Failed(_) => false,
        };
        registry.requests[i].installed = Some(installed);
    }
    registry.revision
}
