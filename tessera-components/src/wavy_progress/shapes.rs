// SPDX-License-Identifier: Apache-2.0
// Copyright 2022-2025 The Android Open Source Project
// Adapted to Rust for Tessera.

mod mapping;
mod measure;

#[cfg(test)]
mod tests;

use std::f32::consts::PI;

use glam::Vec2;

use self::{
    mapping::{Mapping, modulo},
    measure::MeasuredPolygon,
};

const DISTANCE_EPSILON: f32 = 0.0001;
const ANGLE_EPSILON: f32 = 0.000001;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Cubic(pub [Vec2; 4]);

impl Cubic {
    fn line(a: Vec2, b: Vec2) -> Self {
        Self([a, a.lerp(b, 1.0 / 3.0), a.lerp(b, 2.0 / 3.0), b])
    }

    pub(super) fn point(self, t: f32) -> Vec2 {
        let u = 1.0 - t;
        self.0[0] * (u * u * u)
            + self.0[1] * (3.0 * t * u * u)
            + self.0[2] * (3.0 * t * t * u)
            + self.0[3] * (t * t * t)
    }

    pub(super) fn split(self, t: f32) -> (Self, Self) {
        let a = self.0[0].lerp(self.0[1], t);
        let b = self.0[1].lerp(self.0[2], t);
        let c = self.0[2].lerp(self.0[3], t);
        let d = a.lerp(b, t);
        let e = b.lerp(c, t);
        let p = self.point(t);
        (Self([self.0[0], a, d, p]), Self([p, e, c, self.0[3]]))
    }

    fn reverse(self) -> Self {
        Self([self.0[3], self.0[2], self.0[1], self.0[0]])
    }

    fn zero_length(self) -> bool {
        let delta = (self.0[3] - self.0[0]).abs();
        delta.x < DISTANCE_EPSILON && delta.y < DISTANCE_EPSILON
    }

    fn arc(center: Vec2, a: Vec2, b: Vec2) -> Self {
        let start = (a - center).normalize();
        let end = (b - center).normalize();
        let cosine = start.dot(end);
        if cosine > 0.999 {
            return Self::line(a, b);
        }
        let direction = if rotate90(start).dot(b - center) >= 0.0 {
            1.0
        } else {
            -1.0
        };
        let k = (a - center).length() * 4.0 / 3.0
            * ((2.0 * (1.0 - cosine)).sqrt() - (1.0 - cosine * cosine).sqrt())
            / (1.0 - cosine)
            * direction;
        Self([a, a + rotate90(start) * k, b - rotate90(end) * k, b])
    }
}

fn rotate90(point: Vec2) -> Vec2 {
    Vec2::new(-point.y, point.x)
}

#[derive(Clone)]
struct Feature {
    curves: Vec<Cubic>,
    convex: Option<bool>,
}

impl Feature {
    fn representative(&self) -> Vec2 {
        (self.curves[0].0[0] + self.curves[self.curves.len() - 1].0[3]) * 0.5
    }
}

struct Polygon {
    features: Vec<Feature>,
    center: Vec2,
}

impl Polygon {
    fn new(vertices: &[(Vec2, f32, f32)]) -> Self {
        let n = vertices.len();
        let corners: Vec<_> = (0..n)
            .map(|i| {
                Corner::new(
                    vertices[(i + n - 1) % n].0,
                    vertices[i].0,
                    vertices[(i + 1) % n].0,
                    vertices[i].1,
                    vertices[i].2,
                )
            })
            .collect();
        let cuts: Vec<_> = (0..n)
            .map(|i| {
                let next = (i + 1) % n;
                let round = corners[i].round_cut + corners[next].round_cut;
                let full = corners[i].full_cut() + corners[next].full_cut();
                let side = (vertices[i].0 - vertices[next].0).length();
                if round > side {
                    (side / round, 0.0)
                } else if full > side {
                    (1.0, (side - round) / (full - round))
                } else {
                    (1.0, 1.0)
                }
            })
            .collect();
        let curves: Vec<_> = (0..n)
            .map(|i| {
                let allowed = |index: usize| {
                    corners[i].round_cut * cuts[index].0
                        + (corners[i].full_cut() - corners[i].round_cut) * cuts[index].1
                };
                corners[i].curves(allowed((i + n - 1) % n), allowed(i))
            })
            .collect();
        let mut features = Vec::with_capacity(n * 2);
        for i in 0..n {
            let next = (i + 1) % n;
            let prev = (i + n - 1) % n;
            let a = vertices[i].0 - vertices[prev].0;
            let b = vertices[next].0 - vertices[i].0;
            features.push(Feature {
                curves: curves[i].clone(),
                convex: Some(a.x * b.y - a.y * b.x > 0.0),
            });
            features.push(Feature {
                curves: vec![Cubic::line(
                    curves[i].last().expect("corner").0[3],
                    curves[next][0].0[0],
                )],
                convex: None,
            });
        }
        Self {
            features,
            center: Vec2::ZERO,
        }
    }

