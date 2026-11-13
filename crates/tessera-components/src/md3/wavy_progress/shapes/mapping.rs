// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 The Android Open Source Project
// Adapted to Rust for Tessera.

use super::{DISTANCE_EPSILON, measure::ProgressFeature};

pub(super) fn modulo(value: f32) -> f32 {
    (value % 1.0 + 1.0) % 1.0
}

fn in_range(value: f32, start: f32, end: f32) -> bool {
    if end >= start {
        value >= start && value <= end
    } else {
        value >= start || value <= end
    }
}

fn distance(a: f32, b: f32) -> f32 {
    let delta = (a - b).abs();
    delta.min(1.0 - delta)
}

pub(super) struct Mapping(Vec<[f32; 2]>);

impl Mapping {
    pub(super) fn new(first: &[ProgressFeature], second: &[ProgressFeature]) -> Self {
        let mut candidates = Vec::new();
        for (i, a) in first.iter().enumerate() {
            for (j, b) in second.iter().enumerate() {
                if a.feature.convex.is_some() && a.feature.convex == b.feature.convex {
                    candidates.push((
                        (a.feature.representative() - b.feature.representative()).length_squared(),
                        i,
                        j,
                    ));
                }
            }
        }
        candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
        if candidates.is_empty() {
            return Self(vec![[0.0, 0.0], [0.5, 0.5]]);
        }
        if candidates.len() == 1 {
            let (_, i, j) = candidates[0];
            let (a, b) = (first[i].progress, second[j].progress);
            return Self(vec![[a, b], [(a + 0.5) % 1.0, (b + 0.5) % 1.0]]);
        }
        let mut mapping: Vec<[f32; 2]> = Vec::new();
        let mut used_a = vec![false; first.len()];
        let mut used_b = vec![false; second.len()];
        for (_, i, j) in candidates {
            if used_a[i] || used_b[j] {
                continue;
            }
            let (a, b) = (first[i].progress, second[j].progress);
            let index = mapping.partition_point(|pair| pair[0] < a);
            let n = mapping.len();
            if n > 0 {
                let before = mapping[(index + n - 1) % n];
                let after = mapping[index % n];
                if distance(a, before[0]) < DISTANCE_EPSILON
                    || distance(a, after[0]) < DISTANCE_EPSILON
                    || distance(b, before[1]) < DISTANCE_EPSILON
                    || distance(b, after[1]) < DISTANCE_EPSILON
                    || (n > 1 && !in_range(b, before[1], after[1]))
                {
                    continue;
                }
            }
            mapping.insert(index, [a, b]);
            used_a[i] = true;
            used_b[j] = true;
        }
        Self(mapping)
    }

    pub(super) fn map(&self, value: f32, reverse: bool) -> f32 {
        let x = usize::from(reverse);
        let y = 1 - x;
        let index = (0..self.0.len())
            .find(|&i| in_range(value, self.0[i][x], self.0[(i + 1) % self.0.len()][x]))
            .expect("progress mapping covers the outline");
        let a = self.0[index];
        let b = self.0[(index + 1) % self.0.len()];
        let dx = modulo(b[x] - a[x]);
        let t = if dx < 0.001 {
            0.5
        } else {
            modulo(value - a[x]) / dx
        };
        modulo(a[y] + modulo(b[y] - a[y]) * t)
    }
}
