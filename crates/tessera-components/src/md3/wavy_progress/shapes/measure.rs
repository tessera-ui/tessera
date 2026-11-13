// SPDX-License-Identifier: Apache-2.0
// Copyright 2023 The Android Open Source Project
// Adapted to Rust for Tessera.

use super::{Cubic, DISTANCE_EPSILON, Feature, Polygon, mapping::modulo};

fn measure(curve: Cubic, threshold: f32) -> (f32, f32) {
    let mut previous = curve.0[0];
    let mut total = 0.0;
    let mut remaining = threshold;
    for i in 1..=3 {
        let t = i as f32 / 3.0;
        let point = curve.point(t);
        let segment = point.distance(previous);
        if segment >= remaining {
            return (t - (1.0 - remaining / segment) / 3.0, threshold);
        }
        remaining -= segment;
        total += segment;
        previous = point;
    }
    (1.0, total)
}

#[derive(Clone)]
pub(super) struct MeasuredCurve {
    pub curve: Cubic,
    pub start: f32,
    pub end: f32,
    length: f32,
}

impl MeasuredCurve {
    fn new(curve: Cubic, start: f32, end: f32) -> Self {
        Self {
            curve,
            start,
            end,
            length: measure(curve, f32::INFINITY).1,
        }
    }

    pub(super) fn cut(&self, progress: f32) -> (Self, Self) {
        let progress = progress.clamp(self.start, self.end);
        let t = measure(
            self.curve,
            (progress - self.start) / (self.end - self.start) * self.length,
        )
        .0;
        let (a, b) = self.curve.split(t);
        (
            Self::new(a, self.start, progress),
            Self::new(b, progress, self.end),
        )
    }
}

#[derive(Clone)]
pub(super) struct ProgressFeature {
    pub progress: f32,
    pub feature: Feature,
}

#[derive(Clone)]
pub(super) struct MeasuredPolygon {
    pub curves: Vec<MeasuredCurve>,
    pub features: Vec<ProgressFeature>,
}

impl MeasuredPolygon {
    fn from_parts(curves: Vec<Cubic>, outline: Vec<f32>, features: Vec<ProgressFeature>) -> Self {
        let mut start = 0.0;
        let mut measured = Vec::new();
        for (i, curve) in curves.into_iter().enumerate() {
            if outline[i + 1] - outline[i] > DISTANCE_EPSILON {
                measured.push(MeasuredCurve::new(curve, start, outline[i + 1]));
                start = outline[i + 1];
            }
        }
        measured.last_mut().expect("nonempty outline").end = 1.0;
        Self {
            curves: measured,
            features,
        }
    }

    pub(super) fn new(polygon: &Polygon) -> Self {
        let mut curves = Vec::new();
        let mut corners = Vec::new();
        for feature in &polygon.features {
            if feature.convex.is_some() {
                corners.push((curves.len() + feature.curves.len() / 2, feature));
            }
            curves.extend_from_slice(&feature.curves);
        }
        let mut outline = vec![0.0];
        for curve in &curves {
            outline.push(outline.last().unwrap() + measure(*curve, f32::INFINITY).1);
        }
        let total = *outline.last().unwrap();
        for progress in &mut outline {
            *progress /= total;
        }
        let features = corners
            .into_iter()
            .map(|(i, feature)| ProgressFeature {
                progress: modulo((outline[i] + outline[i + 1]) * 0.5),
                feature: feature.clone(),
            })
            .collect();
        Self::from_parts(curves, outline, features)
    }

    pub(super) fn cut_and_shift(&self, cut: f32) -> Self {
        if cut < DISTANCE_EPSILON {
            return self.clone();
        }
        let index = self
            .curves
            .iter()
            .position(|curve| cut >= curve.start && cut <= curve.end)
            .expect("cut lies on the outline");
        let (a, b) = self.curves[index].cut(cut);
        let n = self.curves.len();
        let mut curves = vec![b.curve];
        for i in 1..n {
            curves.push(self.curves[(index + i) % n].curve);
        }
        curves.push(a.curve);
        let mut outline = vec![0.0];
        for i in 1..=n {
            outline.push(modulo(self.curves[(index + i - 1) % n].end - cut));
        }
        outline.push(1.0);
        let features = self
            .features
            .iter()
            .map(|feature| ProgressFeature {
                progress: modulo(feature.progress - cut),
                feature: feature.feature.clone(),
            })
            .collect();
        Self::from_parts(curves, outline, features)
    }
}