    fn cubics(&self) -> Vec<Cubic> {
        let mut curves = Vec::new();
        let split = if self.features[0].curves.len() == 3 {
            Some(self.features[0].curves[1].split(0.5))
        } else {
            None
        };
        let mut append = |curve: Cubic| {
            if !curve.zero_length() {
                curves.push(curve);
            } else if let Some(last) = curves.last_mut() {
                last.0[3] = curve.0[3];
            }
        };
        for i in 0..=self.features.len() {
            if i == 0
                && let Some((_, end)) = split
            {
                append(end);
                append(self.features[0].curves[2]);
            } else if i == self.features.len() {
                if let Some((start, _)) = split {
                    append(self.features[0].curves[0]);
                    append(start);
                }
            } else {
                for &curve in &self.features[i].curves {
                    append(curve);
                }
            }
        }
        if let Some(first) = curves.first().map(|curve| curve.0[0])
            && let Some(last) = curves.last_mut()
        {
            last.0[3] = first;
        }
        curves
    }

    fn normalized(mut self) -> Self {
        let curves = self.cubics();
        let (min, max) = bounds(&curves);
        let size = max - min;
        let side = size.x.max(size.y);
        let offset = (Vec2::splat(side) - size) * 0.5 - min;
        for feature in &mut self.features {
            for curve in &mut feature.curves {
                for point in &mut curve.0 {
                    *point = (*point + offset) / side;
                }
            }
        }
        self.center = (self.center + offset) / side;
        self
    }
}

pub(super) fn bounds(curves: &[Cubic]) -> (Vec2, Vec2) {
    let mut low = Vec2::splat(f32::INFINITY);
    let mut high = Vec2::splat(f32::NEG_INFINITY);
    for curve in curves {
        for point in curve.0 {
            low = low.min(point);
            high = high.max(point);
        }
    }
    (low, high)
}

struct Corner {
    previous: Vec2,
    vertex: Vec2,
    next: Vec2,
    before: Vec2,
    after: Vec2,
    radius: f32,
    smoothing: f32,
    round_cut: f32,
}

impl Corner {
    fn new(previous: Vec2, vertex: Vec2, next: Vec2, radius: f32, smoothing: f32) -> Self {
        let a = previous - vertex;
        let b = next - vertex;
        let before = a.normalize_or_zero();
        let after = b.normalize_or_zero();
        let cosine = before.dot(after).clamp(-1.0, 1.0);
        let sine = (1.0 - cosine * cosine).sqrt();
        let round_cut = if sine > 0.001 {
            radius * (cosine + 1.0) / sine
        } else {
            0.0
        };
        Self {
            previous,
            vertex,
            next,
            before,
            after,
            radius,
            smoothing,
            round_cut,
        }
    }

    fn full_cut(&self) -> f32 {
        (1.0 + self.smoothing) * self.round_cut
    }

    fn smoothing_at(&self, cut: f32) -> f32 {
        if cut > self.full_cut() {
            self.smoothing
        } else if cut > self.round_cut {
            self.smoothing * (cut - self.round_cut) / (self.full_cut() - self.round_cut)
        } else {
            0.0
        }
    }

