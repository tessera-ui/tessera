//! Geometry for uncontained collections with different item aspect ratios.
//!
//! ## Usage
//!
//! Browse mixed portrait, landscape, and square media without resizing items
//! while scrolling.

#[derive(Clone, Debug, PartialEq)]
pub(super) struct VariableItems {
    widths: Vec<f32>,
    prefix: Vec<f64>,
    fallback_width: f32,
    minimum_width: f32,
    spacing: f32,
    padding: [f32; 2],
}

pub(super) fn normalized_ratio(ratio: f32) -> f32 {
    if ratio.is_finite() && ratio > 0.0 {
        ratio.clamp(9.0 / 16.0, 16.0 / 9.0)
    } else {
        1.0
    }
}

impl VariableItems {
    pub fn new(height: f32, spacing: f32, padding: [f32; 2], count: usize, ratios: &[f32]) -> Self {
        let height = height.max(1.0);
        let widths: Vec<_> = ratios
            .iter()
            .take(count)
            .map(|ratio| height * normalized_ratio(*ratio))
            .collect();
        let minimum_width = widths.iter().copied().fold(height, f32::min);
        let mut prefix = Vec::with_capacity(widths.len() + 1);
        prefix.push(0.0);
        for width in &widths {
            prefix.push(
                prefix.last().copied().unwrap_or(0.0) + f64::from(*width) + f64::from(spacing),
            );
        }
        Self {
            widths,
            prefix,
            fallback_width: height,
            minimum_width,
            spacing,
            padding,
        }
    }

    pub fn width(&self, index: usize) -> f32 {
        self.widths
            .get(index)
            .copied()
            .unwrap_or(self.fallback_width)
    }

    fn start(&self, index: usize) -> f64 {
        let cached = index.min(self.widths.len());
        self.prefix[cached]
            + index.saturating_sub(cached) as f64
                * (f64::from(self.fallback_width) + f64::from(self.spacing))
    }

    pub fn max_scroll(&self, viewport: f32, count: usize) -> f32 {
        if count == 0 {
            return 0.0;
        }
        (self.start(count - 1)
            + f64::from(self.width(count - 1))
            + f64::from(self.padding[0])
            + f64::from(self.padding[1])
            - f64::from(viewport))
        .max(0.0) as f32
    }

    pub fn snap(&self, index: usize, viewport: f32, count: usize) -> f32 {
        if count == 0 {
            return 0.0;
        }
        (self.start(index.min(count - 1)) as f32).clamp(0.0, self.max_scroll(viewport, count))
    }

    fn index_at(&self, offset: f64, count: usize) -> usize {
        let mut low = 0;
        let mut high = count;
        while low < high {
            let middle = low + (high - low) / 2;
            if self.start(middle) <= offset {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        low.saturating_sub(1).min(count.saturating_sub(1))
    }

    pub fn nearest(&self, offset: f32, viewport: f32, count: usize) -> usize {
        let approximate = self.index_at(f64::from(offset), count);
        (approximate.saturating_sub(1)..approximate.saturating_add(3).min(count))
            .min_by(|left, right| {
                (self.snap(*left, viewport, count) - offset)
                    .abs()
                    .total_cmp(&(self.snap(*right, viewport, count) - offset).abs())
            })
            .unwrap_or(0)
    }

    pub fn masks(&self, offset: f32, viewport: f32, count: usize) -> Vec<(usize, f32, f32)> {
        if count == 0 {
            return Vec::new();
        }
        let first = self.index_at(f64::from(offset - self.padding[0]), count);
        let limit = (viewport / self.minimum_width).ceil() as usize + 3;
        (first..count)
            .take(limit)
            .map(|index| {
                let width = self.width(index);
                let left = f64::from(self.padding[0]) + self.start(index) - f64::from(offset);
                (index, left, width)
            })
            .take_while(|(_, left, _)| *left < f64::from(viewport))
            .filter_map(|(index, left, width)| {
                (left + f64::from(width) > 0.0).then_some((
                    index,
                    (left + f64::from(width) * 0.5) as f32,
                    width,
                ))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f32, expected: f32) {
        assert!((actual - expected).abs() < 0.001, "{actual} != {expected}");
    }

    #[test]
    fn mixed_ratios_keep_their_dimensions_and_align_both_collection_edges() {
        let items = VariableItems::new(180.0, 8.0, [16.0, 0.0], 3, &[16.0 / 9.0, 9.0 / 16.0, 1.0]);
        close(items.width(0), 320.0);
        close(items.width(1), 101.25);
        close(items.width(2), 180.0);
        close(items.max_scroll(390.0, 3), 243.25);
        let first = items.masks(0.0, 390.0, 3);
        assert_eq!(first.len(), 2);
        close(first[0].1 - first[0].2 * 0.5, 16.0);
        let end = items.masks(items.max_scroll(390.0, 3), 390.0, 3);
        close(end.last().unwrap().1 + end.last().unwrap().2 * 0.5, 390.0);
        for offset in [0.0, 100.0, 200.0, 243.25] {
            for (index, _, width) in items.masks(offset, 390.0, 3) {
                close(width, items.width(index));
            }
        }
    }

    #[test]
    fn missing_ratios_are_square_and_invalid_ratios_are_bounded() {
        let items = VariableItems::new(
            180.0,
            8.0,
            [0.0, 0.0],
            8,
            &[f32::NAN, 0.0, -1.0, f32::INFINITY, 0.1, 10.0],
        );
        for index in [0, 1, 2, 3, 6, 7] {
            close(items.width(index), 180.0);
        }
        close(items.width(4), 101.25);
        close(items.width(5), 320.0);
        assert_eq!(items.max_scroll(390.0, 0), 0.0);
        assert!(items.masks(0.0, 390.0, 0).is_empty());
    }

    #[test]
    fn large_collections_cache_only_explicit_ratios_and_build_visible_items() {
        let count = 1_000_000;
        let items = VariableItems::new(180.0, 8.0, [16.0, 0.0], count, &[16.0 / 9.0, 9.0 / 16.0]);
        assert_eq!(items.prefix.len(), 3);
        let offset = items.snap(900_000, 390.0, count);
        let masks = items.masks(offset, 390.0, count);
        assert!(masks.len() <= 5);
        assert!(
            masks.iter().all(|(index, center, width)| *index >= 899_999
                && center.is_finite()
                && *width > 0.0)
        );
        assert!((items.nearest(offset, 390.0, count) as isize - 900_000).abs() <= 1);
    }

    #[test]
    fn height_spacing_padding_and_scroll_offsets_scale_together() {
        let ratios = [16.0 / 9.0, 9.0 / 16.0, 1.0, 3.0 / 4.0];
        let base = VariableItems::new(180.0, 8.0, [16.0, 4.0], 4, &ratios);
        for density in [1.25, 1.5, 2.0] {
            let scaled = VariableItems::new(
                180.0 * density,
                8.0 * density,
                [16.0 * density, 4.0 * density],
                4,
                &ratios,
            );
            close(
                scaled.max_scroll(390.0 * density, 4),
                base.max_scroll(390.0, 4) * density,
            );
            for (actual, expected) in scaled
                .masks(100.0 * density, 390.0 * density, 4)
                .iter()
                .zip(base.masks(100.0, 390.0, 4))
            {
                assert_eq!(actual.0, expected.0);
                close(actual.1, expected.1 * density);
                close(actual.2, expected.2 * density);
            }
        }
    }
}
