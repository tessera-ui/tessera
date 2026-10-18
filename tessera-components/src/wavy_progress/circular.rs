use std::{
    f32::consts::TAU,
    sync::{Arc, Mutex},
};

use glam::Vec2;
use tessera_ui::{Color, Dp};

use crate::{pipelines::path_stroke::command::PathStrokeCommand, progress::ProgressStrokeCap};

use super::shapes::{CircularMorph, Cubic, bounds, circular_morph};

#[derive(Clone, Default)]
pub(super) struct Cache(Arc<Mutex<Geometry>>);

impl PartialEq for Cache {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Default)]
struct Geometry {
    vertices: usize,
    morph: Option<CircularMorph>,
    requires_morph: bool,
    key: Option<[f32; 4]>,
    active: Vec<Vec2>,
    track: Vec<Vec2>,
}

pub(super) fn vertices(width: f32, height: f32, stroke: f32, wavelength: f32) -> usize {
    if wavelength > 0.0 {
        (TAU * ((width.min(height) - stroke) * 0.5).max(0.0) / wavelength)
            .round()
            .clamp(5.0, 1024.0) as usize
    } else {
        5
    }
}

fn path(mut curves: Vec<Cubic>, pivot: Vec2, size: Vec2, stroke: f32) -> Vec<Vec2> {
    let start = curves[0].0[0] - pivot;
    let rotation = -start.y.atan2(start.x) - TAU * 0.25;
    let rotate = |point: Vec2| Vec2::from_angle(rotation).rotate(point);
    let scale = (size - Vec2::splat(stroke)).max(Vec2::ZERO);
    for curve in &mut curves {
        for point in &mut curve.0 {
            *point = rotate(*point) * scale;
        }
    }
    let (low, high) = bounds(&curves);
    let offset = (size - low - high) * 0.5;
    let mut points = vec![curves[0].0[0] + offset];
    for curve in curves {
        let second = (curve.0[0] - curve.0[1] * 2.0 + curve.0[2])
            .length()
            .max((curve.0[1] - curve.0[2] * 2.0 + curve.0[3]).length())
            * 6.0;
        // The second derivative bounds the distance from each chord to its
        // cubic.
        let count = (second / (8.0 * 0.01)).sqrt().ceil().max(1.0) as usize;
        for i in 1..=count {
            points.push(curve.point(i as f32 / count as f32) + offset);
        }
    }
    let first = points[0];
    *points.last_mut().unwrap() = first;
    points
}

fn length(points: &[Vec2]) -> f32 {
    points
        .windows(2)
        .map(|pair| pair[0].distance(pair[1]))
        .sum()
}

fn segment(points: &[Vec2], start: f32, end: f32) -> Vec<Vec2> {
    if end <= start {
        return Vec::new();
    }
    let total = length(points);
    if total <= 0.000001 {
        return Vec::new();
    }
    let mut distance = 0.0;
    let mut result = Vec::new();
    for i in 0..points.len() * 2 - 2 {
        let a = points[i % (points.len() - 1)];
        let b = points[(i + 1) % (points.len() - 1)];
        let size = a.distance(b);
        if size > 0.000001 && distance + size >= start && distance < end {
            if result.is_empty() {
                result.push(a.lerp(b, ((start - distance) / size).clamp(0.0, 1.0)));
            }
            result.push(a.lerp(b, ((end - distance) / size).clamp(0.0, 1.0)));
        }
        distance += size;
        if distance >= end {
            break;
        }
    }
    result
}

#[derive(Clone, PartialEq)]
pub(super) struct CircularWaveDraw {
    pub cache: Cache,
    pub wavelength: Dp,
    pub progress: f32,
    pub amplitude: f32,
    pub phase: f32,
    pub rotation: f32,
    pub stroke_width: Dp,
    pub track_stroke_width: Dp,
    pub gap: Dp,
    pub stroke_cap: ProgressStrokeCap,
    pub track_stroke_cap: ProgressStrokeCap,
    pub color: Color,
    pub track_color: Color,
    pub reverse: bool,
}

