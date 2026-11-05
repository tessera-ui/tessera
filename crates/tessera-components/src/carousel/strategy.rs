//! Keyline arrangements for adaptive carousel viewports.
//!
//! ## Usage
//!
//! Resolve item masks, edge alignment, and snap positions during scrolling.

// Copyright 2023-2024 The Android Open Source Project.
// Modified for Tessera with a Rust implementation and viewport validation.
// Licensed under Apache-2.0; see assets/material_carousel.LICENSE and .NOTICE.

use super::{CarouselVariant, variable::VariableItems};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Keyline {
    pub size: f32,
    pub offset: f32,
    pub unadjusted: f32,
    focal: bool,
    anchor: bool,
    pivot: bool,
    cutoff: f32,
}

impl Keyline {
    fn lerp(self, other: Self, progress: f32) -> Self {
        Self {
            size: lerp(self.size, other.size, progress),
            offset: lerp(self.offset, other.offset, progress),
            unadjusted: lerp(self.unadjusted, other.unadjusted, progress),
            focal: if progress < 0.5 {
                self.focal
            } else {
                other.focal
            },
            anchor: if progress < 0.5 {
                self.anchor
            } else {
                other.anchor
            },
            pivot: if progress < 0.5 {
                self.pivot
            } else {
                other.pivot
            },
            cutoff: lerp(self.cutoff, other.cutoff, progress),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Keylines(Vec<Keyline>);

impl Keylines {
    fn first_focal(&self) -> usize {
        self.0
            .iter()
            .position(|key| key.focal)
            .expect("focal keyline")
    }
    fn last_focal(&self) -> usize {
        self.0
            .iter()
            .rposition(|key| key.focal)
            .expect("focal keyline")
    }
    fn first_non_anchor(&self) -> usize {
        self.0
            .iter()
            .position(|key| !key.anchor)
            .expect("content keyline")
    }
    fn last_non_anchor(&self) -> usize {
        self.0
            .iter()
            .rposition(|key| !key.anchor)
            .expect("content keyline")
    }
    fn pivot(&self) -> usize {
        self.0
            .iter()
            .position(|key| key.pivot)
            .expect("pivot keyline")
    }

    fn build(items: &[(f32, bool)], viewport: f32, spacing: f32, center: bool) -> Self {
        let large = items
            .iter()
            .filter(|item| !item.1)
            .map(|item| item.0)
            .fold(0.0f32, f32::max);
        let first = items
            .iter()
            .position(|item| !item.1 && item.0 == large)
            .expect("large item");
        let count = items[first..]
            .iter()
            .take_while(|item| item.0 == large)
            .count();
        let offset = if center {
            viewport * 0.5 - (count - 1) as f32 * (large + spacing) * 0.5
        } else {
            large * 0.5
        };
        Self::with_pivot(items, viewport, spacing, first, offset)
    }

    fn with_pivot(
        items: &[(f32, bool)],
        viewport: f32,
        spacing: f32,
        pivot: usize,
        offset: f32,
    ) -> Self {
        let large = items
            .iter()
            .filter(|item| !item.1)
            .map(|item| item.0)
            .fold(0.0f32, f32::max);
        let first = items
            .iter()
            .position(|item| !item.1 && item.0 == large)
            .expect("large item");
        let last = first
            + items[first..]
                .iter()
                .take_while(|item| item.0 == large)
                .count()
            - 1;
        let key = |index: usize, offset: f32, unadjusted: f32| {
            let size = items[index].0;
            let left_cut = offset - size * 0.5 < 0.0 && offset + size * 0.5 > 0.0;
            let right_cut = offset - size * 0.5 < viewport && offset + size * 0.5 > viewport;
            Keyline {
                size,
                offset,
                unadjusted,
                focal: index >= first && index <= last,
                anchor: items[index].1,
                pivot: index == pivot,
                cutoff: if left_cut {
                    if index == pivot {
                        offset - size * 0.5
                    } else {
                        (offset - size * 0.5).abs()
                    }
                } else if right_cut {
                    offset + size * 0.5 - viewport
                } else {
                    0.0
                },
            }
        };
        let mut result = vec![key(pivot, offset, offset); items.len()];
        let mut edge = offset - large * 0.5 - spacing;
        let mut unadjusted = edge;
        for index in (0..pivot).rev() {
            result[index] = key(index, edge - items[index].0 * 0.5, unadjusted - large * 0.5);
            edge -= items[index].0 + spacing;
            unadjusted -= large + spacing;
        }
        edge = offset + large * 0.5 + spacing;
        unadjusted = edge;
        for index in pivot + 1..items.len() {
            result[index] = key(index, edge + items[index].0 * 0.5, unadjusted + large * 0.5);
            edge += items[index].0 + spacing;
            unadjusted += large + spacing;
        }
        Self(result)
    }

    fn shifted(&self, source: usize, destination: usize, viewport: f32, spacing: f32) -> Self {
        let direction = if source > destination { 1 } else { -1 };
        let old_pivot = self.pivot();
        let offset = self.0[old_pivot].offset
            + (self.0[source].size - self.0[source].cutoff + spacing) * direction as f32;
        let pivot = (old_pivot as isize + direction) as usize;
        let mut items: Vec<_> = self.0.iter().map(|key| (key.size, key.anchor)).collect();
        let moved = items.remove(source);
        items.insert(destination, moved);
        Self::with_pivot(&items, viewport, spacing, pivot, offset)
    }

    fn padded(&self, padding: f32, pivot: usize, viewport: f32, spacing: f32) -> Self {
        let count = self.0.iter().filter(|key| !key.anchor).count();
        let reduction = padding / count as f32;
        let items: Vec<_> = self
            .0
            .iter()
            .map(|key| (key.size - reduction.abs(), key.anchor))
            .collect();
        let mut result = Self::with_pivot(
            &items,
            viewport,
            spacing,
            pivot,
            self.0[pivot].offset - reduction * 0.5 + padding,
        );
        for (new, old) in result.0.iter_mut().zip(&self.0) {
            new.unadjusted = old.unadjusted;
        }
        result
    }

    fn steps(&self, viewport: f32, spacing: f32, padding: f32, start: bool) -> Vec<Self> {
        let mut steps = vec![self.clone()];
        let focal = if start {
            self.first_focal()
        } else {
            self.last_focal()
        };
        let at_edge = if start {
            focal == self.first_non_anchor()
                && self.0[focal].offset - self.0[focal].size * 0.5 >= 0.0
        } else {
            focal == self.last_non_anchor()
                && self.0[focal].offset + self.0[focal].size * 0.5 <= viewport
        };
        if at_edge {
            if padding != 0.0 {
                steps.push(self.padded(
                    if start { padding } else { -padding },
                    focal,
                    viewport,
                    spacing,
                ));
            }
            return steps;
        }
        let count = if start {
            focal - self.first_non_anchor()
        } else {
            self.last_non_anchor() - focal
        };
        if count == 0 && self.0[focal].cutoff > 0.0 {
            steps.push(self.shifted(0, 0, viewport, spacing));
            return steps;
        }
        for step in 0..count {
            let previous = steps.last().expect("initial step");
            let destination = if start {
                let original = self.first_non_anchor() + step;
                if original > 0 {
                    let size = self.0[original - 1].size;
                    (previous.last_focal()..previous.0.len())
                        .find(|index| previous.0[*index].size == size)
                        .unwrap_or(previous.0.len() - 1)
                        .saturating_sub(1)
                } else {
                    self.0.len() - 1
                }
            } else {
                let original = self.last_non_anchor() - step;
                if original < self.0.len() - 1 {
                    let size = self.0[original + 1].size;
                    (0..previous.first_focal())
                        .rev()
                        .find(|index| previous.0[*index].size == size)
                        .unwrap_or(0)
                        + 1
                } else {
                    0
                }
            };
            let source = if start {
                self.first_non_anchor()
            } else {
                self.last_non_anchor()
            };
            steps.push(previous.shifted(source, destination, viewport, spacing));
        }
        if padding != 0.0 {
            let last = steps.last_mut().expect("initial step");
            let pivot = if start {
                last.first_focal()
            } else {
                last.last_focal()
            };
            *last = last.padded(
                if start { padding } else { -padding },
                pivot,
                viewport,
                spacing,
            );
        }
        steps
    }
}

#[derive(Clone, Copy)]
struct Arrangement {
    small: f32,
    medium: f32,
    large: f32,
    small_count: usize,
    medium_count: usize,
    large_count: usize,
}

impl Arrangement {
    fn valid(self) -> bool {
        self.large > 0.0
            && (self.small_count == 0 || self.large > self.small)
            && (self.medium_count == 0
                || self.small_count == 0
                || (self.large > self.medium && self.medium > self.small))
    }

    fn fit(
        viewport: f32,
        spacing: f32,
        target: f32,
        small: f32,
        min_small: f32,
        max_small: f32,
        medium: f32,
        counts: [usize; 3],
    ) -> Self {
        let [large_count, medium_count, small_count] = counts;
        let available = viewport
            - (large_count + medium_count + small_count).saturating_sub(1) as f32 * spacing;
        let mut small = small.clamp(min_small, max_small);
        let delta = available
            - target * large_count as f32
            - medium * medium_count as f32
            - small * small_count as f32;
        if small_count > 0 {
            small += (delta / small_count as f32).clamp(min_small - small, max_small - small);
        } else {
            small = 0.0;
        }
        let mut large = (available - (small_count as f32 + medium_count as f32 * 0.5) * small)
            / (large_count as f32 + medium_count as f32 * 0.5);
        let mut medium = (large + small) * 0.5;
        if medium_count > 0 && large != target {
            let target_adjustment = (target - large) * large_count as f32;
            let distribute = target_adjustment
                .abs()
                .min(medium * 0.1 * medium_count as f32)
                * target_adjustment.signum();
            medium -= distribute / medium_count as f32;
            large += distribute / large_count as f32;
        }
        Self {
            small,
            medium,
            large,
            small_count,
            medium_count,
            large_count,
        }
    }

    fn best(
        viewport: f32,
        spacing: f32,
        target: f32,
        small: f32,
        small_range: [f32; 2],
        medium: f32,
        large_range: [usize; 2],
        medium_counts: &[usize],
        small_count: usize,
    ) -> Self {
        let mut best = None;
        let mut cost = f32::INFINITY;
        let mut priority = 1;
        for large in (large_range[0]..=large_range[1]).rev() {
            for &medium_count in medium_counts {
                let candidate = Self::fit(
                    viewport,
                    spacing,
                    target,
                    small,
                    small_range[0],
                    small_range[1],
                    medium,
                    [large, medium_count, small_count],
                );
                let candidate_cost = if candidate.valid() {
                    (target - candidate.large).abs() * priority as f32
                } else {
                    f32::INFINITY
                };
                if best.is_none() || candidate_cost < cost {
                    best = Some(candidate);
                    cost = candidate_cost;
                }
                if cost == 0.0 {
                    return candidate;
                }
                priority += 1;
            }
        }
        let result = best.expect("at least one candidate");
        if result.valid() {
            result
        } else {
            Self {
                small: 0.0,
                medium: 0.0,
                large: viewport,
                small_count: 0,
                medium_count: 0,
                large_count: 1,
            }
        }
    }

    fn keylines(self, viewport: f32, spacing: f32, anchors: [f32; 2], center: bool) -> Keylines {
        let mut items = vec![(anchors[0], true)];
        if center {
            items.extend(std::iter::repeat_n(
                (self.small, false),
                self.small_count / 2,
            ));
            items.extend(std::iter::repeat_n(
                (self.medium, false),
                self.medium_count / 2,
            ));
        }
        items.extend(std::iter::repeat_n((self.large, false), self.large_count));
        items.extend(std::iter::repeat_n(
            (self.medium, false),
            if center {
                self.medium_count / 2
            } else {
                self.medium_count
            },
        ));
        items.extend(std::iter::repeat_n(
            (self.small, false),
            if center {
                self.small_count / 2
            } else {
                self.small_count
            },
        ));
        items.push((anchors[1], true));
        Keylines::build(&items, viewport, spacing, center)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct StrategyConfig {
    pub variant: CarouselVariant,
    pub viewport: f32,
    pub preferred: Option<f32>,
    pub spacing: f32,
    pub density: f32,
    pub count: usize,
    pub small_range: [f32; 2],
    pub padding: [f32; 2],
    pub item_height: f32,
    pub aspect_revision: u64,
}

#[derive(Clone, Debug, PartialEq)]
struct KeylinePlan {
    default: Keylines,
    start: Vec<Keylines>,
    end: Vec<Keylines>,
    start_points: Vec<f32>,
    end_points: Vec<f32>,
    start_shift: f32,
    end_shift: f32,
}

#[derive(Clone, Debug, PartialEq)]
enum StrategyKind {
    Keylines(KeylinePlan),
    Variable(VariableItems),
    FullScreen,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct Strategy {
    kind: StrategyKind,
    pub item_size: f32,
    pub viewport: f32,
    pub spacing: f32,
}

impl Strategy {
    pub fn new(config: StrategyConfig, ratios: &[f32]) -> Self {
        if config.variant == CarouselVariant::UncontainedMultiAspectRatio {
            return Self {
                kind: StrategyKind::Variable(VariableItems::new(
                    config.item_height,
                    config.spacing,
                    config.padding,
                    config.count,
                    ratios,
                )),
                item_size: config.item_height.max(1.0),
                viewport: config.viewport,
                spacing: config.spacing,
            };
        }
        if config.variant == CarouselVariant::FullScreen {
            return Self {
                kind: StrategyKind::FullScreen,
                item_size: config.viewport,
                viewport: config.viewport,
                spacing: config.spacing,
            };
        }
        let StrategyConfig {
            variant,
            viewport,
            preferred,
            spacing,
            density,
            count,
            small_range,
            padding,
            ..
        } = config;
        let target = preferred.unwrap_or(viewport).max(1.0).min(viewport);
        let anchor = 10.0 * density;
        let default = if variant == CarouselVariant::Uncontained {
            let large = (target + spacing).min(viewport);
            let large_count = (viewport / large).floor().max(1.0) as usize;
            let remaining = viewport - large_count as f32 * large;
            let mut medium = (remaining * 1.5).max(anchor);
            if medium > large * 0.85 {
                medium = (large * 0.85).max(remaining * 1.2).min(large);
            }
            Arrangement {
                large,
                medium,
                small: 0.0,
                large_count,
                medium_count: usize::from(remaining > 0.0),
                small_count: 0,
            }
            .keylines(
                viewport,
                spacing,
                [anchor.min(target).max(medium * 0.5), anchor],
                false,
            )
        } else {
            let small = (target / 3.0).clamp(small_range[0], small_range[1]);
            let multi = variant == CarouselVariant::MultiBrowse;
            let centered = variant == CarouselVariant::CenteredHero && count >= 3;
            let mut small_count = if count <= 1 && !multi {
                0
            } else if centered {
                2
            } else {
                1
            };
            if viewport
                < if multi {
                    small_range[0] * 2.0
                } else {
                    small_range[0] * (small_count as f32 + 1.25)
                }
            {
                small_count = 0;
            }
            let medium = if multi { (target + small) * 0.5 } else { 0.0 };
            let minimum_space = viewport
                - if multi {
                    medium + small_range[1] * small_count as f32
                } else {
                    small_range[0] * small_count as f32
                };
            let large_range = [
                (minimum_space / target).floor().max(1.0) as usize,
                (viewport / target).ceil().max(1.0) as usize,
            ];
            let medium_counts: &[usize] = if multi { &[1, 0] } else { &[0] };
            let mut arrangement = Arrangement::best(
                viewport,
                spacing,
                target,
                small,
                small_range,
                medium,
                large_range,
                medium_counts,
                small_count,
            );
            if multi
                && arrangement.large_count + arrangement.medium_count + arrangement.small_count
                    > count
                && arrangement.small_count > 0
            {
                arrangement = Arrangement::best(
                    viewport,
                    spacing,
                    target,
                    small,
                    small_range,
                    medium,
                    large_range,
                    &[arrangement.medium_count],
                    0,
                );
            }
            let center = centered
                && count
                    >= arrangement.large_count + arrangement.medium_count + arrangement.small_count;
            arrangement.keylines(viewport, spacing, [anchor, anchor], center)
        };
        let item_size = default.0[default.first_focal()].size;
        let start = default.steps(viewport, spacing, padding[0], true);
        let end = default.steps(viewport, spacing, padding[1], false);
        let start_shift = (start.last().expect("start step").0[0].unadjusted
            - start[0].0[0].unadjusted)
            .max(padding[0]);
        let end_shift = (end[0].0.last().expect("anchor").unadjusted
            - end
                .last()
                .expect("end step")
                .0
                .last()
                .expect("anchor")
                .unadjusted)
            .max(padding[1]);
        let start_points = shift_points(&start, start_shift, true);
        let end_points = shift_points(&end, end_shift, false);
        Self {
            kind: StrategyKind::Keylines(KeylinePlan {
                default,
                start,
                end,
                start_points,
                end_points,
                start_shift,
                end_shift,
            }),
            item_size,
            viewport,
            spacing,
        }
    }

    pub fn item_width(&self, index: usize) -> f32 {
        match &self.kind {
            StrategyKind::Variable(items) => items.width(index),
            _ => self.item_size,
        }
    }

    pub fn variable_nearest(&self, offset: f32, count: usize) -> Option<usize> {
        match &self.kind {
            StrategyKind::Variable(items) => Some(items.nearest(offset, self.viewport, count)),
            _ => None,
        }
    }

    pub fn max_scroll(&self, count: usize) -> f32 {
        if let StrategyKind::Variable(items) = &self.kind {
            return items.max_scroll(self.viewport, count);
        }
        (self.item_size * count as f32 + self.spacing * count.saturating_sub(1) as f32
            - self.viewport)
            .max(0.0)
    }

    pub fn snap(&self, index: usize, count: usize) -> f32 {
        let plan = match &self.kind {
            StrategyKind::Variable(items) => return items.snap(index, self.viewport, count),
            StrategyKind::FullScreen => {
                return (index as f32 * (self.item_size + self.spacing))
                    .clamp(0.0, self.max_scroll(count));
            }
            StrategyKind::Keylines(plan) => plan,
        };
        let mut focal = plan.default.0[plan.default.first_focal()];
        if index < plan.start.len() {
            let step = &plan.start[plan.start.len() - 1 - index];
            focal = step.0[step.first_focal()];
        }
        let last = count.saturating_sub(1);
        if index >= last.saturating_sub(plan.end.len() - 1)
            && count > plan.default.last_focal() - plan.default.first_focal() + 1
        {
            let step = &plan.end[(plan.end.len() - 1).saturating_sub(last.saturating_sub(index))];
            focal = step.0[step.last_focal()];
        }
        let offset = (focal.unadjusted - self.item_size * 0.5 + 0.5).floor();
        (index as f32 * (self.item_size + self.spacing) - offset).clamp(0.0, self.max_scroll(count))
    }

    fn keylines(&self, offset: f32, max: f32) -> Keylines {
        let StrategyKind::Keylines(plan) = &self.kind else {
            unreachable!("keyline geometry")
        };
        let offset = offset.max(0.0);
        let end_offset = (max - plan.end_shift).max(0.0);
        if offset >= plan.start_shift && offset <= end_offset {
            return plan.default.clone();
        }
        let (steps, points, progress) = if offset > end_offset {
            let progress = map_range(0.0, 1.0, end_offset, max, offset);
            if end_offset < 0.01 && plan.start.len() == 2 && plan.end.len() == 2 {
                return interpolate(
                    plan.start.last().expect("start"),
                    plan.end.last().expect("end"),
                    progress,
                );
            }
            (&plan.end, &plan.end_points, progress)
        } else {
            (
                &plan.start,
                &plan.start_points,
                map_range(1.0, 0.0, 0.0, plan.start_shift, offset),
            )
        };
        for index in 1..steps.len() {
            if progress <= points[index] {
                return interpolate(
                    &steps[index - 1],
                    &steps[index],
                    map_range(0.0, 1.0, points[index - 1], points[index], progress),
                );
            }
        }
        steps[0].clone()
    }

    pub fn masks(&self, offset: f32, count: usize) -> Vec<(usize, f32, f32)> {
        if let StrategyKind::Variable(items) = &self.kind {
            return items.masks(offset, self.viewport, count);
        }
        if matches!(self.kind, StrategyKind::FullScreen) {
            let step = self.item_size + self.spacing;
            let first = (offset.max(0.0) / step).floor() as usize;
            return (first..first.saturating_add(2).min(count))
                .filter_map(|index| {
                    let left = index as f32 * step - offset;
                    (left < self.viewport && left + self.item_size > 0.0).then_some((
                        index,
                        left + self.item_size * 0.5,
                        self.item_size,
                    ))
                })
                .collect();
        }
        let keys = self.keylines(offset, self.max_scroll(count));
        let step = self.item_size + self.spacing;
        let first_unadjusted = keys.0[0].unadjusted;
        let last_unadjusted = keys.0.last().expect("anchor").unadjusted;
        let first = ((first_unadjusted + offset - self.item_size * 0.5) / step)
            .floor()
            .max(0.0) as usize;
        let end = (((last_unadjusted + offset - self.item_size * 0.5) / step)
            .ceil()
            .max(0.0) as usize)
            .saturating_add(1)
            .min(count);
        (first.saturating_sub(1)..end)
            .filter_map(|index| {
                let center = index as f32 * step + self.item_size * 0.5 - offset;
                let before = keys
                    .0
                    .iter()
                    .rfind(|key| key.unadjusted < center)
                    .unwrap_or(&keys.0[0]);
                let after = keys
                    .0
                    .iter()
                    .find(|key| key.unadjusted >= center)
                    .unwrap_or(keys.0.last().expect("anchor"));
                let progress = if before == after {
                    1.0
                } else {
                    (center - before.unadjusted) / (after.unadjusted - before.unadjusted)
                };
                let key = before.lerp(*after, progress);
                let position = key.offset
                    + if before == after {
                        (center - key.unadjusted) / key.size.max(0.001)
                    } else {
                        0.0
                    };
                let width = key.size.max(0.0);
                (width > 0.0
                    && position + width * 0.5 > 0.0
                    && position - width * 0.5 < self.viewport)
                    .then_some((index, position, width))
            })
            .collect()
    }
}

fn interpolate(start: &Keylines, end: &Keylines, progress: f32) -> Keylines {
    Keylines(
        start
            .0
            .iter()
            .zip(&end.0)
            .map(|(start, end)| start.lerp(*end, progress))
            .collect(),
    )
}

fn shift_points(steps: &[Keylines], distance: f32, start: bool) -> Vec<f32> {
    let mut result = vec![0.0];
    if distance == 0.0 {
        return result;
    }
    for index in 1..steps.len() {
        let delta = if start {
            steps[index].0[0].unadjusted - steps[index - 1].0[0].unadjusted
        } else {
            steps[index - 1].0.last().expect("anchor").unadjusted
                - steps[index].0.last().expect("anchor").unadjusted
        };
        result.push(if index == steps.len() - 1 {
            1.0
        } else {
            result[index - 1] + delta / distance
        });
    }
    result
}

fn lerp(start: f32, end: f32, fraction: f32) -> f32 {
    start + (end - start) * fraction
}
fn map_range(start: f32, end: f32, low: f32, high: f32, value: f32) -> f32 {
    if value <= low {
        start
    } else if value >= high {
        end
    } else {
        lerp(start, end, (value - low) / (high - low))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plan(strategy: &Strategy) -> &KeylinePlan {
        let StrategyKind::Keylines(plan) = &strategy.kind else {
            panic!("expected keyline geometry")
        };
        plan
    }

    fn build_strategy(config: StrategyConfig) -> Strategy {
        Strategy::new(config, &[])
    }

    fn config(
        variant: CarouselVariant,
        viewport: f32,
        preferred: Option<f32>,
        count: usize,
    ) -> StrategyConfig {
        StrategyConfig {
            variant,
            viewport,
            preferred,
            spacing: 0.0,
            density: 1.0,
            count,
            small_range: [40.0, 56.0],
            padding: [0.0; 2],
            item_height: 180.0,
            aspect_revision: 0,
        }
    }

    #[test]
    fn multi_browse_matches_reference_spacing_and_keylines() {
        let mut config = config(CarouselVariant::MultiBrowse, 380.0, Some(186.0), 10);
        config.spacing = 8.0;
        let strategy = build_strategy(config);
        assert_eq!(strategy.item_size, 186.0);
        assert_eq!(
            plan(&strategy)
                .default
                .0
                .iter()
                .map(|key| key.unadjusted)
                .collect::<Vec<_>>(),
            [-101.0, 93.0, 287.0, 481.0, 675.0]
        );
        assert_eq!(plan(&strategy).default.0[3].size, 56.0);
        assert_eq!(plan(&strategy).default.0[3].offset, 352.0);
        assert_eq!(strategy.max_scroll(10), 1552.0);
    }

    #[test]
    fn multi_browse_fits_small_viewports_and_short_collections() {
        let small = build_strategy(config(CarouselVariant::MultiBrowse, 100.0, Some(200.0), 10));
        assert_eq!(plan(&small).default.0.len(), 4);
        assert_eq!(plan(&small).default.0[2].size, 40.0);
        let tiny = build_strategy(config(CarouselVariant::MultiBrowse, 40.0, Some(200.0), 10));
        assert_eq!(tiny.item_size, 40.0);
        let short = build_strategy(config(CarouselVariant::MultiBrowse, 512.0, Some(200.0), 3));
        assert_eq!(plan(&short).default.0.len(), 5);
        assert!(plan(&short).default.0[1].focal && plan(&short).default.0[2].focal);
        assert!(plan(&short).default.0[3].size < short.item_size);
        assert_eq!(
            build_strategy(config(CarouselVariant::MultiBrowse, 500.0, Some(120.0), 10)).item_size,
            120.0
        );
    }

    #[test]
    fn centered_hero_matches_reference_small_item_geometry() {
        let strategy = build_strategy(config(CarouselVariant::CenteredHero, 180.0, None, 6));
        assert_eq!(strategy.item_size, 100.0);
        assert_eq!(
            plan(&strategy)
                .default
                .0
                .iter()
                .map(|key| key.size)
                .collect::<Vec<_>>(),
            [10.0, 40.0, 100.0, 40.0, 10.0]
        );
        assert_eq!(plan(&strategy).default.0[2].offset, 90.0);
        let first = strategy.masks(0.0, 6);
        assert_eq!(first[0], (0, 50.0, 100.0));
        let last = strategy.masks(strategy.max_scroll(6), 6);
        assert_eq!(last.last().copied(), Some((5, 130.0, 100.0)));
        let short = build_strategy(config(CarouselVariant::CenteredHero, 180.0, None, 2));
        assert_eq!(plan(&short).default.first_focal(), 1);
        assert!(short.item_size > 100.0);
        assert_eq!(
            build_strategy(config(CarouselVariant::CenteredHero, 120.0, None, 1)).item_size,
            120.0
        );
        assert_eq!(
            build_strategy(config(CarouselVariant::CenteredHero, 120.0, None, 6)).item_size,
            120.0
        );
    }

    #[test]
    fn centered_hero_spacing_keeps_both_outer_items_aligned() {
        for spacing in [0.0, 12.0] {
            let strategy = build_strategy(StrategyConfig {
                spacing,
                ..config(CarouselVariant::CenteredHero, 980.0, Some(300.0), 10)
            });
            let first = plan(&strategy).default.0[plan(&strategy).default.first_non_anchor()];
            let last = plan(&strategy).default.0[plan(&strategy).default.last_non_anchor()];
            assert_eq!(first.offset, 20.0);
            assert_eq!(last.offset, 960.0);
        }
    }

    #[test]
    fn uncontained_matches_reference_cutoff_sizes() {
        let strategy = build_strategy(config(CarouselVariant::Uncontained, 400.0, Some(125.0), 10));
        assert_eq!(plan(&strategy).default.0.len(), 6);
        assert_eq!(plan(&strategy).default.0[0].size, 18.75);
        assert_eq!(plan(&strategy).default.0[0].offset, -9.375);
        assert_eq!(plan(&strategy).default.0[4].size, 37.5);
        assert_eq!(plan(&strategy).default.0[4].offset, 393.75);
        let large = build_strategy(config(
            CarouselVariant::Uncontained,
            1000.0,
            Some(501.0),
            10,
        ));
        assert_eq!(plan(&large).default.0[0].offset, -125.25);
        assert_eq!(plan(&large).default.0[2].size, 501.0);
        assert_eq!(plan(&large).default.0[3].offset, 1007.0);
        let full = build_strategy(config(CarouselVariant::Uncontained, 400.0, Some(500.0), 10));
        assert_eq!(full.item_size, 400.0);
        assert_eq!(plan(&full).default.0.len(), 3);
    }

    #[test]
    fn padded_end_steps_match_reference_two_large_items() {
        let strategy = build_strategy(StrategyConfig {
            spacing: 8.0,
            padding: [16.0; 2],
            ..config(CarouselVariant::MultiBrowse, 444.0, Some(186.0), 10)
        });
        assert_eq!(strategy.item_size, 186.0);
        let start = plan(&strategy).start.last().expect("start step");
        assert!((start.0[3].offset + start.0[3].size * 0.5 - 444.0).abs() < 0.001);
        let end = plan(&strategy).end.last().expect("end step");
        assert!((end.0[1].size + end.0[2].size + end.0[3].size + 16.0 - 428.0).abs() < 0.001);
        assert!((end.0[1].offset - end.0[1].size * 0.5).abs() < 0.001);
    }

    #[test]
    fn density_preserves_mask_geometry() {
        let logical = config(CarouselVariant::MultiBrowse, 380.0, Some(186.0), 10);
        let base = build_strategy(logical);
        for density in [1.25, 1.5, 2.0] {
            let scaled = build_strategy(StrategyConfig {
                viewport: logical.viewport * density,
                preferred: logical.preferred.map(|width| width * density),
                density,
                small_range: logical.small_range.map(|width| width * density),
                ..logical
            });
            for fraction in [0.0, 0.1, 0.5, 0.9, 1.0] {
                let original = base.masks(base.max_scroll(10) * fraction, 10);
                let physical = scaled.masks(scaled.max_scroll(10) * fraction, 10);
                assert_eq!(original.len(), physical.len());
                for (original, physical) in original.iter().zip(physical) {
                    assert_eq!(original.0, physical.0);
                    assert!((original.1 - physical.1 / density).abs() < 0.001);
                    assert!((original.2 - physical.2 / density).abs() < 0.001);
                }
            }
        }
    }

    #[test]
    fn full_screen_pages_keep_their_width_between_snap_positions() {
        let strategy = build_strategy(StrategyConfig {
            spacing: 16.0,
            padding: [16.0, 16.0],
            ..config(CarouselVariant::FullScreen, 390.0, Some(186.0), 5)
        });
        assert_eq!(strategy.item_size, 390.0);
        assert_eq!(strategy.max_scroll(5), 1624.0);
        assert_eq!(strategy.snap(4, 5), 1624.0);
        assert_eq!(strategy.masks(0.0, 5), vec![(0, 195.0, 390.0)]);
        assert_eq!(
            strategy.masks(203.0, 5),
            vec![(0, -8.0, 390.0), (1, 398.0, 390.0)]
        );
        assert_eq!(strategy.masks(1624.0, 5), vec![(4, 195.0, 390.0)]);
    }

    #[test]
    fn masks_remain_finite_ordered_and_virtualized_while_scrolling() {
        for variant in [
            CarouselVariant::MultiBrowse,
            CarouselVariant::Uncontained,
            CarouselVariant::UncontainedMultiAspectRatio,
            CarouselVariant::Hero,
            CarouselVariant::CenteredHero,
            CarouselVariant::FullScreen,
        ] {
            for width in [40.0, 100.0, 180.0, 380.0, 980.0] {
                for count in [1, 2, 3, 10, 10_000] {
                    let strategy = build_strategy(config(variant, width, Some(186.0), count));
                    for frame in 0..101 {
                        let offset = strategy.max_scroll(count) * frame as f32 / 100.0;
                        let masks = strategy.masks(offset, count);
                        assert!(masks.len() <= (width / strategy.item_size).ceil() as usize + 6);
                        for (index, center, size) in &masks {
                            assert!(*index < count && center.is_finite() && size.is_finite());
                            assert!(*size > 0.0 && *size <= strategy.item_width(*index) + 0.001);
                        }
                        assert!(
                            masks
                                .windows(2)
                                .all(|pair| pair[0].0 < pair[1].0 && pair[0].1 < pair[1].1)
                        );
                    }
                }
            }
        }
    }
}
