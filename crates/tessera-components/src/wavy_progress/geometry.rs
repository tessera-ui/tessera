use tessera_ui::{Color, Dp};

use crate::{pipelines::linear_wave::command::LinearWaveCommand, progress::ProgressStrokeCap};

#[derive(Clone, Copy, PartialEq)]
pub(super) struct LinearWaveDraw {
    pub fractions: [[f32; 2]; 2],
    pub count: usize,
    pub amplitude: f32,
    pub phase: f32,
    pub wavelength: Dp,
    pub stroke_width: Dp,
    pub track_stroke_width: Dp,
    pub gap: Dp,
    pub stop: Dp,
    pub stroke_cap: ProgressStrokeCap,
    pub track_stroke_cap: ProgressStrokeCap,
    pub color: Color,
    pub track_color: Color,
    pub reverse: bool,
}

fn path_x(nominal: f32, half_wave: f32, control_height: f32) -> f32 {
    let half_wave = f64::from(half_wave);
    let control = f64::from(control_height).abs();
    if control < 0.000_001 {
        return nominal;
    }
    let primitive = |v: f64| {
        (v * (half_wave * half_wave + v * v).sqrt()
            + half_wave * half_wave * (v / half_wave).asinh())
            * 0.5
    };
    let a = primitive(2.0 * control);
    let arc = |t| (a - primitive(2.0 * control * (1.0 - 2.0 * t))) / (4.0 * control);
    let cell = (f64::from(nominal) / half_wave).floor();
    let target = (f64::from(nominal) / half_wave - cell) * arc(1.0);
    let (mut low, mut high) = (0.0, 1.0);
    for _ in 0..24 {
        let t = (low + high) * 0.5;
        if arc(t) < target {
            low = t;
        } else {
            high = t;
        }
    }
    ((cell + (low + high) * 0.5) * half_wave) as f32
}

impl LinearWaveDraw {
    pub(super) fn commands(&self, width: f32, height: f32) -> Vec<LinearWaveCommand> {
        if width <= 0.0 || height <= 0.0 {
            return Vec::new();
        }
        let stroke = self.stroke_width.to_pixels_f32();
        let track_stroke = self.track_stroke_width.to_pixels_f32();
        let cap = if (self.stroke_cap == ProgressStrokeCap::Butt
            && self.track_stroke_cap == ProgressStrokeCap::Butt)
            || height > width
        {
            0.0
        } else {
            (stroke.max(track_stroke) * 0.5).min(width * 0.5)
        };
        let wavelength = self.wavelength.to_pixels_f32();
        let control_height = if wavelength > 0.000_001 && self.amplitude > 0.0 {
            (height - stroke) * self.amplitude
        } else {
            0.0
        };
        let shift = if control_height != 0.0 {
            self.phase * wavelength
        } else {
            0.0
        };
        let command = LinearWaveCommand {
            color: self.color,
            start: 0.0,
            end: 0.0,
            stroke,
            wavelength,
            control_height,
            shift,
            round: self.stroke_cap == ProgressStrokeCap::Round,
            reverse: self.reverse,
        };
        let track = LinearWaveCommand {
            color: self.track_color,
            stroke: track_stroke,
            control_height: 0.0,
            shift: 0.0,
            round: self.track_stroke_cap == ProgressStrokeCap::Round,
            ..command
        };
        let mut tracks = Vec::with_capacity(3);
        let mut active = Vec::with_capacity(2);
        let mut track_end = width - cap;
        let head = self.fractions[0][1] * width;
        let spacing = if head < cap {
            0.0
        } else {
            (head - cap).min(self.gap.to_pixels_f32()) + cap * 2.0
        };
        for [tail, head] in &self.fractions[..self.count] {
            let (tail, head) = (tail * width, head * width);
            let (start, end) = (tail.clamp(cap, width - cap), head.clamp(cap, width - cap));
            if track_end > end + spacing {
                tracks.push(LinearWaveCommand {
                    start: end + spacing,
                    end: track_end,
                    ..track
                });
            }
            if end > start {
                let position = |x| {
                    if control_height == 0.0 {
                        x
                    } else {
                        path_x(x + shift, wavelength * 0.5, height - stroke) - shift
                    }
                };
                active.push(LinearWaveCommand {
                    start: position(start),
                    end: position(end),
                    ..command
                });
            }
            if head > tail {
                track_end = cap.max(start - spacing);
            }
        }
        if track_end > cap {
            tracks.push(LinearWaveCommand {
                start: cap,
                end: track_end,
                ..track
            });
        }
        tracks.sort_by(|a, b| a.start.total_cmp(&b.start));
        let mut merged: Vec<LinearWaveCommand> =
            Vec::with_capacity(tracks.len() + active.len() + 1);
        for track in tracks {
            if let Some(previous) = merged.last_mut()
                && track.start <= previous.end
            {
                previous.end = previous.end.max(track.end);
            } else {
                merged.push(track);
            }
        }
        let mut tracks = merged;
        tracks.extend(active);
        let mut stop = track_stroke.min(self.stop.to_pixels_f32());
        let inset = if stop == track_stroke {
            0.0
        } else {
            track_stroke * 0.25
        };
        let mut x = width - stop - inset;
        let head = self.fractions[0][1] * width + cap;
        if x <= head {
            stop = (stop - (head - x)).max(0.0);
            x = head;
        }
        if stop > 0.0 {
            let center = x + stop * 0.5;
            tracks.push(LinearWaveCommand {
                start: if track.round { center } else { x },
                end: if track.round { center } else { x + stop },
                stroke: stop,
                color: self.color,
                ..track
            });
        }
        tracks.retain(|command| command.stroke > 0.0 && command.color.a > 0.0);
        tracks
    }
}

impl tessera_ui::DrawModifierNode for LinearWaveDraw {
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