impl CircularWaveDraw {
    pub(super) fn commands(&self, width: f32, height: f32) -> Vec<PathStrokeCommand> {
        if width <= 0.0 || height <= 0.0 {
            return Vec::new();
        }
        let stroke = self.stroke_width.to_pixels_f32();
        let track_stroke = self.track_stroke_width.to_pixels_f32();
        let mut geometry = self.cache.0.lock().unwrap();
        let count = vertices(width, height, stroke, self.wavelength.to_pixels_f32());
        if geometry.vertices != count {
            geometry.vertices = count;
            geometry.morph = Some(circular_morph(count));
            geometry.key = None;
        }
        let requires_morph =
            geometry.requires_morph || (self.amplitude > 0.0 && self.amplitude < 1.0);
        let key = [width, height, stroke, self.amplitude];
        if geometry.key != Some(key) || requires_morph != geometry.requires_morph {
            geometry.requires_morph = requires_morph;
            let morph = geometry.morph.as_ref().unwrap();
            let size = Vec2::new(width, height);
            let active = if requires_morph {
                let mut curves: Vec<_> = morph
                    .pairs
                    .iter()
                    .map(|pair| {
                        Cubic(std::array::from_fn(|i| {
                            pair[0].0[i].lerp(pair[1].0[i], self.amplitude)
                        }))
                    })
                    .collect();
                let first = curves[0].0[0];
                curves.last_mut().unwrap().0[3] = first;
                path(curves, Vec2::splat(0.5), size, stroke)
            } else if self.amplitude == 1.0 {
                path(morph.active.clone(), morph.active_center, size, stroke)
            } else {
                path(morph.track.clone(), morph.track_center, size, stroke)
            };
            let track = path(morph.track.clone(), morph.track_center, size, stroke);
            geometry.active = active;
            geometry.track = track;
            geometry.key = Some(key);
        }
        let active_length = length(&geometry.active);
        let track_length = length(&geometry.track);
        let stop = self.progress * active_length;
        let inset = if (self.stroke_cap == ProgressStrokeCap::Butt
            && self.track_stroke_cap == ProgressStrokeCap::Butt)
            || height > width
        {
            0.0
        } else {
            (stroke.max(track_stroke) * 0.5).min(stop)
        };
        let spacing = 2.0 * inset + self.gap.to_pixels_f32().min(stop);
        let phase = self.phase.rem_euclid(1.0);
        let mut commands = Vec::with_capacity(2);
        let center = Vec2::new(width, height) * 0.5;
        let transform = |points: Vec<Vec2>, rotation: f32| -> Arc<[Vec2]> {
            let rotation = Vec2::from_angle(rotation);
            points
                .into_iter()
                .map(|point| {
                    let mut point = center + rotation.rotate(point - center);
                    if self.reverse {
                        point.x = width - point.x;
                    }
                    point
                })
                .collect()
        };
        let track = segment(
            &geometry.track,
            self.progress * track_length + spacing,
            track_length - spacing,
        );
        if track.len() > 1 && track_stroke > 0.0 && self.track_color.a > 0.0 {
            commands.push(PathStrokeCommand {
                points: transform(track, self.rotation),
                color: self.track_color,
                stroke: track_stroke,
                round: self.track_stroke_cap == ProgressStrokeCap::Round,
                closed: self.progress == 0.0,
            });
        }
        let active = segment(
            &geometry.active,
            phase * active_length,
            (phase + self.progress) * active_length,
        );
        if active.len() > 1 && stroke > 0.0 && self.color.a > 0.0 {
            commands.push(PathStrokeCommand {
                points: transform(active, self.rotation - phase * TAU),
                color: self.color,
                stroke,
                round: self.stroke_cap == ProgressStrokeCap::Round,
                closed: self.progress == 1.0,
            });
        }
        commands
    }
}

impl tessera_ui::DrawModifierNode for CircularWaveDraw {
    fn draw(
        &self,
        ctx: &mut tessera_ui::DrawModifierContext<'_, '_>,
        content: &mut dyn tessera_ui::DrawModifierContent,
    ) {
        let size = ctx
            .render_input
            .metadata_mut()
            .computed_data()
            .unwrap_or(tessera_ui::ComputedData::ZERO);
        for command in self.commands(size.width.to_f32(), size.height.to_f32()) {
            ctx.render_input
                .metadata_mut()
                .fragment_mut()
                .push_draw_command(command);
        }
        content.draw(ctx.render_input);
    }
}