    fn curves(&self, cut0: f32, cut1: f32) -> Vec<Cubic> {
        let cut = cut0.min(cut1);
        if self.round_cut < DISTANCE_EPSILON
            || cut < DISTANCE_EPSILON
            || self.radius < DISTANCE_EPSILON
        {
            return vec![Cubic::line(self.vertex, self.vertex)];
        }
        let actual_cut = cut.min(self.round_cut);
        let radius = self.radius * actual_cut / self.round_cut;
        let center = self.vertex
            + ((self.before + self.after) * 0.5).normalize()
                * (radius * radius + actual_cut * actual_cut).sqrt();
        let before = self.vertex + self.before * actual_cut;
        let after = self.vertex + self.after * actual_cut;
        let flank = |side: Vec2, a: Vec2, b: Vec2, smoothing: f32| {
            let direction = (side - self.vertex).normalize();
            let start = self.vertex + direction * actual_cut * (1.0 + smoothing);
            let p = a.lerp((a + b) * 0.5, smoothing);
            let end = center + (p - center).normalize() * radius;
            let tangent = rotate90(end - center);
            let rotated = rotate90(tangent);
            let denominator = direction.dot(rotated);
            let numerator = (end - side).dot(rotated);
            let anchor = if denominator.abs() < DISTANCE_EPSILON
                || denominator.abs() < DISTANCE_EPSILON * numerator.abs()
            {
                a
            } else {
                side + direction * (numerator / denominator)
            };
            Cubic([start, (start + anchor * 2.0) / 3.0, anchor, end])
        };
        let first = flank(self.previous, before, after, self.smoothing_at(cut0));
        let last = flank(self.next, after, before, self.smoothing_at(cut1)).reverse();
        vec![first, Cubic::arc(center, first.0[3], last.0[0]), last]
    }
}

fn circle(vertices: usize) -> Polygon {
    let radius = 1.0 / (PI / vertices as f32).cos();
    let points: Vec<_> = (0..vertices)
        .map(|i| {
            let angle = 2.0 * PI / vertices as f32 * i as f32;
            (Vec2::new(angle.cos(), angle.sin()) * radius, 1.0, 0.0)
        })
        .collect();
    Polygon::new(&points).normalized()
}

fn star(vertices: usize) -> Polygon {
    let points: Vec<_> = (0..vertices * 2)
        .map(|i| {
            let angle = PI / vertices as f32 * i as f32;
            let (radius, rounding, smoothing) = if i % 2 == 0 {
                (1.0, 0.35, 0.4)
            } else {
                (0.75, 0.5, 0.0)
            };
            (
                Vec2::new(angle.cos(), angle.sin()) * radius,
                rounding,
                smoothing,
            )
        })
        .collect();
    Polygon::new(&points).normalized()
}

pub(super) struct CircularMorph {
    pub track: Vec<Cubic>,
    pub track_center: Vec2,
    pub active: Vec<Cubic>,
    pub active_center: Vec2,
    pub pairs: Vec<[Cubic; 2]>,
}

pub(super) fn circular_morph(vertices: usize) -> CircularMorph {
    assert!(
        vertices >= 3,
        "circular morph requires at least three vertices"
    );
    let circle = circle(vertices);
    let star = star(vertices);
    let first = MeasuredPolygon::new(&circle);
    let second = MeasuredPolygon::new(&star);
    let mapping = Mapping::new(&first.features, &second.features);
    let cut = mapping.map(0.0, false);
    let second = second.cut_and_shift(cut);
    let mut pairs = Vec::new();
    let (mut i, mut j) = (1, 1);
    let mut a = first.curves.first().cloned();
    let mut b = second.curves.first().cloned();
    while let (Some(mut left), Some(mut right)) = (a.take(), b.take()) {
        let end_a = if i == first.curves.len() {
            1.0
        } else {
            left.end
        };
        let end_b = if j == second.curves.len() {
            1.0
        } else {
            mapping.map(modulo(right.end + cut), true)
        };
        let end = end_a.min(end_b);
        if end_a > end + ANGLE_EPSILON {
            let (part, remaining) = left.cut(end);
            left = part;
            a = Some(remaining);
        } else {
            a = first.curves.get(i).cloned();
            i += 1;
        }
        if end_b > end + ANGLE_EPSILON {
            let (part, remaining) = right.cut(modulo(mapping.map(end, false) - cut));
            right = part;
            b = Some(remaining);
        } else {
            b = second.curves.get(j).cloned();
            j += 1;
        }
        pairs.push([left.curve, right.curve]);
    }
    assert!(
        a.is_none() && b.is_none(),
        "circular morph must consume both outlines"
    );
    CircularMorph {
        track: circle.cubics(),
        track_center: circle.center,
        active: star.cubics(),
        active_center: star.center,
        pairs,
    }
}
