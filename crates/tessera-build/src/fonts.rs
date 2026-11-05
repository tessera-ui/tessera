use std::{collections::BTreeMap, fs, io::Write, path::Path};

use anyhow::{Context, Result, ensure};
use read_fonts::{FontRef, collections::IntSet};
use sha2::{Digest, Sha256};
use skera::{Plan, SubsetFlags, subset_font};

use crate::{FontSubset, PreparedAsset};

pub(crate) fn prepare(
    source: &Path,
    relative: &str,
    ranges: &[(u32, u32)],
    embedded: bool,
    cache: &Path,
) -> Result<Vec<PreparedAsset>> {
    let data = fs::read(source)?;
    let face = ttf_parser::Face::parse(&data, 0).context("Invalid subset font")?;
    ensure!(
        face.tables().glyf.is_some(),
        "Font subsetting requires TrueType outlines: {relative}"
    );
    ensure!(
        ranges
            .iter()
            .all(|&(start, end)| start <= end && end <= 0x10ffff),
        "Invalid Unicode range for {relative}"
    );
    let family = [
        ttf_parser::name_id::TYPOGRAPHIC_FAMILY,
        ttf_parser::name_id::FAMILY,
    ]
    .into_iter()
    .find_map(|id| {
        face.names()
            .into_iter()
            .filter(|name| name.name_id == id)
            .find_map(|name| name.to_string())
    })
    .context("Subset font has no Unicode family name")?;
    let mut blocks: BTreeMap<u32, IntSet<u32>> = BTreeMap::new();
    for subtable in face.tables().cmap.context("Font has no cmap")?.subtables {
        if subtable.is_unicode() {
            subtable.codepoints(|codepoint| {
                if (ranges.is_empty()
                    || ranges
                        .iter()
                        .any(|&(start, end)| (start..=end).contains(&codepoint)))
                    && face
                        .glyph_index(char::from_u32(codepoint).unwrap_or('\0'))
                        .is_some()
                {
                    blocks.entry(codepoint / 256).or_default().insert(codepoint);
                }
            });
        }
    }
    ensure!(
        !blocks.is_empty(),
        "Font subset ranges contain no glyphs: {relative}"
    );
    let mut hash = Sha256::new();
    hash.update(b"skera-0.9.0-block256-v3");
    hash.update(&data);
    for &(start, end) in ranges {
        hash.update(start.to_le_bytes());
        hash.update(end.to_le_bytes());
    }
    let cache = cache.join(format!("{:x}", hash.finalize()));
    fs::create_dir_all(&cache)?;
    let font = FontRef::new(&data)?;
    let mut assets = Vec::new();
    for (block, unicodes) in blocks {
        let name = format!("{block:x}.ttf");
        let source = cache.join(&name);
        if !source.is_file() {
            let mut requested = unicodes.clone();
            requested.extend(0xfe00..=0xfe0f);
            requested.extend(0xe0100..=0xe01ef);
            let plan = Plan::new(
                &IntSet::empty(),
                &requested,
                &font,
                SubsetFlags::default(),
                &skera::DEFAULT_DROP_TABLES.iter().copied().collect(),
                &IntSet::all(),
                &IntSet::all(),
                &IntSet::all(),
                &IntSet::all(),
            );
            let mut temporary = tempfile::NamedTempFile::new_in(&cache)?;
            temporary.write_all(&subset_font(&font, &plan)?)?;
            if let Err(error) = temporary.persist_noclobber(&source) {
                ensure!(
                    error.error.kind() == std::io::ErrorKind::AlreadyExists,
                    "Failed to cache font subset: {error}"
                );
            }
        }
        assets.push(PreparedAsset {
            embedded,
            source,
            relative: format!("{relative}.subsets/{name}"),
            font: Some(FontSubset {
                family: family.clone(),
                ranges: unicodes
                    .iter()
                    .map(|codepoint| (codepoint, codepoint))
                    .fold(Vec::<(u32, u32)>::new(), |mut ranges, (start, end)| {
                        if let Some(last) = ranges.last_mut().filter(|last| last.1 + 1 == start) {
                            last.1 = end;
                        } else {
                            ranges.push((start, end));
                        }
                        ranges
                    }),
            }),
        });
    }
    Ok(assets)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subsets_keep_selected_coverage_and_cache_by_font_content() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("font.ttf");
        fs::write(&source, font_test_data::AHEM).unwrap();
        let cache = root.path().join("cache");
        let make =
            |ranges: &[(u32, u32)]| prepare(&source, "font.ttf", ranges, false, &cache).unwrap();
        let assets = make(&[(0x41, 0x42), (0x3a9, 0x3a9)]);
        assert_eq!(assets.len(), 2);
        for (asset, expected, absent) in [(&assets[0], 'A', 'Ω'), (&assets[1], 'Ω', 'A')] {
            let bytes = fs::read(&asset.source).unwrap();
            let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
            assert!(
                face.glyph_bounding_box(face.glyph_index(expected).unwrap())
                    .is_some()
            );
            assert!(face.glyph_index(absent).is_none());
        }
        let modified = fs::metadata(&assets[0].source).unwrap().modified().unwrap();
        let cached = make(&[(0x41, 0x42), (0x3a9, 0x3a9)]);
        assert_eq!(assets[0].source, cached[0].source);
        assert_eq!(
            modified,
            fs::metadata(&cached[0].source).unwrap().modified().unwrap()
        );
        let changed = make(&[(0x41, 0x41)]);
        let bytes = fs::read(&changed[0].source).unwrap();
        let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
        assert!(face.glyph_index('A').is_some());
        assert!(face.glyph_index('B').is_none());
        fs::write(&source, font_test_data::TINOS_SUBSET).unwrap();
        let changed = make(&[(0x41, 0x41)]);
        assert_ne!(face.glyph_hor_advance(face.glyph_index('A').unwrap()), {
            let bytes = fs::read(&changed[0].source).unwrap();
            let face = ttf_parser::Face::parse(&bytes, 0).unwrap();
            face.glyph_hor_advance(face.glyph_index('A').unwrap())
        });
        fs::write(&source, font_test_data::NOTO_SANS_JP_CFF).unwrap();
        assert!(prepare(&source, "font.ttf", &[], false, &cache).is_err());
    }
}
